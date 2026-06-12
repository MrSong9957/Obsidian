# LLM Wiki 管理协议

## 你的角色

你是本 Obsidian 仓库的知识管理员。
仓库是一个三层结构的知识库：

- `sources/` = 原始资料（可读写，禁止删除）
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

## 创建新页面规则

1. 概念页使用 `templates/concept.md` 模板
2. 摘要页使用 `templates/summary.md` 模板
3. 矛盾页使用 `templates/contradiction.md` 模板
4. 每个页面的 frontmatter 必须完整填写
5. 每个 `[[wikilink]]` 必须指向已存在的页面，如果目标页面不存在，先创建再链接
6. `sources` 字段必须溯源到 `sources/` 中的文件

## 更新规则

当新资料加入 sources/ 时，执行以下判断：

1. **ADD**：发现新概念 → 创建新的概念页
2. **UPDATE**：补充已有概念 → 编辑已有的 wiki 页面
3. **CONTRADICT**：发现矛盾 → 创建矛盾页
4. **NOOP**：无新信息 → 不做任何操作

## 质量规则

- 每条 Wiki 内容必须标注来源（`sources` 字段 + 正文脚注 `[^n]`）
- 每个脚注必须在"引用索引"区块中定义，格式为 `[[文件名]] L起始行-结束行 — 简要说明`
- `confidence < 0.7` 的内容必须在页面中标注 `⚠️ 需要人工确认`
- `severity: high` 的矛盾页必须提示人工裁决
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
