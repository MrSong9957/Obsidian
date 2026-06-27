---
name: MiMo Code UI 渲染管道
description: MiMo Code 终端 UI 渲染架构、屏幕缓冲区系统、布局计算、光标定位
type: concept
updated: 2026-06-27
tags: mimocode, ui, rendering, terminal, tui
source: [[sources/projects/ECC-MiMo-Code.md]]
---

# MiMo Code UI 渲染管道

## 架构概览

MiMo Code 使用**自定义渲染器**（非 React/Ink），基于屏幕缓冲区 + diff 系统。

### 渲染流程

```
buildTree() → renderTree() → diffBuffers() → writePatches()
```

**关键原则**：所有内容必须通过此管道；直接 stdout 写入绕过 diff 系统，导致布局混乱。

## 终端 UI 物理本质

终端是一个**固定大小的黑板**（如 24 行 × 80 列），需要在上面画出：横幅、消息区、输入框、状态栏。

## 布局计算原理

```
黑板总高度 = terminalHeight (如 24 行)
固定区域 = 横幅(4) + 输入框(3) + 状态栏(1) + 间距(2) = 10 行
消息区可用 = terminalHeight - 10
```

## 消息区："滚动窗口"原理

- 窗口大小固定（maxMessageLines 行）
- 消息从下往上"滚入"窗口
- 窗口满了，最旧的消息从顶部"滚出"
- 只显示最近的 N 条消息：`startIdx = Math.max(0, messages.length - maxMessageLines)`

## 光标定位："GPS 定位"原理

光标需要精确定位到输入框内，使用 ANSI 转义序列 `\x1b[{row};{col}H` 手动移动。

### 行号计算（从上往下数）

```
第 1 行：root 上边距
第 2 行：横幅上边距
第 3-5 行：横幅内容
第 6 行：消息区上边距
第 7-X 行：消息区内容（实际显示的行数）
第 X+1 行：输入框上边距
第 X+2 行：输入框上边框
第 X+3 行：输入框内容（光标位置）
```

### 列号计算（从左往右数）

```
第 1 列：root 左边距
第 2 列：输入框左边框
第 3 列：输入框内边距
第 4-5 列："❯ "字符
第 6 列+：用户输入的文本宽度
```

## 关键代码：positionCursor()

```typescript
function positionCursor() {
  // 1. 计算消息区实际占用的行数（考虑多行消息）
  let messageAreaLines = 0;
  for (const msg of messages) {
    messageAreaLines += msg.split('\n').length;
  }

  // 2. 限制在最大高度内
  const totalMessageLines = Math.min(maxMessageLines, messageAreaLines);

  // 3. 计算光标行号
  const inputRow = 1 + 1 + 3 + 1 + totalMessageLines + 1 + 1;

  // 4. 计算光标列号
  const beforeWidth = chars.slice(0, cursorPos).reduce((w, c) => w + charWidth(c), 0);
  const inputCol = 2 + 1 + 1 + 2 + beforeWidth;

  // 5. 移动光标
  process.stdout.write(`\x1b[${inputRow};${inputCol}H`);
}
```

## 架构选择：命令式 vs 声明式

MiMo Code 选择**命令式**（手动计算位置）而非**声明式**（React/Ink 自动定位），原因：
1. **零依赖**：不依赖 React/Ink 框架
2. **性能极致**：直接操作终端缓冲区
3. **完全控制**：可以实现任何布局效果

## 关键教训

1. **所有内容必须通过 UI 树渲染**：直接写 stdout 会绕过 diff 系统，导致布局混乱
2. **渲染器必须支持多行消息**：不能只取第一行，需要按 `\n` 分割后逐行渲染
3. **光标位置必须动态计算**：消息区高度变化时，光标位置也要随之更新

## 终端高度开销

固定开销 = 11 行（root paddingY=2, banner=4, input=3, statusBar=1, marginY=1）
可用消息行数 = `terminalHeight - 11`