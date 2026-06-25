---
name: wiki-mcp
description: |
  使用 WIKI MCP 工具搜索、同步和管理 Obsidian 知识库。当用户要求搜索知识、
  查找文档、同步索引、去重检查、提炼记忆到 wiki、或接入 wiki-query 时，使用此技能。
  触发短语：搜 wiki、查知识库、wiki search、同步索引、去重、提炼记忆、接入 wiki-query。
depends: []
---

# Wiki MCP — Obsidian 知识库 MCP 工具编排技能

你通过 WIKI MCP 的 5 个工具与 Obsidian 知识库交互。核心原则：**Markdown 是权威源，SQLite 是派生索引**（可随时 `rm + sync` 重建）。wiki-query 只管索引，你用自己的文件工具读写 vault 正文。

## 0. 工具一览

| 工具 | 一句话 | 典型场景 |
|------|--------|---------|
| `wiki_search` | FTS5 全文检索，返回 top10 元数据 | 找知识、写入前查重 |
| `wiki_list` | 按 tag/type/dir 结构化浏览 | 列出某类文档、浏览目录 |
| `wiki_sync` | 同步索引 + 查重报告 | 写入后同步、去重 |
| `wiki_get_project_rules` | 获取注入到项目根的规则文本 | 首次接入 wiki-query |
| `wiki_check_scope` | 判断文件是否在 wiki 范围内 | 提炼前检查 |

## 1. Vault 结构速查

```
vault/
├── sources/    # 详细文档（完整过程、调试记录，可读写，禁止删除）
├── wiki/       # 入口文档（精简全面，LLM 可读写）
└── templates/  # 页面模板（只读）
```

**写入目标判断：**

```
新内容 X
  ├─ 精简全面，足以完成任务？→ wiki/
  ├─ 详细过程/调试记录？    → sources/（-references.md）
  ├─ 既有入口又有详细？     → 分离：入口 → wiki/，详细 → sources/
  └─ 不确定？               → 保守处理：详细放 sources/
```

**禁止：** 将冗长的详细过程直接写入 wiki/（wiki 应精简全面）。

## 2. 工具使用指南

### 2.1 wiki_search

**参数：**
- `query`（string）：搜索关键词，支持中文，FTS5 全文检索
- 多词默认 AND（全部命中），需任一命中用大写 `OR` 分隔（如 `mimo OR provider`）

**返回：** 前 10 条匹配文档的元数据列表（路径/标题/描述/类型/标签）。**不含正文**，需另用文件工具读全文。

**使用场景：**
- 用户问知识问题 → `wiki_search` 找相关文档 → 文件工具读全文 → 综合回答
- 写入前查重 → `wiki_search` 检查是否已有相似内容
- 探索知识库 → 用关键词发现相关文档

### 2.2 wiki_list

**参数（均可选）：**
- `tag`（string）：按标签过滤（子串匹配），如 `agent`
- `type`（string）：按 type 过滤（精确匹配），如 `concept`
- `dir`（string）：按目录前缀过滤，如 `wiki/` 或 `sources/dev/`
- `limit`（i64）：最大返回条数，默认 100

**返回：** 元数据列表，按 `updated` 降序排列。

**与 wiki_search 的区别：** search 是全文语义检索，list 是精确字段过滤。

**使用场景：**
- "列出所有 agent 相关文档" → `wiki_list(tag="agent")`
- "sources/dev/ 下有什么" → `wiki_list(dir="sources/dev/")`
- "所有 concept 类型" → `wiki_list(type="concept")`

### 2.3 wiki_sync

**参数：**
- `scope`（string）：`incremental`（默认，只查本次新入库文档）或 `full`（全库两两查重）

**返回：** 同步报告（同步数、更新数、裁剪数）+ 查重报告（相似文档对）。

**关键：** 查重报告只提示，不自动合并。合并决策归你。

**使用场景：**
- 写入新文档后 → `wiki_sync()` 让索引跟上 + 检查重复
- 用户说"去重一下" → `wiki_sync(scope="full")` 全库查重
- 定期刷新索引 → `wiki_sync()`

