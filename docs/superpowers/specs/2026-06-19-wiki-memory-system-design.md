# Obsidian 记忆系统中枢设计(wiki-query 演进 v2)

## 概述

把现有 `wiki-query.exe` 从"查询工具"升级为"个人知识库的跨工具记忆系统中枢",让 Claude Code / MiMo-Code / Cursor / Cline / ZCode 等 AI agent 像调用内置工具一样读写本 Obsidian 仓库。

**本设计基于成熟方案调研**(见末尾"参考实现"),非闭门造车。核心架构与 Claude Code auto-memory、Letta Filesystem、Cline Memory Bank 同属"Markdown 权威 + 派生索引"第一梯队朴素派。

**基线**:本设计是 [`2026-06-18-wiki-query-design.md`](./2026-06-18-wiki-query-design.md)(已实现:sync/search/show)的增量演进。

**v1 → v2 变更**:经调研,采纳 CLI+MCP 双前端(取代"双胶水")、Mem0 四态写入决策、FTS5+CJK 检索。

## 已确认决策

| 决策点 | 选择 | 依据 |
|--------|------|------|
| 读写语义 | 受控回写 | 用户需求 |
| 目标工具 | CC/MiMo/Cursor/Cline/ZCode 全要 | 用户需求 |
| 权威源 | **Markdown**(SQLite 派生索引) | Letta benchmark 背书,避免双向同步复杂度 |
| 接入协议 | **CLI + MCP 双前端**(共用 Rust 核心,MCP 走本地 stdio) | MCP 是 agent 接入外部知识的既成标准;**stdio 是纯本地子进程,无云无服务器**(MCP 官方规范:Clients SHOULD support stdio whenever possible) |
| MiMo-Code 接入 | 可走 MCP 或原生 tool(二选一,见下) | MiMo 是主力 |
| 写入逻辑 | **四态决策**(ADD/UPDATE/NOOP/CONTRADICT) | Mem0 50k 星标准答案,解决重复写入污染 |
| 检索引擎 | **FTS5 + CJK tokenizer**(取代 LIKE) | 两份调研一致推荐;解决中文检索 |
| 一致性路线 | Markdown 权威,接受读时陈旧 | 主力是 MiMo 但 post-commit hook 兜底 |
| frontmatter 校验 | write 时校验 5 字段非空 | 极简原则 |
| 向量检索 | **不做**(YAGNI) | Letta 证明文件方案够强;几千篇 FTS5 足够 |
| 知识图谱 | **不做** | git + sources 禁删已留历史 |

## 架构

```
   所有 agent (CC/MiMo/Cursor/Cline/ZCode) + 人工
            │
   ┌────────┴─────────┐
   │                  │
   ▼                  ▼
 CLI 前端          MCP 前端(stdio)
 (wiki-query.exe)  (同一二进制的 serve 模式,
   search/show       或独立 server)
   write/sync              │
   serve(mcp)              │
            │              │
            └──────┬───────┘
                   ▼
         wiki-query 核心 (Rust 库)
         ├── search(FTS5)
         ├── show
         ├── write(四态决策 + frontmatter 校验)
         ├── delete(保留,受 policy 限制)
         └── sync(Markdown→SQLite)
                   │
                   ▼
   Markdown (sources/wiki/templates/, 权威源, git)
                   │
                   ▼  sync
   SQLite (db/wiki.db, FTS5 派生索引,可重建)
```

### 两个根本性简化(保持极简)

1. **无双向同步**:Markdown 永远权威,SQLite 是 `sync` 可重建的派生索引。人工编辑 → commit → post-commit hook sync;AI 写也是写 Markdown → write 内部 sync 该文件。方向单一,零冲突。
2. **核心单一,Rust 库共享**:CLI 和 MCP 是同一 Rust 核心的两个前端,不是两套实现。校验逻辑、检索、四态决策全在核心里,单一真相源。

### 与 cyanheads/obsidian-mcp 的关键区别

