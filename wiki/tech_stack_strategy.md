---
type: feedback
name: 技术栈与语言策略
description: 原型用 HTML/CSS/JS，正式前端 Flutter，后端默认 Python，高性能场景 Rust
updated: 2026-04-10
tags: [技术栈, 语言策略, Rust, Flutter, 界面原型]
status: deprecated
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# 技术栈与语言策略

> 内容已迁移至 [[wiki/frontend.md|frontend]]。

## 技术栈

| 层 | 默认 | 高性能场景 |
|---|---|---|
| 原型 | HTML/CSS/JavaScript | — |
| 正式前端 | Flutter | — |
| 后端 | Python | Rust |

通过掌握底层逻辑 + Claude Code 辅助，可覆盖不熟悉的编程语言。

## Rust 策略

**Why:** 用户不会 Rust，无法直接审查 Rust 代码的正确性。直接让 AI 写 Rust 容易因语义不清导致幻觉。

**How to apply:**
1. 即使最终要 Rust，也必须先用 Python 完整实现（用户能审查对错）
2. 彻底完成后再翻译成 Rust
3. Python vs Rust 由 Claude 结合场景主动分析并与用户讨论后决定，不擅自选

## 界面原型策略

**Why:** 用户能直接审查 HTML 的布局和交互流程，但看不懂框架代码。

**How to apply:**
先用 HTML/CSS/JavaScript 写原型，验证布局和交互流程无误后，再翻译到 Flutter。
