---
name: 流式架构模式
description: MiMo Code 和 Claude Code 的流式处理架构、EventBus 模式、渲染器分离、压缩集成
type: concept
updated: 2026-06-27
tags: mimocode, claude-code, streaming, architecture, event-bus
source: [[sources/projects/ECC-MiMo-Code.md]]
---

# 流式架构模式

## MiMo Code 流式架构

### EventBus off-pattern

StreamEventBus 使用 `emitter.removeListener()` 进行每个 `off*` 方法。每个 `on*` 方法都有匹配的 `off*`，接受相同的 handler 引用。`removeAllListeners()` 仅用于紧急清理。

### 渲染器分离模式

StreamEventRenderer 在 `attach()` 期间将 handler 引用存储在 `handlers` 对象中，在 `detach()` 中使用它们进行精确移除。避免依赖 `removeAllListeners()`。

### 工具计时模式

使用 `Map<string, number>` 在发出 `ToolCallEvent` 时跟踪每个工具 ID 的 `startTime`。在发出 `ToolResultEvent` 时计算 `duration = Date.now() - startTime`。

### 压缩集成

`compression.ts` 中的 `runCompaction()` 执行 L1（snip）+ L2（micro）压缩。在代理循环中每次消息历史更新后调用，以防止无限增长。返回 `{ needsL4: boolean }` 用于完整上下文压缩。

### isStreamEvent 类型守卫

在 `types.ts` 中集中定义为 `STREAM_EVENT_TYPES` Set + 导出函数。避免在 query-engine.ts 和 streaming-query.ts 中重复检查。

## Claude Code 流式架构

### 4 层流式管道

```
API (claude.ts) → QueryEngine (QueryEngine.ts) → QueryLoop (query.ts) → UI (StreamingMarkdown.tsx)
```

每层向上 yield AsyncGenerator 值。

### 稳定/不稳定区域分离

在 StreamingMarkdown 中：
- `stablePrefixRef` 跟踪已完成的 markdown 块
- `unstableSuffix` 是增长的尾部
- 使用 `marked.lexer()` 找到边界 — 每帧只重新解析不稳定部分
- 复杂度：O(unstable_length) 而非 O(total_length)

### StreamingToolExecutor 并发模型

- 只读工具（FileReadTool、GlobTool、GrepTool）：`isConcurrencySafe=true`，并行运行
- 写入工具（write_file、bash）：`isConcurrencySafe=false`，必须串行
- 基于队列，`processQueue()` 检查 `canExecuteTool()`

### 助手消息 fire-and-forget

`recordTranscript(messages)` 对助手消息使用 `void`（fire-and-forget），避免阻塞流式输出。其他消息类型使用 `await` 确保持久化。

## mi-code UI 渲染管道

### 屏幕缓冲区 + diff 系统

```
buildTree() → renderTree() → diffBuffers() → writePatches()
```

所有内容必须通过此管道；直接 stdout 写入绕过 diff 系统。

### Markdown 流式渲染

`MarkdownStreamRenderer.renderToken()` 通过状态机处理增量 tokens。处理标题、粗体、斜体、代码块、引用、列表、链接。

### 代码高亮使用 ANSI

`CodeHighlighter.highlight()` 通过 ANSI 转义码应用语法颜色（关键字=青色、字符串=绿色、注释=灰色、数字=黄色）。