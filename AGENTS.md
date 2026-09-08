# LLM Wiki 管理协议

## 你的角色

你是本 Obsidian 仓库的知识管理员。
仓库是一个三层结构的知识库：

- `sources/` = 详细文档（可读写，禁止删除）
- `wiki/` = 结构化知识（你可以读写）
- `templates/` = 页面模板（创建新页面时遵循）

## 权限矩阵

| 操作 | sources/ | wiki/ | templates/ |
|------|----------|-------|------------|
| 读取 | ✅ | ✅ | ✅ |
| 创建 | ✅ | ✅ | ❌ |
| 编辑 | ✅ | ✅ | ❌ |
| 删除 | ❌ | 仅标记 deprecated | ❌ |

## Sources 目录分类规则

`sources/` 根目录只保留 `_index.md`，所有资料文件必须归入子目录。

### 现有子目录

| 子目录 | 用途 | 关键词 |
|--------|------|--------|
| `agent/` | Agent 架构、多代理系统、ShareAI 系列 | agent, proxy, team, workflow, harness |
| `claude-code/` | Claude Code 配置、CLI、MCP 集成、system-prompts | claude, mcp, plugin, superpowers, hook |
| `DeepSeek-Harness/` | DeepSeek Harness、AI IDE 设计资料 | deepseek, harness, context, ai-ide |
| `dev/` | 编程语言、开发实践、前端/后端技术 | rust, frontend, tdd, debugging, code-quality |
| `learning/` | 学习方法论、思维模型、认知科学 | learning, methodology, cognition, mental-model |
| `projects/` | 项目规划、进度记录、发布方案 | project, roadmap, publish, progress |
| `security/` | 安全实践、密钥管理、漏洞分析 | security, token, credential, vulnerability |
| `tools/` | CLI 工具、编辑器、开发环境配置 | yazi, obsidian, opencode, token-management |

### 分类判断流程

新资料加入 `sources/` 时：

1. 读取文件 frontmatter 的 `tags`、`description` 字段
2. 按上表关键词匹配子目录
3. 若无匹配，评估是否创建新子目录（需与用户确认）
4. 使用 `git mv` 移动，保持文件名不变

### 约束

- 禁止将文件移回 `sources/` 根目录
- 移动文件后检查 wiki 层的 `[[wikilink]]` 是否仍有效（Obsidian 通常自动跟踪重命名）
- `_index.md` 始终留在 `sources/` 根目录

### 文档组织

每个模块两层文档：

