---
name: Terminal Inline Renderer 架构原则
description: Application-driven Layout 底层约束——DECAWM、wordWrap、宽度计算、ANSI 处理等不变原则 + 架构演进记录（cursorUp 覆写 → cell 级双缓冲）
type: reference
updated: 2026-07-14
tags: tui, inline-renderer, decawm, wordwrap, ansi, cursor, layout-engine, terminal, architecture-principles, grid, double-buffer
source: mi-code 项目 docs/Terminal-Inline-Renderer-架构原则.md
---

> **架构演进说明（2026-07-14）**：本文档最初定下的原则大部分仍然有效（DECAWM、wordWrap、宽度计算等底层约束不变）。但有三处已被架构演进推翻：
> 1. ~~禁止 cell-based terminal emulator~~ → 已建立 InlineGridRenderer（cell 级双缓冲），解决了 resize 堆叠
> 2. 渲染模型三层（State→Layout→ANSI）→ 现为四层（State→Layout→Diff/DoubleBuffer→Emit）
> 3. 动态区域"覆盖刷新"（cursorUp + overwriteLine）→ footer 改用 cell diff + 绝对坐标
>
> 详见 [[inline-grid-footer-render]]。下面标注了每节的当前状态。

# Terminal Inline Renderer 架构原则

## 核心原则

终端 UI 渲染必须采用 **Application-driven Layout（应用驱动布局）**，禁止依赖终端自身的自动排版行为。

不要让程序猜测 Terminal Emulator 的最终显示结果。

错误模式：

```

应用计算文本高度
↓
猜测终端自动换行结果
↓
cursorUp 定位
↓
覆盖重绘

```

正确模式：

```

应用决定布局
↓
计算最终物理行
↓
输出 ANSI 指令
↓
终端只负责显示

```

---

# 根本问题

终端自动换行（DECAWM）属于 Terminal Emulator 行为，不同环境可能不同：

- xterm/Linux
- Windows ConPTY
- macOS Terminal
- VS Code Terminal

对超宽文本的处理可能不一致。

因此：

禁止使用：

- simulateTerminalWrap
- 根据字符长度猜测物理行数
- 假设终端一定自动 wrap

因为：

```

应用预测布局 != 终端真实布局

```

最终导致：

- cursor 漂移
- footer 高度错误
- border 堆叠
- 重绘错位

---

# 渲染模型

> **⚠️ 架构演进**：以下三层模型已被四层取代（State → Layout → Diff/DoubleBuffer → Emit）。footer 渲染走 cell 级双缓冲（InlineGridRenderer），不再用 cursorUp 覆盖刷新。静态行（logo/消息）仍走 appendLine 直写 stdout。下面的三层数据流对静态行仍有效。

InlineRenderer 必须采用：

```

Input State
↓
Layout Engine
↓
ANSI Renderer
↓
Terminal

````

职责：

## Input State

负责：

- 用户输入内容
- 光标逻辑位置
- 多行状态


## Layout Engine

负责：

- wordWrap
- 行拆分
- footer 高度计算
- cursor 物理位置计算


## ANSI Renderer

负责：

- 输出 ANSI 控制序列
- cursor 移动
- 覆盖刷新


Terminal 不参与布局计算。

---

# DECAWM 策略

InlineRenderer 生命周期：

启动：

```ansi
ESC[?7l
````

关闭终端自动换行。

退出：

```ansi
ESC[?7h
```

恢复默认行为。

目的：

让终端只执行：

```
write()
newline()
cursor movement()
```

不自动改变布局。

必须保证：

* 正常退出恢复
* SIGINT 恢复
* SIGTERM 恢复
* crash cleanup 恢复

避免用户 shell 永久关闭 wrap。

---

# Word Wrap 规则

所有动态区域必须由应用主动折行：

包括：

* 输入框
* status
* footer 内容

禁止：

```ts
sliceAnsi()
```

作为主要布局方案。

因为截断会丢失用户内容。

规则：

```
physical rows = application wrapped rows
```

即：

应用生成多少行，终端显示多少行。

---

# 宽度计算

禁止：

```ts
text.length
```

计算显示宽度。

必须使用：

```ts
string-width
```

原因：

需要正确处理：

* CJK
* emoji
* 全角字符
* ANSI

统一：

```ts
usableWidth = cols - 1
```

保留安全列，避免终端 pending wrap 状态。

---

# ANSI 处理

禁止：

先 strip ANSI，再重新拼接。

原因：

可能破坏：

* SGR 状态
* 颜色连续性

推荐：

ANSI token stream：

```
tokenize
    ↓
计算字符宽度
    ↓
保留 ANSI token
    ↓
重新组合
```

ANSI/control token：

显示宽度：

```
0
```

字符 token：

使用：

```
stringWidth()
```

---

# 光标定位

禁止：

根据终端模拟计算：

```ts
simulateTerminalWrap()
```

计算 cursor 位置。

必须基于应用布局：

```
文本
 ↓
wordWrap()
 ↓
找到 cursor 所在 wrapped line
 ↓
计算 row / col
```

cursor index 必须统一：

推荐：

```
Unicode code point index
```

禁止混用：

* UTF-16 index
* code point index

处理 emoji 时必须安全：

```ts
[...text]
```

避免 surrogate pair 错位。

---

# 渲染区域规则

## 动态区域

需要：

应用布局。

包括：

* 输入框
* footer
* status

## 历史区域

允许：

终端自然显示。

包括：

* 已提交消息
* streaming 输出

不要为了统一模型重写稳定区域。

---

# Suggestion 菜单规则

Suggestion 属于交互菜单，不属于正文。

策略：

* 保持单行
* 保持对齐
* 可以 truncate

不要为了完整显示破坏菜单布局。

---

# 禁止过度设计

> **⚠️ 已推翻（2026-07-14）**：下方"禁止 cell-based terminal emulator"已被推翻。为了解决 resize 堆叠（7 次 cursorUp 相对定位方案全部失败），建立了 InlineGridRenderer（cell 级双缓冲 + 绝对坐标定位）。原因是 inline 模式的行数账本（footerHeight）被终端 reflow 单方面破坏，cell 双缓冲让应用持有权威屏幕状态。其余禁止项（DSR 主渲染、完整虚拟终端模拟器）仍然有效。

当前阶段不要引入：

* DSR 实时查询光标位置
* alt-screen 重构
* cell-based terminal emulator
* 完整虚拟终端模拟器

原因：

应用层布局已经可以解决核心问题。

DSR 只适合作为异常恢复机制，不作为主渲染循环。

---

# 测试要求

必须覆盖：

## 字符宽度

* ASCII
* CJK
* emoji
* 混合文本

## Wrap

* 正常换行
* 空格断行
* 长 token
* 边界宽度

## Cursor

* 行首
* 行尾
* 换行边界
* emoji 后定位

## Regression

验证：

* border 不堆叠
* cursor 不漂移
* resize 正常
* 多行输入正常

---

# 核心认知

Terminal UI 的稳定性来自：

```
不要模拟终端
不要猜终端

自己管理布局
让终端执行布局
```

Inline Renderer 本质不是字符串打印器，而是：

```
一个小型 Layout Engine + ANSI Renderer
```

```

这版适合作为长期项目约束。它不是某一次修复方案，而是以后所有 CLI/TUI 开发都应该遵守的底层渲染原则。
