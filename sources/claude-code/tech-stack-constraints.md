# Claude Code 源码技术栈约束（源码验证版）

> 基于 v2.1.88 源码逐条验证

Claude Code 借鉴了 Ink 的组件 API 设计（Box/Text/ScrollBox 等），但整个渲染后端都是自研的 — 包括 Reconciler、Yoga 绑定、cell buffer、diff 引擎、输出管线。

目的是极致性能（cell 级 intern、blit 优化、最小 ANSI 补丁）和功能扩展（文本选择、搜索高亮、bidi 支持、OSC 8 超链接等 Ink 原版没有的能力）。

---

## 一、运行时与语言

| 约束 | 值 | 源码依据 |
|------|------|---------|
| Node.js | >= 18.0.0 | `package.json` engines |
| 模块系统 | ESM (`"type": "module"`) | `package.json` |
| TypeScript | target ES2022, strict: false, module ESNext, bundler resolution | `tsconfig.json` |
| 包管理 | pnpm + npm 双支持 | `pnpm-lock.yaml` + `package-lock.json` 共存 |
| 构建（原始） | Bun 编译时内联（`feature()`、`MACRO`、`bun:bundle`） | `scripts/build.mjs` 注释 |
| 构建（重建） | esbuild `--platform=node --target=node18 --format=esm --bundle` | `scripts/build.mjs` |

---

## 二、核心框架依赖

| 框架 | 版本 | 说明 | 源码依据 |
|------|------|------|---------|
| **React** | ^19.2.7 | UI 层核心，所有组件、hooks 基础 | `package.json` |
| **Ink** | ^7.1.0 | 声明在 `package.json` 但运行时不导入；`src/ink/` 是自研替代实现，保留 Ink 组件 API 风格 | `package.json`、`src/ink/` |
| react-dom | ^19.2.7 | React DOM 支持 | `package.json` |
| react-reconciler | ^0.33.0 | 自定义 reconciler 驱动终端渲染 | `src/ink/reconciler.ts` |
| Commander.js | ^13.1.0 | CLI 参数解析，配合 `@commander-js/extra-typings` | `package.json` |
| Zod | ^4.4.3 | Schema 验证，工具输入用 `z.strictObject` | `src/tools/` |
| lodash-es | ^4.18.1 | ESM-only 工具库 | `package.json` |
| chalk | ^5.6.2 | ESM-only 行内样式 | `package.json` |
| strip-ansi | ^7.2.0 | 可见字符宽度计算、光标定位 | `package.json` |
| usehooks-ts | ^3.1.1 | React hooks 工具集 | `package.json` |

---

## 三、UI 渲染架构（React + 自研渲染引擎）

### 3.1 渲染管线

Claude Code 的终端 UI 基于 **React + 自研 Ink-like 渲染引擎**构建。`ink` npm 包虽然在 `package.json` 中声明，但运行时**从不导入**——`src/ink/` 是完整的自研实现，保留 Ink 的组件 API 设计但替换了全部渲染后端：

```
React 组件树 (<Box>, <Text>, <ThemedText>...)
  ↓ JSX 编译
src/ink/reconciler.ts (自定义 react-reconciler, 直接对接 React 19)
  ↓ Fiber commit → DOM 树
src/ink/dom.ts (ink-text / ink-box 节点, textStyles 结构体)
  ↓ onComputeLayout
src/ink/layout/yoga.ts (Yoga flexbox 计算位置/尺寸)
  ↓ 遍历 DOM 树
src/ink/render-node-to-output.ts (1462行: squish → wrap → applyTextStyles → 写入 Output)
  ↓
src/ink/output.ts → src/ink/screen.ts (cell buffer + CharPool/StylePool intern)
  ↓ 前后帧对比
src/ink/log-update.ts (cell 级 diff, DECSTBM 滚动优化)
  ↓
src/ink/optimizer.ts (合并光标移动, 丢弃空操作)
  ↓
src/ink/terminal.ts → writeDiffToTerminal()
  ↓
process.stdout.write()
```

