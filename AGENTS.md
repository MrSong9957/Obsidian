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
| 编辑 | ✅ | ✅ | 仅限 Schema/模板维护任务 |
| 删除 | ❌ | 仅标记 deprecated | ❌ |

`templates/` 的编辑仅在 Schema / 模板维护任务（用户明确授权的治理任务）中允许，普通知识任务禁止；创建与删除始终禁止。

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

命名约定：
- 为 Wiki 整理的配套详细文档：以入口文档名为前缀，加 `-references` 后缀（如 `ecc.md` → `ecc-references.md`）
- 原始资料：保留原始语义名称，不强制改造为 `-references` 命名

## 创建新页面规则

1. Wiki 页面使用 `templates/wiki.md` 模板（通过 `type` 字段区分概念页/矛盾页）
2. 详细文档使用 `templates/sources-reference.md` 模板（遵循 `-references` 命名约定）
3. 原始资料按 Sources 分类规则直接放入对应 `sources/<子目录>/`；若为 md 文件，在开头添加 frontmatter 元数据
4. 每个页面的 frontmatter 必须完整填写
5. 每个 `[[wikilink]]` 必须指向真实存在的页面。目标不存在时，先判断该内容是否具有独立、长期价值：满足建页条件才先创建再链接，否则不添加该 wikilink；禁止为满足链接完整性创建空壳或无意义页面
6. `source` 字段必须真实溯源：从 `sources/` 资料提炼 → `[[sources/文件名]]`；有库外原始资料 → URL；无独立原始资料（自身整理、用户原创沉淀、多上下文综合且无法对应唯一 Source）→ `self`，含义是"该 Wiki 没有需要存入 `sources/` 的独立原始资料"。存在真实 Source 时必须优先链接，`self` 不是绕过溯源的兜底；禁止为满足字段要求捏造 Source、创建空壳 Source、把 Wiki 正文复制到 `sources/` 或捏造不存在的 wikilink

## 通用写入流程（写前 search，写后 sync）

对 wiki/ 与 sources/ 的创建和编辑均适用，不限于记忆提炼：

1. 写入前调 `wiki_search` 检查已有知识与重复内容
2. 按结果判断：`ADD`（新建）/ `UPDATE`（并入已有页）/ `NOOP`（已有内容已覆盖，不写）/ `CONTRADICT`（与已有知识矛盾，走矛盾页或向用户提出）
3. 优先 UPDATE / NOOP，避免重复造页
4. 写入完成后调 `wiki_sync` 同步索引
5. 检查同步返回的查重报告，有意外重复按报告处理

## 质量规则

- 每条 Wiki 必须填写 `source` 字段；正文引用与脚注 `[^n]` 要求按 `source` 类型执行
- 脚注格式根据 `source` 类型区分：
  - `source: "[[sources/...]]"`（有独立 Sources 文件）：脚注引用原文，格式为 `[[文件名]] L起始行-结束行 — 简要说明`
  - `source: URL`（库外原始资料）：脚注引用 URL，无需行号
  - `source: self`（无独立原始资料）：不强制虚构行号脚注；如需标注，可引用正文自身说明
- 一条 Wiki 来自多个真实 Sources 时：主 Source 写入 `source` 字段，其余真实来源以正文脚注精确记录；禁止把明确来自多个真实 Sources 的内容归为 `source: self`
- 禁止捏造不存在的 `[[wikilink]]`

## 行号索引规则

行号索引是 Wiki 层的核心溯源机制，确保 AI 和人都能直达原文。作用范围：`source: "[[sources/...]]"` 的 Wiki——来自 Sources 的知识要点必须可精确溯源到原文行号；`source: URL` 按 URL 引用规则（无需行号），`source: self` 不要求虚构行号：

- **格式**：`L{起始行}-{结束行}`，例如 `L42-58`
- **每个来自 Sources 的要点都要标注**：正文中用 `[^1]`、`[^2]` 等脚注标记，底部"引用索引"区块中定义
- **行号是原文的实际行号**：LLM 在读取 sources/ 文件时记录关键信息所在行，生成 Wiki 页时写入
- **行号会随原文变化失效**：如果原始资料被替换（新版本），相关 Wiki 页的行号需要重新标注

## 格式规则

- 使用 Obsidian 原生语法：wikilinks、frontmatter、callout、tags、脚注
- 不使用 HTML 标签
- 不使用自定义语法
- Markdown 格式遵循 CommonMark 规范

