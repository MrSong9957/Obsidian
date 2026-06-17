---
name: ecc-references
description: ECC Hooks 完整参考 — 所有 Hook 作用、ECC_HOOK_PROFILE 档位对照
type: reference
updated: 2026-06-18
tags: [ecc, hooks, claude-code, profile]
source: https://github.com/affaan-m/ECC
---

# ECC Hooks 完整参考

> 入口文档：[[wiki/ecc]]

## 概述

ECC 的 Hooks 系统通过 `hooks.json` 定义，实际脚本位于 `scripts/hooks/` 目录（约 48 个文件）。核心调度机制由 `run-with-flags.js` 实现，根据 `ECC_HOOK_PROFILE` 环境变量决定是否执行各 hook。

## ECC_HOOK_PROFILE 档位

定义位置：`scripts/lib/hook-flags.js`

```javascript
const VALID_PROFILES = new Set(['minimal', 'standard', 'strict']);
```

| 档位 | 说明 |
|------|------|
| `minimal` | 仅启用生命周期和安全 hook，最低开销 |
| `standard` | **默认值**。平衡质量检查与安全防护 |
| `strict` | 启用额外提醒和更严格的防护栏 |

未设置时默认为 `standard`。

## 其他运行时控制变量

| 变量 | 说明 |
|------|------|
| `ECC_DISABLED_HOOKS` | 逗号分隔的 hook ID 列表，禁用指定 hook |
| `ECC_GATEGUARD=off` | 禁用 GateGuard |
| `ECC_SESSION_START_MAX_CHARS` | 限制 SessionStart 上下文大小（默认 8000） |
| `ECC_SESSION_START_CONTEXT=off` | 禁用 SessionStart 附加上下文 |
| `ECC_CONTEXT_MONITOR_COST_WARNINGS=off` | 禁用 API 成本估算警告 |
| `ECC_GOVERNANCE_CAPTURE=1` | 启用治理事件捕获 |
| `ECC_DRY_RUN=1` | 预览模式，不实际执行 hook |

## PreToolUse Hooks

在工具执行前触发。

| Hook ID | 匹配器 | 作用 |
|---------|--------|------|
| `pre:bash:dispatcher` | `Bash` | Bash 预检调度器，运行子 hook：block-no-verify、auto-tmux-dev、tmux-reminder、git-push-reminder、commit-quality、GateGuard fact-force |
| `pre:write:doc-file-warning` | `Write` | 警告非标准 `.md`/`.txt` 文件创建（允许 README、CLAUDE、CONTRIBUTING、CHANGELOG、LICENSE、SKILL、docs/、skills/） |
| `pre:edit-write:suggest-compact` | `Edit\|Write` | 每约 50 次工具调用建议手动 `/compact` |
| `pre:observe:continuous-learning` | `*` | 捕获工具使用观察用于持续学习（异步，非阻塞） |
| `pre:governance-capture` | `Bash\|Write\|Edit\|MultiEdit` | 捕获治理事件（密钥、策略违规、审批请求），需 `ECC_GOVERNANCE_CAPTURE=1` |
| `pre:config-protection` | `Write\|Edit\|MultiEdit` | 阻止修改 linter/formatter 配置文件，引导 agent 修复代码而非削弱配置 |
| `pre:mcp-health-check` | `*` | MCP 工具执行前检查服务器健康状态，阻止不健康的 MCP 调用 |
| `pre:edit-write:gateguard-fact-force` | `Edit\|Write\|MultiEdit` | 事实强制门控：阻止对每个文件的首次 Edit/Write/MultiEdit，要求先调查（导入者、数据模式、用户指令） |

## PreCompact Hooks

上下文压缩前触发。

| Hook ID | 匹配器 | 作用 |
|---------|--------|------|
| `pre:compact` | `*` | 上下文压缩前保存会话状态 |

## SessionStart Hooks

会话启动时触发。

| Hook ID | 匹配器 | 作用 |
|---------|--------|------|
| `session:start` | `*` | 加载上次上下文，检测包管理器 |

## PostToolUse Hooks

工具执行成功后触发。

