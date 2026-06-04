---
name: OpenCode 桥接
description: OpenCode plugin 平台行为限制、input.tool 契约、CLAUDE_SETTINGS_PATH override、ECC 桥接关键结论（log 修复/parseHookOutput/aggregateResults/output.args 写回）；代理配置（添加/覆盖/禁用代理）
type: concept
updated: 2026-06-03
tags: opencode, claude-code, hooks, plugin, bridge, deploy, agent
source: [sources/claude-code/opencode-bridge-v2.md]
---

# OpenCode 桥接

> 过程文档：[[sources/claude-code/opencode-bridge-v2.md|OpenCode 桥接 v2 升级过程]]
>
> 配套仓库：`E:\Files\Code\opencode-claude-bridge` | 部署报告：`~/.config/opencode/MIGRATION_NOTES.md`

## 平台行为（OpenCode CLI 模式）

| 行为 | 状态 | 原因 |
|---|---|---|
| `session.idle` / `session.deleted` 触发 | ❌ 不会触发 | OpenCode CLI 是无状态单次运行，无 idle 状态 |
| `experimental.session.compacting` 触发 | ✅ 用户日志可触发 | 手动 `/compact` 命令 |
| `tool.execute.before` / `tool.execute.after` 触发 | ✅ 持续触发 | 工具调用必触发 |
| `permission.asked` 触发 | ✅ 权限请求时触发 | OpenCode 内置权限系统 |

**关键推断**：`input.tool` 是**字符串**（`"Bash"`、`"Read"`），不是 `{name, arguments}` 对象。证据：`~/AppData/Local/Temp/ecc-metrics-*.json` 中 `recent_tools[].tool` 字段是字符串。

## 可用事件（7 个）

| 事件 | 触发场景 | 用途 |
|---|---|---|
| `tool.execute.before` | 工具调用前 | PreToolUse 等价 |
| `tool.execute.after` | 工具调用后 | PostToolUse 等价 |
| `tool.execute.after` (error) | 工具失败 | PostToolUseFailure 等价 |
| `experimental.session.compacting` | 上下文压缩 | PreCompact 等价，注入 `output.context` |
| `session.created` | 会话创建 | SessionStart 等价 |
| `session.idle` | 会话空闲 | **CLI 模式不触发** |
| `session.deleted` | 会话结束 | SessionEnd 等价，**CLI 模式不触发** |
| `permission.asked` | 权限请求 | PermissionRequest 等价，写回 `output.status` |

## v2 桥接关键能力

| 能力 | 实现 |
|---|---|
| `log()` 修复 | `console.log` 永远执行（v1 早返回导致 CLI 模式静默）|
| `parseHookOutput(stdout)` | 解析 hook 返回的 Claude Code 风格 JSON (`{hookSpecificOutput: {...}}`)|
| `aggregateResults(results)` | 浅合并多 hook 结果：updatedInput 字段 + permissionDecision 第一个 deny 优先 |
| `output.args` 写回 | `{...output.args, ...result.updatedInput}` |
| `_blocked` 标记 | `permissionDecision=deny` 时注入 `_blocked: true, _blockReason` |
| 配置缓存 | 60s TTL `loadSettingsWithCache`，session 内只读 1 次 |
| `CLAUDE_SETTINGS_PATH` 覆盖 | 环境变量覆盖默认 `~/.claude/settings.json` 路径 |

## 输入契约（已确认）

```typescript
tool.execute.before: async (
  input: { tool: string; sessionID?: string; callID?: string },
  output: { args: Record<string, unknown> },
) => { ... }
```

⚠️ **关键**：`input.tool` 是**字符串**，**不是对象**。TypeScript 类型 `{ tool: string }` 正确，Bun 运行时不做类型检查但 OpenCode 实际传字符串。

## 失败教训

### 早期 v1 静默（已修）

**禁止**：在 `log()` 函数顶部用 `if (!ctx.client) return;` 早返回。
**原因**：CLI 模式下 `ctx.client` 是 `undefined`，整个 plugin 加载日志全部丢失，调试时看不到任何东西。
**替代**：把 `console.log` 放在 `if (ctx.client) {...}` 外面，让它**永远执行**。