关键自研模块说明：
- **`render-node-to-output.ts`**（1462行）：核心绘制器，遍历 DOM 树，将 `ink-text` 节点的文本 squish 成 `StyledSegment[]`，应用换行/截断，通过 `applyTextStyles()` 转成 ANSI，写入 Output
- **`screen.ts`**（1486行）：cell 级缓冲区，`CharPool`/`StylePool` 做字符串 intern 以最小化内存和加速 diff 比较
- **`log-update.ts`**（773行）：cell 级前后帧 diff，支持 DECSTBM 滚动优化和 BSU/ESU 原子更新
- **`optimizer.ts`**：diff 后优化，合并连续光标移动、丢弃空操作

### 3.2 关键目录

| 目录 | 职责 |
|------|------|
| `src/ink/` | 自研渲染引擎（reconciler、DOM、screen、diff、输出管线，51个文件，替代 Ink 运行时） |
| `src/ink/components/` | 基础 UI 原语（Box、Text、Button、Link、ScrollBox 等） |
| `src/components/` | 业务组件（数百个 `.tsx` 文件） |
| `src/hooks/` | React hooks（状态、副作用、键盘、语音等） |
| `src/context/` | React Context providers |
| `src/screens/` | 页面级组件（REPL、Doctor、ResumeConversation） |

### 3.3 渲染模式

- **默认模式**：Inline TUI，行级增量重绘，不使用备用屏
- **备用屏模式**：支持 `ENTER_ALT_SCREEN` / `EXIT_ALT_SCREEN`（`src/ink/termio/dec.ts`）
- `src/ink/components/AlternateScreen.tsx` 提供备用屏组件
- 两种模式共存，非互斥

### 3.4 `process.stdout.write` 的实际用途

`process.stdout.write` 在 22 个文件中使用，但**不是主渲染循环**，而是：
- 桥接层输出（`src/bridge/`）
- 流式原始输出（`src/cli/print.ts`）
- 调试/诊断边界
- 主渲染由自研引擎的 `writeDiffToTerminal()` 驱动（`src/ink/terminal.ts`）

---

## 四、工具系统架构

### 4.1 定义模式

- 核心接口 `Tool<Input, Output, P>` 含约 25-28 个方法
- 使用 `buildTool()` 构建器模式，`TOOL_DEFAULTS` 提供默认值
- Schema 用 `lazySchema(() => z.strictObject({...}))` 延迟构建

### 4.2 注册与发现

- `getAllBaseTools()` 硬编码所有内置工具数组（`src/tools.ts:193`）
- `process.env.USER_TYPE === 'ant'` 控制内部专属工具
- `feature('FLAG')` 控制实验性工具（编译时替换为 `false`）
- `assembleToolPool()` 合并内置工具 + MCP 工具（`src/tools.ts:345`）

### 4.3 权限检查链

工具执行前的三层校验：
1. `validateInput` — 输入合法性
2. `checkPermissions` — 工具特定权限逻辑
3. `ToolPermissionContext` — 通用权限规则

### 4.4 结果序列化

- `mapToolResultToToolResultBlockParam` 将 Output 转为 API 格式
- `maxResultSizeChars` 系统级上限 50000（`DEFAULT_MAX_RESULT_SIZE_CHARS`），各工具可单独声明更低值（BashTool 30K、GrepTool 20K），FileReadTool 设为 Infinity；超阈值自动持久化到磁盘

---

## 五、服务层与 MCP

### 5.1 MCP 客户端

- 3 种标准传输：StdioClientTransport、SSEClientTransport、StreamableHTTPClientTransport
- `InProcessTransport`（`src/services/mcp/InProcessTransport.ts`）：同进程内 `queueMicrotask` 异步双向通信
- 7 种配置作用域：local / user / project / dynamic / enterprise / claudeai / managed
- 完整 OAuth 2.0 流程

### 5.2 API 服务层

- 多云客户端工厂：Anthropic 直连 / AWS Bedrock / Azure Foundry / Vertex AI
- 统一 Anthropic SDK 接口（`@anthropic-ai/sdk ^0.107.0`）
- 重试策略：默认 10 次，529 过载最多 3 次

---

## 六、权限与安全模型

### 6.1 权限模式

`default | plan | acceptEdits | bypassPermissions | dontAsk | auto`

### 6.2 Auto 模式

- 两级快速路径：acceptEdits 快速通道 + safe-tool 白名单
- AI 分类器 `classifyYoloAction()` 做最终裁决
- 连续 3 次拒绝或累计 20 次拒绝 → 强制回退手动审批

### 6.3 沙箱