cyanheads 依赖 Obsidian Local REST API 插件 + 常驻进程,**我们不改文件 I/O 方式**——核心仍是 Rust 直读直写 Markdown + SQLite 索引 + 单二进制。我们只借鉴它的**工具面设计**(工具分组、section patch、path policy、delete 确认),不引入 REST API 那套。

## 组件设计

### 1. 核心库(Rust,lib)

把现有 `db.rs`/`sync.rs`/`models.rs` 的逻辑抽成可被 CLI 和 MCP 共用的库函数。核心函数:

| 函数 | 职责 |
|------|------|
| `search(query, filters)` | FTS5 检索,返回元数据 + snippet |
| `show(path)` | 读单个文档元数据 + 正文 |
| `write(input)` | 四态决策 + frontmatter 校验 + 落盘 + 即时 sync 该文件 |
| `delete(path)` | 受 path policy 限制(sources 禁删) |
| `sync()` | 全量增量同步 Markdown→SQLite |

### 2. write 命令(四态决策,学 Mem0)

**CLI 接口**:
```
wiki-query write --path <相对路径> --name <标题> --body -(stdin)
                 [--description] [--type] [--tags]
                 [--mode auto|add|update]   # 默认 auto(四态自动决策)
```

**四态决策逻辑**(`--mode auto` 时):

```
write(input) {
  先用 name/tags 做 FTS5 search,看是否已有相关文档:
  
  ├─ 无命中且 path 不存在     → ADD(新建)
  ├─ path 已存在且主题一致     → UPDATE(覆盖 frontmatter+body)
  ├─ 已有等价记录(NOOP)       → 跳过,返回"已存在,未写入"
  └─ 检测到与现有文档冲突      → MVP:返回 status=conflict + 候选文档列表,
                                    不自动处理,交给调用方/人工决策
}
```

> **MVP 范围说明**:本期只实现 ADD / UPDATE / NOOP 三态的自动处理。CONTRADICT(矛盾页 + deprecate 旧文档)作为第四态**仅返回 conflict 信号和候选列表,不自动落盘**——因为"何时算矛盾"的判定主观,自动 deprecate 风险高。完整的矛盾页工作流列入演进路径。

**frontmatter 校验**(通过后才进入四态决策):

| 字段 | 规则 |
|------|------|
| name | 非空 |
| description | 非空 |
| type | ∈ 白名单(concept/reference/contradiction/...) |
| updated | 自动填今天 |
| tags | 非空 |

**不做**的校验(留给 git 审阅):wikilink 目标存在、脚注完整、正文符合模板。

**path policy**(借鉴 cyanheads):

| 目录 | 读 | 写 | 删 |
|------|----|----|-----|
| sources/ | ✅ | ✅(append 优先) | ❌ |
| wiki/ | ✅ | ✅ | 仅 deprecated 标记 |
| templates/ | ✅ | ❌ | ❌ |

**输出**:JSON,`{"status": "add\|update\|noop\|contradict", "path": "...", "synced": true, "reason": "..."}`。

### 3. MCP 前端(本地 stdio,无云无服务器)

> **澄清常见误解**:MCP 默认走 stdio 传输——agent 把 `wiki-query serve` 当**本地子进程**拉起,通过 stdin/stdout 管道传 JSON。整个过程在单机完成:无云、无服务器、无端口、无认证配置。HTTP/SSE 传输只用于"远程团队共享知识库"场景,本期不做。
>
> 与 CLI 的区别仅在通信方式:CLI 是一次性命令,MCP-stdio 是常驻子进程持续对话。两者都本地、都调同一 Rust 核心。

二进制增加 `serve` 子命令,启动 stdio MCP server,工具面借鉴 cyanheads 的成熟设计(只借工具面,不引入其 REST API 依赖):

| 工具 | 对应核心函数 | 说明 |
|------|------------|------|
| `wiki_search` | search | FTS5 检索 |
| `wiki_read` | show | 读单文档 |
| `wiki_write` | write | 四态决策写入 |
| `wiki_list` | (新) | 按 tag/type/dir 列出文档 |
| `wiki_patch` | (新) | section 级编辑(借鉴 cyanheads) |
| `wiki_delete` | delete | 受 policy,需确认 |

