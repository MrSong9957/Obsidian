---
name: Claude Code 配置与工作流
description: settings.json/CLAUDE.md/rules/记忆系统/插件/Hooks/子代理/MCP/并行工作流 — Claude Code 使用的核心参考
type: concept
updated: 2026-06-17
tags: claude-code, settings, claudemd, memory, rules, hooks, subagents, mcp, workflow, plugins
source: [sources/claude-code/claude-code-references.md, wiki/ecc.md]
---

# Claude Code 配置与工作流

> 详细参考：[[sources/claude-code/claude-code-references.md|Claude Code 配置详细参考]]

## settings.json 关键字段

- `showClearContextOnPlanAccept: true` — Plan 模式审批时显示清空上下文选项，释放窗口空间
- `autoMemoryEnabled: true` — 启用自动记忆
- `fileCheckpointingEnabled: true` — 编辑前自动快照，支持 `/rewind` 撤销
- `autoCompactEnabled: true` — 上下文满时自动压缩，避免会话中断
- `todoFeatureEnabled: true` — 启用任务追踪面板，多步骤任务更清晰
- `statusLine` — 状态栏配置（type: command/template）
- `claudeMdExcludes` — monorepo 排除不相关 CLAUDE.md
- `autoMemoryDirectory` — 自定义记忆存储位置（仅 user/local settings 可配）

完整设置清单见 [[sources/claude-code/settings-reference.md|Claude Code 完整设置清单]]（100+ 配置项）。

## 常见配置场景

```json
// 场景 1：开启自动记忆
{ "autoMemoryEnabled": true, "autoMemoryDirectory": "~/.claude/projects/my-project/memory" }

// 场景 2：允许 npm 命令不弹确认
{ "permissions": { "allow": ["Bash(npm *)"] } }

// 场景 3：配置自动格式化 Hook
{ "hooks": { "PostToolUse": [{ "matcher": "Write|Edit", "hooks": [{ "type": "command", "command": "prettier --write $FILE" }] }] } }
```

## 三大持久化系统

| | CLAUDE.md | .claude/rules/ | Auto Memory |
|---|---|---|---|
| **谁写的** | 你（人工） | 你（人工） | Claude（自动） |
| **内容** | 指令和规则 | 模块化规则 | 学习和经验 |
| **作用域** | 全局/项目/组织 | 用户级/项目级 | 每个 git 仓库 |
| **加载方式** | 每次会话完整加载 | 无条件：启动时；scoped：按需 | 前 200 行/25KB |
| **Compaction 后** | 从磁盘重新加载，完整恢复 | 同 CLAUDE.md | 不重新注入，丢失 |

**简单理解：CLAUDE.md 是规章制度，rules 是分册细则，MEMORY.md 是 Claude 的笔记本。**

## CLAUDE.md 结构规则

- 三级作用域：组织级 > 用户级 > 项目级（更具体的位置覆盖更宽泛的）
- 每次会话完整加载，无截断限制；compaction 后从磁盘重新读取并注入
- 每个文件不超过 200 行；超长内容用 `@path` 导入或拆分到 rules/
- 写具体可验证的指令，不写模糊指令；定期审查消除矛盾
- 子目录 CLAUDE.md 按需加载（读取该子目录文件时才触发）

## .claude/rules/ 使用结论

- 无条件规则：启动时全部加载，compaction 后重新加载
- Scoped 规则（带 `paths:`）：Claude 读取匹配路径文件时触发，不是每个工具调用
- 核心策略：CLAUDE.md 防全局误解，scoped rules 防局部错误
- 反模式：全部写成无条件规则 = 把 1000 行 CLAUDE.md 搬了个位置

## 记忆系统行为

- MEMORY.md 加载前 200 行或 25KB，超限部分永远不加载；compaction 后不会重新注入
- 主题文件不启动加载，Claude 按 Read 工具按需读取
- 持久化优先级：Hooks > CLAUDE.md > 无条件 rules > scoped rules > MEMORY.md 前 200 行 > 主题文件 > 口头约定
- Active Recall 四阶段：扫描（200 文件前 30 行）→ 摘要 → Sonnet 选最多 5 条 → 注入（单文件 4KB 截断）
- 实用命令：`/memory`（查看已加载文件）、"记住这个"（保存到 auto memory）、"加到 CLAUDE.md"（写入 CLAUDE.md）

## Superpowers 核心技能

using-superpowers / writing-plans / executing-plans / systematic-debugging / test-driven-development / brainstorming / subagent-driven-development / dispatching-parallel-agents / requesting-code-review / receiving-code-review / verification-before-completion / finishing-a-development-branch / using-git-worktrees / writing-skills / ui-ux-pro-max

## Safety Net 核心能力

