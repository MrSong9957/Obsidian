---
name: 前端与 UI
description: UI设计原则/终端UI测试/Playwright自动化/技术栈策略 — 遇到前端设计、UI测试、技术选型问题时参考
type: concept
updated: 2026-04-17
tags: [frontend, ui, testing, playwright, tech-stack]
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# 前端与 UI

## UI 设计原则

- 先问用途再写代码——给谁用、什么场景
- 先定极端审美方向——brutalist/editorial/luxury/playful/industrial，不准走中间路线
- 先想"记忆点"——用户打开页面会记住什么
- 先画 5 种截然不同的线框方案，用户选定后再编码
- 字体：禁止 Inter/Roboto/Arial/Space Grotesk/system-ui，必须配对（标题+正文），字重要有层级
- 颜色：CSS 变量定义，一主色 + 一锐利强调色，禁止紫色渐变白底和 Tailwind 默认蓝
- 布局：禁止居中 hero + 三栏卡片（AI 默认套路），空间要么慷慨要么密集
- 动效：CSS 优先，交错入场编排 > 散落微交互，hover 要惊喜
- 质感：必须加噪点纹理/渐变网格/几何图案中的至少一样
- 自动迭代闭环：截图→分析→改代码→重截，设迭代上限和收敛标准

> 详细内容见 sources/dev/frontend.md

## 无头 vs 有头模式

- 原则：无头做苦力，有头做裁判
- 无头：间距/对齐/颜色量化检查、代码 lint、自动迭代截图对比
- 有头：审美定稿、动效体验、最终交付验收

> 详细内容见 sources/dev/frontend.md

## 终端 UI 测试

- 推荐方案：VHS + 双智能体循环（Generator → VHS → Evaluator → 反馈）
- VHS 核心价值：声明式 .tape 脚本 + PNG 截图供 AI 视觉分析
- 评估维度：布局对齐、颜色对比度、文本溢出、交互状态、残留 Bug
- 关键发现：简单交互循环 + 二元奖励即有效；环境规模比算法复杂度重要；循环行为是主要失败模式（39%）

> 详细内容见 sources/dev/frontend.md

## Playwright CLI 能力摘要

- 核心操作：open/screenshot/snapshot/fill/click/select/resize/route/state-save/state-load
- 高级能力：Mock API（route）、多会话并发、网络监控、视频/轨迹录制、弱网模拟
- 常用命令速查见 sources/dev/frontend.md

> 详细内容见 sources/dev/frontend.md

## 技术栈策略

| 层 | 默认 | 高性能场景 |
|---|---|---|
| 原型 | HTML/CSS/JavaScript | — |
| 正式前端 | Flutter | — |
| 后端 | Python | Rust |

- Rust 策略：先用 Python 完整实现（用户能审查），完成后再翻译成 Rust
- 界面原型：先用 HTML/CSS/JS 验证布局和交互流程，再翻译到 Flutter

> 详细内容见 sources/dev/frontend.md
