---
name: claude-code
description: Superpowers/SafetyNet/Statusline/子代理创建与配置/子代理技能注入 — 遇到Claude Code插件、安全网、状态栏、子代理创建、子代理技能问题时参考
metadata: 
  node_type: memory
  type: concept
  updated: 2026-05-30
  tags: "claude-code, plugins, superpowers, safety-net, subagent, skill, agent"
  source: https://example.com
  originSessionId: ec32b087-a1b9-49c9-9eb6-7c196d718298
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
- Hook 参数：v0.9.0+ 用 `--claude-code`（从 stdin 读 JSON），旧参数 `--hook "$TOOL_INPUT"` 已废弃，会报 `Unknown option: --hook` ← [[sources/claude-code/2026-05-13-safety-net-hook-fix.md]] L32-46

## StatusLine 关键变量

- stdin JSON 字段：`model.display_name` / `workspace.current_dir` / `context_window.used_percentage`
- 配置：settings.json 中 `statusLine.type` = command 或 template
- 推荐 Python 版本（比 Bash 版本更稳定）；脚本放 `~/.claude/scripts/statusline.py`
- 多状态栏合并：用管道 `py statusline.py | npx cc-safety-net --statusline`（Windows 必须用 `py`，`python` 返回 exit code 49）

### Worktree 兼容

- **分支检测 bug：** `get_git_branch()` 用 `os.path.isdir(".git")` 查找仓库，但 worktree 中 `.git` 是文件（指向主仓库），导致跳过 worktree 向上找到主仓库，返回错误分支。修复：改为 `isdir(...) or isfile(...)`
- **目录显示：** worktree 路径 `<project>/.claude/worktrees/<name>/` 过长且无意义，检测后压缩为 `项目名:worktree名` 格式（如 `project:persona-style`）
- **普通目录：** 只保留最后 2 层（如 `~/a/b/c` → `b/c`），不足 2 层保持原样

## 切换 API KEY 后诊断流程

切换 `ANTHROPIC_AUTH_TOKEN` 后按以下优先级排查，99% 问题出在 MCP token：

### 1. MCP 断连（99% 是 token 问题）

**症状：** `claude mcp list` 显示服务器 Disconnected

**快速修复：**
- 智谱 HTTP MCP（zread/web-reader/web-search-prime）的 `Authorization` header 和 zai-mcp-server 的 `Z_AI_API_KEY` env 均已配置为 `${ANTHROPIC_AUTH_TOKEN}` 环境变量引用，切换 token 后**无需手动更新**
- 如果仍断连，检查环境变量是否生效：`echo $ANTHROPIC_AUTH_TOKEN`，确认值是最新 token
- 只有硬编码 token 的情况才需手动改 `.claude.json`

**验证：** `claude mcp list` 确认全部 Connected

### 2. StatusLine 失效（Windows `py` vs `python`）

**症状：** 底部状态栏不显示或报错 exit code 49

**快速修复：**
- Windows 上 `python` 命令返回 exit code 49（不存在），必须用 `py`
- 确认 settings.json 中 `statusLine.command` 使用 `py statusline.py`
- 快速验证：
  ```bash
  echo '{"model":{"display_name":"test"},"workspace":{"current_dir":"."},"context_window":{"remaining_percentage":80},"session_id":""}' | py ~/.claude/scripts/statusline.py
  ```

### 3. Hooks 报错

**症状：** `claude mcp list` 显示 "hook references ${CLAUDE_PLUGIN_ROOT} but not associated with a plugin"

**诊断与修复：**
1. **原因：** ECC 插件升级后 `plugins/cache/` 路径变化（如 `everything-claude-code/1.8.0` → `ecc/2.0.0-rc.1`），settings.json 中的旧 hooks 失效
2. **确认插件自解析：** ECC v2+ 自带 `hooks/hooks.json`，能自动解析正确路径
3. **清理 settings.json：** 删除所有 `${CLAUDE_PLUGIN_ROOT}` 引用的 ECC hooks，只保留 Safety Net 和 block-no-verify
4. **更新插件路径：** 用 Glob 搜索 `plugins/cache/` 确认 `CLAUDE_PLUGIN_ROOT` 和 `SAFETY_NET_PLUGIN_ROOT` 的实际版本路径

**最终验证：** `claude mcp list` 确认全部 Connected 且无 hook 错误

## GitHub MCP 插件认证

**问题：** `github@claude-plugins-official` 启动报错 `Incompatible auth server: does not support dynamic client registration`
**原因：** 插件 `.mcp.json` 用 `${GITHUB_PERSONAL_ACCESS_TOKEN}` header 认证，变量缺失时回退到 OAuth 动态注册，而 Copilot 端点不支持
**解决：** 在 `~/.claude/settings.json` 的 `env` 中添加 `"GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxx"`（值取自项目 `docker/.env`），`/reload-plugins` 后生效

