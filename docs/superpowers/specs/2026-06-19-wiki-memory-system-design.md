# Obsidian 记忆系统中枢设计(wiki-query 演进)

## 概述

把现有 `wiki-query.exe` 从"查询工具"升级为"个人知识库的跨工具记忆系统中枢",让 Claude Code / MiMo-Code / IDE 类 agent / ZCode 四类工具像调用内置工具一样读写本 Obsidian 仓库。

**基线**:本设计是 [`2026-06-18-wiki-query-design.md`](./2026-06-18-wiki-query-design.md) 的增量演进,不推翻已有实现。

## 已确认决策(brainstorming 阶段冻结)

| 决策点 | 选择 | 影响 |
|--------|------|------|
| 读写语义 | 受控回写 | 需要校验逻辑,纯只读方案否决 |
| 目标工具 | 四类全要(CC/MiMo/IDE/ZCode) | 锁定通用接口,否决单工具私有方案 |
| 接入机制 | 本地 SQLite + 薄胶水 | 否决 MCP server,否决常驻进程 |
| 权威源 | **Markdown** | 否决 SQLite 权威方案,无双向同步 |
| 统一入口 | 复用并扩展 `wiki-query.exe` | 校验逻辑只写一次,所有工具共享 |
| 检索引擎 | 复用 wiki-query(MVP 用 LIKE) | FTS5 列为可选升级,不阻塞 MVP |
| 写入校验 | 只校 frontmatter | wikilink/脚注完整性留给 git 审阅 |

## 架构

```
   Claude Code / MiMo-Code / IDE / ZCode / 人工
                    │
                    │  各自薄胶水,调用同一二进制
                    ▼
          wiki-query.exe  (Rust,统一 CLI,唯一校验入口)
          ├── search       查元数据(已有,LIKE)
          ├── show         读正文(已有)
          ├── write        受控写 Markdown + frontmatter 校验(新增)
          └── sync         Markdown→SQLite 索引(已有,单向)
                    │
                    ▼
   Markdown (sources/wiki/,权威源,git 兜底)
                    │
                    ▼  sync
   SQLite (db/wiki.db,可重建派生索引)
```

### 两个根本性简化

1. **无双向同步**:Markdown 永远权威,SQLite 是 `sync` 可重建的派生索引。人工编辑 Markdown → 跑 sync 更新索引;AI 写也是写 Markdown → 再 sync。方向单一,零冲突。
2. **无常驻进程**:wiki-query.exe 就是统一入口。每个工具的"接入"仅是一行 shell 调用,不存在 MCP server 或独立服务。

## 组件设计

### 1. write 命令(新增,唯一校验归宿)

**职责**:AI/人工向知识库写入新文档或更新已有文档,落盘前校验 frontmatter。

**CLI 接口**:

```
wiki-query write --path <相对路径> --name <标题> [--description <..>] [--type <..>] [--tags <..>] [--body <..>]
```

- `--path`:必填,目标 `.md` 相对路径(如 `wiki/xxx.md`、`sources/agent/yyy.md`)
- `--name`:必填,frontmatter name 字段
- `--description`/`--type`/`--tags`:frontmatter 其余字段
- `--body`:正文(第二个 `---` 之后的内容)。从 stdin 读取以支持长文本。

**校验规则(MVP,只校 frontmatter)**:

| 规则 | 失败行为 |
|------|----------|
| `name` 非空 | 拒绝,返回错误 |
| `description` 非空 | 拒绝 |
| `type` ∈ {concept, reference, ...}(白名单) | 拒绝 |
| `updated` 自动填今天日期 | 自动补全 |
| `tags` 非空 | 拒绝 |

**不做**的校验(留给 git 审阅流程):
- `[[wikilink]]` 目标是否存在
- `[^n]` 脚注是否完整
- 正文是否符合模板

**幂等性**:`write` 对已存在路径执行更新(覆盖 frontmatter + body),对不存在路径执行创建。从不删除(sources/ 禁删规则)。

**输出**:JSON,`{"status": "created"\|"updated", "path": "...", "synced": true}`。

**索引一致性**:write 落盘后,直接对单文件执行 sync 逻辑(复用 `sync.rs` 的 `sync_file`,而非全量 `scan_directory`),保证该文档在 SQLite 中即时更新,无需等待 post-commit。`sync_file` 当前是 `sync.rs` 内部函数,write 命令直接调用即可,无需新增命令。

### 2. search / show(已有,不改)