---

# 本地 Wiki 检索工具说明

仓库内置了一个高性能的 Rust 查询工具,源码与产物路径为 `./mcp/wiki-query/`(可执行文件 `./mcp/wiki-query/wiki-query.exe`)。该二进制为本地构建产物,Git 不携带;新 clone 需先在仓库根执行 `bash mcp/wiki-query/build.sh` 构建,MCP 由 `.mcp.json` 通过 `mcp/wiki-query/serve.sh` 启动器拉起(见 `mcp/wiki-query/README.md`)。

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

AI agent 通常自带项目记忆(如 Claude Code 的 `CLAUDE.md` / `.claude/`、Cursor 的 `.cursor/rules/`、ZCode 的 `.zcode/`),用于存放瞬时偏好、会话进度、临时笔记。`wiki/` 与 `sources/` 构成 Obsidian 永久知识库;wiki-query 是其索引与检索服务。两者**各司其职**,不重叠。

### 分工表

| 内容类型 | 放哪里 | 性质 | 溯源 |
|---------|--------|------|------|
| 瞬时偏好(语言、风格、这次任务范围) | agent 自带记忆 | 临时,会话级 | 无 |
| 会话进度(做到哪一步、待办) | agent 自带记忆 | 临时,会话级 | 无 |
| 当前项目约定(本仓库的协作规则) | agent 自带记忆 | 半永久,随项目 | 无 |
| 长期技术知识(配置方法、原理、决策依据) | `wiki/` | 永久 | ✅ 按 Wiki `source` 规则溯源（`[[sources/...]]` / URL / `self`） |
| 详细原始资料(完整过程、代码、踩坑) | `sources/` | 永久 | 自身即源头 |
| 需跨会话/跨 agent 共享的决策 | `wiki/` | 永久 | ✅ |

### 判断原则

1. **这次会话结束后还有用吗?** 没用 → agent 自带记忆;有用 → 永久知识库(`wiki/` / `sources/`)。
2. **需要溯源到某份资料吗?** 需要 → 按 Wiki `source` 规则进入 `wiki/` 或 `sources/`;不需要 → 视情况。
3. **换一个 agent / 新开会话还要用到吗?** 是 → 永久知识库(`wiki/` / `sources/`);否 → agent 自带记忆。

### 记忆提炼(瞬时 → 永久)

当 agent 自带记忆中沉淀出**长期有价值**的内容(例如某次会话敲定的技术决策、踩过的坑、摸索出的配置方法),用户可主动触发"提炼":

> "把这次会话的关键决策记到知识库"

v5 提炼工作流(6 步,详见 `templates/project-rules.md`):
1. agent 读自己的记忆系统(文件工具)
2. 逐篇查重(固定流程,不可省):先调 `wiki_check_scope` 判断路径是否在 wiki 范围内(sources/wiki/templates,在范围内说明已是 wiki 内容,跳过),不在范围内再调 `wiki_search` 查相似
3. 用文件工具写 vault Markdown,按 Wiki `source` 规则填写 frontmatter:
   - 有真实 Sources 文件 → `source: "[[sources/...]]"`
   - 有外部资料 → `source: URL`
   - 纯会话决策 / 自身整理且无独立原始资料 → `source: self`
   - **禁止**让 `source` 指向会被删除的临时 Agent memory 文件
4. 调 `wiki_sync` 同步索引 + 拿查重报告
5. 清理已提炼的源内容(避免重复提炼;数据安全优先,仅在提炼并成功 `wiki_sync` 后执行):
   - 只清理本次已成功沉淀的临时记忆条目
   - 不得因提炼其中一部分内容而删除整个共享记忆文件;文件仍含未提炼内容时,只移除对应条目
   - 不得删除 CLAUDE.md / AGENTS.md 等项目规则文件
   - 记忆系统不支持安全的条目级删除时,保留原文件,不以去重为理由破坏其他记忆
   - 仅明确属于一次性、独立、已完整提炼的临时记忆文件,才可按其系统规则整体删除
6. 告知用户

**读写 vault 正文是 agent 用自己的文件工具干的事,wiki-query 不碰正文。** wiki-query 只负责索引 + 查重报告兜底。首次接入时调 `wiki_get_project_rules` 把这套规则注入到当前项目根的 CLAUDE.md/AGENTS.md。

**提炼是手动触发,不自动**——避免把噪声(临时偏好、未定型的想法)灌进知识库。
