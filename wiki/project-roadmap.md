---
name: Email Assistant 项目路线图
description: 项目发展方向的三阶段战略决策 — 开源 Build in Public → AI Skill 框架 → SaaS，含多平台发布方案
type: concept
updated: 2026-05-14
tags: project, roadmap, strategy, email-assistant, open-source, publish, social-media
source: [sources/projects/project-roadmap.md]], [[sources/projects/publish-plan.md]
---

# Email Assistant 项目路线图

## 定位

"AI × 终端工作流" 的实战案例，而非"又一个邮件客户端"。

← [[sources/projects/project-roadmap.md]] L1-20

## 三阶段路线图

### 阶段 A：开源项目 + Build in Public（当前阶段）

- 邮件助手本身不卖钱，开发过程有价值
- 打磨高质量开源项目 + 多平台（GitHub/抖音/即刻）内容输出
- 变现路径：品牌 → 咨询/合作/工作机会
- 时间：1-2 个月集中投入

← [[sources/projects/project-roadmap.md]] L22-43

### 阶段 B：AI Skill 开发框架

- 从 email_cli.py 抽象通用 Skill 骨架 → Claude Code Skill 模板/脚手架
- 让其他开发者快速构建日历/RSS/记账等 AI 助手
- 定位升级：从「一个工具」到「一种方法论」

← [[sources/projects/project-roadmap.md]] L45-58

### 阶段 C：AI 邮件助手 SaaS

- Web 产品，多用户，订阅制
- 仅在 A+B 发现明确市场信号时启动

← [[sources/projects/project-roadmap.md]] L60-72

## 阶段切换信号

| 切换 | 信号 |
|------|------|
| A → B | 社区有人问"我也想做一个类似的" |
| B → C | A+B 阶段发现明确的付费意愿/市场需求 |

← [[sources/projects/project-roadmap.md]] L74-79

## 应用场景洞察

### 出海贸易沟通（核心场景）

- **时差兼容：** 跨时区沟通天然容忍 AI 思考/处理延迟（几秒到几十秒），不影响体验
- **语言无障碍：** AI 自动翻译，实现与全球任何语言客户的实时邮件沟通
- **陌生拜访拓客：** AI 撰写个性化 cold email，批量但非群发，符合营销场景需求
- **场景匹配度：** 出海贸易 = 时差 + 语言障碍 + 邮件为主 → 邮件助手完美契合

### 场景化方向

- 参考设计文档中的场景化路线，针对具体使用场景优化功能

## Build in Public 内容策略

**核心定位**：展示「一个人用 AI 能做出什么」，主角是 AI + 开发过程，邮件助手只是载体。

← [[sources/projects/project-roadmap.md]] L76-80

### 平台分工

| 平台 | 内容 | 目的 |
|------|------|------|
| GitHub | 项目、Wiki、Issue | 技术 credibility 根基 |
| 即刻 / X | 短帖：进展、踩坑 | 圈子曝光、互动 |
| 抖音 / B站 | 2-5 分钟录屏+讲解 | 触达更广技术受众 |
| 公众号 / 博客 | 长文：设计决策解析 | SEO 沉淀、深度信任 |

← [[sources/projects/project-roadmap.md]] L82-90

### 内容节奏（前 2 个月）

- 每周 1-2 条短帖（star 里程碑、bug 解决、AI code review 对比）
- 每 2 周 1 个视频（功能演示、安装教程、从零实现录屏）
- 每月 1 篇长文（如「我用 Claude Code 做了一个邮件助手，以下是学到的事」）

← [[sources/projects/project-roadmap.md]] L92-97

### 关键原则

- 做开发日记，不做产品宣传
- 展示「遇到问题 → 分析 → 解决」的思维过程
- 视频 hook：「我用 AI 写了一个能帮我管邮件的终端工具」

← [[sources/projects/project-roadmap.md]] L99-103

## 多平台发布方案（10 个平台）

国内外各 5 个平台，按内容类型四列对齐：

| 内容类型 | 国内 | 国外 |
|----------|------|------|
| 代码分发 | GitHub | GitHub |
| 长文 | 知乎 | Dev.to |
| 社交图文 | 小红书 / 公众号 | X / Reddit |
| 视频 | 抖音 / B站 | YouTube |

### 通用发布格式

- **文字 + 图片 = 最小通用集**，所有平台都支持
- **文字图片可合成视频**（FFmpeg / remotion / 手动录屏），覆盖抖音/B站/YouTube
- 一次写好 `post.md`（frontmatter + body），脚本按平台规则裁剪分发
- 每个平台通过 frontmatter 开关控制：`true`(自动) / `draft`(草稿) / `manual`(手动) / `false`(跳过)

### 一键发布工具 `publish.sh`

- 解析 `release-notes/v{version}/post.md` 统一模板
- 自动平台：GitHub Release（`gh` CLI）、Dev.to（REST API）
- 手动平台：格式化输出到剪贴板，复制即发
- 视频平台：检查视频文件，输出上传指引

← [[sources/projects/publish-plan.md]] L1-232

### 平台评估结论

- 即刻、V2EX 为小众平台，不在首批发布范围
- 掘金（开发者社区）替换为知乎（更主流，SEO 更好）
- Hacker News、Product Hunt 降级为 launch day 备选，不作为常规渠道
- 抖音必须纳入（用户量最大，覆盖面最广）
