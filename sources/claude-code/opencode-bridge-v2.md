---
name: OpenCode 桥接 v2 升级过程
description: opencode-claude-bridge 从 v1（只读观察器）升级到 v2（读写控制器）的完整过程、调研发现、探针验证方法、部署脚本设计
type: summary
updated: 2026-06-02
tags: opencode, claude-code, hooks, plugin, bridge, deploy, probe
source: https://github.com/magarcia/opencode-claude-hooks
---

# OpenCode 桥接 v2 升级过程

> 配套结论文档：[[wiki/opencode-bridge.md|OpenCode 桥接关键发现]]
> 部署文档：`E:\Files\Code\opencode-claude-bridge\PROBE_REPORT.md` + `MIGRATION_NOTES.md`

## 背景

用户在 2026-06-02 从 Claude Code 迁移到 OpenCode。OpenCode 的 plugin 体系与 Claude Code 不兼容：
- 无 plugin marketplace
- 无 agent teams
- 大量 Claude Code hooks 不触发

自研 `opencode-claude-bridge` v1 解决了一部分问题（28 个 hook 桥接 + 6 个事件），但只是"只读观察器"：
- hook stdout JSON 完全忽略
- PreToolUse 退出码 2 不响应
- `log()` 早返回导致 SessionStart 静默
- 配置每次 tool call 读盘

## 调研：参考项目

参考 magarcia/opencode-claude-hooks（同期唯一活跃的同类项目）。

**发现的 4 个隐藏 bug**：
1. `prompt` hook 类型是空头 — TypeScript 类型定义有，但实际不处理
2. `permission.ask` 拼写错误 — 应是 `permission.ask` → `permission.asked`
3. 字符串错误检测：`if (typeof x === 'object')` 而非 `if (typeof x !== 'string')`
4. `dist/` 目录不存在 — README 引用 `dist/index.js` 但项目用 tsx/esbuild noEmit

**为什么不切换 magarcia**：
- 4 个隐藏 bug + 1 commit/4 月不活跃
- 12 个事件中 6 个我们用不到
- 我们仓库已有 90+ 单元测试，是真资产

**决策**：保留自研，移植 magarcia 核心能力到我们的 v2。

## v2 升级步骤

### 1. 修复 v1 已知 bug

`log()` 函数原代码：
```typescript
const log = async (level, message, extra) => {
  if (!ctx.client) return;        // ← 早返回
  try {
    await ctx.client.app.log({ ... });
  } catch {}
};
```

修复为：
```typescript
const log = async (level, message, extra) => {
  if (ctx.client) {
    try {
      await ctx.client.app.log({ ... });
    } catch {}
  }
  console.log(`[claude-code-hooks] ${level}: ${message}`);  // ← 永远执行
};
```

### 2. 移植 magarcia 核心能力

新增 4 个函数：
- `loadSettingsWithCache(home)` + `clearSettingsCache()`：60s TTL 配置缓存
- `parseHookOutput(stdout)`：解析 hook 返回的 Claude Code 风格 JSON
- `aggregateResults(results)`：浅合并多 hook 结果
- `dispatch(event, tool, payload, hooks, env)`：async 返回 AggregatedHookResult

### 3. 真正读写 output.args

`tool.execute.before` handler（核心改动）：
```typescript
const result = await dispatch("PreToolUse", input.tool, payload, settings.hooks, baseEnv);
if (result.updatedInput) {
  output.args = { ...output.args, ...result.updatedInput };  // ← 真正改写
}
if (result.permissionDecision === "deny") {
  output.args = { ...output.args, _blocked: true, _blockReason: ... };
}
```

### 4. 新增事件

- `experimental.session.compacting`：注入 `output.context` (PreCompact equivalent)
- `permission.asked`：处理 OpenCode 权限请求，返回 `output.status` (allow/deny/ask)

### 5. 部署脚本

`scripts/deploy.mjs`：
- 原子化（`copyFile` → `rename`，无中间状态）
- 支持 `DEPLOY_TARGET_DIR` 环境变量（便于测试）
- 4 个退出码：1=源缺失/空，2=目录创建失败，3=复制/重命名失败

### 6. 探针验证（三层）

不依赖 OpenCode 真实调用，直接 import 部署文件：

- **L1 静态**：grep 14 关键代码段（14/14 通过）
- **L2 动态-基础**：mock OpenCode plugin + 触发 `tool.execute.before`（CLI/TUI 双模式）
- **L2 动态-E2E**：mock `CLAUDE_SETTINGS_PATH` settings + 真 hook 命令（输出 JSON）
- **L3 真实 session**：`~/AppData/Local/Temp/ecc-metrics-*.json` mtime 证据

## 探针 E2E 关键证据

```text
[e2e] before output.args: {"command":"echo v2-e2e-test"}
[e2e] after  output.args: {"command":"echo v2-e2e-test","newField":"v2-was-here"}
[e2e] newField value: v2-was-here
```

`updatedInput` 经 `parseHookOutput` → `aggregateResults` → 合并入 `output.args` 完整链路已端到端验证。

## 部署链

```bash
cd E:\Files\Code\opencode-claude-bridge
npm test            # 99/99
npm run typecheck   # 0 errors
npm run deploy      # 原子化复制 src/index.ts → ~/.config/opencode/plugins/
```

部署后**必须重启 OpenCode** 才能加载新版本。

## 探针脚本保留

`tests/probes/` 4 个 .mjs 脚本 + README：
- `probe-v2.mjs`：基础探针（CLI/TUI）
- `probe-v2-e2e.mjs`：端到端探针
- `inspect-hooks.mjs`：检视 `~/.claude/settings.json` 结构
- `inspect-recent.mjs`：检视 ecc-metrics mtime

`npm run test:probes` 一键跑前 2 个。

## 后续工作

- 考虑在 OpenCode 升级后做兼容测试
- magarcia 4 个 bug 已记录但未修（保留决策）
- 可能扩展：把 `parser.ts`/`runner.ts`/`payload.ts` 合并到 `index.ts`（单文件更易部署）

## 文件清单

| 文件 | 改动 |
|---|---|
| `src/index.ts` | 332→515 行（log 修复、stdout 解析、aggregateResults、output.args 写回、permission.asked、配置缓存）|
| `tests/dispatch.test.ts` | 199→326 行（+4 测试）|
| `tests/deploy.test.ts` | 新建（+5 测试）|
| `scripts/deploy.mjs` | 新建（2293 bytes）|
| `tests/probes/*.mjs` | 4 个探针脚本（从根目录迁入）|
| `~/.config/opencode/plugins/claude-code-hooks.ts` | 部署：12033→16714 bytes（v2）|
| `~/.config/opencode/plugins/claude-code-hooks.ts.bak-20260602` | v1 备份（保留）|
| `~/.config/opencode/MIGRATION_NOTES.md` | 164→278 行 |
| `PROBE_REPORT.md` | 新建（5695 bytes 验证报告）|
