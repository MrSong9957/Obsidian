---
name: wiki-mcp
description: |
  使用 WIKI MCP 工具搜索、同步和管理 Obsidian 知识库。当用户要求搜索知识、
  查找文档、同步索引、去重检查、提炼记忆到 wiki、或接入 wiki-query 时，使用此技能。
  触发短语：搜 wiki、查知识库、wiki search、同步索引、去重、提炼记忆、接入 wiki-query。
depends: []
---

# Wiki MCP — Obsidian 知识库 MCP 工具编排技能

wiki-query 只管索引，你用自己的文件工具读写 vault 正文。文档组织、frontmatter、溯源、写入和记忆清理规则遵循当前项目的 `AGENTS.md` / `CLAUDE.md`。

## 1. 工具使用指南

### 1.1 wiki_search

**参数：**

- `query`（string，必填）：搜索关键词，支持中文，FTS5 全文检索
- 多词默认 AND（全部命中），需任一命中用大写 `OR` 分隔（如 `mimo OR provider`）

**返回：** 前 10 条匹配文档的元数据列表（路径/名称/描述/类型/更新日期/标签）。**不含正文**，需另用文件工具读全文。

### 1.2 wiki_list

**参数（均可选，多个条件同时满足）：**

- `tag`（string）：按标签过滤（子串匹配），如 `agent`
- `type`（string）：按 type 过滤（精确匹配），如 `concept`
- `dir`（string）：按目录前缀过滤，分隔符须与索引返回的 `path` 一致
- `limit`（i64）：最大返回条数，默认 100

**返回：** 元数据列表（不含正文），按 `updated` 降序排列。不传过滤条件则返回最近更新的文档。

### 1.3 wiki_sync

**参数：**

- `scope`（string，可选）：只控制查重候选范围；`incremental`（默认，本次新增/更新的文档）或 `full`（全库文档）

**返回：** 同步报告（同步数、更新数、裁剪数）+ 查重报告（相似文档对）。

**关键：** 查重报告只提示，不自动合并。合并决策归你。

### 1.4 wiki_get_project_rules

**参数：**

- `agent_type`（string，可选）：`claude` 对应 CLAUDE.md；省略或其他值对应 AGENTS.md

**返回：** 包裹在 `<!-- wiki-query-rules -->` 标记中的规则文本，以及注入指引。工具不直接修改目标文件。

**首次接入时：**

1. 调用工具获取规则文本
2. 读取目标文件（CLAUDE.md 或 AGENTS.md）
3. 检查是否已含 `<!-- wiki-query-rules -->` 标记
4. 未含 → 追加含标记的规则块到文件末尾；已含 → 跳过

### 1.5 wiki_check_scope

**参数：**

- `path`（string，必填）：优先传以 `/` 分隔的 vault 相对路径；传绝对路径时须同时提供 `vault_root`
- `vault_root`（string，可选）：vault 根路径，用于去除匹配的根路径前缀

**返回：**

- `path`（string）：用于判断的路径
- `in_scope`（bool）：是否在 wiki 范围内
- `scope_dir`（string|null）：所在目录（sources/wiki/templates）
- `suggestion`（string）：建议文本

范围检查只判断路径，不验证文件是否存在。提炼流程遵循仓库规则。
