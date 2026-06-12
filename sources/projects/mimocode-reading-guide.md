---
name: MiMo-Code 源码阅读指南
description: MiMo-Code（OpenCode fork）源码阅读路线，参考 Learn ShareAI s01-s19 课程结构，从最小闭环逐步扩展到多 Agent 平台
type: reference
updated: 2026-06-13
tags: [mimocode, opencode, typescript, monorepo, architecture, agent-loop]
---

# MiMo-Code 源码阅读指南

> 项目定位：基于 OpenCode fork 的 AI 编程智能体，具有跨会话记忆功能。
> 仓库地址：`E:\Files\GitHub\MiMo-Code`
> 参考课程：[Learn ShareAI — Learn Claude Code](https://learn.shareai.run/zh/s01/)

## 阅读路线

按**从最小闭环逐步扩展**的顺序组织为 4 个模块 19 章。
每一章对应 Learn ShareAI 的同名章节，标注 MiMo-Code 中的具体文件和关键函数。

### 模块一：核心闭环（先搭最小可用 agent）

> 没有循环，就没有 agent。先把 `messages → model → tool_result → next turn` 这条回路搭起来。

#### s01 Agent 循环

**核心概念**：while(true) 主循环——发请求→处理工具→结果写回→下一轮。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `session/prompt.ts` | `runLoop` (L1698-2959) | 主循环入口，while(true) 处理消息→LLM→工具→结果写回 |
| `session/processor.ts` | `create` (L181-941) | 流处理器：消费 LLM stream，处理 tool-call/tool-result/text 事件 |
| `session/llm.ts` | `run` (L298-696), `stream` (L645-696) | 调用 AI SDK streamText()，带指数退避重试 |
| `session/session.ts` | `create` (L577-596), `messages` (L682-695) | 会话生命周期，消息存取 |
| `cli/cmd/run.ts` | `execute` (L397-671) | CLI 入口：订阅事件→启动 loop→发送 prompt |

**消息流**：`用户输入 → SessionPrompt.prompt() → runLoop() → LLM.stream() → processor.handleEvent() → tool 执行 → 结果写回 messages → 下一轮`

#### s02 工具使用

**核心概念**：工具注册表 + dispatch map，主循环不关心具体工具实现。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `tool/tool.ts` | `define()`, `init()` | 工具核心类型：Def = id + description + parameters + execute |
| `tool/registry.ts` | `Service` (L1-413) | 注册中心：21 个内建工具 + 插件工具 + MCP 工具，按模型/provider 过滤 |
| `tool/bash.ts` | `execute` | bash 工具：Tree-sitter 解析、权限、子进程、输出截断 |
| `tool/read.ts` | `execute` | read 工具：目录/文件读取、二进制检测、图片 PDF |
| `tool/write.ts` | `execute` | write 工具：写入 + 格式化 + LSP 诊断 |
| `tool/edit.ts` | `execute` | edit 工具：局部替换、9 种容错匹配策略、Semaphore 并发锁 |

#### s03 待办写入

**核心概念**：会话级任务追踪，防止多步任务漂移。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `task/registry.ts` | `create`, `start`, `done`, `block`, `unblock` | 任务注册表：SQLite 持久化，树状 ID（T1, T1.1） |
| `task/gate.ts` | `TaskGate.decide()` | 完成门：actor 结束前检查遗留任务，最多 3 次 ReAct |
| `task/schema.ts` | `Task`, `TaskStatus` | 状态机：open → in_progress → done/abandoned/blocked |
| `task/events.ts` | `TaskCreated`, `TaskUpdated` | 任务总线事件 |

#### s04 子代理

**核心概念**：子 agent 用干净上下文处理子任务，摘要返回父 agent。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `actor/spawn.ts` | `spawn`, `cancel` (L1-741) | Actor 核心：spawnPeer（子 session）或 spawnSubagent（共享父 session） |
| `actor/registry.ts` | `register`, `get`, `listBySession` | Actor 注册表：SQLite 持久化，孤儿恢复，卡住检测（5 分钟） |
| `actor/waiter.ts` | `wait` (L1-166) | Actor 等待器：Bus 事件订阅 + Deferred 竞争，带超时 |
| `actor/turn.ts` | `runTurn()` | Turn 包装器：uninterruptible 外壳保证状态清理 |
| `actor/schema.ts` | `Actor`, `ActorStatus`, `SpawnMode` | 类型定义：status=pending/running/idle, mode=peer/subagent |

#### s05 技能系统

**核心概念**：先轻发现（目录列表）再深加载（SKILL.md 正文注入）。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `skill/index.ts` | `get`, `all`, `dirs`, `available` | 技能核心：扫描 compose bundle + 外部目录 + 配置路径 + 远程 URL |
| `skill/discovery.ts` | `pull` | 远程技能发现：从 URL 拉取 index.json，下载到本地缓存 |
| `skill/compose/extract.ts` | `extractComposeBundle()` | 内建技能 bundle：Bun macro 编译时打包 ~15 个技能 |

#### s06 上下文压缩

**核心概念**：大结果落盘 + 旧结果占位 + 整体摘要，保持活跃上下文小而稳。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `session/compaction.ts` | `prune`, `process`, `create` (L1-543) | 压缩核心：prune 回溯擦除旧工具输出，process 生成摘要保留尾部 |
| `session/checkpoint.ts` | `tryStartCheckpointWriter` (L500-908) | 检查点：生成 checkpoint-writer 子 agent 维护 checkpoint.md |
| `session/checkpoint.ts` | `renderRebuildContext` (L1008-1133) | 上下文重建：从 checkpoint + memory + tasks + notes 注入 |

**关键常量**：`PRUNE_MINIMUM=20_000`, `PRUNE_PROTECT=40_000`, `DEFAULT_TAIL_TURNS=2`

---

### 模块二：系统加固（安全与可观测）

> 核心闭环能跑之后，加安全闸门、扩展点、记忆、输入管线、容错。

#### s07 权限系统

**核心概念**：工具调用前必过权限管道，三层规则合并（config + session + 用户批准）。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `permission/index.ts` | `ask`, `reply`, `evaluate` | 权限核心：ask 发布事件阻塞等待，reply 处理 once/always/reject |
| `permission/evaluate.ts` | `evaluate()` | 纯函数规则匹配：从 rulesets 中 findLast 匹配，默认返回 ask |
| `permission/arity.ts` | `BashArity.prefix()` | 命令前缀词典（~160 条），从 shell 命令提取人类可读命令名 |

#### s08 Hook 系统

**核心概念**：不改主循环也能扩展，hook 按注册顺序串行执行。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `plugin/index.ts` | `trigger`, `triggerActorPreStop`, `triggerActorPostStop` | 插件核心：加载内部/外部插件，钩子串行执行 |
| `plugin/index.ts` | `aggregateDecision()` (L364-458) | 聚合 actor stop 钩子：收集 continue=true + reason |
| `plugin/matcher.ts` | `matchesActor()` | Actor 匹配器：按 agentType/mode/lifecycle 过滤触发 |

**内置插件**：MimoAuth, CodexAuth, CopilotAuth, GitlabAuth, CheckpointSplitover, SubagentProgressChecker

#### s09 记忆系统

**核心概念**：SQLite FTS5 全文搜索，四层记忆（global/projects/sessions/cc）。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `memory/service.ts` | `search`, `reconcile` | 记忆核心：BM25 排序 + 相对分数阈值过滤，双向同步磁盘↔索引 |
| `memory/reconcile.ts` | `reconcileMemory()`, `walkMemoryDir()` | 索引协调：索引新文件、修剪已删除文件的 FTS 行 |
| `memory/paths.ts` | `parsePath()`, `buildPath()` | 路径解析：scope 分 global/projects/sessions/cc |
| `memory/fts-query.ts` | `buildFtsQuery()` | FTS5 查询构建：tokenize + 短语引用 + OR 连接 |

#### s10 系统提示词

**核心概念**：多来源逐段组装——agent prompt + system + memory + skills。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `session/llm.ts` | `buildSystemArray` (L234-296) | 构建 system prompt 数组：agent prompt + custom + user + memory |
| `agent/prompt/` | 各 .txt 文件 | Agent 专属 prompt 模板（build/plan/compose/compaction 等） |
| `agent/agent.ts` | Agent 定义 (L112-369) | 11 个内置 agent，每个有权限规则、mode、可选 prompt |

#### s11 错误恢复

**核心概念**：截断续写 / 上下文压缩重试 / 指数退避重试。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `session/llm.ts` | `persistentRetrySchedule` (L74-79) | 指数退避：500ms × 2，上限 5min，最多 10 次 |
| `session/compaction.ts` | `isOverflow` | 溢出检测：触发上下文压缩后重试 |
| `session/processor.ts` | `handleEvent` 中的 error 分支 | 流处理错误：retry/overflow/error 三种恢复路径 |

---

### 模块三：任务运行时（持久化与异步）

> 会话内 todo 升级为可持久化的依赖关系图，慢命令后台跑。

#### s12 任务系统

**核心概念**：树状任务 ID（T1, T1.1），状态机 open→in_progress→done/abandoned/blocked。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `task/registry.ts` | 全文件 | SQLite 持久化，create/list/get/start/block/unblock/done/abandon/rename |
| `task/gate.ts` | `TaskGate.decide()` | 完成门：ReAct 循环防止遗留任务，主 agent 最多 3 次 |
| `task/gate-state.ts` | `get`, `bump`, `clear` | 每 session 的 ReAct 计数器 |
| `task/schema.ts` | `Task`, `TaskStatus`, `TaskEvent` | 类型定义 |

#### s13 后台任务

**核心概念**：子 agent 后台执行，主循环不阻塞。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `actor/spawn.ts` | `spawnSubagent` | 子 agent 在共享父 session 中执行，后台不阻塞主循环 |
| `actor/waiter.ts` | `wait` | 带超时的 actor 完成等待 |
| `actor/registry.ts` | `listActive`, `updateStatus` | 活跃 actor 追踪，卡住检测 |

#### s14 定时调度

**核心概念**：MiMo-Code 无独立 cron，通过 `/dream` 和 `/distill` 命令手动触发。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `agent/agent.ts` | `dream`, `distill` agent 定义 | dream：扫描会话提取知识到记忆；distill：发现重复工作流打包成 skill |
| `cli/cmd/` | `/dream`, `/distill` 命令 | 手动触发，非自动定时 |

---

### 模块四：多 Agent 平台（协作与扩展）

> 从单 agent 扩展到专职角色团队、结构化协作、自主认领、外部能力接入。

#### s15 Agent 团队

**核心概念**：build/plan/compose/explore/checkpoint-writer 等 11 个专职角色。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `agent/agent.ts` | Agent 定义 (L112-369) | 11 个内置 agent：build, plan, compose, general, explore, title, summary, compaction, checkpoint-writer, dream, distill |
| `agent/agent.ts` | `list`, `defaultAgent` | agent 列表排序，默认 primary agent |

#### s16 团队协议

**核心概念**：完成门——actor 结束前检查遗留任务，最多 3 次 ReAct 重入。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `actor/spawn.ts` | preStop/postStop ReAct 循环 | actor 生命周期：preStop（执行后、完成门之前）→ postStop（完成门之后） |
| `task/gate.ts` | `TaskGate.decide()` | 构建 reentry 文本提示 agent 完成或放弃遗留任务 |
| `task/gate-state.ts` | `bump`, `clear` | ReAct 计数器防无限循环 |

#### s17 自主代理

**核心概念**：specs-driven 开发流程，brainstorm→plan→tdd→verify→merge。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `skill/compose/` | 各 SKILL.md | 内建 ~15 个 compose 技能：brainstorm, plan, tdd, debug, verify, review, merge, parallel, subagent 等 |
| `skill/compose/extract.ts` | `composeSkillsBlock()` | 技能注入：编译时打包，运行时提取到磁盘 |

#### s18 Worktree 隔离

**核心概念**：MiMo-Code 无独立 worktree，通过 session 隔离实现。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `session/session.ts` | `create` | 每个 session 独立上下文 |
| `actor/spawn.ts` | `spawnPeer` vs `spawnSubagent` | peer 模式创建子 session（隔离），subagent 模式共享父 session |

#### s19 MCP 与插件

**核心概念**：MCP 客户端 + 外部 npm 插件，统一路由到工具注册表。

| 文件 | 关键函数 | 说明 |
|------|---------|------|
| `mcp/index.ts` | `connect`, `tools`, `add`, `disconnect` | MCP 客户端：支持 stdio 和 StreamableHTTP/SSE 传输，OAuth 2.0 认证 |
| `plugin/index.ts` | `trigger`, `list`, `init` | 插件系统：内部插件 + 外部 npm 插件 |
| `plugin/loader.ts` | `loadExternal()` | 外部插件加载：npm install + 兼容性检查 + 动态 import |

---

## 关键技术栈

- 运行时: Bun
- 语言: TypeScript
- ORM: Drizzle ORM
- 数据库: SQLite
- UI 框架: SolidJS (TUI) + React (Web)
- AI SDK: Vercel AI SDK
- 效果系统: Effect-TS

---

## 学习路径总览

```
核心闭环:  s01(循环) → s02(工具) → s03(任务) → s04(子代理) → s05(技能) → s06(压缩)
系统加固:  → s07(权限) → s08(Hook) → s09(记忆) → s10(提示词) → s11(恢复)
任务运行时: → s12(任务图) → s13(后台) → s14(调度)
多Agent:  → s15(团队) → s16(协议) → s17(自治) → s18(隔离) → s19(MCP)
```

**架构演进主线**：单 agent 循环 → 工具分发 → 会话规划 → 上下文隔离 → 按需知识 → 上下文管理 → 安全权限 → 可扩展 hook → 跨会话记忆 → 输入流水线 → 容错恢复 → 持久任务图 → 异步执行 → 时间触发 → 长期队友 → 结构化协作 → 自治认领 → 文件系统隔离 → 外部能力接入。

---

## 完整文件清单

> 所有路径相对于 `packages/opencode/src/`

### 模块一：核心闭环

```
s01 Agent 循环
├── session/prompt.ts          runLoop 主循环 (L1698-2959)
├── session/processor.ts       流处理器 create (L181-941)
├── session/llm.ts             LLM 调用 run/stream (L298-696)
├── session/session.ts         会话生命周期 create/messages
└── cli/cmd/run.ts             CLI 入口 execute (L397-671)

s02 工具使用
├── tool/tool.ts               工具核心类型 define/init
├── tool/registry.ts           工具注册中心 Service (L1-413)
├── tool/bash.ts               bash 工具：Tree-sitter 解析、子进程
├── tool/read.ts               read 工具：目录/文件、二进制检测
├── tool/write.ts              write 工具：写入 + 格式化 + LSP
└── tool/edit.ts               edit 工具：9 种容错匹配、Semaphore

s03 待办写入
├── task/registry.ts           任务注册表 SQLite 持久化
├── task/gate.ts               完成门 TaskGate.decide()
├── task/gate-state.ts         ReAct 计数器
├── task/schema.ts             类型定义 Task/TaskStatus
└── task/events.ts             总线事件 TaskCreated/Updated

s04 子代理
├── actor/spawn.ts             Actor 核心 spawn/cancel (L1-741)
├── actor/registry.ts          注册表 SQLite、孤儿恢复
├── actor/waiter.ts            等待器 Bus+Deferred (L1-166)
├── actor/turn.ts              Turn 包装器 uninterruptible
└── actor/schema.ts            类型定义 Actor/Status/SpawnMode

s05 技能系统
├── skill/index.ts             技能核心 get/all/dirs/available
├── skill/discovery.ts         远程发现 pull
└── skill/compose/extract.ts   内建 bundle extractComposeBundle

s06 上下文压缩
├── session/compaction.ts      压缩核心 prune/process (L1-543)
└── session/checkpoint.ts      检查点 tryStartCheckpointWriter (L500-908)
```

### 模块二：系统加固

```
s07 权限系统
├── permission/index.ts        权限核心 ask/reply/evaluate
├── permission/evaluate.ts     纯函数规则匹配
└── permission/arity.ts        命令前缀词典 (~160 条)

s08 Hook 系统
├── plugin/index.ts            插件核心 trigger/aggregateDecision
├── plugin/matcher.ts          Actor 匹配器
└── plugin/loader.ts           外部插件加载 npm install

s09 记忆系统
├── memory/service.ts          记忆核心 search/reconcile
├── memory/reconcile.ts        索引协调 reconcileMemory
├── memory/paths.ts            路径解析 scope/type
└── memory/fts-query.ts        FTS5 查询构建

s10 系统提示词
├── session/llm.ts             buildSystemArray (L234-296)
├── agent/agent.ts             Agent 定义 (L112-369)
└── agent/prompt/*.txt         7 个 prompt 模板

s11 错误恢复
├── session/llm.ts             persistentRetrySchedule (L74-79)
├── session/compaction.ts      isOverflow 溢出检测
└── session/processor.ts       handleEvent error 分支
```

### 模块三：任务运行时

```
s12 任务系统
├── task/registry.ts           全文件 SQLite 持久化
├── task/gate.ts               完成门 ReAct 防遗留
└── task/schema.ts             状态机 open→in_progress→done

s13 后台任务
├── actor/spawn.ts             spawnSubagent 后台执行
├── actor/waiter.ts            带超时等待
└── actor/registry.ts          活跃 actor 追踪

s14 定时调度
└── agent/agent.ts             dream/distill agent 定义
```

### 模块四：多 Agent 平台

```
s15 Agent 团队
└── agent/agent.ts             11 个内置 agent 定义 (L112-369)

s16 团队协议
├── actor/spawn.ts             preStop/postStop ReAct 循环
└── task/gate.ts               完成门 reentry 文本

s17 自主代理
└── skill/compose/             ~15 个内建 compose 技能

s18 Worktree 隔离
├── session/session.ts         session 独立上下文
└── actor/spawn.ts             spawnPeer vs spawnSubagent

s19 MCP 与插件
├── mcp/index.ts               MCP 客户端 connect/tools
├── plugin/index.ts            插件系统 trigger/list
└── plugin/loader.ts           外部插件加载
```

---

## 建议

1. 先运行 `bun install` 和 `bun run dev` 体验功能
2. 从 s01 Agent 循环开始，重点读 `session/prompt.ts` 的 `runLoop`
3. 每读完一章，用 IDE 跳转追踪调用链验证理解
4. 遇到 Effect-TS 概念时，参考 `effect/` 目录的 InstanceState/Runner
