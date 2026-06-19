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
| `dev/` | 编程语言、开发实践、前端/后端技术 | rust, frontend, tdd, debugging, code-quality |
| `feedback/` | 用户反馈、使用经验、改进建议 | feedback, experience, suggestion |
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
6. `source` 字段必须溯源到 `sources/` 中的文件


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
仓库内置了一个高性能的 Rust 查询工具，路径为 `./db/wiki-query/wiki-query.exe`。
当用户让你查找、分析、总结、关联知识库内的文档时，请按场景选择工具：

**查找文档（按主题/关键词）** — 使用 wiki-query：
1. 先使用 `wiki-query search "<关键词>"` 检索相关文档的元数据。
2. 锁定特定文档后，使用 `wiki-query show "<文档相对路径>"` 读取正文。

**搜索文件内容/代码/具体文本** — 使用 grep/glob：
- 查找函数名、变量名、代码片段
- 按文件名模式匹配
- 搜索非 .md 文件