- Hook 机制在命令执行前拦截，CLAUDE.md 的"软"规则无法替代"硬"技术约束
- 拦截：git checkout -- / git reset --hard / git push --force / rm -rf（cwd 外）/ git branch -D / find -delete
- 放行：git checkout -b / git branch -d / git push --force-with-lease / rm -rf ./...
- 安装：`/plugin install safety-net@cc-marketplace` → 配置 PreToolUse Hook

## ECC

ECC（Everything Claude Code）是独立插件，提供基础设施层（编码规范、上下文管理、安全审查、持续学习），补充 Superpowers 不覆盖的领域。

## 上下文窗口管理（最重要）

- 200k 上下文窗口，启用太多 MCP/插件后可能只剩 70k 可用，性能严重下降
- 经验法则：配置 20-30 个 MCP 没问题，**同时启用 <10 个 MCP**，**活跃工具 <80 个**
- 用 `disabledMcpServers` 按项目禁用不用的 MCP（在 `~/.claude.json` 的 `projects.[path]` 下配置）

## Hooks 配置范式

- **PreToolUse**：长时间命令 → 提醒用 tmux；写 .md → 阻止不必要文件；git push → 打开编辑器审查
- **PostToolUse**：编辑 .ts/.tsx → 自动 prettier + tsc --noEmit；任何编辑 → 检查 console.log
- **Stop**：审计修改过的文件，确保没有 console.log 遗留
- 小技巧：`/hookify` 插件可以对话式创建 Hook，不用手写 JSON

## Skill 开发与部署

- 存放路径：`~/.claude/skills/<skill-name>/SKILL.md`，放好后用 `/reload-plugins` 刷新生效
- **双目录部署**：项目目录（开发）和 `~/.claude/skills/`（运行）是两份独立副本，改代码后必须 `cp` 同步
- **技能链调用**：Skill 内引用其他 Skill 时必须用 Skill 工具加载完整内容，不能跳过
- **子代理限制**：子代理/团队成员无法调用 Skill 工具，父代理必须预加载技能内容并注入 prompt

## Subagents 目录结构

```
~/.claude/agents/
  planner.md / architect.md / tdd-guide.md / code-reviewer.md / security-reviewer.md
  build-error-resolver.md / e2e-runner.md / refactor-cleaner.md / doc-updater.md
```

核心原则：限制工具 = 聚焦执行；子代理 + 技能 = 自动化组合

## MCP 管理策略

- 全部放用户级配置（`~/.claude.json`），按项目禁用不用的
- HTTP 类型 MCP 不需要本地安装，直接 URL 连接
- `npx -y` 类型 MCP 首次调用时自动安装

## Skills vs Commands

| | Skills | Commands |
|---|---|---|
| 位置 | `~/.claude/skills/` | `~/.claude/commands/` |
| 规模 | 宽泛工作流，可多文件目录 | 快速单次 prompt |
| 调用 | `/skill-name` | `/command-name` |

## 并行工作流

- `/fork` — 分叉当前对话，并行处理不重叠任务
- **Git Worktrees** — 每个分支一个独立 checkout，跑多个 Claude 实例互不干扰
- **tmux** — 让 Claude 的服务器在 tmux 里跑，随时 detach/reattach 看日志

## 编辑器选择

- **Zed**（推荐）：Rust 写的，秒开，Agent Panel 实时追踪，CMD+Shift+R 命令面板
- **VS Code / Cursor**：终端模式 + `\ide` 同步，或扩展模式

## 常用快捷键

| 快捷键 | 功能 |
|--------|------|
| Ctrl+U | 删除整行 |
| ! | 快速 bash 命令 |
| @ | 搜索文件 |
| / | 斜杠命令 |
| Shift+Enter | 多行输入 |
| Tab | 切换思考过程显示 |
| Esc Esc | 中断/恢复 |

## 五条核心建议

1. **别搞太复杂** — 配置像微调不是架构，简单就好
2. **上下文窗口很珍贵** — 不用的 MCP 和插件都禁用
3. **能并行就并行** — /fork 分叉对话，git worktree 并行开发
4. **自动化重复的事** — Hooks 处理格式化、lint、提醒
5. **限制子代理范围** — 工具越少执行越聚焦

## 进阶技术

以下技术从实战中提炼，按需使用：

- **Session Memory**：跨会话恢复 `~/.claude/sessions/YYYY-MM-DD-topic.tmp`
- **Strategic Compacting**：禁用自动压缩，在逻辑节点手动 `/compact`
- **Dynamic System Prompt**：`claude --system-prompt "$(cat dev.md)"` 按场景切换
- **Continuous Learning**：Stop Hook 自动评估会话，提取模式到 `~/.claude/skills/learned/`
- **Token 经济学**：Haiku + Opus 组合性价比最高（5x 价差）；mgrep 替代 grep 减少 ~50% token
- **Evaluations**：Checkpoint-Based（线性验证）vs Continuous（持续测试）；pass@k vs pass^k

详细内容见 [[sources/claude-code/claude-code-references.md|Claude Code 配置详细参考]]
