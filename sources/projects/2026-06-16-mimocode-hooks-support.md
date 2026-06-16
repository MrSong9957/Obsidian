---
name: MiMo-Code Hooks 支持情况调研
description: 确凿证据判定——MiMo-Code 已有开发者级 plugin hook（含 tool.execute.before/after，等价 PreToolUse/PostToolUse），但缺 Claude Code 式用户声明式配置与 shell 运行时
type: reference
updated: 2026-06-16
tags: [mimocode, opencode, hooks, plugin, claude-code]
source: https://github.com/XiaomiMiMo/MiMo-Code
---

# MiMo-Code Hooks 支持情况调研

> 调研对象：MiMo-Code（OpenCode fork），分支 `feat/support-agents-local-md` @ `2917744`，upstream `XiaomiMiMo/MiMo-Code@ab53d06`（2026-06-16 fetch）。
> 相关：[[mimocode-reading-guide.md|MiMo-Code 源码阅读指南]]、[[stop-hook-design.md|Stop Hook 设计]]、[[opencode.md|OpenCode]]

## 判定结论

**MiMo-Code 支持 hooks（开发者级），但不支持 Claude Code 式的「用户声明式配置 + shell 命令运行时」。**

⚠️ **修正一处常见误判**：仅凭「`config/` 目录无 `hook.ts`」容易得出「MiMo-Code 缺 hooks」的结论——**这是错的**。plugin 系统的 `Hooks` 接口里**已包含** `tool.execute.before` / `tool.execute.after`，正是 Claude Code `PreToolUse` / `PostToolUse` 的等价物。真正的 gap 在「用户入口」，不在「hook 能力本身」。

## 要点

1. `Hooks` 接口已覆盖工具执行前后、用户消息、权限、actor 生命周期等核心场景[^1]。
2. `Config` 类型无 `hooks` 字段——用户无法在配置文件里声明 hooks[^2]。
3. 全仓无 Claude Code 式事件名，且 hook 只能是 TS 函数，无 shell 脚本运行时[^3]。

## 详细内容

### 证据①：`Hooks` 接口已有工具/会话级钩子（铁证）

`packages/plugin/src/index.ts` 的 `Hooks` 接口（L302-427）定义了插件可注册的全部 hook 点[^1]：

| MiMo-Code hook | 行号 | input → output（可改的部分加粗） | Claude Code 对应 |
|---|---|---|---|
| `tool.execute.before` | L345 | `{tool,sessionID,callID}` → **`{args}`**（可改参数） | **PreToolUse** |
| `tool.execute.after` | L353 | `{tool,sessionID,callID,args}` → **`{title,output,metadata}`**（可改输出） | **PostToolUse** |
| `command.execute.before` | L341 | `{command,sessionID,arguments}` → `{parts}` | PreToolUse(Bash) |
| `chat.message` | L313 | 消息接收 → `{message,parts}` | UserPromptSubmit |
| `chat.params` / `chat.headers` | L326 / L336 | 改 LLM 参数 / 请求头 | — |
| `permission.ask` | L340 | `Permission` → `{ask\|deny\|allow}` | 权限钩子 |
| `shell.env` | L349 | `{cwd,sessionID,callID}` → `{env}` | — |
| `tool.definition` | L412 | 改工具描述/参数 schema | — |
| `actor.preStop` / `actor.postStop` | L419 / L427 | actor 结束前后，可注入续跑 | Stop |
| `experimental.chat.system.transform` | L370 | 改系统提示 | — |
| `experimental.session.compacting` | L383 | 自定义压缩 prompt | — |

→ **工具执行前后、用户消息、权限、actor 生命周期——Claude Code 的核心 hook 场景已被覆盖。**

### 证据②：`Config` 类型无 `hooks` 字段（铁证）

`packages/opencode/src/config/config.ts` 的 Config 类型字段含 `command`(L103) / `skills`(L106) / `plugin`(L117) / `mcp`(L193) / `permission`(L209) / `tools`(L210) 等——**`grep hook` 0 命中**[^2]。

→ 用户**无法**在 `opencode.json` 或任何配置文件里声明 hooks。

### 证据③：无 Claude Code 式事件名，hook 只能是 TS 函数

- 全仓 `.ts` 搜索 `PreToolUse` / `PostToolUse` / `UserPromptSubmit` / `SessionStart` / `SessionEnd` **0 命中**（命中项均为 LSP/MCP/ACP 协议层 notification，非用户钩子）[^3]。
- hook 注册形态是 `Plugin = (input) => Promise<Hooks>`[^1]——**必须写 TS 代码、打包、`opencode plugin install`**，没有「指向外部 shell/node 脚本」的运行时。

### 真实 gap

| 能力 | Claude Code | MiMo-Code |
|---|---|---|
| 用户声明式配置（settings.json `hooks`） | ✅ | ❌（Config 无 hooks 字段） |
| matcher 按工具名匹配 | ✅ | ❌（仅 actor matcher） |
| 运行任意 shell/node 脚本 | ✅ | ❌（仅 TS 函数） |
| 工具执行前后钩子 | PreToolUse / PostToolUse | ✅ `tool.execute.before/after` |
| 阻断 / 放行 / 改参 | ✅（脚本返回 JSON） | 部分（output 字段可改） |

→ **gap 不是「没有 hooks」，而是「缺用户级入口 + shell 运行时」。**

### ECC 兼容性结论

ECC（`affaan-m/ECC`）的 hooks 是 **Claude Code 式**：`settings.json` 声明 → matcher 匹配工具 → 跑 node 脚本 → 返回 JSON 阻断/放行（如 GateGuard、governance-capture、mcp-health-check）。

**这些 hooks 无法直接在 MiMo-Code 上运行**，因为 MiMo-Code 缺声明式配置层与 shell 运行时。迁移路径只有两条：

1. 把每条 ECC hook 改写为 MiMo-Code TS plugin（注册到 `tool.execute.before/after` 等事件）；
2. 为 MiMo-Code 新增「声明式 hooks → 已有 plugin hook」的桥接层（更小的功能，复用全部已有基础设施——推荐方向）。

> 旁证：本次调研会话中触发的 GateGuard（Fact-Forcing）、governance-capture 等全部依赖 Claude Code `PreToolUse` 机制——它们正是「Claude Code 式 hook 生态」的实例，反证 MiMo-Code 缺这层入口。

## 引用索引

[^1]: `packages/plugin/src/index.ts` L302-427 — `Hooks` 接口完整定义（`@mimo-ai/plugin` 包）；插件注册形态 `Plugin = (input) => Promise<Hooks>` 见同文件 L75。
[^2]: `packages/opencode/src/config/config.ts` L97-261 — Config 类型字段清单；`grep -nE "hooks|Hooks"` 于本文件 0 命中（2026-06-16）。
[^3]: `grep -rnE "PreToolUse|PostToolUse|UserPromptSubmit|SessionStart|SessionEnd" packages/opencode/src --include="*.ts"` 0 命中（2026-06-16，分支 `feat/support-agents-local-md`）。