现状即满足需求,本期不动。FTS5 升级列为可选(见"演进路径")。

### 3. 薄胶水(每工具一段,不重复校验)

胶水的唯一职责:**调用 wiki-query + 决定注入时机**。校验全在二进制里。

| 工具 | 胶水形态 | 注入时机 |
|------|---------|---------|
| Claude Code | `SessionStart` hook 跑 `wiki-query search "<会话主题>"`;自定义 slash 命令 `/wiki` 调 write | 会话开始自动注入 |
| MiMo-Code | `AGENTS.md` 指示 agent 用 wiki-query 查询;plugin 在 `tool.execute.before` 注入(可选) | prompt 级 + 可选自动 |
| IDE agent(Cursor/Cline 等) | 快捷指令 / 任务模板调 `wiki-query search\|show` | 手动触发 |
| ZCode | `AGENTS.md` 指示 + 可选 Skill 封装 | prompt 级 |

胶水本身**不含任何业务逻辑**,只是"在合适时机 spawn 二进制"。这样升级二进制即全工具受益。

## 数据流

### 读流程(AI 查询知识)

```
AI 决定查询 → 胶水 spawn → wiki-query search "关键词"
  → SQLite LIKE 查询 → 返回元数据 JSON
  → (按需) wiki-query show <path> → 返回正文 JSON
```

### 写流程(AI 受控回写)

```
AI 决定记录 → 胶水 spawn → wiki-query write --path ... --name ... --body ...(stdin)
  → frontmatter 校验
  │  失败 → 返回错误,不落盘
  │  通过 → 写 Markdown 文件 + sync 该文件入索引
  → 返回 {"status": "created\|updated", "path": "...", "synced": true}
```

### 同步流程(人工编辑后)

```
人工在 Obsidian 编辑 Markdown → git commit
  → post-commit hook → wiki-query sync → 全量增量扫描入 SQLite
```

(已有,不改动)

## 错误处理

| 场景 | 处理 |
|------|------|
| write 的 frontmatter 校验失败 | 退出码 1,stderr 打印缺哪个字段,**不落盘** |
| write 目标目录不存在 | 自动创建(`std::fs::create_dir_all`) |
| write 的 `--body` 从空 stdin 读取 | 用空字符串,允许创建只有 frontmatter 的骨架 |
| write 路径含 `..` 路径穿越 | 拒绝(sources 禁删规则的延伸防御) |
| sync 时单个文件 frontmatter 缺失 | 已有行为:警告 + 跳过(不中断) |

## 测试策略

- write 的 frontmatter 校验:表驱动测试(合法/非法各 N 组 case)
- write 的幂等性:同 path 两次 write,第二次为 updated
- write 后索引一致性:write 后立即 `show` 该 path 能取回最新内容
- 路径穿越防御:`--path ../etc/passwd` 应被拒
- 各工具胶水:每个工具单独验证"能调通 + 返回正确"(集成测试,非单测)

## 演进路径(非本期范围)

| 升级 | 触发条件 | 工作 |
|------|---------|------|
| FTS5 全文检索 | 文档破万 或 LIKE 召回不满足 | 加 `documents_fts` 虚拟表,search 切 FTS5 |
| wikilink 完整性校验 | AI 写入开始出现断链 | write 前查 SQLite 验证 `[[xxx]]` 目标 |
| 脚注完整性校验 | `[^n]` 溯源开始失准 | write 前正则配对脚注与引用索引 |
| MiMo-Code 自动注入 | 确认 plugin 价值 | 写 `tool.execute.before` plugin,注入 search 结果 |

本期只做 write 命令 + 四工具最小胶水。

## 复杂度评估

| 工作 | 估计 |
|------|------|
| write 命令(frontmatter 解析重组 + 校验 + 落盘 + 即时 sync) | 3h |
| Claude Code SessionStart hook + slash 命令 | 1h |
| MiMo-Code / IDE / ZCode 胶水(主要是 prompt + 脚本) | 1.5h |
| write 测试 | 1.5h |
| **总计** | **7h** |

## 风险

| 风险 | 可能性 | 缓解 |
|------|--------|------|
| AI 重复/低质写入污染知识库 | 中 | write 不校验内容质量,依赖 git 审阅 + 定期 `git revert` |
| frontmatter 白名单 type 漏更新 | 低 | type 白名单从现有文档 frontmatter 自动收集 |
| 各工具胶水漂移(版本/路径不一) | 中 | 文档统一约定二进制绝对路径 + git 跟踪胶水脚本 |
