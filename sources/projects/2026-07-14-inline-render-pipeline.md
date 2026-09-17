---
name: Inline 渲染管线统一修复
description: mi-code inline 模式从"两个 stdout 写入者冲突"到"统一单次 write 管线"的完整修复过程，结合 Claude Code 源码的架构决策
type: concept
tags: [mi-code, inline-render, terminal, ansi, claude-code]
source: self
created: 2026-07-14
updated: 2026-07-15
project: mi-code
branch: refactor/inline-render-pipeline
---

# Inline 渲染管线统一修复

## 背景与问题

mi-code 的 inline 模式（main-screen，保留 scrollback）在 `InlineDynamicGrid` 集成后出现三个症状：

1. **启动渲染错乱**：logo 后立刻出现 footer border 重复、status 行混入 assistant 正文、输入框错位
2. **流式闪烁**：流式输出时每次 delta 更新都明显闪烁
3. **resize 边框不自适应**：缩小窗口时 border 超出新宽度被终端折行

### 根因：两个 stdout 写入者并存

inline 模式有两个并行的 stdout 写入子系统：

- **appendLine**（`stdout.write(text + '\n')`）：logo、固化消息、摘要行 — `\n` 推进流，光标在末尾
- **InlineDynamicGrid**（CUP 绝对定位 `\x1b[row;colH` + cell diff）：草稿 + footer — 光标跳转，不推进流

CUP 把光标跳到 footer 区域后，下一次 `appendLine` 的 `\n` 从 **grid 留下的光标位置**继续 — 写到 footer 区域内。这就是 status 行混入正文、border 重复、输入框错位的根因。

## 根因分析（结合 Claude Code 源码）

### Claude Code 的 main-screen 架构：单管线

Claude Code 的 main-screen 模式（`log-update.ts`）是一个纯相对光标的单管线渲染：

```
React 组件树（全部：logo、消息、输入框、footer）
  → yoga 布局（计算每行位置）
  → 整体写入一个 Screen buffer（rows × cols 的 cell 网格）
  → diff(prevScreen, nextScreen) → 逐 cell 比较
  → emit patches（cursorMove 相对移动 + 字符写入）
  → stdout.write（一次调用）
```

关键证据：
- `log-update.ts:187`："We have to use purely relative operations to manipulate the cursor since we don't know its starting point."
- `ink.tsx:574`："Main-screen can't do this — cursor.y tracks scrollback rows CSI H can't reach."
- 没有 `appendLine`，没有 CUP 绝对定位。所有内容通过 Screen buffer → diff → 一次 write。

### CUP vs \r\n 的本质矛盾

- **CUP 绝对定位**（`\x1b[row;colH`）：不推进流，内容不滚动，历史消失（不进 scrollback）
- **\r\n 相对推进**：推进流，终端自动滚动，旧行进 scrollback

Claude Code 的 main-screen 用 cursorMove（相对移动）+ `\r\n` 推进。我们的 emit 用 CUP。两者是不同模型，不能混合。

### conhost 不支持 DEC 2026

`terminal.ts:70-117` 的检测逻辑：
- `WT_SESSION` 环境变量 → Windows Terminal → 支持
- `TMUX` → 不支持（跳过）
- conhost（传统 cmd/PS）不在支持列表 → 返回 false

BSU/ESU（`\x1b[?2026h`/`\x1b[?2026l`）在 conhost 上被当作未知序列忽略。**单次 write 是防闪烁的核心手段**——即使不支持 DEC 2026，终端处理一个完整字符串时中间状态最少。

`terminal.ts:205` 注释："Buffer all writes into a single string to avoid multiple write calls"。

## 修复方案

### 决策：回归 cursorUp 全行覆写 + \n 推进

废弃 `InlineDynamicGrid`（CUP cell diff），回归 `InlineRenderer` 已验证的 cursorUp 全行覆写模式。

**为什么不用 cell diff + 相对移动**：子代理发现 5 个 Blocker：
1. virtualCursor 状态需跨 emit 调用持久化（emit 当前是无状态函数）
2. glyph width 未知（全角字符后光标需 +2，但 Patch 不携带宽度信息）
3. scroll-count 追踪（`\n` 滚动后 grid 屏幕原点变化，topRow 失准）
4. `\n` vs `\x1b[B` 语义混淆（前者滚动，后者不滚动）
5. DEC 2026 + scrollback 交互未验证

**为什么不用 InlineDynamicGrid CUP 模式**：CUP 不推进流，和 appendLine 的 `\n` 推进冲突 — 这就是根因。

**全行覆写的可行性**：活跃区小（草稿 + footer，通常 5-15 行），全行覆写性能完全够用。`writeFooter` 的 cursorUp + `\r\x1b[2K` 模式在此项目中已验证。

### 改动清单

#### 1. InlineRenderer.ts：恢复 footer 写入 + BSU/ESU + 单次 write

`commit()` 恢复 `writeFooter` 调用（之前注释说"已移到 InlineGridRenderer"）。引入 `writeBuf` 缓冲机制：

