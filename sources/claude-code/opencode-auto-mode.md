# OpenCode Auto Mode 实现过程

## 背景

需要在 OpenCode 中实现 "auto 模式"——一个长时间无人监管的自主执行模式。核心思路：通过两阶段 LLM 判断机制，自动放行安全操作，仅对危险操作请求用户确认。

## 实现步骤

### 1. 配置添加代理（最简方案）

在 `~/.config/opencode/opencode.jsonc` 的 `agent` 部分添加：

```jsonc
"auto": {
  "description": "Autonomous mode. Auto-approves safe operations, asks for dangerous ones.",
  "mode": "primary",
  "model": "opencode/minimax-m3-free",
  "tools": {
    "write": true,
    "edit": true,
    "bash": true,
    "read": true,
    "changed-files": true
  }
}
```

重启 OpenCode 后，`auto` 代理出现在切换列表中。

### 2. 源码修改（自定义判断逻辑）

如果需要自动判断功能，需要修改源码：

#### 2.1 新建 `packages/opencode/src/permission/auto-judge.ts`

两阶段判断模块：
- Stage 1: 规则分类器（正则匹配 safe/dangerous/unknown）
- Stage 2: LLM 推理判断（generateObject，temperature:0）

#### 2.2 修改 `packages/opencode/src/permission/service.ts`

`Request` schema 添加 `agent?: string` 字段。

#### 2.3 修改 `packages/opencode/src/permission/next.ts`

`ask` 函数添加 AutoJudge 拦截逻辑。

#### 2.4 修改 `packages/opencode/src/session/prompt.ts`

`ctx.ask` 传递 `agent: input.agent.name`。

#### 2.5 修改 `packages/opencode/src/agent/agent.ts`

添加 `auto` 代理定义（可选，配置已覆盖）。

#### 2.6 TUI 增强

- `permission.tsx`: 权限对话框显示 "Auto-mode: judged dangerous"
- `footer.tsx`: 添加 `⚡ AUTO` 指示器

### 3. SDK 类型更新

- `packages/sdk/js/src/v2/gen/types.gen.ts`: `PermissionRequest` 添加 `agent?: string`
- `packages/sdk/openapi.json`: `PermissionRequest` schema 添加 `agent` 字段

## 关键发现

1. **代理配置优先级**：配置 > 内置定义。配置中的 `agent` 部分会合并覆盖内置代理。
2. **TUI 过滤条件**：`mode !== "subagent" && !hidden` 的代理才会出现在切换列表。
3. **AI SDK 参数**：`generateObject` 使用 `maxOutputTokens` 而非 `maxTokens`。
4. **依赖问题**：`tsgo` 需要 `@typescript/native-preview-win32-x64` 包，标准 `tsc` 无法替代。

## 验证结果

- 配置添加 auto 代理：✅ 成功
- 代理出现在切换列表：✅ 成功
- typecheck 通过：⚠️ 预存问题（effect 模块依赖），无新增错误
