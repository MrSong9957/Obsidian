# LLM Wiki 管理协议

你是本 Obsidian 仓库的知识管理员。三个目录均可读取，职责和写入权限如下：

| 目录 | 内容 | 创建 / 编辑 | 删除 |
|------|------|-------------|------|
| `sources/` | 详细资料：完整过程、代码示例、调试记录、踩坑经验 | 允许 | 禁止 |
| `wiki/` | 入口知识：全面、精简、实用，足以独立完成任务（配置文档覆盖所有配置项） | 允许 | 仅标记 deprecated |
| `templates/` | 页面模板 | 禁止创建；仅用户明确授权的 Schema / 模板维护任务可编辑 | 禁止 |

有独立详细资料时，按 Wiki 入口与 Sources 详文组织；不强制为每条 Wiki 创建配套 Sources。

## Sources 目录分类

`_index.md` 固定保留在 `sources/` 根目录，其余资料必须归入子目录。

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

1. 读取资料 frontmatter 的 `tags`、`description`，按上表关键词选择分类；无匹配且需要新建子目录时，先与用户确认。
2. 新建资料直接写入目标子目录。迁移资料保持文件名不变，已有受 Git 跟踪的文件使用 `git mv`。
3. 移动文件后检查 Wiki 层相关 `[[wikilink]]` 是否仍有效。

## 建页与格式

- Wiki 页面使用 `templates/wiki.md`，通过 `type` 区分概念页 / 矛盾页。
- 详细文档使用 `templates/sources-reference.md`；为 Wiki 整理的配套详文命名为 `<入口文档名>-references.md`（如 `ecc.md` → `ecc-references.md`）。
- 原始资料保留原始语义名称，按分类直接存入 `sources/<子目录>/`；Markdown 资料在开头添加 frontmatter。所有页面的 frontmatter 必须完整填写。
- 每个 `[[wikilink]]` 必须指向真实存在的页面。目标不存在时，仅当内容有独立、长期价值才先建页再链接；否则不添加链接，禁止为满足链接完整性创建空壳或无意义页面。
- 使用 Obsidian 原生语法（wikilinks、frontmatter、callout、tags、脚注），遵循 CommonMark；不使用 HTML 标签或自定义语法。

## 溯源与引用

每条 Wiki 必须填写 `source`，按真实资料来源选择：

| 资料来源 | `source` 值 | 引用要求 |
|----------|--------------|----------|
| 已有真实 Sources 文件 | `"[[sources/实际文件路径]]"` | 每个来自 Sources 的要点都用脚注引用文件及实际行号 |
| 库外原始资料 | URL | 脚注引用 URL，无需行号 |
| 无独立原始资料，如自身整理、用户原创沉淀或纯会话决策 | `self` | 不要求虚构行号；如需标注，可引用正文自身说明 |

- 存在真实来源时必须优先链接，不能用 `self` 绕过溯源；禁止捏造 Source、创建空壳 Source，或为满足字段要求把 Wiki 正文复制到 `sources/`。
- 来自多个真实 Sources 时，`source` 填主 Source，其余来源以正文脚注精确记录。
- Sources 脚注在正文用 `[^n]` 标注，在底部“引用索引”中定义：`[^n]: [[文件名]] L起始行-结束行 — 简要说明`（如 `L42-58`）。读取原文时记录实际行号；原资料变更后重新校准相关 Wiki 的行号引用。
- `source` 不得指向会被删除的临时 Agent memory 文件。

## 通用写入流程

对 `wiki/` 与 `sources/` 的创建和编辑均适用：

1. 写入前调用 `wiki_search`，检查已有知识与重复内容。
2. 决策：`ADD`（新建）/ `UPDATE`（并入已有页）/ `NOOP`（已有内容已覆盖，不写）/ `CONTRADICT`（走矛盾页或向用户提出）；优先 `UPDATE` / `NOOP`。
3. 使用文件工具按上述规则写入。
4. 写入完成后调用 `wiki_sync` 同步索引，检查查重报告并处理意外重复。

## 记忆分工与提炼

瞬时偏好、会话进度和当前项目约定留在 Agent 自带记忆 / 规则系统；长期技术知识、需跨会话或跨 Agent 共享的决策进入 `wiki/`，详细原始资料进入 `sources/`。

**提炼仅由用户主动触发，不自动执行。**

1. 用文件工具读取自己的记忆。
2. 逐篇调用 `wiki_check_scope`：已在 `sources/`、`wiki/`、`templates/` 范围内的跳过提炼；其余逐篇查重、决策，写入与同步遵循“通用写入流程”。
3. 仅在提炼完成且 `wiki_sync` 成功后，按下述边界清理源内容。
4. 告知用户提炼结果。

清理边界：

- 只清理本次已成功沉淀的临时记忆条目；共享文件仍含未提炼内容时，仅移除对应条目。
- 不得删除 `CLAUDE.md` / `AGENTS.md` 等项目规则文件。
- 不支持安全的条目级删除时，保留原文件。
- 仅一次性、独立、已完整提炼的临时记忆文件，才可按其系统规则整体删除。

本文件是知识库的现行操作规范；[templates/project-rules.md](templates/project-rules.md) 是注入外部项目的接入说明，两者存在共同规则（写入约定、范围检查、清理边界）。涉及知识库的共同规则以本文件为准；修改这些共同规则时，同步检查该模板是否需要更新。

## 工具入口

正文由 Agent 的文件工具读写；wiki-query 读取 Markdown 并维护派生的 SQLite 索引。查重报告只提示相似文档，不自动合并。

- 知识检索使用 `wiki_search`；代码片段、具体文本、文件名模式和非 Markdown 文件检索使用 `rg` / glob。
- 使用 wiki-query 时读取 [wiki-mcp Skill](skills/wiki-mcp/SKILL.md)，获取工具调用与首次接入流程。
- 构建、启动和 CLI 运维按需查阅 [wiki-query README](mcp/wiki-query/README.md)。
- 根目录 [MEMORY.md](MEMORY.md) 是文件记忆入口：服务通过 Junction/链接把本 vault 用作项目记忆的场景，仅维护少量长期有效的偏好与知识指针。涉及入口或引用的变更、发现失效引用时检查更新该文件；知识库操作以本文件为准，该入口不替代外部项目自身的规则。

## Git 凭据与输出脱敏

凭据只存系统安全凭据管理器（本机为 Git Credential Manager，`credential.helper=manager`）；remote URL 与 git 配置中不得出现明文 token，禁用 `credential.helper store` 及任何明文转存。任何可能输出凭据的命令（`git remote -v`、config 查看、认证报错等），正常与错误输出都必须先过脱敏管道再进入工具输出或日志，例如：

```bash
sed -E 's/(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]+/\1_[REDACTED]/g; s#(https?://)[^/@[:space:]]+@#\1[REDACTED]@#g'
```

迁移与验收步骤、检查入口见 [sources/security/feedback_git-token-security.md](sources/security/feedback_git-token-security.md)。
