---
name: Agent 工具初始化配置
description: 新 Agent 环境两步初始化清单——双层规则配置与 grill-me 技能安装，适用于 Codex、Claude Code、ZCode、MiMo Code 等工具
type: concept
updated: 2026-10-20
tags: [agent, setup, agents-md, claude-md, checklist]
source: self
---

# Agent 工具初始化配置

新 Agent 工具环境的初始化清单，共两步：先配规则，再装技能。

### 初始化步骤

1. 所需内容都在 Obsidian 仓库，优先找到本地仓库，如果没有则使用远程 GitHub 仓库。
	- 本地默认路径：宿主机 `D:\AgentLab\shared\Obsidian` / Docker 容器 `/vault`。
	- GitHub 仓库：https://github.com/MrSong9957/Obsidian

2. **配置规则文件**：按 [[wiki/AGENTS-md-base|AGENTS.md 双层架构模板]] 的目录对照表，配置用户全局规则与项目根目录的 `AGENTS.md`。默认全局示例为 Codex 的 `~/.codex/AGENTS.md`；Claude Code、ZCode、MiMo Code 等工具使用各自实际的规则目录，不能把程序安装目录当成规则目录。Claude Code 的 `CLAUDE.md` 用一行 `@AGENTS.md` 导入同目录正文；ZCode、MiMo Code 使用原生规则正文或各自支持的显式加载配置。Claude Code 的导入条件见 [[wiki/agents-md-claude-md|AGENTS.md 与 CLAUDE.md 配置关系]]。

3. **安装技能**：检查并安装 [[wiki/grill-me最小必要技能安装清单|grill-me 最小必要技能安装清单]]。

4. 安装 Agent 智能体代理：将 Obsidian 仓库根目录 `agents/` 文件夹下的智能体代理安装到 Agent 工具的全局目录。

5. 配置规格文档：将 `规格文档实战法.md` 复制到项目路径 `docs\规格文档实战法.md`。

**完成后验证**：新建会话，确认当前 Agent 实际加载了用户全局与项目两层规则文件；Claude Code 用 `/context` 查看 Memory files 清单，其他工具按其加载日志或指令来源清单核对。技能按 [[wiki/grill-me最小必要技能安装清单|安装清单]]「四、安装后验证」执行。

### 相关概念

- [[wiki/AGENTS-md-base|AGENTS.md 双层架构模板]] — 第 1 步使用的双层规则模板
- [[wiki/grill-me最小必要技能安装清单|grill-me 最小必要技能安装清单]] — 第 2 步执行的技能安装指令
- [[wiki/agents-md-claude-md|AGENTS.md 与 CLAUDE.md 配置关系]] — `@AGENTS.md` 导入方案的层级规则与四档配置
