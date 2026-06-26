# MiMo-Code 项目地图

> 这是一份帮助你快速理解 MiMo-Code 源码结构的导航文档。无论你是想了解某个功能的实现，还是想贡献代码，都可以从这里开始。

---

## 目录

- [项目概览](#项目概览)
- [Monorepo 结构](#monorepo-结构)
- [核心包：packages/opencode/src/](#核心包packagesopencodesrc)
- [核心模块导航](#核心模块导航)
- [常用代码搜索模式](#常用代码搜索模式)
- [推荐阅读顺序](#推荐阅读顺序)
- [相关文档索引](#相关文档索引)

---

## 项目概览

**MiMo Code**（MiMoCode）是小米开源的终端原生 AI 编程助手，基于 [OpenCode](https://github.com/anomalyco/opencode) 分叉而来。它能读写代码、运行命令、管理 Git，并通过持久化记忆系统跨会话理解你的项目。

| 项目 | 说明 |
|------|------|
| 语言 | TypeScript（严格模式） |
| 运行时 | [Bun](https://bun.sh) |
| TUI 框架 | [OpenTUI](https://github.com/nicholasgriffintn/opentui) + [SolidJS](https://www.solidjs.com/) |
| 包管理 | Bun Workspaces + Turborepo |
| 包名 | `@mimo-ai/cli` |
| CLI 命令 | `mimo` |

**与 OpenCode 的关系：** MiMoCode 保留了 OpenCode 的所有核心能力（多 Provider、TUI、LSP、MCP、插件），并新增了持久化记忆、智能上下文管理、子 Agent 编排、目标驱动自主循环、Compose 工作流和自我改进（Dream/Distill）。

---

## Monorepo 结构

```
MiMo-Code/
├── packages/                  # 所有包（monorepo 核心）
│   ├── opencode/              # 核心 CLI/TUI 包（最重要！）
│   ├── plugin/                # 插件 API 定义
│   ├── shared/                # 共享工具函数
│   ├── sdk/                   # JavaScript SDK + OpenAPI 定义
│   │   ├── js/                # JS SDK
│   │   └── openapi.json       # API Schema
│   ├── app/                   # Web 应用（SolidJS + Vite）
│   ├── desktop/               # 桌面应用（Electron）
│   ├── ui/                    # 共享 UI 组件
│   ├── console/               # 控制台/Web 面板
│   ├── web/                   # Web 相关
│   ├── enterprise/            # 企业版功能
│   ├── identity/              # 身份认证
│   ├── function/              # Serverless 函数
│   ├── extensions/            # IDE 扩展
│   ├── containers/            # 容器化部署
│   ├── slack/                 # Slack 集成
│   ├── script/                # 构建/工具脚本
│   └── storybook/             # UI 组件文档
│
├── docs/                      # 项目文档
├── docker/                    # Docker 配置
├── infra/                     # 基础设施（SST）
├── nix/                       # Nix 配置
├── script/                    # 根级脚本
├── sdks/                      # 其他 SDK
├── assets/                    # 静态资源
├── patches/                   # 依赖补丁
├── package.json               # 根配置（workspaces）
├── turbo.json                 # Turborepo 配置
├── sst.config.ts              # SST 基础设施配置
├── tsconfig.json              # TypeScript 配置
├── AGENTS.md                  # Agent 开发规范
└── CLAUDE.md                  # Claude Code 指令
```

---

## 核心包：packages/opencode/src/

这是整个项目的**心脏**，包含 CLI、TUI、Agent、Tool、Session 等所有核心逻辑。

```
packages/opencode/src/
├── index.ts                  # 主入口（yargs CLI 解析）
│
├── cli/                      # CLI 层
│   ├── bootstrap.ts          # 启动引导
│   ├── cmd/                  # 所有 CLI 子命令
│   │   ├── run.ts            # mimo run（非交互模式）
│   │   ├── serve.ts          # mimo serve（HTTP 服务器）
│   │   ├── models.ts         # mimo models
│   │   ├── providers.ts      # mimo providers
│   │   ├── mcp.ts            # mimo mcp
│   │   ├── agent.ts          # mimo agent
│   │   ├── session.ts        # mimo session
│   │   ├── db.ts             # mimo db
│   │   ├── upgrade.ts        # mimo upgrade
│   │   ├── export.ts         # mimo export
│   │   ├── import.ts         # mimo import
│   │   ├── github.ts         # mimo github
│   │   ├── pr.ts             # mimo pr
│   │   ├── plug.ts           # mimo plug（插件管理）
│   │   ├── acp.ts            # mimo acp
│   │   ├── debug/            # 调试命令
│   │   └── tui/              # TUI 界面（最大的子目录！）
│   │       ├── app.tsx       # TUI 根组件
│   │       ├── routes/       # 路由页面
│   │       ├── component/    # UI 组件
│   │       ├── context/      # SolidJS Context
│   │       ├── ui/           # 通用 UI 基础组件
│   │       ├── plugin/       # TUI 插件系统
│   │       ├── feature-plugins/ # 功能插件
│   │       ├── config/       # TUI 配置
│   │       ├── i18n/         # 国际化
│   │       ├── util/         # 工具函数
│   │       └── worker.ts     # Worker 线程
│   ├── effect/               # Effect 运行时封装
│   └── ...
│
├── agent/                    # Agent 系统
│   ├── agent.ts              # Agent 定义和管理
│   ├── config.ts             # Agent 配置
│   └── prompt/               # 系统提示词模板
│
├── session/                  # 会话系统（最复杂的模块之一）
│   ├── session.ts            # 会话核心逻辑
│   ├── message-v2.ts         # 消息格式（V2）
│   ├── checkpoint.ts         # 检查点/快照
│   ├── compaction.ts         # 上下文压缩
│   ├── processor.ts          # 消息处理器
│   ├── llm.ts                # LLM 调用
│   ├── prompt/               # 提示词管理
│   └── ...
│
├── tool/                     # 工具系统（~60 个文件）
│   ├── tool.ts               # 工具基础类型
│   ├── registry.ts           # 工具注册中心
│   ├── bash.ts               # Bash 工具
│   ├── read.ts               # 读文件
│   ├── write.ts              # 写文件
│   ├── edit.ts               # 编辑文件
│   ├── glob.ts               # 文件名搜索
│   ├── grep.ts               # 内容搜索
│   ├── webfetch.ts           # 网页抓取
│   ├── websearch/            # 网页搜索
│   ├── codesearch.ts         # 代码搜索
│   ├── actor.ts              # 子 Agent
│   ├── task.ts               # 任务管理
│   ├── skill.ts              # 技能调用
│   ├── workflow.ts           # 工作流
│   ├── lsp.ts                # LSP 集成
│   ├── memory.ts             # 内存操作
│   ├── question.ts           # 向用户提问
│   ├── apply_patch.ts        # 应用补丁
│   └── ...
│
├── provider/                 # LLM Provider 系统
│   ├── provider.ts           # Provider 核心（1788 行！）
│   ├── models.ts             # 模型定义
│   ├── schema.ts             # 类型定义
│   ├── auth.ts               # Provider 认证
│   ├── transform.ts          # 数据转换
│   └── sdk/                  # 各 Provider SDK 封装
│
├── config/                   # 配置系统
│   ├── config.ts             # 配置核心
│   ├── agent.ts              # Agent 配置
│   ├── provider.ts           # Provider 配置
│   ├── mcp.ts                # MCP 配置
│   ├── permission.ts         # 权限配置
│   ├── skills.ts             # 技能配置
│   ├── keybinds.ts           # 快捷键配置
│   └── ...
│
├── memory/                   # 持久化记忆系统
│   ├── index.ts              # 内存核心
│   ├── service.ts            # 内存服务
│   ├── fts-query.ts          # FTS5 全文搜索
│   ├── reconcile.ts          # 记忆协调
│   └── paths.ts              # 路径管理
│
├── skill/                    # 技能系统
│   ├── index.ts              # 技能核心
│   ├── discovery.ts          # 技能发现
│   ├── builtin/              # 内置技能
│   └── compose/              # Compose 技能（来自 superpowers/karpathy）
│
├── workflow/                 # 工作流系统
│   ├── runtime.ts            # 运行时
│   ├── sandbox.ts            # 沙箱执行
│   ├── builtin/              # 内置工作流
│   │   ├── compose.js        # Compose 工作流
│   │   └── deep-research.js  # 深度研究工作流
│   └── ...
│
├── mcp/                      # MCP 协议集成
│   ├── index.ts              # MCP 核心
│   ├── auth.ts               # MCP 认证
│   └── oauth-provider.ts     # OAuth 提供者
│
├── lsp/                      # LSP 集成
│   ├── client.ts             # LSP 客户端
│   ├── server.ts             # LSP 服务器
│   ├── language.ts           # 语言定义
│   └── diagnostic.ts         # 诊断信息
│
├── storage/                  # 存储层（SQLite + Drizzle ORM）
│   ├── db.ts                 # 数据库核心
│   ├── schema.ts             # Schema 定义
│   ├── schema.sql.ts         # SQL Schema
│   ├── storage.ts            # 存储接口
│   └── ...
│
├── permission/               # 权限系统
│   ├── schema.ts             # 权限 Schema
│   ├── evaluate.ts           # 权限评估
│   └── index.ts              # 权限核心
│
├── actor/                    # Actor 系统（子 Agent 编排）
│   ├── spawn.ts              # Actor 生成
│   ├── turn.ts               # Actor 回合
│   ├── registry.ts           # Actor 注册
│   └── schema.ts             # Actor Schema
│
├── effect/                   # Effect 运行时
│   ├── runtime.ts            # 运行时核心
│   ├── bridge.ts             # Effect 桥接
│   ├── instance-state.ts     # 实例状态
│   └── ...
│
├── project/                  # 项目管理
│   ├── project.ts            # 项目核心
│   ├── instance.ts           # 项目实例
│   ├── workspace-trust.ts    # 工作区信任
│   └── vcs.ts                # 版本控制
│
├── server/                   # HTTP 服务器
│   ├── server.ts             # 服务器核心
│   ├── routes/               # API 路由
│   ├── auth.ts               # 认证
│   └── ...
│
├── bus/                      # 事件总线
├── file/                     # 文件操作（ripgrep、watcher）
├── git/                      # Git 操作
├── command/                  # 命令模板
├── auth/                     # 认证系统
├── snapshot/                 # 快照系统
├── sync/                     # 同步系统
├── share/                    # 分享功能
├── inbox/                    # 收件箱
├── id/                       # ID 生成
├── flag/                     # 特性标志
├── util/                     # 工具函数
└── ...
```

---

## 核心模块导航

### 1. CLI 启动流程

**你想了解：** 用户输入 `mimo` 命令后发生了什么？

```
用户输入 mimo 命令
  │
  ▼
packages/opencode/src/index.ts          ← 主入口，yargs 解析命令
  │  （数据库迁移、Claude Code 会话导入）
  │
  ▼
packages/opencode/src/cli/cmd/tui/      ← 默认进入 TUI 模式
  │  （或 mimo run 进入非交互模式）
  │
  ▼
packages/opencode/src/cli/cmd/tui/app.tsx  ← TUI 根组件（SolidJS + OpenTUI）
  │
  ▼
packages/opencode/src/cli/cmd/tui/routes/  ← 路由页面
  ├── home.tsx     ← 首页（Logo + 输入框）
  └── session/     ← 会话页面（对话界面）
```

**关键文件：**
- `src/index.ts` — CLI 主入口（yargs）
- `src/cli/cmd/tui/app.tsx` — TUI 根组件
- `src/cli/cmd/tui/routes/home.tsx` — 首页
- `src/cli/cmd/tui/routes/session/index.tsx` — 会话页面

---

### 2. Agent 系统 (agent/)

**你想了解：** Agent 是什么？有哪些内置 Agent？

Agent 是 AI 的"人格"，定义了它能用什么工具、用什么提示词、有什么权限。

| Agent | 说明 |
|-------|------|
| **build** | 默认 Agent，完整工具权限，用于开发 |
| **plan** | 只读分析模式，用于代码探索和方案设计 |
| **compose** | 编排模式，用于规格驱动开发 |

```
src/agent/
├── agent.ts        # Agent 定义和管理（574 行）
├── config.ts       # Agent 配置
└── prompt/         # 系统提示词模板
    ├── explore.txt
    ├── dream.txt
    ├── distill.txt
    ├── summary.txt
    ├── compaction.txt
    └── title.txt
```

**关键文件：**
- `src/agent/agent.ts` — Agent 核心（`Info` Schema、`Interface`、`runtimePermission`）

---

### 3. Session 系统 (session/)

**你想了解：** 对话是怎么管理的？检查点是什么？

Session 是对话的核心容器，管理消息历史、检查点、上下文压缩等。

```
src/session/
├── session.ts              # 会话核心（908 行）
├── message-v2.ts           # 消息格式 V2
├── checkpoint.ts           # 检查点/快照系统
├── compaction.ts           # 上下文压缩
├── processor.ts            # 消息处理器
├── llm.ts                  # LLM 调用
├── goal.ts                 # 目标/停止条件
├── max-mode.ts             # Max Mode（并行推理）
├── prompt/                 # 提示词管理
│   └── ...
├── auto-dream.ts           # 自动 Dream
├── session.sql.ts          # 数据库 Schema
└── ...
```

**关键文件：**
- `src/session/session.ts` — 会话核心
- `src/session/checkpoint.ts` — 检查点系统
- `src/session/compaction.ts` — 上下文压缩

---

### 4. Tool 系统 (tool/)

**你想了解：** Agent 能做什么？（读文件、写文件、运行命令……）

每个工具都是一个独立的模块，定义了输入 Schema、执行逻辑和权限模型。

| 类别 | 工具 | 作用 |
|------|------|------|
| **文件** | `read.ts` | 读取文件 |
| | `write.ts` | 写入文件 |
| | `edit.ts` | 编辑文件 |
| | `apply_patch.ts` | 应用补丁 |
| **搜索** | `glob.ts` | 文件名模式匹配 |
| | `grep.ts` | 内容搜索（ripgrep） |
| | `codesearch.ts` | 代码搜索 |
| **网络** | `webfetch.ts` | 网页抓取 |
| | `websearch/` | 网页搜索 |
| **执行** | `bash.ts` | Shell 命令执行 |
| **Agent** | `actor.ts` | 子 Agent 生成 |
| | `task.ts` | 任务管理 |
| **扩展** | `skill.ts` | 技能调用 |
| | `workflow.ts` | 工作流 |
| | `lsp.ts` | LSP 集成 |
| | `mcp-exa.ts` | MCP Exa 搜索 |
| **交互** | `question.ts` | 向用户提问 |
| | `memory.ts` | 内存操作 |
| | `plan.ts` | 计划模式切换 |

**关键文件：**
- `src/tool/tool.ts` — 工具基础类型（`Def`、`Context`、`ExecuteResult`）
- `src/tool/registry.ts` — 工具注册中心

---

### 5. Provider 系统 (provider/)

**你想了解：** 支持哪些 LLM？怎么切换模型？

Provider 系统管理所有 LLM 提供商的连接和模型选择。

```
src/provider/
├── provider.ts       # Provider 核心（1788 行！）
├── models.ts         # 模型定义
├── schema.ts         # 类型定义（ModelID、ProviderID）
├── auth.ts           # Provider 认证
├── transform.ts      # 数据转换
└── sdk/              # 各 Provider SDK 封装
```

**支持的 Provider：**
- Anthropic (Claude)
- OpenAI (GPT)
- Google (Gemini)
- Amazon Bedrock
- Azure
- xAI (Grok)
- Groq
- Cohere
- Mistral
- DeepInfra
- Together AI
- Perplexity
- OpenRouter
- Alibaba (通义)
- MiMo (小米)
- 自定义 OpenAI 兼容 API

**关键文件：**
- `src/provider/provider.ts` — Provider 核心逻辑

---

### 6. Config 系统 (config/)

**你想了解：** 配置文件怎么写？有哪些配置项？

配置通过 `.mimocode/mimocode.json`（项目级）或 `~/.config/mimocode/mimocode.json`（全局）管理。

```
src/config/
├── config.ts         # 配置核心
├── agent.ts          # Agent 配置
├── provider.ts       # Provider 配置
├── model-id.ts       # 模型 ID
├── mcp.ts            # MCP 配置
├── permission.ts     # 权限配置
├── skills.ts         # 技能配置
├── keybinds.ts       # 快捷键
├── layout.ts         # 布局配置
├── lsp.ts            # LSP 配置
├── parse.ts          # 配置解析
├── paths.ts          # 路径配置
└── ...
```

**关键文件：**
- `src/config/config.ts` — 配置核心

---

### 7. Memory 系统 (memory/)

**你想了解：** 怎么让 AI 记住之前的对话？

持久化记忆系统基于 SQLite FTS5 全文搜索，支持跨会话记忆。

```
src/memory/
├── index.ts          # 内存核心
├── service.ts        # 内存服务
├── fts-query.ts      # FTS5 全文搜索查询
├── reconcile.ts      # 记忆协调（去重、合并）
└── paths.ts          # 路径管理
```

**记忆类型：**
- **项目记忆** (`MEMORY.md`) — 持久化项目知识、规则、架构决策
- **会话检查点** (`checkpoint.md`) — 结构化状态快照
- **临时笔记** (`notes.md`) — Agent 的临时记录
- **任务进度** (`tasks/<id>/progress.md`) — 每个任务的日志

**关键文件：**
- `src/memory/service.ts` — 内存服务
- `src/memory/fts-query.ts` — FTS5 搜索

---

### 8. Skill 系统 (skill/)

**你想了解：** 技能是什么？和工具、命令有什么区别？

技能是可复用的工作流程，可以是内置的或用户自定义的。

```
src/skill/
├── index.ts              # 技能核心
├── discovery.ts          # 技能发现
├── builtin/              # 内置技能
│   ├── .bundle/          # 打包的技能
│   └── bundle.macro.ts   # 打包宏
└── compose/              # Compose 技能
    ├── .bundle/          # 打包的技能
    ├── bundle.macro.ts   # 打包宏
    ├── LICENSE-karpathy  # karpathy 技能许可
    └── LICENSE-superpowers  # superpowers 技能许可
```

**关键文件：**
- `src/skill/index.ts` — 技能核心
- `src/skill/discovery.ts` — 技能发现

---

### 9. Workflow 系统 (workflow/)

**你想了解：** 工作流是什么？怎么执行？

工作流是更复杂的自动化流程，可以在沙箱中执行。

```
src/workflow/
├── runtime.ts        # 运行时
├── sandbox.ts        # 沙箱执行
├── builtin.ts        # 内置工作流注册
├── builtin/          # 内置工作流
│   ├── compose.js    # Compose 工作流
│   └── deep-research.js  # 深度研究工作流
├── events.ts         # 事件系统
├── persistence.ts    # 持久化
├── resolve.ts        # 解析
└── ...
```

**关键文件：**
- `src/workflow/runtime.ts` — 运行时核心
- `src/workflow/sandbox.ts` — 沙箱执行

---

### 10. TUI 层 (cli/cmd/tui/)

**你想了解：** 终端界面是怎么渲染的？

TUI 基于 SolidJS + OpenTUI 构建，使用响应式 UI 模式。

```
src/cli/cmd/tui/
├── app.tsx               # TUI 根组件（1140 行）
├── routes/               # 路由页面
│   ├── home.tsx          # 首页
│   └── session/          # 会话页面
│       ├── index.tsx     # 会话主页面（3061 行！）
│       ├── sidebar.tsx   # 侧边栏
│       ├── footer.tsx    # 底部栏
│       ├── permission.tsx # 权限对话框
│       ├── question.tsx  # 问题对话框
│       └── ...
├── component/            # UI 组件（39 个）
│   ├── prompt/           # 输入框组件
│   │   ├── index.tsx     # 输入框核心
│   │   ├── autocomplete.tsx # 自动补全
│   │   └── ...
│   ├── dialog-*.tsx      # 各种对话框
│   ├── logo.tsx          # Logo 组件
│   ├── spinner.tsx       # 加载动画
│   ├── task-item.tsx     # 任务项
│   ├── todo-item.tsx     # 待办项
│   └── ...
├── context/              # SolidJS Context（19 个）
│   ├── sdk.tsx           # SDK 连接
│   ├── project.tsx       # 项目信息
│   ├── theme.tsx         # 主题
│   ├── keybind.tsx       # 快捷键
│   ├── route.tsx         # 路由
│   ├── event.ts          # 事件
│   ├── sync.tsx          # 同步
│   ├── language.tsx      # 国际化
│   └── ...
├── ui/                   # 通用 UI 基础组件（10 个）
│   ├── dialog.tsx        # 对话框基础
│   ├── dialog-alert.tsx  # 警告对话框
│   ├── dialog-confirm.tsx # 确认对话框
│   ├── toast.tsx         # Toast 提示
│   └── ...
├── plugin/               # TUI 插件系统
├── feature-plugins/      # 功能插件
│   ├── home/             # 首页插件
│   ├── sidebar/          # 侧边栏插件
│   └── system/           # 系统插件
├── config/               # TUI 配置
├── i18n/                 # 国际化
├── util/                 # 工具函数（17 个）
│   ├── clipboard.ts      # 剪贴板
│   ├── editor.ts         # 编辑器
│   ├── voice.ts          # 语音
│   ├── vad.ts            # 语音活动检测
│   └── ...
└── worker.ts             # Worker 线程
```

**关键文件：**
- `src/cli/cmd/tui/app.tsx` — TUI 根组件
- `src/cli/cmd/tui/routes/session/index.tsx` — 会话页面（3061 行）
- `src/cli/cmd/tui/context/sdk.tsx` — SDK 连接

---

### 11. MCP 集成 (mcp/)

**你想了解：** MCP（Model Context Protocol）是怎么工作的？

```
src/mcp/
├── index.ts          # MCP 核心
├── auth.ts           # MCP 认证
├── oauth-callback.ts # OAuth 回调
└── oauth-provider.ts # OAuth 提供者
```

**关键文件：**
- `src/mcp/index.ts` — MCP 核心

---

### 12. LSP 集成 (lsp/)

**你想了解：** 语言服务器协议是怎么集成的？

```
src/lsp/
├── client.ts         # LSP 客户端
├── server.ts         # LSP 服务器
├── language.ts       # 语言定义（支持的语言列表）
├── diagnostic.ts     # 诊断信息
├── launch.ts         # 启动器
└── lsp.ts            # LSP 核心
```

**关键文件：**
- `src/lsp/client.ts` — LSP 客户端
- `src/lsp/language.ts` — 支持的语言

---

### 13. Storage 层 (storage/)

**你想了解：** 数据是怎么持久化的？

基于 SQLite + Drizzle ORM 的存储层。

```
src/storage/
├── db.ts             # 数据库核心
├── db.bun.ts         # Bun 运行时数据库
├── db.node.ts        # Node 运行时数据库
├── schema.ts         # Schema 定义
├── schema.sql.ts     # SQL Schema
├── storage.ts        # 存储接口
├── read-sqlite.ts    # SQLite 读取
└── json-migration.ts # JSON 迁移
```

**关键文件：**
- `src/storage/db.ts` — 数据库核心

---

### 14. Permission 系统 (permission/)

**你想了解：** 权限控制是怎么做的？

```
src/permission/
├── index.ts          # 权限核心
├── schema.ts         # 权限 Schema
├── evaluate.ts       # 权限评估
└── arity.ts          # 权限元数
```

**关键文件：**
- `src/permission/evaluate.ts` — 权限评估逻辑

---

### 15. Actor 系统 (actor/)

**你想了解：** 子 Agent 是怎么管理的？

```
src/actor/
├── spawn.ts          # Actor 生成
├── turn.ts           # Actor 回合
├── registry.ts       # Actor 注册
├── schema.ts         # Actor Schema
├── events.ts         # 事件系统
├── waiter.ts         # 等待器
└── return-header.ts  # 返回头
```

**关键文件：**
- `src/actor/spawn.ts` — Actor 生成

---

### 16. Effect 运行时 (effect/)

**你想了解：** 项目为什么用 Effect？它是什么？

[Effect](https://effect.website/) 是一个 TypeScript 函数式编程框架，提供依赖注入、错误处理、并发控制等能力。

```
src/effect/
├── runtime.ts            # 运行时核心
├── bridge.ts             # Effect 桥接
├── instance-state.ts     # 实例状态
├── instance-registry.ts  # 实例注册
├── app-runtime.ts        # 应用运行时
├── bootstrap-runtime.ts  # 引导运行时
└── ...
```

**关键文件：**
- `src/effect/runtime.ts` — Effect 运行时核心

---

### 17. Server 层 (server/)

**你想了解：** HTTP 服务器是怎么工作的？

```
src/server/
├── server.ts         # 服务器核心
├── adapter.bun.ts    # Bun 适配器
├── adapter.node.ts   # Node 适配器
├── routes/           # API 路由
├── auth.ts           # 认证
├── event.ts          # 事件系统
├── middleware.ts      # 中间件
└── ...
```

**关键文件：**
- `src/server/server.ts` — 服务器核心

---

### 18. 构建与发布

**你想了解：** 项目是怎么构建和发布的？

```bash
# 安装依赖
bun install

# 开发模式
bun run dev

# 类型检查
bun turbo typecheck

# 构建
bun run --cwd packages/opencode script/build.ts

# 测试（从包目录运行）
cd packages/opencode && bun test
```

**构建工具：**
- [Bun](https://bun.sh) — 运行时和包管理器
- [Turborepo](https://turbo.build/) — Monorepo 构建编排
- [Drizzle ORM](https://orm.drizzle.team/) — 数据库 ORM
- [OpenTUI](https://github.com/nicholasgriffintn/opentui) — TUI 框架
- [SolidJS](https://www.solidjs.com/) — UI 框架
- [Effect](https://effect.website/) — 函数式编程框架

---

## 常用代码搜索模式

当你想找特定功能时，这些命令很有用：

```bash
# 找到所有工具定义
rg "export.*Tool" packages/opencode/src/tool/

# 找到所有 Agent 定义
rg "Info = z" packages/opencode/src/agent/

# 找到所有 Session 操作
rg "export.*Session" packages/opencode/src/session/

# 找到所有 Provider
rg "providerID" packages/opencode/src/provider/

# 找到所有权限检查
rg "permission" packages/opencode/src/permission/

# 找到所有 SolidJS 组件
rg "^export function" packages/opencode/src/cli/cmd/tui/component/

# 找到所有 Context
rg "createSimpleContext" packages/opencode/src/cli/cmd/tui/context/

# 找到所有 Effect 服务
rg "extends Context.Service" packages/opencode/src/

# 找到所有数据库 Schema
rg "sqliteTable" packages/opencode/src/

# 找到所有配置项
rg "z\." packages/opencode/src/config/
```

---

## 推荐阅读顺序

根据你的目标，选择不同的学习路径：

### 路径 1：快速了解项目全貌（30 分钟）

1. 读 `README.md` — 了解项目背景
2. 读本文档 — 建立整体认知
3. 读 `AGENTS.md` — 了解开发规范
4. 读 `packages/opencode/src/index.ts` — 理解 CLI 入口

### 路径 2：理解 TUI 系统（1 小时）

1. 读 `src/cli/cmd/tui/app.tsx` — TUI 根组件
2. 读 `src/cli/cmd/tui/routes/home.tsx` — 首页
3. 读 `src/cli/cmd/tui/routes/session/index.tsx` 前 200 行 — 会话页面
4. 读 `src/cli/cmd/tui/context/sdk.tsx` — SDK 连接

### 路径 3：理解工具系统（1 小时）

1. 读 `src/tool/tool.ts` — 工具基础类型
2. 读 `src/tool/bash.ts` — 一个简单工具的实现
3. 读 `src/tool/registry.ts` — 工具注册
4. 读 `src/permission/evaluate.ts` — 权限评估

### 路径 4：理解 Agent 和 Session（1 小时）

1. 读 `src/agent/agent.ts` — Agent 定义
2. 读 `src/session/session.ts` — 会话核心
3. 读 `src/session/checkpoint.ts` — 检查点系统
4. 读 `src/session/compaction.ts` — 上下文压缩

### 路径 5：理解 Provider 和 LLM 调用（1 小时）

1. 读 `src/provider/provider.ts` — Provider 核心
2. 读 `src/provider/models.ts` — 模型定义
3. 读 `src/session/llm.ts` — LLM 调用
4. 读 `src/session/processor.ts` — 消息处理

### 路径 6：理解记忆和技能系统（1 小时）

1. 读 `src/memory/service.ts` — 内存服务
2. 读 `src/memory/fts-query.ts` — FTS5 搜索
3. 读 `src/skill/index.ts` — 技能核心
4. 读 `src/workflow/runtime.ts` — 工作流运行时

---

## 相关文档索引

| 文档 | 路径 | 内容 |
|------|------|------|
| README | [README.md](../README.md) | 项目介绍、快速开始 |
| 中文 README | [README.zh.md](../README.zh.md) | 中文版项目介绍 |
| 开发规范 | [AGENTS.md](../AGENTS.md) | Agent 开发规范、代码风格 |
| 贡献指南 | [CONTRIBUTING.md](../CONTRIBUTING.md) | 如何贡献代码 |
| 安全政策 | [SECURITY.md](../SECURITY.md) | 安全报告 |
| 使用限制 | [USE_RESTRICTIONS.md](../USE_RESTRICTIONS.md) | 使用限制条款 |
| Compose 文档 | [docs/compose/](compose/) | Compose 模式文档 |
| SDK OpenAPI | [packages/sdk/openapi.json](../packages/sdk/openapi.json) | API Schema |

---

> 最后更新：2026-06-27