工具名加 `wiki_` 前缀避免与 agent 内置工具冲突。MCP 工具直接调核心库函数,与 CLI 完全等价。

### 4. CLI 前端(现有 + 新增)

```
wiki-query search <query>     # 已有,改用 FTS5
wiki-query show <path>        # 已有
wiki-query sync               # 已有
wiki-query write ...          # 新增
wiki-query serve              # 新增,启动本地 stdio MCP server(无云无服务器)
```

### 5. MiMo-Code 接入

MiMo 是主力。**本期推荐 MCP 方式**(与全工具接入一致):

- **本期:MCP**——在 `mimocode.jsonc` 配 `mcp.wiki`(指向 `wiki-query serve`)。与 Claude Code/Cursor/Cline 接入方式统一。
- **可选演进:原生 tool**——`~/.config/mimocode/tool/wiki.ts` 包装核心调用(`registry.ts:180-193` 扫描 `tool/*.ts` 自动注册)。和 memory 工具平起平坐,读路径可直读 SQLite(接受陈旧索引)。本期不实现,列为演进项。

### 6. 其余工具接入

| 工具 | 接入 |
|------|------|
| Claude Code | MCP(配 `.mcp.json`)+ SessionStart hook 自动注入 search 结果 |
| Cursor | MCP(Cursor 原生支持 MCP)|
| Cline | MCP 或快捷指令调 CLI |
| ZCode | MCP 或 AGENTS.md 指示 + Skill |

## 数据流

### 读(AI 查询)
```
AI → MCP wiki_search / CLI search → 核心 search(FTS5) → 返回元数据+snippet
   → (按需) wiki_read / show → 返回正文
```

### 写(AI 受控回写)
```
AI → MCP wiki_write / CLI write --body - → 核心 write:
   frontmatter 校验 → 四态决策 → 落盘 Markdown → sync 该文件入 FTS5
   → 返回 {"status": "add|update|noop|contradict", ...}
```

### 同步(人工编辑后)
```
人工编辑 Markdown → git commit → post-commit hook → wiki-query sync → 全量增量入 FTS5
```

## FTS5 设计(取代 LIKE)

**虚拟表**:
```sql
CREATE VIRTUAL TABLE documents_fts USING fts5(
  path, name, description, type, tags, content,
  content='documents', content_rowid='rowid',
  tokenize='<CJK tokenizer>'   -- 如 wangfenjin/simple
);
```

- 复用 `documents` 表(contentless external content table),不重复存正文
- sync 时同步更新 FTS5 索引
- search 用 `MATCH` + `bm25()` 排序 + `snippet()` 返回片段
- CJK tokenizer 解决中文分词(参考 wangfenjin/simple,~1k 星,Rust)

## 错误处理

| 场景 | 处理 |
|------|------|
| frontmatter 校验失败 | 退出码 1,stderr 指出缺哪个字段,不落盘 |
| 四态决策为 NOOP | 退出码 0,返回"已存在,未写入",不落盘 |
| 四态决策检测到 conflict(MVP) | 退出码 0,返回 status=conflict + 候选文档列表,不落盘 |
| write path 含 `..` | 拒绝(路径穿越防御) |
| write 目标目录不存在 | 自动创建 |
| MCP 传输错误 | 返回标准 MCP error response |
| sync 单文件 frontmatter 缺失 | 警告 + 跳过(不中断) |

## 测试策略

- **核心库**:纯函数测试,CLI/MCP 都受益
  - write 四态决策:表驱动(命中各态的 case)
  - frontmatter 校验:合法/非法 case
  - FTS5 检索:中文/英文/混合查询
  - path policy:各目录读写删权限
- **CLI 前端**:集成测试(spawn 二进制,验证 JSON 输出)
- **MCP 前端**:用 MCP inspector 或简单 stdio client 验证每个工具
- **跨前端等价性**:同一操作经 CLI 和 MCP 结果必须一致

## 演进路径(非本期)