```ts
private writeBuf: string[] | null = null;

private write(s: string): void {
  if (this.writeBuf !== null) {
    this.writeBuf.push(s);
  } else {
    this.stdout.write(s);
  }
}

commit(frame: CommitFrame): void {
  this.writeBuf = [];
  this.write('\x1b[?2026h');  // BSU

  // 所有子方法（commitFooter / eraseStreamingLines / appendLine /
  // rewriteStreamingLines / writeFooter）的 stdout.write 改为 this.write → 进 buffer
  // ...

  this.write('\x1b[?2026l');  // ESU
  this.stdout.write(this.writeBuf.join(''));  // 一次 write
  this.writeBuf = null;
}
```

`executeOperations`、`writeFooter`、`eraseStreamingLines` 中的 `this.stdout.write` 全部改为 `this.write`。

#### 2. InlineApp.tsx：废弃 dynamicGrid，回归 renderer.commit

- 移除 `dynamicGrid` prop 及所有调用（commit/clear/reset/promoteDraft）
- 移除 `totalContentRowsRef`（不再需要 topRow 计算）
- 恢复 `renderer.commit({ streamingLines, ... })`（传回草稿行，不再 null）
- 固化路径简化：不区分 assistant/thinking，统一由 commit 内部处理

#### 3. cols 加回 effect 依赖（resize 边框自适应）

`cols` 在 effect 依赖数组中 → resize 时 effect 立即重跑 → footer 用新 cols 布局 → border 自适应。

#### 4. 废弃文件

- `inline-dynamic-grid.ts`：CUP cell diff 模式，与 appendLine 不兼容
- `grid-renderer.ts`：已是死代码
- `streaming-grid-renderer.ts`：已是死代码

## 关键设计点

### 单次 write 防闪烁

`writeBuf` 机制：commit 期间所有 write 进 buffer，结束时一次 `stdout.write`。

这是防闪烁的**核心手段**。conhost 不支持 DEC 2026，BSU/ESU 被忽略，但单次 write 让终端在一个完整字符串内完成渲染——中间状态最少。

之前 `commit()` 内部有十几次独立的 `stdout.write`（executeOperations 每个 op 一次、writeFooter 光标定位 4 次、appendLine 一次）。每次 write 都可能触发终端刷新 → 闪烁。

### 固化路径统一

不再区分 assistant / thinking 两条路径。`commit` 内部序列：

```
commitFooter（擦除旧 footer）
  → eraseStreamingLines（擦除草稿，thinking 折叠时）
  → appendLine（固化消息/摘要行，\n 推进进 scrollback）
  → rewriteStreamingLines（草稿覆写）
  → writeFooter（footer 全行覆写 + 光标定位）
```

全部用 `\n` 推进 + cursorUp，光标状态一致 → 无冲突。

assistant 固化时草稿和正式消息格式一致（都用 `wrapStreamingText`），`eraseStreamingLines` 删除草稿行后 `appendLine` 从该位置写入固化消息——光标一致。

### resize 处理

`cols` 在 effect 依赖 → resize 立即触发 effect 重跑 → footer 用新宽度重画。

ConPTY reflow 堆叠问题暂未完美解决：cursorUp 全行覆写在 conhost reflow 后行数变化时光标可能算错。但用户环境是独立 cmd/PS，reflow 影响有限。

## 止血修复（2026-07-15）

统一管线重构分三步走。第一步止血修复了三个实际问题：

### Bug A：流式堆叠（needEraseDraft 条件错误）

**症状**：流式输出时同一句话重复几十遍。

**根因**：`InlineApp.tsx` 的 `needEraseDraft` 条件包含 `state.renderedCount > 0 && prevLastFinalizedRef.current === false`。流式中这两个条件每帧都为 true（有已固化消息 + 上一帧在流式）→ 每帧 `clearStreamingHeight()` → `lastStreamingHeight = 0` → `rewriteStreamingLines` 走追加分支（`diffStreamingOverlay(0, lines)` → 逐行 appendLine）→ 堆叠。

**修复**：`needEraseDraft = justFinalized`（删除第二个条件）。只在实际固化转换帧（从流式转到非流式）时擦除草稿。

### Bug B：resize 后堆叠（光标状态断链）

**症状**：缩小放大窗口后 status 行重复 3 次。

**根因**：cursorUp 全行覆写模式在 conhost reflow 后无法自纠正——`state.footerHeight` / `state.lastStreamingHeight` 是旧值，但屏幕实际行数已变 → cursorUp 距离算错 → 覆写变成追加。

**Claude Code 的答案**（`log-update.ts:142`）：resize 后不维护光标状态，直接 `fullResetSequence_CAUSES_FLICKER`（清屏重画）。