### 2.4 wiki_get_project_rules

**参数：**
- `agentType`（string）：`claude`（注入到 CLAUDE.md）或其他（注入到 AGENTS.md）

**返回：** 规则文本，包裹在 `<!-- wiki-query-rules -->` 标记中。

**使用流程：**
1. 调用工具获取规则文本
2. 读取目标文件（CLAUDE.md 或 AGENTS.md）
3. 检查是否已含 `<!-- wiki-query-rules -->` 标记
4. 未含 → 追加到文件末尾；已含 → 跳过

**使用场景：** 首次接入 wiki-query、用户说"连接 wiki-query"。

### 2.5 wiki_check_scope

**参数：**
- `path`（string）：文件路径（相对或绝对）
- `vault_root`（string，可选）：vault 根路径，传入后自动去除前缀

**返回：**
- `in_scope`（bool）：是否在 wiki 范围内
- `scope_dir`（string|null）：所在目录（sources/wiki/templates）
- `suggestion`（string）：建议文本

**使用场景：** 提炼记忆前检查文件是否已在 wiki 范围内（在范围内则跳过）。

## 3. 常见工作流

### 3.1 搜索并阅读

```
用户问知识问题
  → wiki_search(query) → 获取元数据列表
  → 选择相关结果 → 文件工具读全文
  → 综合回答用户
```

### 3.2 写入并索引

```
用户说"记住这个" / "保存到知识库"
  → 1. 判断目标：wiki/ vs sources/（见 §1 判断流程）
  → 2. wiki_search(keywords) → 查重检查
  → 3. 文件工具写文档（带完整 frontmatter）
  → 4. wiki_sync() → 更新索引 + 获取查重报告
  → 5. 处理查重报告中的相似对
  → 6. 判断是否需要更新 MEMORY.md（见 §6）
```

### 3.3 提炼记忆到 Wiki

这是最复杂的工作流，将 agent 自带记忆（.claude/、.cursor/ 等）提炼到永久知识库。

```
用户说"提炼记忆" / "refine to wiki"
  → 1. 文件工具读取自己的记忆文件
  → 2. 对每篇记忆文件：
     a. wiki_check_scope(path) → 已在范围内则跳过
     b. wiki_search(keywords) → 查重
     c. 决策：合并到已有文档 / 新建文档
     d. 文件工具写入 vault（frontmatter 带 source 字段溯源）
  → 3. wiki_sync() → 更新索引 + 验证无意外重复
  → 4. 删除源记忆文件（避免下次重复提炼）
  → 5. 报告用户："已提炼 N 篇，删源 M 个"
```

**source 字段格式：** `source: [[.claude/memory/xxx.md]]`（溯源到记忆出处）

### 3.4 去重

```
用户说"去重一下" / "检查重复"
  → wiki_sync(scope="full") → 获取全库查重报告
  → 审查相似对
  → 对每对：读两篇文档 → 决定合并/跳过
  → 如合并：文件工具编辑 → wiki_sync() 再次验证
```

### 3.5 结构化浏览

```
用户说"列出所有 agent 相关文档" / "sources/dev/ 下有什么"
  → wiki_list(tag="agent") 或 wiki_list(dir="sources/dev/")
  → 展示结果给用户
```

### 3.6 首次接入 Wiki-Query

```
用户说"连接 wiki-query" / "接入知识库"
  → wiki_get_project_rules(agentType="claude")
  → 读取项目 CLAUDE.md → 检查 <!-- wiki-query-rules --> 标记
  → 无标记 → 追加规则文本到文件末尾
  → 有标记 → 跳过，告知用户已接入
```

## 4. Frontmatter 规范

每个写入 vault 的文档必须有 YAML frontmatter：

```yaml
---
name: 页面名称（简短，用于索引查找）
description: 一句话描述 — 覆盖哪些问题/场景
type: concept | reference | summary | feedback | guide | debugging-story
updated: YYYY-MM-DD
tags: [标签1, 标签2]
source: [[sources/xxx.md]]   # 可选，溯源到来源资料
---
```

