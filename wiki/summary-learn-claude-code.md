---
name: Learn Claude Code 学习笔记摘要
description: shareAI-lab 仓库 12 个渐进式脚本的摘要，从 Agent Loop 到工作树隔离——理解 Claude Code harness 构建时查阅
type: summary
source: "[[sources/learn-claude-code]]"
confidence: 0.95
updated: 2026-04-06
tags: [Agent, Claude-Code, 教学笔记, harness]
status: deprecated
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# 摘要：Learn Claude Code（s01-s12）

> 内容已迁移至 [[wiki/agent-arch-p1.md|agent-arch-p1]]。

## 一句话总结

从 LLM API 调用到多智能体系统的 12 个渐进式教学脚本，每个脚本在前者基础上叠加一个核心机制，最终构建出完整的 Agent harness。[^0]

## 四阶段递进关系

### 阶段一：核心循环（s01-s02）

建立 Agent 的基本骨架——while 循环 + 工具分发。

- **s01 Agent 循环**：整个 Agent 的核心秘密就是 `while stop_reason == "tool_use"` 循环，把工具执行结果喂回模型直到模型自己决定停下来。[^1]
- **s02 工具调用**：不改循环本身，用 `TOOL_HANDLERS` 字典（dispatch map）路由多个工具（bash/read/write/edit），实现可扩展的工具注册。[^2]

### 阶段二：状态与压缩（s03-s06）

让 Agent 能处理长任务——追踪进度、管理上下文、按需加载知识。

- **s03 进度追踪**：引入 `TodoWrite` 工具让模型自己维护任务列表，解决"忘了要做什么"的问题。[^3]
- **s04 子代理**：给子任务一个全新的 `messages=[]`，子代理干完只返回摘要，父代理的上下文保持干净。通过工具层面移除 `task` 工具防止无限递归。[^4]
- **s05 技能按需加载**：两层设计——系统提示词只放 name+description（约 100 tokens/技能），模型选择后再加载完整内容，避免全部塞进 prompt 又贵又慢。[^5]
- **s06 上下文压缩**：三层压缩策略——micro_compact（每轮静默替换旧结果）、auto_compact（超阈值时保存+LLM摘要+替换）、compact 工具（模型主动触发）。[^6]

### 阶段三：持久化与协作（s07-s10）

让 Agent 能跨会话工作、后台运行、团队协作。

- **s07 任务系统**：任务持久化到 `.tasks/` 目录（JSON 文件），支持 blockedBy 依赖关系，Agent 崩溃后可恢复。[^7]
- **s08 后台任务**：`threading.Thread` 让子代理在后台运行，父代理通过 `task_output` 文件获取结果，实现并行执行。[^8]
- **s09 团队协作**：持久化的 teammate（非临时子代理），每个 teammate 有独立对话历史和双向消息通道。[^9]
- **s10 团队协议**：用自然语言定义团队规则（谁来做什么、什么顺序），模型按协议协调多个 teammate。[^10]

### 阶段四：自治与隔离（s11-s12）

让 Agent 系统真正自主运行。

- **s11 自主代理**：任务看板 + 空闲轮询 + 自动认领，空闲 teammate 每隔几秒检查 `.tasks/` 目录，看到没人认领的任务就自己拿来做。[^11]
- **s12 工作树隔离**：用 `git worktree` 为每个 teammate 创建独立的工作目录，避免多人同时修改同一文件时的冲突。[^12]

## 关键数据

- 12 个脚本，总计 3194 行源材料
- s01 核心循环仅约 150 行 Python，之后每个脚本增加约 200 行
- 子代理上限 30 轮（安全防护）
- 上下文压缩阈值 50000 tokens
- 保留最近 3 条工具结果（micro_compact）

## 涉及的概念

- [[harness-engineering.md]] — "模型就是 Agent，代码是 harness" 的方法论
- [[multi-agent-framework.md]] — 多智能体协作的四角色迭代框架
- [[natural-language-programming.md]] — s10 用自然语言定义团队协议
- [[Everything-Claude-Code.md]] — ECC 技能集覆盖了 s05 技能加载的生产版实现
- [[learning-methods.md]] — 12 个脚本的螺旋上升学习路径设计

## 引用索引

[^0]: [[sources/learn-claude-code/s01-Agent循环]] `L1-10` — 核心问题和解决方案概述
[^1]: [[sources/learn-claude-code/s01-Agent循环]] `L9-19` — Agent Loop 的 while 循环模式
[^2]: [[sources/learn-claude-code/s02-工具调用]] `L1-20` — 工具分发表的设计
[^3]: [[sources/learn-claude-code/s03-进度追踪]] `L1-30` — TodoWrite 进度追踪机制
[^4]: [[sources/learn-claude-code/s04-子代理]] `L1-30` — 子代理的独立上下文设计
[^5]: [[sources/learn-claude-code/s05-技能按需加载]] `L1-20` — 两层按需加载架构
[^6]: [[sources/learn-claude-code/s06-上下文压缩]] `L1-45` — 三层压缩策略
[^7]: [[sources/learn-claude-code/s07-任务系统]] `L1-30` — 任务持久化设计
[^8]: [[sources/learn-claude-code/s08-后台任务]] `L1-30` — 后台线程执行
[^9]: [[sources/learn-claude-code/s09-团队协作]] `L1-30` — 持久化 teammate 设计
[^10]: [[sources/learn-claude-code/s10-团队协议]] `L1-30` — 自然语言团队协议
[^11]: [[sources/learn-claude-code/s11-自主代理]] `L1-30` — 任务看板与自动认领
[^12]: [[sources/learn-claude-code/s12-工作树隔离]] `L1-30` — git worktree 隔离机制