| Hook ID | 匹配器 | 作用 |
|---------|--------|------|
| `post:bash:dispatcher` | `Bash` | Bash 后处理调度器，运行子 hook：command-log-audit、command-log-cost、pr-created、build-complete（异步） |
| `post:quality-gate` | `Edit\|Write\|MultiEdit` | 文件编辑后运行质量门检查（异步） |
| `post:edit:design-quality-check` | `Edit\|Write\|MultiEdit` | 前端编辑偏向通用模板风格时发出警告 |
| `post:edit:accumulator` | `Edit\|Write\|MultiEdit` | 记录编辑的 JS/TS 文件路径，Stop 时批量格式化+类型检查 |
| `post:edit:console-warn` | `Edit` | 编辑后警告 `console.log` 语句 |
| `post:governance-capture` | `Bash\|Write\|Edit\|MultiEdit` | 从工具输出捕获治理事件，需 `ECC_GOVERNANCE_CAPTURE=1` |
| `post:session-activity-tracker` | `*` | 跟踪每会话工具调用和文件活动（ECC2 指标） |
| `post:observe:continuous-learning` | `*` | 捕获工具使用结果用于持续学习（异步） |
| `post:ecc-metrics-bridge` | `*` | 维护运行中会话指标聚合，供状态栏和上下文监控使用 |
| `post:ecc-context-monitor` | `*` | 上下文耗尽、高成本、范围蔓延或工具循环时注入 agent 警告 |

## PostToolUseFailure Hooks

工具执行失败后触发。

| Hook ID | 匹配器 | 作用 |
|---------|--------|------|
| `post:mcp-health-check` | `*` | 跟踪失败的 MCP 工具调用，标记不健康服务器，尝试重连 |

## Stop Hooks

每次响应结束后触发。

| Hook ID | 匹配器 | 作用 |
|---------|--------|------|
| `stop:format-typecheck` | `*` | 批量格式化（Biome/Prettier）和类型检查（tsc）本轮编辑的所有 JS/TS 文件 |
| `stop:check-console-log` | `*` | 检查修改文件中的 `console.log` |
| `stop:session-end` | `*` | 每次响应后持久化会话状态（异步） |
| `stop:evaluate-session` | `*` | 评估会话可提取的模式（持续学习）（异步） |
| `stop:cost-tracker` | `*` | 跟踪每会话 token 和成本指标（异步） |
| `stop:desktop-notify` | `*` | 发送桌面通知（macOS/WSL），包含任务摘要（异步） |

## SessionEnd Hooks

会话结束时触发。

| Hook ID | 匹配器 | 作用 |
|---------|--------|------|
| `session:end:marker` | `*` | 会话结束生命周期标记和清理日志（异步） |

## Bash Dispatcher 子 Hook

`pre:bash:dispatcher` 和 `post:bash:dispatcher` 内部包含多个子 hook，各自有独立的 profile 门控。

### Pre-Bash 子 Hook

| 子 Hook ID | 启用档位 | 作用 |
|------------|----------|------|
| `pre:bash:block-no-verify` | minimal, standard, strict | 阻止 git 命令中的 `--no-verify` |
| `pre:bash:auto-tmux-dev` | 始终启用 | 开发命令自动创建 tmux 会话 |
| `pre:bash:tmux-reminder` | strict | 长时间运行命令建议使用 tmux |
| `pre:bash:git-push-reminder` | strict | git push 前提醒审查更改 |
| `pre:bash:commit-quality` | strict | git commit 前运行质量检查（lint 暂存文件、验证提交消息格式、检测 console.log/debugger/secrets） |
| `pre:bash:gateguard-fact-force` | standard, strict | GateGuard 事实强制门控 |

### Post-Bash 子 Hook

| 子 Hook ID | 启用档位 | 作用 |
|------------|----------|------|
| `post:bash:command-log-audit` | 始终启用 | 命令审计日志 |
| `post:bash:command-log-cost` | 始终启用 | 命令成本日志 |
| `post:bash:pr-created` | standard, strict | `gh pr create` 后记录 PR URL 和审查命令 |
| `post:bash:build-complete` | standard, strict | 构建命令后后台分析 |

## Profile 档位 Hook 启用对照表

### minimal 档位

仅启用带 `minimal` 标记或无 profile 限制的 hook：