- 封装 `@anthropic-ai/sandbox-runtime`（bubblewrap / Seatbelt）
- 文件系统四象限：allowWrite / denyWrite / allowRead / denyRead
- 始终 denyWrite 所有 settings.json 路径

### 6.4 安全存储

- macOS：Keychain 优先 + 明文降级
- Linux/Windows：纯明文 `.credentials.json`，文件权限 `chmod 0o600`

---

## 七、构建系统

### 7.1 原始构建（Bun）

Claude Code 原始使用 Bun 的编译时内联机制：
- `feature('X')` → 编译时求值，`false` 时触发死代码消除
- `MACRO.VERSION` 等替换为字符串字面量
- `bun:bundle` 提供模块内联

### 7.2 开源重建（esbuild）

`scripts/build.mjs` 提供近似重建，5 阶段：
1. copy src → build-src
2. transform（`feature()` → `false`，MACRO 替换，bun:bundle 移除）
3. create entry wrapper
4. scan imports + 自动 stub 缺失模块
5. esbuild bundle → dist/cli.js

esbuild 参数：`--platform=node --target=node18 --format=esm --bundle`

### 7.3 启动入口

- CLI 入口 → 主入口，大量 fast-path（`--version` / `daemon` / `bridge` 等）
- 仅非 fast-path 时动态 import 主逻辑
- 顶层副作用：`profileCheckpoint`、`startMdmRawRead`、`startKeychainPrefetch`

---

## 八、关键代码模式

| 模式 | 约束 | 源码依据 |
|------|------|---------|
| Schema 定义 | `lazySchema(() => z.strictObject({...}))` | `src/entrypoints/sdk/coreSchemas.ts` |
| Feature Flag | `feature('FLAG_NAME')` 编译时替换 | `scripts/build.mjs` |
| 延迟加载 | `await import()` 为主，`require()` 用于同步场景 | `src/cli/print.ts` 等 |
| 终端渲染 | React + 自研 Ink-like 渲染引擎，`ink` npm 包运行时不使用 | `src/ink/`（51个自研文件）、`src/components/` |
| 副作用标记 | 顶层副作用需 `eslint-disable` + 原因说明 | 源码惯例 |
| 遥测埋点 | `profileCheckpoint('name')` + `logEvent()` | `src/bootstrap/state.ts` |
| 错误处理 | `toError()` 规范化 + `logError()` 记录 | `src/utils/log.ts` |
| 日志 | 优先 `logForDebugging` / `logForDiagnosticsNoPII`，`console.log` 在边界场景存在 | `src/bridge/bridgeMain.ts` |
| 结果持久化 | `maxResultSizeChars` 超阈值自动写磁盘 | `src/tools.ts` |

---

## 九、注意事项（非绝对禁止）

| 事项 | 实际约束 | 源码现实 |
|------|---------|---------|
| React/Ink | **核心依赖**，`ink` npm 包声明但运行时不导入；`src/ink/` 是自研替代 | React 250+ 文件 import；Ink 零运行时导入 |
| `require()` | **在用**，用于同步延迟加载 | `src/cli/print.ts` 等 20+ 处 |
| `console.log` | 优先用 `logForDebugging`，但边界场景允许 | `src/bridge/bridgeMain.ts` |
| localStorage/sessionStorage | Node.js 环境无此 API，自然不存在 | N/A |
| 备用屏 | 支持但非默认，inline 为主模式 | `src/ink/components/AlternateScreen.tsx` |
| `Array.toSorted` | 需兼容 Node 18，用 copy-then-sort | 运行时约束 |

---

## 十、依赖清单（核心）

```
运行时:
  react ^19.2.7          react-dom ^19.2.7       react-reconciler ^0.33.0
  ink ^7.1.0             (声明但运行时不导入)    usehooks-ts ^3.1.1      commander ^13.1.0
  zod ^4.4.3             lodash-es ^4.18.1       chalk ^5.6.2
  strip-ansi ^7.2.0      @anthropic-ai/sdk ^0.107.0
  @anthropic-ai/bedrock-sdk ^0.31.0               @anthropic-ai/vertex-sdk ^0.18.0
  @anthropic-ai/foundry-sdk ^0.4.0                sharp ^0.35.2

开发:
  esbuild ^0.27.4        typescript ^6.0.2
```

---