### 字符串 vs 对象契约混淆

**禁止**：写 `input.tool.name` 或 `input.tool.arguments`。
**原因**：OpenCode 传字符串 `input.tool = "Bash"`，不是对象。访问 `.name` 会返回 `undefined`。
**替代**：用 `input.tool` 直接传字符串到 `pascalCase(tool)`。

### `output.args` 浅合并陷阱

**禁止**：假设 `output.args` 修改会被 OpenCode 接收。
**原因**：要返回的不是修改 `output.args`（in-place 写 OpenCode 不一定读取），而是要用 `output.args = { ...output.args, ...result.updatedInput }` **重新赋值**。
**替代**：始终用 spread + 重新赋值（immutability pattern）。

### Exit code 2 不响应（已知限制）

**禁止**：依赖 Claude Code 的 `process.exit(2)` 来阻断 PreToolUse。
**原因**：OpenCode 不读取子进程退出码，只读 stdout JSON。
**替代**：用 `output.args._blocked = true` 标记 + `_blockReason` 写理由。**注意**：OpenCode 是否真的读取 `_blocked` 字段未确认（探针验证了 `output.args` 写回，但 `_blocked` 拦截未实测）。

## 测试方法（端到端探针）

不依赖 OpenCode 真实调用，直接 import 部署文件：

```javascript
import plugin from "file:///C:/Users/<user>/.config/opencode/plugins/claude-code-hooks.ts";
const handlers = await plugin({ client: undefined, ... });
await handlers["tool.execute.before"](
  { tool: "Bash", sessionID: "x" },
  { args: { command: "echo test" } }
);
```

`tests/probes/probe-v2.mjs` + `probe-v2-e2e.mjs` 已保留。

## 实测验证（2026-06-02 最终确认）

`watch-bridge.mjs` 在用户 PowerShell 窗口实时观测：

- **40+ 次 PostToolUse 写入**（35 秒内），证明 v2 桥接 100% 被触发
- **5 个 session 同时活跃**：用户旧 session + 2 个新 session + Claude Code session
- **原子写模式确认**：`.tmp` → `.json` 替换（与 deploy.mjs 相同）
- **文件字节递增**：342→479，每个工具调用增加 1-34 bytes
- **LSP 已启用**：用户确认「LSP显示已开始」

**最强证据**：我的 4 个测试调用（`echo test-1`、4 个 Call）在 watcher 中 1:1 对应 mtime 变化。

## 部署

```bash
cd E:\Files\Code\opencode-claude-bridge
npm test            # 99/99
npm run typecheck   # 0 errors
npm run deploy      # 原子化复制
```

部署后**必须重启 OpenCode**。

## 回滚

```powershell
# v1 → 当前（v2）
Copy-Item C:\Users\<user>\.config\opencode\plugins\claude-code-hooks.ts.bak-20260602 `
          C:\Users\<user>\.config\opencode\plugins\claude-code-hooks.ts -Force