| 事件 | Hook ID | 说明 |
|------|---------|------|
| PreToolUse | `pre:bash:dispatcher` | 运行子 hook：block-no-verify、auto-tmux-dev |
| SessionStart | `session:start` | 加载上次上下文 |
| PostToolUse | `post:bash:dispatcher` | 运行子 hook：command-log-audit、command-log-cost |
| PostToolUse | `post:ecc-metrics-bridge` | 会话指标聚合 |
| Stop | `stop:session-end` | 持久化会话状态 |
| Stop | `stop:evaluate-session` | 模式提取 |
| Stop | `stop:cost-tracker` | 成本跟踪 |
| SessionEnd | `session:end:marker` | 生命周期标记 |

**总计：约 8 个 hook 活跃。** 无质量检查、无提醒、无治理、无上下文监控。

### standard 档位（默认）

在 minimal 基础上增加：

| 事件 | Hook ID | 说明 |
|------|---------|------|
| PreToolUse | `pre:write:doc-file-warning` | 文档文件警告 |
| PreToolUse | `pre:edit-write:suggest-compact` | 压缩建议 |
| PreToolUse | `pre:observe:continuous-learning` | 学习观察 |
| PreToolUse | `pre:governance-capture` | 治理事件 |
| PreToolUse | `pre:config-protection` | 配置文件保护 |
| PreToolUse | `pre:mcp-health-check` | MCP 健康检查 |
| PreToolUse | `pre:edit-write:gateguard-fact-force` | 事实强制门控 |
| PreCompact | `pre:compact` | 压缩前保存状态 |
| PostToolUse | `post:quality-gate` | 质量门检查 |
| PostToolUse | `post:edit:design-quality-check` | UI 漂移检测 |
| PostToolUse | `post:edit:accumulator` | 批量格式化累加器 |
| PostToolUse | `post:edit:console-warn` | console.log 警告 |
| PostToolUse | `post:governance-capture` | 治理捕获 |
| PostToolUse | `post:session-activity-tracker` | 活动跟踪 |
| PostToolUse | `post:observe:continuous-learning` | 学习观察 |
| PostToolUse | `post:ecc-context-monitor` | 上下文耗尽警告 |
| PostToolUseFailure | `post:mcp-health-check` | MCP 故障跟踪 |
| Stop | `stop:format-typecheck` | 批量格式化+类型检查 |
| Stop | `stop:check-console-log` | console.log 审计 |
| Stop | `stop:desktop-notify` | 桌面通知 |

**总计：约 28 个 hook 活跃。** 平衡质量与安全。

### strict 档位

在 standard 基础上增加 strict 专属子 hook：

| 事件 | 子 Hook ID | 说明 |
|------|------------|------|
| PreToolUse | `pre:bash:tmux-reminder` | tmux 建议 |
| PreToolUse | `pre:bash:git-push-reminder` | git push 审查提醒 |
| PreToolUse | `pre:bash:commit-quality` | 提交前质量检查 |

**总计：约 31 个 hook 活跃。** 最大防护栏和提醒。

## 架构要点

1. **Profile 门控机制**：每个 hook 声明允许的 profile 列表（CSV 格式），`isHookEnabled()` 函数检查当前 `ECC_HOOK_PROFILE` 是否在允许列表中
2. **合并调度器**：`pre-bash-dispatcher.js` 和 `post-bash-dispatcher.js` 将多个子 hook 打包为 `hooks.json` 中的单个条目，减少进程启动开销
3. **静默跳过**：不在允许 profile 集中的 hook 被静默跳过（exit 0 透传），profile 切换不会破坏流程
4. **优先级**：`ECC_DISABLED_HOOKS` 提供的逐 hook 覆盖优先于 profile 门控

## 引用索引

[^1]: [ECC hooks/README.md](https://github.com/affaan-m/ECC/blob/main/hooks/README.md) — Hook 官方文档
[^2]: [ECC scripts/lib/hook-flags.js](https://github.com/affaan-m/ECC/blob/main/scripts/lib/hook-flags.js) — Profile 定义和门控逻辑
[^3]: [ECC hooks/hooks.json](https://github.com/affaan-m/ECC/blob/main/hooks/hooks.json) — Hook 图定义