| 升级 | 触发条件 |
|------|---------|
| sqlite-vec 混合检索 | FTS5 召回不满足,出现"关键词对不上"痛点 |
| wikilink 完整性校验 | AI 写入出现断链 |
| 脚注完整性校验 | `[^n]` 溯源失准 |
| wikilink 当 PageRank 排序图(学 Aider) | 检索排序质量需提升 |
| 轻量偏好通道(学 Claude auto-memory) | 需要记一句话偏好,write 太重 |
| 会话级短期记忆层(学 Cline activeContext) | 需要记"当前在干嘛" |

## 复杂度评估

| 工作 | 估计 |
|------|------|
| 核心逻辑抽成库 + write 四态决策 + frontmatter 校验 | 5h |
| FTS5 迁移 + CJK tokenizer | 3h |
| MCP serve 前端(6 工具) | 4h |
| CLI write/serve 子命令 | 1h |
| MiMo/CC/Cursor 接入配置 | 2h |
| 测试(核心 + 跨前端等价) | 4h |
| **总计** | **~19h** |

(v1 估 7h;v2 因吸收 MCP + 四态 + FTS5,增至 ~19h)

## 风险

| 风险 | 可能性 | 缓解 |
|------|--------|------|
| 四态决策的"冲突"判定主观(何时算 CONTRADICT) | 中 | MVP 先只实现 ADD/UPDATE/NOOP,CONTRADICT 留给人工或简单关键词冲突检测 |
| CJK tokenizer 集成到 rusqlite | 中 | 先验证 wangfenjin/simple 能否作为 FTS5 tokenizer 加载;备选 jieba-rs |
| MCP serve 增加二进制复杂度 | 低 | serve 是可选模式,不影响 CLI 主体 |
| MiMo 直读 SQLite 陈旧 | 低 | post-commit hook 兜底;或 MiMo 走 MCP(每次经核心,可加 sync) |
| AI 低质写入 | 中 | 四态 NOOP 兜底 + git 审阅 + git revert |

## 参考实现(调研依据)

**接入协议**:
- [cyanheads/obsidian-mcp-server](https://github.com/cyanheads/obsidian-mcp-server)(3.5k 星)— 工具面设计标杆(read/write/search/list/patch/delete + section patch + path policy)。我们借鉴工具面,不引入其 REST API 依赖。
- [sweir1/obsidian-brain](https://mcpservers.org/servers/sweir1/obsidian-brain)— 另一现成方案,**不依赖 Obsidian 开着**(直接读文件系统),有语义搜索+RRF 混合检索。评估过但未采用:行号溯源、三层权限矩阵、矛盾页工作流等核心需求它不建模,仍需自建领域语义层。
- [MCP Transports 规范](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports)— stdio 是本地子进程传输,无云无服务器,客户端应首选。

**写入逻辑**:
- [Mem0](https://github.com/mem0ai/mem0)(50k 星)— ADD/UPDATE/DELETE/NOOP 四态决策引擎。

**检索**:
- [wangfenjin/simple](https://github.com/wangfenjin/simple)(1k 星)— CJK FTS5 tokenizer(Rust)。
- [pvliesdonk/markdown-vault-mcp](https://github.com/pvliesdonk/markdown-vault-mcp)— FTS5 Markdown schema 参考。
- [sqlite-vec](https://github.com/asg017/sqlite-vec)(4k 星)— 未来向量检索升级路径。

**架构哲学背书**:
- [Letta: Is a Filesystem All You Need?](https://www.letta.com/blog/benchmarking-ai-agent-memory/) — benchmark 证明"文件方案"打赢多数向量/图谱系统,背书 Markdown 权威选择。
- Claude Code auto-memory(Markdown 权威 + grep 派生检索,我们的超集)。
- Cline Memory Bank(Markdown 权威 + AI 读写同哲学)。

**MiMo-Code 原生 tool 机制**:
- `packages/opencode/src/tool/registry.ts:180-193` — 扫描配置目录 `tool/*.ts` 自动注册为工具,与 builtin 平起平坐。
