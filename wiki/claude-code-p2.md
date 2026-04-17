---
name: Claude Code 工具
description: ECC技能集/Superpowers/SafetyNet/Statusline — 遇到Claude Code插件、安全网、状态栏问题时参考
type: concept
updated: 2026-04-17
tags: [claude-code, plugins, ecc, superpowers, safety-net]
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# Claude Code 工具

> 前篇：[[wiki/claude-code-p1.md|Claude Code 配置]]

## ECC 技能集概要

- 112 个 Skills（110 ECC 核心 + 2 官方插件）+ 25 个智能体 + 57 条指令
- 分类：流程工作流、代码开发、测试、安全、数据库、框架、前端、Agent 工程、内容营销、iOS/Swift、媒体处理、API/MCP、供应链物流
- 安装：`/plugin marketplace add affaan-m/everything-claude-code` → `/plugin install` → 手动合并 rules 和 hooks
- 教训：rules 不能通过插件分发需手动安装；hooks 必须追加不能覆盖；Windows 需配 `CLAUDE_PLUGIN_ROOT` 环境变量

## Superpowers 核心技能

- using-superpowers — 所有任务起点，1% 可能适用就必须调用
- writing-plans / executing-plans — 编写和执行分步实现计划
- systematic-debugging — 系统化定位和修复 bug
- test-driven-development — TDD 流程（红→绿→重构）
- brainstorming — 需求分析和方案设计，优先于实现
- subagent-driven-development — 分任务调度子代理
- dispatching-parallel-agents — 并行代理调度
- requesting-code-review / receiving-code-review — 代码审查
- verification-before-completion — 完成前验证
- finishing-a-development-branch — 分支完成处理
- using-git-worktrees — Worktree 隔离开发
- writing-skills — 创建新技能
- ui-ux-pro-max — UI/UX 设计

## Safety Net 核心能力

- Hook 机制在命令执行前拦截，CLAUDE.md 的"软"规则无法替代"硬"技术约束
- 拦截：git checkout -- / git reset --hard / git push --force / rm -rf（cwd 外）/ git branch -D
- 放行：git checkout -b / git branch -d / git push --force-with-lease / rm -rf ./...
- 高级：Shell 包装检测、解释器单行检测、审计日志、自定义规则（`~/.cc-safety-net/config.json`）
- 安装：`/plugin install safety-net@cc-marketplace` → 配置 PreToolUse Hook

## StatusLine 关键变量

- stdin JSON 字段：`model.display_name` / `workspace.current_dir` / `context_window.used_percentage`
- 配置：settings.json 中 `statusLine.type` = command 或 template
- 推荐 Python 版本（比 Bash 版本更稳定）；脚本放 `~/.claude/scripts/statusline.py`
- 多状态栏合并：用管道 `python statusline.py | npx cc-safety-net --statusline`

## 安装教训

- ECC：用 `"source": "directory"` 代替 `"source": "github"` 避免 GitHub 访问失败
- Safety Net：Windows 需手动配 Hook 和 `SAFETY_NET_PLUGIN_ROOT` 环境变量
- 任何插件安装前备份：`cp -r ~/.claude/rules ~/.claude/rules.backup`
- Rules 和 Hooks 都是追加合并，绝不能直接覆盖

> 详细内容见 sources/claude-code-p2.md
