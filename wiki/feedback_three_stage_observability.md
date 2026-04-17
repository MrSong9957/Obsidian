---
name: 三段式可观测性
description: 修复 BUG 时使用 before/after/failed 三段式事件记录作为主要观测手段
type: feedback
updated: 2026-04-10
tags: [feedback, 调试, 可观测性]
status: deprecated
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# 三段式可观测性

> 内容已迁移至 [[wiki/dev-practice.md|dev-practice]]。

修复 BUG 时，优先用 before/after/failed 三段式事件日志建立可观测性，再定位问题。

**Why:** 用户从 s12 的 EventBus 获得灵感——append-only JSONL 在关键操作前后及失败时各留一条记录，能完整回溯"发生了什么"，是最轻量但最有效的调试手段。

**How to apply:**
- 修 BUG 前，先确认目标代码有没有 before/after/failed 日志点；没有就先加
- 日志格式：每行一个 JSON，包含 event 名、时间戳、关键上下文（参数、状态）
- 用 before/after 确认操作是否完成，用 failed 捕获异常现场
- 不需要复杂框架，一个 JSONL 文件 + append 就够
