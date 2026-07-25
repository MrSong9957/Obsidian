---
name: Inline Grid Footer 渲染架构
description: cell 级双缓冲 + 绝对坐标定位彻底解决 inline 模式 resize 堆叠——footerTopRow 计算、resize 链路、emit yBias、InlineGridRenderer 组件职责
type: concept
updated: 2026-07-14
tags: tui, inline-renderer, grid, double-buffer, resize, footer, conhost, reflow, absolute-positioning
source: mi-code 项目 src/tui/inline/grid-renderer.ts
---

# Inline Grid Footer 渲染架构

> **核心结论**：inline 模式 footer 用 cell 级双缓冲 + 绝对坐标定位渲染，彻底解决 resize 堆叠。`footerTopRow = min(内容行数 + 1, rows - footerHeight + 1)`。

## 问题本质

终端 resize 时，conhost/ConPTY 会用新宽度 reflow 整个屏幕——把旧的超宽 border 行折成多行。应用层的行数账本（footerHeight）和终端真实物理行数不一致，导致 cursorUp 相对定位失效，footer 堆叠。

**7 次失败**的根因：全部假设"应用能通过 stdout 序列修复 reflow 后的状态"，但应用层连 reflow 发生了都不知道。

## 解决方案

### 两层替换

| 维度 | 旧（cursorUp 相对定位） | 新（grid 双缓冲 + 绝对坐标） |
|---|---|---|
| 屏幕真相 | 终端物理行 + 应用账本（假设一致） | DoubleBuffer（应用持有权威） |
| 定位方式 | `cursorUp(N)` 相对移动 | `\x1b[row;colH` 绝对坐标 |
| diff 粒度 | 行级（overwriteLine 整行擦写） | cell 级（只输出变化的格子） |
| resize 时 | 账本失效 → 堆叠 | clearForResize 清旧区域 → 绝对坐标重画 |

### footerTopRow 计算（关键公式）

```
contentLines = 3(logo) + 1(hook) + renderer.state.renderedCount(消息) + lastStreamingHeight(流式草稿)
footerTopRow = min(contentLines + 1, rows - footerHeight + 1)
```

**第一性原理**：footer 紧跟在最后一个内容行的下一行。
- 内容没填满屏幕时，footer 在内容正下方（如 row 5）
- 内容超出屏幕时，`min` 保证 footer 不超出屏幕底部

**错误公式**（alt-screen 全屏假设）：`footerTopRow = rows - footerHeight + 1`——把 footer 强制放屏幕底部，inline 模式下内容很少时中间留大片空白，logo 视觉上"消失"。

### resize 完整链路

```
用户缩小窗口
  ↓
conhost reflow：把旧超宽 border 折成多行（终端自己干的，应用看不见）
  ↓
cols 变化 → InlineApp useEffect 重跑 → 检测 cols !== prevColsRef
  ↓
gridRenderer.clearForResize():
  CUP(旧footerTopRow, 1)  ← 绝对定位到旧 footer 位置
  ED（\x1b[0J）            ← 从这里清到屏幕底，旧 footer + reflow 碎片全清
  db = null               ← 丢弃双缓冲
  ↓
commitFooter(新layout, 新footerTopRow, 新cols):
  新建 DoubleBuffer（新cols 宽度，front 全 0）
  back ← blitAnsi（新宽度的 border/输入框/status）
  diff(front=全0, back) → 全量 Patch（每个 cell 都变了）
  emit → \x1b[row;colH 逐 cell 绝对坐标写入（不依赖光标当前位置）
  swap（back→front）
```

**为什么绝对坐标能解决 reflow**：`cursorUp(N)` 依赖"光标当前在哪"+"旧内容占几行"，reflow 后两者都变了。`\x1b[row;colH` 直接指定屏幕坐标，不管 reflow 怎么折腾，clearForResize 清完后 emit 逐 cell 精确重画。

## 架构组件

### InlineGridRenderer（`src/tui/inline/grid-renderer.ts`）

持有 footer 区域的 DoubleBuffer（`new DoubleBuffer(height, cols)`），复用 `src/render/` 的渲染核心。

**核心接口：**
- `commitFooter(layout, footerTopRow, cols)`：blitAnsi 写 back → diff front/back → optimize → emit（yBias = footerTopRow - 1）
- `clearForResize()`：CUP + ED 清旧 footer 区域，丢弃 buffer
- `dispose()`：unmount 时清除 footer

### emit.ts yBias

`EmitContext.yBias` 可选参数。所有 CUP 定位（patch 定位 + 末尾光标定位）加 yBias 偏移：
- alt-screen 模式：yBias=0（默认，行为不变）
- inline 模式：yBias = footerTopRow - 1（局部网格坐标 → 屏幕绝对坐标）

`\x1b[K`（EL）不加 yBias——它只擦当前行，依赖前置 CUP 已定位。

### 渲染流程（InlineApp effect）

```
新消息到达时：
  1. gridRenderer.clearForResize()     ← 擦旧 footer 让位
  2. renderer.commit()                  ← appendLine 新消息 + 流式草稿（不含 footer）
  3. gridRenderer.commitFooter()        ← 在新位置画 footer（grid diff）

纯输入/spinner 更新时：
  1. gridRenderer.commitFooter()        ← diff 增量更新 footer（db 存在，只输出变化的 cell）

resize 时：
  1. gridRenderer.clearForResize()      ← 清旧 footer + reflow 碎片
  2. gridRenderer.commitFooter()        ← db=null → 全量重画（绝对坐标）
```

## 关键约束

- `InlineRenderer.commit()` **不再写 footer**（writeFooter 已删除）——footer 完全由 gridRenderer 管
- 静态行（logo/消息）继续走 `appendLine` 直写 stdout，进 scrollback 不归应用管
- 流式草稿仍用 `cursorUp` 相对定位（第一阶段不处理，resize 时接受短暂异常）
- DECAWM OFF（`\x1b[?7l`）保证写入时应用控制折行，物理行 = 应用行

## 相关文件

- `src/tui/inline/grid-renderer.ts` — InlineGridRenderer
- `src/render/emit.ts` — yBias 支持
- `src/render/screen.ts` — DoubleBuffer（复用，零改动）
- `src/render/diff.ts` — cell 级 diff（复用）
- `src/tui/inline/InlineApp.tsx` — footerTopRow 计算 + resize 检测
- `src/tui/inline/layout.ts` — layoutFooter 纯函数（复用）
- `docs/superpowers/specs/2026-07-14-inline-grid-footer-design.md` — 完整设计文档
