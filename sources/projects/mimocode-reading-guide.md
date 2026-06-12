---
name: MiMo-Code 源码阅读指南
description: MiMo-Code（OpenCode fork）完整源码阅读顺序、核心架构、关键文件路径
type: reference
updated: 2026-06-12
tags: [mimocode, opencode, typescript, monorepo, architecture]
---

# MiMo-Code 源码阅读指南

> 项目定位：基于 OpenCode fork 的 AI 编程智能体，具有跨会话记忆功能。
> 仓库地址：`E:\Files\GitHub\MiMo-Code`

## 推荐阅读顺序

### 第一阶段：理解项目概况

1. `README.md` / `README.zh.md` — 了解项目定位和核心特性
2. `package.json` — 了解项目依赖和脚本
3. `AGENTS.md` — 了解开发规范和代码风格

### 第二阶段：入口点和 CLI

1. `packages/opencode/src/index.ts` — 程序入口，了解 CLI 命令注册
2. `packages/opencode/src/cli/cmd/run.ts` — 核心 run 命令实现
3. `packages/opencode/src/cli/cmd/` — 其他命令实现

### 第三阶段：核心架构

1. `packages/opencode/src/agent/` — 代理系统（核心）
   - `agent.ts` — 代理定义和管理
   - `config.ts` — 代理配置
   - `prompt/` — 各种提示词模板
2. `packages/opencode/src/session/` — 会话管理（核心）
   - `session.ts` — 会话生命周期
   - `checkpoint.ts` — 检查点系统
   - `compaction.ts` — 上下文压缩
   - `llm.ts` — LLM 调用逻辑
3. `packages/opencode/src/tool/` — 工具系统
   - 各种工具实现（bash、read、write、edit、grep、glob 等）

### 第四阶段：特色功能

1. `packages/opencode/src/memory/` — 跨会话记忆系统
2. `packages/opencode/src/skill/` — 技能系统
3. `packages/opencode/src/workflow/` — 工作流系统
4. `packages/opencode/src/actor/` — 子智能体系统

### 第五阶段：基础设施

1. `packages/opencode/src/storage/` — 数据存储（SQLite + Drizzle ORM）
2. `packages/opencode/src/config/` — 配置管理
3. `packages/opencode/src/provider/` — LLM Provider 集成
4. `packages/opencode/src/server/` — HTTP 服务器
5. `packages/opencode/src/bus/` — 事件总线

### 第六阶段：UI 和其他包

1. `packages/app/` — Web 应用 UI
2. `packages/desktop/` — 桌面应用
3. `packages/sdk/` — SDK
4. `packages/shared/` — 共享工具函数

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

## 建议

1. 先运行 `bun install` 和 `bun run dev` 体验功能
2. 从 `run.ts` 命令开始追踪执行流程
3. 重点关注 `agent/`、`session/`、`tool/` 三个目录
4. 使用 IDE 的跳转功能追踪调用链
