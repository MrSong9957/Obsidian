---
name: Claude Code 项目地图
description: Claude Code 源码结构导航——目录总览、核心入口、按功能模块导航与推荐阅读顺序
type: reference
updated: 2026-06-27
tags: [claude-code, source-map, navigation]
source: self
---

# Claude Code 项目地图

> 这是一份帮助你快速理解 Claude Code 源码结构的导航文档。无论你是想了解某个功能的实现，还是想贡献代码，都可以从这里开始。

---

## 目录

- [项目概览](#项目概览)
- [目录结构总览](#目录结构总览)
- [核心入口文件](#核心入口文件)
- [按功能模块导航](#按功能模块导航)
- [常用代码搜索模式](#常用代码搜索模式)
- [推荐阅读顺序](#推荐阅读顺序)
- [相关文档索引](#相关文档索引)

---

## 项目概览

**Claude Code** 是 Anthropic 官方的 CLI（命令行）AI 编程助手。它能直接在终端里帮你编辑文件、运行命令、搜索代码、管理 Git 等。

| 项目 | 说明 |
|------|------|
| 语言 | TypeScript（严格模式） |
| 运行时 | [Bun](https://bun.sh)（不是 Node.js） |
| 终端 UI | [React](https://react.dev) + [Ink](https://github.com/vadimdemedes/ink) |
| 代码规模 | ~1,900 个文件，512,000+ 行代码 |
| 包名 | `@anthropic-ai/claude-code` |

---

## 目录结构总览

```
claude-code-source-code-1/
├── src/                    # 所有源代码（核心）
│   ├── main.tsx            # CLI 主入口（Commander.js + React/Ink）
│   ├── QueryEngine.ts      # LLM API 调用引擎（~46K 行，最核心的文件）
│   ├── Tool.ts             # 工具类型定义（~29K 行）
│   ├── commands.ts         # 命令注册中心（~25K 行）
│   ├── tools.ts            # 工具注册中心
│   ├── context.ts          # 系统/用户上下文收集
│   ├── cost-tracker.ts     # Token 成本追踪
│   │
│   ├── tools/              # ~40 个 Agent 工具的实现
│   ├── commands/           # ~85 个斜杠命令的实现
│   ├── components/         # ~140 个 Ink UI 组件
│   ├── hooks/              # React Hooks（含权限检查）
│   ├── screens/            # 全屏 UI 界面
│   ├── services/           # 外部服务集成
│   ├── types/              # TypeScript 类型定义
│   ├── utils/              # 工具函数（329 个文件！）
│   │
│   ├── bridge/             # IDE 集成（VS Code、JetBrains）
│   ├── coordinator/        # 多 Agent 编排
│   ├── plugins/            # 插件系统
│   ├── skills/             # 技能系统
│   ├── tasks/              # 任务管理
│   ├── server/             # 服务器模式
│   ├── remote/             # 远程会话
│   ├── memdir/             # 持久化内存目录
│   ├── state/              # 状态管理
│   ├── query/              # 查询管道
│   │
│   ├── voice/              # 语音输入
│   ├── vim/                # Vim 模式
│   ├── keybindings/        # 快捷键配置
│   ├── schemas/            # 配置 Schema（Zod）
│   ├── migrations/         # 配置迁移
│   ├── entrypoints/        # 初始化入口
│   ├── ink/                # Ink 渲染器封装
│   ├── context/            # React Context 定义
│   ├── buddy/              # 伴侣精灵（彩蛋 🐣）
│   ├── native-ts/          # 原生 TypeScript 工具
│   ├── outputStyles/       # 输出样式
│   └── upstreamproxy/      # 代理配置
│
├── docs/                   # 项目文档
├── tests/                  # 测试（集成 + 冒烟）
├── scripts/                # 构建脚本
├── mcp-server/             # MCP 探索服务器
├── docker/                 # Docker 配置
├── grafana/                # Grafana 监控面板
├── helm/                   # Kubernetes Helm Chart
├── prompts/                # 提示词模板
├── web/                    # Web 相关资源
├── package.json            # 项目配置
├── tsconfig.json           # TypeScript 配置
├── biome.json              # Linter/Formatter 配置
├── vitest.config.ts        # 测试框架配置
├── drizzle.config.ts       # 数据库 ORM 配置
└── Dockerfile              # Docker 构建文件
```

---

## 核心入口文件

这几个文件是理解整个项目的关键起点：

| 文件 | 行数 | 作用 | 类比 |
|------|------|------|------|
| `src/main.tsx` | — | CLI 主入口，Commander.js 解析命令行参数，初始化 React/Ink 渲染器 | 一个 App 的 `main()` 函数 |
| `src/entrypoints/cli.tsx` | 303 | 引导入口，处理快速路径（如 `--version`），然后加载 `main.tsx` | 启动器/引导程序 |
| `src/QueryEngine.ts` | ~46K | 核心 LLM 引擎：流式响应、工具调用循环、重试、Token 计数 | 项目的心脏 |
| `src/Tool.ts` | ~29K | 所有工具的基础类型和接口定义 | 工具的"模具" |
| `src/commands.ts` | ~25K | 命令注册中心，管理所有斜杠命令 | 命令的"通讯录" |
| `src/tools.ts` | — | 工具注册中心，管理所有 Agent 工具 | 工具的"通讯录" |
| `src/context.ts` | 190 | 收集系统上下文（Git 状态、CLAUDE.md 等） | 环境信息收集器 |
| `src/cost-tracker.ts` | — | 追踪 Token 使用量和费用 | 账单计算器 |

---

## 按功能模块导航

### 1. CLI 启动流程

**你想了解：** 用户输入 `claude` 命令后发生了什么？

```
用户输入 claude 命令
  │
  ▼
src/entrypoints/cli.tsx          ← 引导入口，快速路径分流
  │  （检查 --version、--daemon、--bridge 等特殊参数）
  │
  ▼
src/main.tsx                     ← 主入口，Commander.js 解析所有参数
  │  （并行预取：MDM 设置、Keychain、GrowthBook）
  │
  ▼
src/entrypoints/init.ts          ← 初始化：配置、遥测、OAuth、MDM 策略
  │
  ▼
src/replLauncher.tsx             ← 启动 REPL 交互界面
  │
  ▼
src/screens/REPL.tsx             ← 主 REPL 屏幕（你看到的那个界面）
  │
  ▼
src/QueryEngine.ts               ← 等待用户输入，调用 LLM API
```

**关键文件：**
- `src/entrypoints/cli.tsx` — 引导入口
- `src/main.tsx` — 主入口（很长，~1000+ 行）
- `src/replLauncher.tsx` — REPL 启动器
- `src/screens/REPL.tsx` — REPL 界面

---

### 2. 工具系统 (tools/)

**你想了解：** Claude 能做什么？（读文件、写文件、运行命令……）

每个工具都是一个独立的目录，结构如下：

```
src/tools/BashTool/
├── BashTool.ts      ← 核心执行逻辑
├── UI.tsx           ← 终端里的渲染效果
├── prompt.ts        ← 系统提示词中关于这个工具的描述
└── index.ts         ← 导出入口
```

**所有工具一览：**

| 类别 | 工具 | 作用 |
|------|------|------|
| **文件 I/O** | `FileReadTool` | 读取文件（支持图片、PDF） |
| | `FileWriteTool` | 创建/覆盖文件 |
| | `FileEditTool` | 部分修改（字符串替换） |
| | `NotebookEditTool` | 编辑 Jupyter Notebook |
| **搜索** | `GlobTool` | 文件名模式匹配 |
| | `GrepTool` | 基于 ripgrep 的内容搜索 |
| | `WebSearchTool` | 网页搜索 |
| | `WebFetchTool` | 获取网页内容 |
| **执行** | `BashTool` | 运行 Shell 命令 |
| | `PowerShellTool` | 运行 PowerShell 命令 |
| | `SkillTool` | 执行技能 |
| | `MCPTool` | 调用 MCP 服务器工具 |
| | `LSPTool` | 语言服务器协议集成 |
| **Agent** | `AgentTool` | 生成子 Agent |
| | `SendMessageTool` | Agent 间消息传递 |
| | `TeamCreateTool` | 创建团队 |
| | `TaskCreateTool` | 创建任务 |
| **模式** | `EnterPlanModeTool` | 进入计划模式 |
| | `ExitPlanModeTool` | 退出计划模式 |
| | `EnterWorktreeTool` | 进入 Git Worktree 隔离环境 |
| | `SleepTool` | 主动等待模式 |
| | `ToolSearchTool` | 延迟工具发现 |
| | `CronCreateTool` | 定时触发器 |

**关键文件：**
- `src/Tool.ts` — 工具类型定义（`buildTool()` 工厂函数）
- `src/tools.ts` — 工具注册中心
- `src/tools/` — 所有工具实现

---

### 3. 命令系统 (commands/)

**你想了解：** 在 REPL 里输入 `/xxx` 会发生什么？

命令有三种类型：

| 类型 | 说明 | 例子 |
|------|------|------|
| **PromptCommand** | 发送格式化提示词给 LLM | `/review`、`/commit` |
| **LocalCommand** | 在进程内运行，返回纯文本 | `/cost`、`/version` |
| **LocalJSXCommand** | 在进程内运行，返回 React JSX | `/doctor`、`/install` |

**常用命令：**

| 命令 | 作用 |
|------|------|
| `/commit` | Git 提交 |
| `/review` | 代码审查 |
| `/compact` | 压缩上下文 |
| `/mcp` | MCP 服务器管理 |
| `/config` | 设置 |
| `/doctor` | 环境诊断 |
| `/memory` | 持久化内存 |
| `/skills` | 技能管理 |
| `/tasks` | 任务管理 |
| `/diff` | 查看变更 |
| `/cost` | 查看使用费用 |
| `/vim` | Vim 模式切换 |

**关键文件：**
- `src/commands.ts` — 命令注册中心
- `src/commands/` — 所有命令实现（102 个目录/文件）

---

### 4. 服务层 (services/)

**你想了解：** Claude Code 如何与外部系统交互？

```
src/services/
├── api/                  # Anthropic API 客户端
├── mcp/                  # MCP 协议连接管理
├── oauth/                # OAuth 2.0 认证
├── lsp/                  # 语言服务器协议管理
├── analytics/            # GrowthBook 特性标志 & 分析
├── plugins/              # 插件加载器
├── compact/              # 对话上下文压缩
├── extractMemories/      # 自动内存提取
├── teamMemorySync/       # 团队内存同步
├── tokenEstimation.ts    # Token 数量估算
├── policyLimits/         # 组织策略限制
├── remoteManagedSettings/ # 远程管理设置
├── tips/                 # 使用提示
└── ...
```

**关键文件：**
- `src/services/api/` — API 调用核心
- `src/services/mcp/` — MCP 协议实现
- `src/services/oauth/` — 认证系统

---

### 5. UI 层 (components/ + screens/ + hooks/)

**你想了解：** 终端界面是怎么渲染出来的？

Claude Code 用 **React + Ink** 构建终端 UI，和写网页一样！

```
src/screens/
├── REPL.tsx                # 主交互界面（你最常看到的）
├── Doctor.tsx              # 环境诊断界面
└── ResumeConversation.tsx  # 会话恢复界面

src/components/             # ~140 个 UI 组件
├── App.tsx                 # 根组件
├── Messages.tsx            # 消息列表
├── Message.tsx             # 单条消息
├── PromptInput/            # 输入框
├── Markdown.tsx            # Markdown 渲染
├── HighlightedCode.tsx     # 代码高亮
├── diff/                   # 差异对比显示
├── design-system/          # 设计系统基础组件
└── ...

src/hooks/                  # ~85 个 React Hooks
├── toolPermission/         # 权限检查 Hooks
├── useTextInput.ts         # 输入处理
├── useVimInput.ts          # Vim 模式输入
├── useIDEIntegration.tsx   # IDE 集成
├── useVoice.ts             # 语音输入
└── ...
```

**关键文件：**
- `src/screens/REPL.tsx` — 主界面
- `src/components/App.tsx` — 根组件
- `src/hooks/` — 所有 Hooks

---

### 6. IDE 桥接 (bridge/)

**你想了解：** Claude Code 如何与 VS Code / JetBrains 集成？

```
src/bridge/
├── bridgeMain.ts                  # 主循环
├── bridgeMessaging.ts             # 消息协议
├── bridgePermissionCallbacks.ts   # 权限回调
├── replBridge.ts                  # REPL 会话桥接
├── jwtUtils.ts                    # JWT 认证
├── sessionRunner.ts               # 会话执行
└── ...
```

**关键文件：**
- `src/bridge/bridgeMain.ts` — 桥接主循环
- `src/bridge/bridgeMessaging.ts` — 消息协议

---

### 7. MCP 集成

**你想了解：** MCP（Model Context Protocol）是怎么工作的？

MCP 是一个开放协议，让 AI 模型能调用外部工具。Claude Code 同时是 MCP **客户端**和**服务器**。

```
作为客户端：                    作为服务器：
src/services/mcp/              src/entrypoints/mcp.ts
├── client.ts                  └── 让其他 MCP 客户端调用 Claude Code
├── config.ts
├── types.ts
└── ...

调用 MCP 工具：
src/tools/MCPTool/
src/tools/ListMcpResourcesTool/
src/tools/ReadMcpResourceTool/
```

**关键文件：**
- `src/services/mcp/` — MCP 客户端
- `src/entrypoints/mcp.ts` — MCP 服务器入口
- `src/tools/MCPTool/` — MCP 工具调用

---

### 8. 插件系统 (plugins/)

**你想了解：** 如何扩展 Claude Code 的功能？

```
src/plugins/
├── builtinPlugins.ts       # 内置插件列表
└── bundled/                # 打包的插件代码

src/services/plugins/       # 插件加载和管理
```

**关键文件：**
- `src/plugins/builtinPlugins.ts` — 内置插件
- `src/services/plugins/` — 插件加载器

---

### 9. 技能系统 (skills/)

**你想了解：** 技能（Skill）是什么？和工具、命令有什么区别？

技能是可复用的工作流程，定义在 `skills/` 目录中，通过 `SkillTool` 执行。

```
src/skills/
├── bundled/                # 内置技能
│   ├── batch.ts
│   ├── claudeApi.ts
│   ├── debug.ts
│   ├── remember.ts
│   ├── verify.ts
│   └── ...
├── bundledSkills.ts        # 技能注册
├── loadSkillsDir.ts        # 技能加载器
└── mcpSkillBuilders.ts     # 从 MCP 构建技能
```

**关键文件：**
- `src/skills/bundled/` — 内置技能
- `src/skills/bundledSkills.ts` — 技能注册

---

### 10. 任务管理 (tasks/)

**你想了解：** 子 Agent 和任务是怎么管理的？

```
src/tasks/
├── DreamTask/                  # Dream 任务
├── InProcessTeammateTask/      # 进程内队友任务
├── LocalAgentTask/             # 本地 Agent 任务
├── LocalMainSessionTask.ts     # 本地主会话任务
├── LocalShellTask/             # 本地 Shell 任务
├── RemoteAgentTask/            # 远程 Agent 任务
├── types.ts                    # 任务类型定义
└── stopTask.ts                 # 停止任务

src/coordinator/
└── coordinatorMode.ts          # 多 Agent 编排模式
```

**关键文件：**
- `src/tasks/` — 任务实现
- `src/coordinator/` — Agent 编排

---

### 11. 内存/持久化 (memdir/)

**你想了解：** Claude Code 如何记住之前的对话？

```
src/memdir/
├── memdir.ts               # 内存目录核心
├── findRelevantMemories.ts # 查找相关记忆
├── memoryScan.ts           # 记忆扫描
├── memoryTypes.ts          # 记忆类型定义
├── memoryAge.ts            # 记忆年龄管理
├── paths.ts                # 路径管理
├── teamMemPaths.ts         # 团队记忆路径
└── teamMemPrompts.ts       # 团队记忆提示词
```

**关键文件：**
- `src/memdir/memdir.ts` — 内存目录核心

---

### 12. 认证与安全

**你想了解：** 用户登录和权限控制是怎么做的？

```
认证：
src/services/oauth/           # OAuth 2.0 认证
src/utils/auth.ts             # 认证工具函数
src/utils/secureStorage/      # 安全存储（Keychain）

权限控制：
src/hooks/toolPermission/     # 工具权限检查
src/utils/permissions/        # 权限工具函数
  ├── PermissionMode.ts       # 权限模式定义
  └── permissionSetup.ts      # 权限初始化
```

**关键文件：**
- `src/services/oauth/` — OAuth 认证
- `src/hooks/toolPermission/` — 权限检查
- `src/utils/permissions/` — 权限工具

---

### 13. 配置与迁移

**你想了解：** 配置是怎么管理的？版本升级时怎么迁移？

```
配置 Schema：
src/schemas/
└── hooks.ts                  # Hooks 配置 Schema

配置迁移：
src/migrations/
├── migrateAutoUpdatesToSettings.ts
├── migrateBypassPermissionsAcceptedToSettings.ts
├── migrateSonnet45ToSonnet46.ts
└── ... (11 个迁移文件)

配置工具：
src/utils/config.ts           # 全局配置
src/utils/settings/           # 设置管理
src/utils/claudemd.ts         # CLAUDE.md 文件处理
```

**关键文件：**
- `src/utils/config.ts` — 全局配置
- `src/utils/settings/` — 设置管理
- `src/migrations/` — 配置迁移

---

### 14. 遥测与分析

**你想了解：** 使用数据是怎么收集的？

```
src/services/analytics/
├── growthbook.ts             # GrowthBook 特性标志
├── index.ts                  # 分析事件记录
├── sink.ts                   # 分析数据接收器
└── config.ts                 # 分析配置

src/utils/telemetry/          # 遥测工具
src/services/policyLimits/    # 组织策略限制
```

**关键文件：**
- `src/services/analytics/` — 分析服务

---

### 15. 语音功能 (voice/)

**你想了解：** 语音输入是怎么实现的？

```
src/voice/
└── voiceModeEnabled.ts       # 语音模式开关

src/services/
├── voice.ts                  # 语音服务
├── voiceKeyterms.ts          # 语音关键词
└── voiceStreamSTT.ts         # 流式语音转文字
```

**关键文件：**
- `src/services/voice.ts` — 语音服务核心

---

### 16. 构建与发布

**你想了解：** 项目是怎么构建和发布的？

```bash
# 构建命令（package.json 中定义）
bun run build          # 开发构建
bun run build:prod     # 生产构建（压缩）
bun run build:web      # Web 版本构建

# 代码质量
bun run typecheck      # TypeScript 类型检查
bun run lint           # Biome Linter 检查
bun run check          # 类型 + Lint 一起跑

# 测试
bun run test           # 运行测试
bun run test:watch     # 监听模式测试

# 发布
bun run package:npm    # 打包 npm 包
```

**构建工具：**
- [Bun](https://bun.sh) — 运行时和包管理器
- [esbuild](https://esbuild.github.io/) — 打包器
- [Biome](https://biomejs.dev/) — Linter + Formatter
- [Vitest](https://vitest.dev/) — 测试框架
- [Drizzle ORM](https://orm.drizzle.team/) — 数据库 ORM

**关键文件：**
- `scripts/build-bundle.ts` — 构建脚本
- `biome.json` — Linter 配置
- `vitest.config.ts` — 测试配置

---

## 常用代码搜索模式

当你想找特定功能时，这些 grep 命令很有用：

```bash
# 找到所有工具定义
rg "buildTool\(" src/tools/

# 找到所有命令定义
rg "satisfies Command" src/commands/

# 找到特性标志使用
rg "feature\(" src/

# 找到所有 React Hooks
rg "^export function use" src/hooks/

# 找到所有 Zod Schema
rg "z\.object\(" src/schemas/

# 找到系统提示词贡献
rg "prompt\(" src/tools/*/prompt.ts

# 找到权限检查模式
rg "checkPermissions" src/tools/

# 找到所有 API 调用
rg "query\(" src/services/api/

# 找到所有事件记录
rg "logEvent\(" src/

# 找到所有错误处理
rg "logError\(" src/
```

---

## 推荐阅读顺序

根据你的目标，选择不同的学习路径：

### 路径 1：快速了解项目全貌（30 分钟）

1. 读 `README.md` — 了解项目背景
2. 读本文档 — 建立整体认知
3. 读 `src/entrypoints/cli.tsx` — 理解启动流程
4. 读 `src/main.tsx` 前 200 行 — 理解初始化

### 路径 2：理解工具系统（1 小时）

1. 读 `src/Tool.ts` 前 200 行 — 理解 `buildTool()` 接口
2. 读 `src/tools/FileReadTool/` — 一个简单工具的完整实现
3. 读 `src/QueryEngine.ts` 搜索 "tool" — 理解工具调用循环
4. 读 `src/hooks/toolPermission/` — 理解权限系统

### 路径 3：理解 UI 系统（1 小时）

1. 读 `src/screens/REPL.tsx` — 主界面
2. 读 `src/components/App.tsx` — 根组件
3. 读 `src/hooks/useTextInput.ts` — 输入处理
4. 读 `src/ink/ARCHITECTURE.md` — Ink 渲染器架构

### 路径 4：理解 IDE 集成（1 小时）

1. 读 `src/bridge/bridgeMain.ts` — 桥接主循环
2. 读 `src/bridge/bridgeMessaging.ts` — 消息协议
3. 读 `src/bridge/bridgePermissionCallbacks.ts` — 权限回调
4. 读 `src/bridge/replBridge.ts` — REPL 会话桥接

### 路径 5：理解 MCP 和插件（1 小时）

1. 读 `src/services/mcp/` — MCP 客户端实现
2. 读 `src/tools/MCPTool/` — MCP 工具调用
3. 读 `src/entrypoints/mcp.ts` — MCP 服务器入口
4. 读 `src/plugins/builtinPlugins.ts` — 内置插件
5. 读 `src/skills/bundled/` — 内置技能

---

## 相关文档索引

| 文档 | 路径 | 内容 |
|------|------|------|
| 架构详解 | [docs/architecture.md](architecture.md) | 核心管道、状态管理、UI 层、构建系统 |
| 工具目录 | [docs/tools.md](tools.md) | 所有 ~40 个工具的完整目录 |
| 命令目录 | [docs/commands.md](commands.md) | 所有 ~85 个斜杠命令 |
| 子系统指南 | [docs/subsystems.md](subsystems.md) | Bridge、MCP、权限、插件、技能、任务、内存、语音 |
| 探索指南 | [docs/exploration-guide.md](exploration-guide.md) | 如何浏览代码库 |
| Bridge 文档 | [docs/bridge.md](bridge.md) | IDE 桥接系统 |
| 目录结构 | [docs/directory-structure.md](directory-structure.md) | 目录结构详细说明 |
| 贡献指南 | [CONTRIBUTING.md](../CONTRIBUTING.md) | 如何贡献代码 |
| MCP Server | [mcp-server/README.md](../mcp-server/README.md) | MCP 探索服务器使用说明 |

---

> 最后更新：2026-06-27