## 首次安装配置清单

首次安装 Claude Code 后必做配置：官方插件市场（GitHub/context7/skill-creator/learning-output-style/superpowers/frontend-design）→ Safety Net 插件（cc-marketplace）→ StatusLine（带安全网管道）→ 智谱 MCP（4 个服务器）。多数插件安装需用户在 `/plugin` 交互操作，Hook 和 settings.json 可自动配置。完整步骤见 [[wiki/new-project-setup.md#Claude Code 首次安装配置（一次性）|新项目初始化 - 首次安装配置]]。

> 当前已安装的插件和配置状态见 [[wiki/claude-code-setup.md|Claude Code 当前配置清单]]。

## 安装教训

- Safety Net：Windows 需手动配 Hook 和 `SAFETY_NET_PLUGIN_ROOT` 环境变量
- 任何插件安装前备份：`cp -r ~/.claude/rules ~/.claude/rules.backup`
- Rules 和 Hooks 都是追加合并，绝不能直接覆盖

## 创建自定义子代理

创建自定义 subagent 有两种方式：交互式 `/agents` 命令（推荐）和手动创建 Markdown 文件。详见 [[sources/claude-code/creating-subagents.md|创建自定义 Subagents 指南]]。

### 创建方法

- **交互式**：`/agents` → Library → Create new agent → Generate with Claude → 配置工具/模型/内存 → 保存
- **手动**：在 `~/.claude/agents/` 或 `./.claude/agents/` 下创建 `.md` 文件，包含 YAML frontmatter + Markdown 系统提示词

### Frontmatter 核心参数

| 参数 | 必填 | 说明 |
|------|------|------|
| `name` | 是 | 唯一标识符，小写字母和连字符 |
| `description` | 是 | **最关键** — Claude 据此决定何时自动委托任务 |
| `tools` | 否 | 允许的工具列表（限制工具 = 聚焦执行） |
| `model` | 否 | `sonnet`（复杂推理）或 `haiku`（简单研究，快速便宜） |
| `memory` | 否 | `project` 或 `user`，开启持久内存 |

### 项目级 vs 用户级

- **项目级** `./.claude/agents/` — 团队通用代理，可提交 Git 共享
- **用户级** `~/.claude/agents/` — 个人偏好代理，所有项目可用
- 配置变更后需重启会话生效

### 五大效率提升特性

1. **隔离高容量操作** — 委托给 subagent 在独立窗口处理，只返回摘要，保持主会话干净
2. **路径规则按需加载** — `.claude/rules/` 配合 `paths` 字段，只在处理匹配文件时加载规则
3. **持久内存** — `memory: project` 让代理跨会话记住项目架构和 Bug 模式，越用越聪明
4. **显式调用** — `@code-reviewer` 直接启动特定代理，无心智负担
5. **灵活搭配模型** — 简单研究任务用 `haiku`，速度快费用低

### 持久内存最佳实践

- 启用 `memory: project` 后，代理自动创建 `MEMORY.md`，启动时加载前 200 行/25KB
- **工作前**："帮我审查 PR，顺便检查你的 memory，看看有没有类似代码模式"
- **完工后**："把你在这次调试中学到的新知识保存到你的 memory 中"

## 子代理 Skill 工具可用性

**结论：** 子代理和团队成员都不能使用 Skill 工具（不在直接工具列表，也不在 ToolSearch 延迟列表）。父代理必须预加载技能内容注入 prompt。 ← [[sources/claude-code/subagent-skill-test-2026-05-10.md]]

**禁止：** 在子代理 prompt 中要求其调用 Skill 工具
**原因：** Skill 是主会话运行时内置工具，不传递给子代理上下文。ToolSearch("Skill") 返回假阳性空 schema
**替代：** 父代理先用 Skill 加载技能完整内容，用 `---<SKILL_NAME> START/END---` 标记包裹注入子代理 prompt

**注入方式：**
- 子代理（`--print`）：技能内容嵌入 `-p` 参数
- 团队成员（`--agents`）：技能内容嵌入 `prompt` 字段 ← [[sources/claude-code/subagent-skill-test-2026-05-10.md]]

**失败的方案（勿再尝试）：**
- ToolSearch 加载 Skill → 假阳性
- `--allowedTools Skill` → 参数被忽略
- `--plugin-dir superpowers/` → 插件提供内容不提供工具

## ECC 补强

ECC（Everything Claude Code）是独立插件，提供基础设施层（编码规范、上下文管理、安全审查、持续学习），补充 Superpowers 不覆盖的领域。详见 [[wiki/ecc-superpowers-supplement.md|ECC 补强指南]]。