# 重启 OpenCode
```

完整回滚到迁移前：见 `~/.config/opencode/MIGRATION_NOTES.md` "回滚" 章节。

## ECC Agent 格式兼容性

> 详细调试过程：[[sources/claude-code/ecc-opencode-install.md|ECC-OpenCode 集成踩坑]]

ECC agent `.md` 文件的 frontmatter 格式与 OpenCode 不兼容。安装 ECC 到 OpenCode 时必须处理以下差异：

### frontmatter 字段对照

| 字段 | ECC 格式 | OpenCode 格式 | 处理 |
|------|---------|--------------|------|
| `model` | `opus` / `sonnet` / `anthropic/mimo-v2.5-pro` | `opencode/minimax-m3-free` | **删除此行**，让全局 model 生效 |
| `tools` | YAML 数组 `["Read", "Grep", "Glob"]` | YAML 对象 `Read: true, Grep: true` | 必须转换格式 |
| `color` | CSS 颜色名 `red`, `green`, `purple` | 枚举 `primary`, `secondary`, `error`, `success` 等 | 必须映射 |
| `name` | 存在 | 可选 | 保留 |
| `description` | 存在 | 必须 | 保留 |

### color 映射表

| ECC | OpenCode |
|-----|----------|
| `red` | `error` |
| `green` | `success` |
| `purple` | `accent` |
| `teal` | `info` |
| `orange` | `warning` |

### Agent 注册策略

**禁止**：把 ECC 全部 61 个 agent `.md` 文件复制到 `~/.config/opencode/agents/`。
**原因**：OpenCode 把 `agents/` 目录下所有 `.md` 文件当作可切换模式（Ctrl+Tab），61 个 agent 会让模式菜单爆满。
**替代**：只在 `opencode.jsonc` 的 `agent` 段落定义需要的 agent（约 25 个），不放 `.md` 文件到 agents 目录。agent 通过命令 `agent:` 字段调用，不需要出现在模式菜单中。

### Bridge plugin output 防御

`tool.execute.after` handler 的 `output` 参数可能为 `undefined`（工具执行异常/被取消时）。必须加防御：

```typescript
"tool.execute.after": async (input, output?) => {
  if (!output) return;  // 防御 undefined
  const hasError = output.error != null;
  // ...
}
```

## 代理配置（Agent Configuration）

> 过程文档：[[sources/claude-code/opencode-auto-mode.md|OpenCode Auto Mode 实现过程]]

### 通过配置添加代理

OpenCode 支持纯配置添加代理，无需修改源码。配置文件：`~/.config/opencode/opencode.jsonc`

```jsonc
{
  "agent": {
    "auto": {
      "description": "Autonomous mode. Auto-approves safe operations, asks for dangerous ones.",
      "mode": "primary",
      "model": "opencode/minimax-m3-free",
      "tools": { "write": true, "edit": true, "bash": true, "read": true }
    }
  }
}
```

### 配置处理逻辑（agent.ts:221-248）

| 配置字段 | 类型 | 说明 |
|----------|------|------|
| `mode` | `"subagent" \| "primary" \| "all"` | 代理模式，`primary` 出现在切换列表 |
| `description` | string | 代理描述 |
| `model` | string | 使用的模型（`provider/model` 格式） |
| `prompt` | string | 系统提示词，支持 `{file:path}` 语法 |
| `permission` | object | 权限配置，与默认权限合并 |
| `hidden` | boolean | 是否隐藏（不出现在切换列表） |
| `disable` | boolean | 禁用代理（从列表删除） |
| `tools` | object | 已废弃，转为 permission |

**关键行为：**
- key 匹配内置代理（`build`、`auto`、`plan`）→ 合并覆盖
- key 是新名称 → 创建新代理（`mode: "all"`，`native: false`）
- `disable: true` → 删除代理

### TUI 代理过滤（local.tsx:38）

```typescript
sync.data.agent.filter((x) => x.mode !== "subagent" && !x.hidden)
```

只有 `mode !== "subagent"` 且 `!hidden` 的代理出现在切换列表。

### 源码修改 vs 配置

| 需求 | 方案 |
|------|------|
| 添加新代理 | 配置即可 |
| 覆盖内置代理 | 配置即可 |
| 自定义判断逻辑 | 需要源码修改（如 auto-judge.ts） |
| 修改权限流 | 需要源码修改（如 next.ts） |

## 引用索引

- `[[sources/claude-code/opencode-bridge-v2.md]]` — v2 升级完整过程
- `[[sources/claude-code/opencode-auto-mode.md]]` — Auto Mode 实现过程
- `[[sources/claude-code/ecc.md]]` — ECC 完整文档（包含 hooks JSON schema 参考）
- `[[sources/claude-code/ecc-opencode-install.md]]` — ECC-OpenCode 集成踩坑完整过程
- `[[sources/claude-code/settings-reference.md]]` — settings.json 字段参考
- `[[wiki/claude-code-p3.md]]` — Claude Code 进阶工作流（含 Hooks 配置范式）
- `[[wiki/dev-practice.md]]` — TDD/调试/代码质量基础