**修复**：resize 时（`cols !== prevColsRef`）：
1. `\x1b[2J\x1b[3J\x1b[H` 清屏 + 清 scrollback + 光标归位
2. 重写 logo
3. 重置所有渲染状态（`renderedLines.clear()`, `footerHeight=0`, `lastStreamingHeight=0`, `cursorToTop=0`）
4. `prevLastFinalizedRef = undefined`（重置流式状态）
5. effect 继续：所有消息变 pending → appendLine 全部 → writeFooter

### Bug C：resize 内容重复（scrollback 累积 + 重复触发）

**症状**：缩小放大一次后整个屏幕内容复制 3 遍。

**根因**：
1. `\x1b[2J` 只清当前屏幕，不清 scrollback。多次 resize → `\n` 推进让内容进 scrollback → scrollback 累积重复内容。
2. `useTerminalSize` 的 resize handler 每次 `setSize` 都触发 re-render。拖拽窗口时连续 resize 事件（不同尺寸）→ 每次都清屏重画 → 重复。

**Claude Code 的答案**：
- `clearTerminal.ts:59-69`：`getClearTerminalSequence()` 用 `\x1b[2J\x1b[3J\x1b[H`（清屏 + 清 scrollback + 归位）。conhost 不支持 `\x1b[3J` 但无害（忽略）。
- `ink.tsx:314-315`：同尺寸去重 `if (cols === this.terminalColumns && rows === this.terminalRows) return`。
- `ink.tsx:303-308`：不做 debounce——"A debounce opens a window where stdout.columns is NEW but Yoga are OLD"。

**修复**：
1. resize 清屏序列加 `\x1b[3J`：`\x1b[2J\x1b[3J\x1b[H`
2. `useTerminalSize` 的 `setSize` 改为函数式更新，尺寸相同时返回 `prev`（不触发 re-render）

## 废弃与遗留

### InlineDynamicGrid

CUP cell diff 模式架构方向正确（Claude Code 也用 cell diff），但与 appendLine 的 `\n` 推进模式不兼容。要完全统一需要废弃 appendLine，所有内容进 grid buffer——这是更大的重构，当前不做。

### emit.ts 的 ERASE_CHAR_ID

`ERASE_CHAR_ID` 从 `\x1b[K`（擦除到行尾）改为写空格 `' '`。原因：`\x1b[K` 擦除从光标到行尾的所有内容，当同行后面还有 diff patch 时已写入的内容被误擦。

这个改动仍保留在 render 层（`emit.ts`/`optimizer.ts`），alt-screen 模式（`createCustomRenderer`）可用。inline 模式不再用 emit（回归 cursorUp 全行覆写）。

### 未来方向

如果要完全统一管线（像 Claude Code 那样），需要：
1. 废弃 `appendLine` 的 `stdout.write(text + '\n')`
2. 所有内容（logo + 固化消息 + 草稿 + footer）写入统一的 Screen buffer
3. `diff` → `emit`（相对移动 + `\r\n` 推进）→ 一次 `stdout.write`
4. 需要解决 5 个 Blocker（virtualCursor 状态、glyph width、scroll-count 等）

## 验证

- tsc：通过
- 测试：150 files, 1566 passed, 0 failed
- 真实终端（cmd/PS）：
  - 渲染正确、不闪烁、边框自适应 ✅
  - 流式不堆叠 ✅（止血 Bug A 后）
  - 缩小放大两次无堆叠/无重复 ✅（止血 Bug B+C 后）

## 后续计划

统一管线重构分三步：
1. ✅ **第一步：止血**（needEraseDraft + resize 清屏重画 + 同尺寸去重）— 已完成
2. ⬜ **第二步：cell diff + cursorMove 底层准备**（CharPool.widthOf、emit cursorMove 模式）
3. ⬜ **第三步：appendLine 改为写进 buffer**（documentRows + viewportY + 统一出口）

## Claude Code 源码参考

| 问题 | 源码位置 | 答案 |
|------|---------|------|
| 一帧输出几次 write | `terminal.ts:205-247` | 一次。Buffer all writes into a single string |
| 全行覆写 vs cell diff | `log-update.ts:580,615` | cell diff 只写变化的 cell；新行用 \r\n 推进 |
| conhost 是否支持 DEC 2026 | `terminal.ts:70-117` | 不支持。conhost 不在支持列表 |
| 不支持时的 fallback | `terminal.ts:74` | 靠 cell diff 减少变化量 + 单次 write + 16ms throttle |
| main-screen 用 CUP 吗 | `log-update.ts:187`, `ink.tsx:574` | 不用。纯相对移动 cursorMove |
| resize 后怎么处理 | `log-update.ts:142` | 清屏重画（fullResetSequence），不维护光标状态 |
| 清屏是否清 scrollback | `clearTerminal.ts:59-69` | 是。`\x1b[2J\x1b[3J\x1b[H`（conhost 例外，只 `\x1b[2J`） |
| resize 有 debounce 吗 | `ink.tsx:303-308` | 没有。同尺寸去重（`ink.tsx:314-315`），不做 debounce |
| cursorUp 模式是否脆弱 | `log-update.ts:187` | 是。Claude Code 从不"擦整行再重写"，只 diff 变化的 cell |
