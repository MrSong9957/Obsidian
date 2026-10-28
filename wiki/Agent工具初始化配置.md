---
name: Agent 工具初始化配置
description: 新 Agent 环境五步初始化清单——定位仓库、双层规则、技能、智能体代理与项目 rules 配置，适用于 Codex、Claude Code、ZCode、MiMo Code 等工具
type: concept
updated: 2026-10-27
tags: [agent, setup, agents-md, claude-md, checklist]
source: self
---

# Agent 工具初始化配置

新 Agent 工具环境的初始化清单，共五步：定位仓库、配置双层规则、安装技能、安装智能体代理、配置项目 `rules/`。

### 初始化步骤

1. **定位配置来源**：优先找到本地 Obsidian 仓库，没有则从远程 GitHub 仓库获取。
   - 本地默认路径：宿主机 `D:\AgentLab\shared\Obsidian` / Docker 容器 `/vault`。
   - GitHub 仓库：[MrSong9957/Obsidian](https://github.com/MrSong9957/Obsidian)。
   - 后续“项目根目录”指待初始化的目标项目；已有规则、技能或同名文件时先比较再合并，保留项目已有配置。

2. **配置规则文件**：按 [[wiki/AGENTS-md-base|AGENTS.md 双层架构模板]] 的目录对照表，配置用户全局规则与项目根目录的 `AGENTS.md`。默认全局示例为 Codex 的 `~/.codex/AGENTS.md`；Claude Code、ZCode、MiMo Code 等工具使用各自实际的规则目录，不能把程序安装目录当成规则目录。Claude Code 的 `CLAUDE.md` 用一行 `@AGENTS.md` 导入同目录正文；ZCode、MiMo Code 使用原生规则正文或各自支持的显式加载配置。Claude Code 的导入条件见 [[wiki/agents-md-claude-md|AGENTS.md 与 CLAUDE.md 配置关系]]。

3. **安装技能**：按 [[wiki/ce + last30days 技能组合|ce + last30days 技能组合]] 安装 Compound Engineering 与 last30days；再补齐项目规则明确要求的本仓库技能，例如 `skills/comprehensive-testing/`。安装前确认当前 Agent 支持的安装方式与实际用户级全局技能目录。

4. **安装智能体代理**：读取仓库 `agents/` 中的代理定义，按当前 Agent 支持的用户级代理目录与格式安装。Claude Code 使用 `~/.claude/agents/` 下带 frontmatter 的 Markdown；Codex 使用 `~/.codex/agents/` 下的 TOML 定义，需转换为其支持的字段。其他工具先确认目录和格式；不支持自定义代理时记录跳过及其对项目流程的影响。[^claude-agents][^codex-agents]

5. **配置项目 `rules/`**：将仓库 `rules/` 文件夹复制到目标项目根目录，得到 `rules/规格文档实战法.md`、`rules/REVIEW.md` 等文件。在项目级 `AGENTS.md` 中写明：开发前读取 `rules/规格文档实战法.md`，代码审查前读取 `rules/REVIEW.md` 并按其执行。仅复制文件不足以确认规则已生效；两份“规格文档实战法”的维护约定见 [[wiki/规格文档实战法|规格文档实战法]]。

6. 安装 SKILL 技能：将 `Obsidian` 仓库目录下 `skills/` 子目录中的技能按需安装到 Agent 全局目录，仅安装：
	- comprehensive-testing
	- security-review

**完成后验证**：新建会话，确认当前 Agent 实际加载了用户全局与项目两层规则文件；Claude Code 用 `/context` 查看 Memory files 清单，其他工具按其加载日志或指令来源清单核对。技能按 [[wiki/ce + last30days 技能组合|ce + last30days 技能组合]]「安装后验证」执行，并确认补充技能已被识别。核对代理的实际安装路径、名称和识别结果；检查项目 `rules/` 文件齐全、项目级 `AGENTS.md` 中的路径正确，并让 Agent 按开发或审查场景实际读取对应规则。报告各项通过、失败或跳过及原因；尚有必需项未就绪时，不能报告初始化全部完成。

### 相关概念

- [[wiki/AGENTS-md-base|AGENTS.md 双层架构模板]] — 第 2 步使用的双层规则模板
- [[wiki/ce + last30days 技能组合|ce + last30days 技能组合]] — 第 3 步执行的技能安装指令
- [[wiki/agents-md-claude-md|AGENTS.md 与 CLAUDE.md 配置关系]] — `@AGENTS.md` 导入方案的层级规则与四档配置
- [[wiki/规格文档实战法|规格文档实战法]] — 第 5 步复制的规则正文及同步维护约定

### 引用索引

[^claude-agents]: https://code.claude.com/docs/en/sub-agents — Claude Code 用户级代理目录与 Markdown frontmatter 格式。
[^codex-agents]: https://learn.chatgpt.com/docs/agent-configuration/subagents — Codex 用户级自定义代理目录与 TOML 格式。