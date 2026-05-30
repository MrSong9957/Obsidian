---
name: Claude Code 进阶工作流
description: 上下文管理/Hooks配置范式/子代理限定/MCP管理/并行工作流/编辑器选择 — 10个月实战经验提炼
type: concept
updated: 2026-05-13
tags: claude-code, hooks, subagents, mcp, context-management, workflow, editor, session-memory, token-optimization, evals
source: [sources/claude-code/ecc.md]
---

# Claude Code 进阶工作流

> 前篇：[[wiki/claude-code-p2.md|Claude Code 工具]]
>
> 来源：Shorthand Guide ← [[sources/claude-code/ecc.md]] | Longform Guide ← [[sources/claude-code/ecc.md]]

## 上下文窗口管理（最重要）

- 200k 上下文窗口，启用太多 MCP/插件后可能只剩 70k 可用，性能严重下降
- 经验法则：配置 20-30 个 MCP 没问题，**同时启用 <10 个 MCP**，**活跃工具 <80 个**
- 用 `disabledMcpServers` 按项目禁用不用的 MCP（在 `~/.claude.json` 的 `projects.[path]` 下配置）
- `/plugins` 或 `/mcp` 命令查看当前状态，随时开关 ← [[sources/claude-code/ecc.md]] §5

## Hooks 配置范式

Hooks 分三种事件，各有典型用法：

### PreToolUse（工具执行前）
- 长时间命令（npm/pnpm/cargo/pytest）→ 提醒用 tmux 保持会话
- 写 .md 文件 → 阻止创建不必要的文件（只允许 README/CLAUDE.md）
- git push → 自动打开编辑器审查

### PostToolUse（工具执行后）
- 编辑 .ts/.tsx/.js/.jsx 文件 → 自动 `prettier --write`
- 编辑 .ts/.tsx 文件 → 自动 `tsc --noEmit` 类型检查
- 任何编辑 → 检查是否有 `console.log` 残留

### Stop（会话结束）
- 审计所有本次修改过的文件，确保没有 `console.log` 遗留

小技巧：`hookify` 插件可以对话式创建 Hook，不用手写 JSON（`/hookify`） ← [[sources/claude-code/ecc.md]] §2

## Subagents 目录结构与限定

推荐目录：
```
~/.claude/agents/
  planner.md              # 拆解需求
  architect.md            # 系统设计
  tdd-guide.md            # 测试驱动
  code-reviewer.md        # 代码审查
  security-reviewer.md    # 安全审查
  build-error-resolver.md # 构建错误
  e2e-runner.md           # 端到端测试
  refactor-cleaner.md     # 清理死代码
  doc-updater.md          # 文档同步
```

核心原则：
- 每个子代理配置允许的工具、MCP 和权限
- 子代理可以调用技能（skills）自主工作
- **限制工具 = 聚焦执行**：给的工具越少，越不容易跑偏
- 子代理 + 技能 = 自动化组合 ← [[sources/claude-code/ecc.md]] §3

## MCP 管理策略

- 全部放用户级配置（`~/.claude.json`），按项目禁用不用的
- HTTP 类型 MCP（Cloudflare、ClickHouse）不需要本地安装，直接 URL 连接
- `npx -y` 类型 MCP 首次调用时自动安装，不需要全局安装
- **Chrome in Claude**：内置浏览器控制 MCP，Claude 自主操控浏览器 ← [[sources/claude-code/ecc.md]] §5

## Skills vs Commands 区别

| | Skills | Commands |
|---|---|---|
| 位置 | `~/.claude/skills/` | `~/.claude/commands/` |
| 规模 | 宽泛工作流，可多文件目录 | 快速单次 prompt |
| 调用 | `/skill-name` | `/command-name` |
| 特点 | 可链式调用（一条消息串联多个） | 轻量快捷 |

实用案例：codemap-updater 技能在关键节点更新代码地图，帮 Claude 快速导航项目不浪费上下文 ← [[sources/claude-code/ecc.md]] §1

## 并行工作流

- `/fork` — 分叉当前对话，并行处理不重叠任务
- **Git Worktrees** — 每个分支一个独立 checkout，跑多个 Claude 实例互不干扰
- **tmux** — 让 Claude 的服务器在 tmux 里跑，随时 detach/reattach 看日志

```bash
git worktree add ../feature-branch feature-branch
tmux new -s dev
# 分屏：终端 Claude Code + 编辑器看文件
``` ← [[sources/claude-code/ecc.md]] §9

## 编辑器选择

### Zed（推荐）
- Rust 写的，秒开，大项目不卡
- Agent Panel 实时追踪 Claude 编辑的文件
- CMD+Shift+R 命令面板快速执行斜杠命令
- `Ctrl+G` 快速打开 Claude 正在编辑的文件
- 开启自动保存确保 Claude 读到最新内容

### VS Code / Cursor
- 终端模式 + `\ide` 同步
- 或扩展模式，UI 更集成
← [[sources/claude-code/ecc.md]] §10

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

← [[sources/claude-code/ecc.md]] §13

## 五条核心建议

1. **别搞太复杂** — 配置像微调不是架构，简单就好
2. **上下文窗口很珍贵** — 不用的 MCP 和插件都禁用
3. **能并行就并行** — /fork 分叉对话，git worktree 并行开发
4. **自动化重复的事** — Hooks 处理格式化、lint、提醒
5. **限制子代理范围** — 工具越少执行越聚焦

