---
name: Claude Code 工具
description: Superpowers/SafetyNet/Statusline/子代理技能注入 — 遇到Claude Code插件、安全网、状态栏、子代理技能问题时参考
type: concept
updated: 2026-05-14
tags: [claude-code, plugins, superpowers, safety-net, subagent, skill]
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# Claude Code 工具

> 前篇：[[wiki/claude-code-p1.md|Claude Code 配置]]
> 续篇：[[wiki/claude-code-p3.md|Claude Code 进阶工作流]]

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
- 拦截：git checkout -- / git reset --hard / git push --force / rm -rf（cwd 外）/ git branch -D / find -delete
- 放行：git checkout -b / git branch -d / git push --force-with-lease / rm -rf ./...
- 绕过方式：Safety Net 阻止批量删除时，先预览目标列表（如 `find ... -print`），再逐个删除（如 `find ... -exec rm {} \;` 或循环）
- 高级：Shell 包装检测、解释器单行检测、审计日志、自定义规则（`~/.cc-safety-net/config.json`）
- 安装：`/plugin install safety-net@cc-marketplace` → 配置 PreToolUse Hook
- Hook 参数：v0.9.0+ 用 `--claude-code`（从 stdin 读 JSON），旧参数 `--hook "$TOOL_INPUT"` 已废弃，会报 `Unknown option: --hook` ← [[sources/2026-05-13-safety-net-hook-fix.md]] L32-46

## StatusLine 关键变量

- stdin JSON 字段：`model.display_name` / `workspace.current_dir` / `context_window.used_percentage`
- 配置：settings.json 中 `statusLine.type` = command 或 template
- 推荐 Python 版本（比 Bash 版本更稳定）；脚本放 `~/.claude/scripts/statusline.py`
- 多状态栏合并：用管道 `python statusline.py | npx cc-safety-net --statusline`

### Worktree 兼容

- **分支检测 bug：** `get_git_branch()` 用 `os.path.isdir(".git")` 查找仓库，但 worktree 中 `.git` 是文件（指向主仓库），导致跳过 worktree 向上找到主仓库，返回错误分支。修复：改为 `isdir(...) or isfile(...)`
- **目录显示：** worktree 路径 `<project>/.claude/worktrees/<name>/` 过长且无意义，检测后压缩为 `项目名:worktree名` 格式（如 `project:persona-style`）
- **普通目录：** 只保留最后 2 层（如 `~/a/b/c` → `b/c`），不足 2 层保持原样

## GitHub MCP 插件认证

**问题：** `github@claude-plugins-official` 启动报错 `Incompatible auth server: does not support dynamic client registration`
**原因：** 插件 `.mcp.json` 用 `${GITHUB_PERSONAL_ACCESS_TOKEN}` header 认证，变量缺失时回退到 OAuth 动态注册，而 Copilot 端点不支持
**解决：** 在 `~/.claude/settings.json` 的 `env` 中添加 `"GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxx"`（值取自项目 `docker/.env`），`/reload-plugins` 后生效

## 首次安装配置清单

首次安装 Claude Code 后必做配置：官方插件市场（GitHub/context7/skill-creator/learning-output-style/superpowers/frontend-design）→ Safety Net 插件（cc-marketplace）→ StatusLine（带安全网管道）→ 智谱 MCP（4 个服务器）。多数插件安装需用户在 `/plugin` 交互操作，Hook 和 settings.json 可自动配置。完整步骤见 [[sources/claude-code-first-time-setup.md]]

← [[sources/claude-code-first-time-setup.md]]

## 安装教训

- Safety Net：Windows 需手动配 Hook 和 `SAFETY_NET_PLUGIN_ROOT` 环境变量
- 任何插件安装前备份：`cp -r ~/.claude/rules ~/.claude/rules.backup`
- Rules 和 Hooks 都是追加合并，绝不能直接覆盖

## 子代理 Skill 工具可用性

**结论：** 子代理和团队成员都不能使用 Skill 工具（不在直接工具列表，也不在 ToolSearch 延迟列表）。父代理必须预加载技能内容注入 prompt。 ← [[sources/subagent-skill-test-2026-05-10.md]]

**禁止：** 在子代理 prompt 中要求其调用 Skill 工具
**原因：** Skill 是主会话运行时内置工具，不传递给子代理上下文。ToolSearch("Skill") 返回假阳性空 schema
**替代：** 父代理先用 Skill 加载技能完整内容，用 `---<SKILL_NAME> START/END---` 标记包裹注入子代理 prompt

**注入方式：**
- 子代理（`--print`）：技能内容嵌入 `-p` 参数
- 团队成员（`--agents`）：技能内容嵌入 `prompt` 字段 ← [[sources/subagent-skill-test-2026-05-10.md]]

**失败的方案（勿再尝试）：**
- ToolSearch 加载 Skill → 假阳性
- `--allowedTools Skill` → 参数被忽略
- `--plugin-dir superpowers/` → 插件提供内容不提供工具

## ECC 补强

ECC（Everything Claude Code）是独立插件，提供基础设施层（编码规范、上下文管理、安全审查、持续学习），补充 Superpowers 不覆盖的领域。详见 [[wiki/everything-claude-code.md|ECC 插件]]。