**关键规则：**
- `name` + `description`：必有，影响搜索展示
- `tags`：分类核心字段，决定同类摘要是否归入此文件
- `type`：建议填写，不强制白名单
- `source`：提炼时溯源用，指向来源文档
- 文件名使用 kebab-case（如 `dev-practice.md`）
- 双链统一带 `.md` 扩展名：`[[wiki/xxx.md|显示名]]`
- 每个页面必须有 H1 标题（`# 标题`）

## 5. 工具选择决策树

```
用户意图
  ├─ 查找 / 搜索 / "有没有..." → wiki_search
  ├─ 列出 / 浏览 / "有什么..." → wiki_list
  ├─ 写入新文档 / 更新文档 → 文件工具 → wiki_sync
  ├─ 提炼记忆 → wiki_check_scope → wiki_search → 文件工具 → wiki_sync
  ├─ 去重 → wiki_sync(scope="full")
  ├─ 接入 wiki-query → wiki_get_project_rules
  └─ 检查文件是否在 wiki → wiki_check_scope
```

## 6. MEMORY.md 管理

### 条目格式

```
- {场景}：{一句话摘要} → [[wiki/xxx.md|详情]]
```

### 入门门槛

不是每条知识都需要 MEMORY.md 入口。满足以下条件之一才添加：

1. **跨会话适用** — 在大多数会话中都适用（用户偏好、习惯、方法论）
2. **跨领域影响** — 影响多个工作领域的教训（如工具选择失误）
3. **高频引用** — 被频繁引用的约束或方法

### 容量限制

- 每个分类最多 10 条
- MEMORY.md 总行数不超过 200 行
- 超限时：删除已废弃条目 → 压缩摘要 → 报告用户

### 更新逻辑

```
wiki 文件有新增内容
  → 新增内容满足入门条件？
     ├─ 否 → 不更新 MEMORY.md
     └─ 是 → 已有相同场景条目？
              ├─ 是 → 更新摘要，不新增行
              └─ 否 → 添加新条目
```

## 7. 索引维护

### wiki_sync 自动处理

- 扫描 vault 所有 Markdown 文件
- 更新 SQLite 索引（新增/修改/删除）
- 裁剪孤立记录（文件已删除但索引还在）
- 运行查重检查（BM25 相似度）

### 何时调用

- 写入新文档后（必须）
- 编辑文档后（建议）
- 用户说"同步索引"时
- 定期刷新（可选）

### 索引重建

如果索引损坏，删除 `db/wiki.db` 后运行 `wiki_sync()` 即可重建。

### wiki_sync vs CLI

| 场景 | 使用方式 |
|------|---------|
| Agent 自动化 | `wiki_sync()`（MCP 工具） |
| 手动/Git hook | `wiki-query sync`（CLI 命令） |

## 8. 故障排除

### MCP 服务器无响应

检查 `.mcp.json` 配置：
```json
{
  "mcpServers": {
    "wiki": {
      "command": "<vault 根目录>/db/wiki-query/wiki-query.exe",
      "args": ["serve"],
      "cwd": "<vault 根目录>"
    }
  }
}
```
验证二进制文件路径正确且可执行。

### 搜索返回空结果

索引可能为空，先运行 `wiki_sync()` 建立索引。

### 新写入的文档搜不到

写入后必须调用 `wiki_sync()` 更新索引。流程：文件工具写入 → `wiki_sync()` → 再搜索。

### 查重报告误报

BM25 相似度阈值为 -2.0，部分主题相似但内容不同的文档可能触发。需要人工判断是否合并。

### 提炼循环（重复提炼）

源记忆文件未删除，下次提炼会重新处理。提炼完成后必须删除源文件。

### 中文搜索效果差

FTS5 使用 jieba 分词处理中文。如果效果不佳，尝试用更精准的关键词，或重建索引（删除 `db/wiki.db` 后 `wiki_sync()`）。