← [[sources/claude-code/ecc.md]] §15

---

> 以下进阶内容来自 Longform Guide ← [[sources/claude-code/ecc.md]]

## Session Memory 持久化

- 跨会话恢复：`~/.claude/sessions/YYYY-MM-DD-topic.tmp`，每会话一个文件
- 文件内容：有效方法（附证据）+ 失败方法 + 未尝试方法 + 剩余待办
- 触发时机：撞上下文限制、天结束时保存、次日提供路径恢复
← [[sources/claude-code/ecc.md]] §1

## Strategic Compacting

- 禁用自动压缩，在逻辑节点手动 `/compact`：探索→执行切换时、里程碑完成后
- 计数器 Hook（绑定 PreToolUse Edit/Write）：达 50 次工具调用时提醒考虑压缩
← [[sources/claude-code/ecc.md]] §2

## Dynamic System Prompt Injection

- `claude --system-prompt "$(cat dev.md)"` 按 CLI 别名切换场景（dev/review/research）
- **指令权重**：System Prompt > User Messages > Tool Results，比 `.claude/rules/` 更高优先级
- 日常区别不大，安全关键/严格行为规则场景才有实际意义
← [[sources/claude-code/ecc.md]] §3

## Memory Persistence Hooks

- **PreCompact**：压缩前保存关键状态 → [[sources/claude-code/ecc.md]] §4
- **SessionStart**：加载近 7 天会话上下文
- **Stop**：持久化每日会话文件
← [[sources/claude-code/ecc.md]] §4

## Continuous Learning

- **Stop Hook 自动评估**：会话结束时提取模式（错误方案、调试技巧、变通方案），保存到 `~/.claude/skills/learned/`
- **`/learn` 手动提取**：会话中途随时运行，即时提取刚解决的模式
- **用 Stop 而非 UserPromptSubmit**：后者每条消息触发，开销大；Stop 只触发一次，评估完整会话
← [[sources/claude-code/ecc.md]] §5

## Token 经济学

- **Haiku + Opus 组合**性价比最高（5x 价差），Sonnet + Opus 只有 1.67x 不值得
- Sonnet 做 90% 编码；首次失败/跨 5+ 文件/架构决策 → Opus；重复/明确/worker → Haiku
- mgrep 替代 grep/ripgrep 减少约 50% token
- 不需 Claude 处理完整输出的命令放 tmux 跑，只复制需要的部分
- 模块化代码库（数百行而非数千行）减少反复读取的 token 浪费
- System Prompt 瘦身可从 ~18k 减至 ~10k tokens（参考 YK 的补丁方法）
← [[sources/claude-code/ecc.md]] §6

## Sub-Agent 上下文问题

- 子代理只有查询字面意思，缺乏背后的目的和推理 → 摘要常遗漏关键细节
- **迭代检索模式**：评估返回值 → 不充分则追问 → 最多 3 轮循环
- **传递目标上下文**，不只是查询——帮子代理判断该包含什么
← [[sources/claude-code/ecc.md]] §7

## 序列阶段编排

```
Research → Plan → Implement → Review → Verify
```
- 每个 agent 一个输入一个输出，输出存文件不靠记忆，agent 间 `/clear` 刷新上下文
← [[sources/claude-code/ecc.md]] §8

## Agent Abstraction 分层

- **Tier 1（直接增益）**：Subagents、Metaprompting、开头多问用户
- **Tier 2（高门槛）**：Long-running agents、Parallel multi-agent、Role-based、Computer use
- 从 Tier 1 开始，有真实需求才进阶 Tier 2
← [[sources/claude-code/ecc.md]] §9

## MCP → Skills 替代

- 很多 MCP 是 CLI 封装（GitHub/Supabase/Vercel），用 skills+commands 包装 CLI 调用可省上下文
- Lazy Loading 已解决启动时上下文占用，但 **token 使用和成本**仍未解决
- CLI + skills 方式在重操作（DB 查询、部署）时仍显著省 token
← [[sources/claude-code/ecc.md]] §10

## Evaluations 体系

- **Checkpoint-Based**：线性工作流，每个检查点验证后继续；**Continuous**：每 N 分钟运行测试套件，发现回归立即停修
- **Grader**：Code-Based（快/客观/脆弱）、Model-Based（灵活/贵/不稳定）、Human（金标准/慢/贵）
- **pass@k**（至少一次成功）vs **pass^k**（全部成功）：前者验证"能用"，后者验证一致性
- 评估路线图：从 20-50 个真实失败用例开始，监控饱和（100% = 需要更多测试）
← [[sources/claude-code/ecc.md]] §11

## 并行实例管理进阶

- **Cascade Method**：新任务向右排列，左到右扫描，同时聚焦最多 3-4 个任务
- **Two-Instance Kickoff**：实例 1 搭骨架（结构/配置/约定），实例 2 深度研究（PRD/架构图/文档引用）
- 用 `/rename` 命名聊天避免混淆，主聊天改代码，分叉做研究
← [[sources/claude-code/ecc.md]] §12

## 可复用模式哲学

- 投资 Subagents/Skills/Commands/Planning patterns/Context engineering > 投资特定模型技巧
- 模式在模型升级后仍有效，可迁移到其他 agent（如 Codex），有复利效应
← [[sources/claude-code/ecc.md]] §14