- **wiki/** — 入口文档：全面、精简、实用，足以独立完成任务（如配置文档虽精简但覆盖所有配置项）
- **sources/** — 详细文档：完整过程、代码示例、调试记录、踩坑经验等详细内容

命名约定：详细文档以入口文档名为前缀，加 `-references` 后缀（如 `ecc.md` → `ecc-references.md`）。

## 创建新页面规则

1. Wiki 页面使用 `templates/wiki.md` 模板（通过 `type` 字段区分概念页/矛盾页）
2. 详细文档使用 `templates/sources-reference.md` 模板（遵循 `-references` 命名约定）
3. 原始资料直接复制到 `sources/`；若为 md 文件，在开头添加 frontmatter 元数据
4. 每个页面的 frontmatter 必须完整填写
5. 每个 `[[wikilink]]` 必须指向已存在的页面，如果目标页面不存在，先创建再链接
6. `source` 字段必须真实溯源：从 `sources/` 资料提炼 → `[[sources/文件名]]`；有库外原始资料 → URL；无独立原始资料（自身整理、用户原创沉淀、多上下文综合且无法对应唯一 Source）→ `self`，含义是"该 Wiki 没有需要存入 `sources/` 的独立原始资料"。存在真实 Source 时必须优先链接，`self` 不是绕过溯源的兜底；禁止为满足字段要求捏造 Source、创建空壳 Source、把 Wiki 正文复制到 `sources/` 或捏造不存在的 wikilink


## 质量规则

- 每条 Wiki 内容必须标注来源（`sources` 字段 + 正文脚注 `[^n]`）
- 每个脚注必须在"引用索引"区块中定义，格式为 `[[文件名]] L起始行-结束行 — 简要说明`
- 禁止捏造不存在的 `[[wikilink]]`

## 行号索引规则

行号索引是 Wiki 层的核心溯源机制，确保 AI 和人都能直达原文：

- **格式**：`L{起始行}-{结束行}`，例如 `L42-58`
- **每个要点都要标注**：正文中用 `[^1]`、`[^2]` 等脚注标记，底部"引用索引"区块中定义
- **行号是原文的实际行号**：LLM 在读取 sources/ 文件时记录关键信息所在行，生成 Wiki 页时写入
- **行号会随原文变化失效**：如果原始资料被替换（新版本），相关 Wiki 页的行号需要重新标注

## 格式规则

- 使用 Obsidian 原生语法：wikilinks、frontmatter、callout、tags、脚注
- 不使用 HTML 标签
- 不使用自定义语法
- Markdown 格式遵循 CommonMark 规范

---

# 本地 Wiki 检索工具说明

仓库内置了一个高性能的 Rust 查询工具,产物路径为 `./db/wiki-query/wiki-query.exe`。该二进制为本地构建产物,Git 不携带;新 clone 需先在仓库根执行 `bash db/wiki-query/build.sh` 构建,MCP 由 `.mcp.json` 通过 `db/wiki-query/serve.sh` 启动器拉起(见 `db/README.md`)。

**v5 纯索引服务**:agent 用自己的文件工具读写 vault Markdown,wiki-query 只管 SQLite 索引。成品只保留两个 CLI 子命令:
- `wiki-query sync` — 重建/更新 SQLite 索引(运维,git post-commit hook 用)
- `wiki-query serve` — 启动 MCP server(agent 通过 stdio 交互)

**5 个 MCP 工具**(agent 自动调用):
- `wiki_search "<关键词>"` — FTS5 全文检索,返回 top 10 元数据(用于找知识 / 写入前查重)
- `wiki_list` — 按 tag/type/dir 结构化浏览
- `wiki_sync` — 同步索引 + 查重报告(scope=incremental/full);agent 写完文件后调
- `wiki_get_project_rules` — 返回可注入到项目根指令文件的规则文本(首次接入时用)
- `wiki_check_scope` — 判断文件是否在 wiki 范围内(sources/wiki/templates);AI 提炼记忆前调用

**读写 vault 正文** — agent 用自己的文件工具干,wiki-query 不碰正文。

**搜索文件内容/代码/具体文本** — 使用 grep/glob:
- 查找函数名、变量名、代码片段
- 按文件名模式匹配
- 搜索非 .md 文件

---

## 记忆系统分工

AI agent 通常自带项目记忆(如 Claude Code 的 `CLAUDE.md` / `.claude/`、Cursor 的 `.cursor/rules/`、ZCode 的 `.zcode/`),用于存放瞬时偏好、会话进度、临时笔记。wiki-query 则是 Obsidian 知识库的永久记忆。两者**各司其职**,不重叠。

### 分工表

| 内容类型 | 放哪里 | 性质 | 溯源 |
|---------|--------|------|------|
| 瞬时偏好(语言、风格、这次任务范围) | agent 自带记忆 | 临时,会话级 | 无 |
| 会话进度(做到哪一步、待办) | agent 自带记忆 | 临时,会话级 | 无 |
| 当前项目约定(本仓库的协作规则) | agent 自带记忆 | 半永久,随项目 | 无 |
| 长期技术知识(配置方法、原理、决策依据) | wiki-query (`wiki/`) | 永久 | ✅ source 字段溯源到 `sources/` |
| 详细原始资料(完整过程、代码、踩坑) | wiki-query (`sources/`) | 永久 | 自身即源头 |
| 需跨会话/跨 agent 共享的决策 | wiki-query | 永久 | ✅ |

### 判断原则

1. **这次会话结束后还有用吗?** 没用 → agent 自带记忆;有用 → wiki-query。
2. **需要溯源到某份资料吗?** 需要 → wiki-query(带 `source`);不需要 → 视情况。
3. **换一个 agent / 新开会话还要用到吗?** 是 → wiki-query;否 → agent 自带记忆。

### 记忆提炼(瞬时 → 永久)

当 agent 自带记忆中沉淀出**长期有价值**的内容(例如某次会话敲定的技术决策、踩过的坑、摸索出的配置方法),用户可主动触发"提炼":

> "把这次会话的关键决策记到知识库"

v5 提炼工作流(6 步,详见 `templates/project-rules.md`):
1. agent 读自己的记忆系统(文件工具)
2. 逐篇调 `wiki_search` 查重(固定流程,不可省)
3. 用文件工具写 vault Markdown(frontmatter 带 `source` 溯源到记忆出处)
4. 调 `wiki_sync` 同步索引 + 拿查重报告
5. 删除自己记忆系统里的源文件(避免重复提炼)
6. 告知用户

**读写 vault 正文是 agent 用自己的文件工具干的事,wiki-query 不碰正文。** wiki-query 只负责索引 + 查重报告兜底。首次接入时调 `wiki_get_project_rules` 把这套规则注入到当前项目根的 CLAUDE.md/AGENTS.md。

**提炼是手动触发,不自动**——避免把噪声(临时偏好、未定型的想法)灌进知识库。
