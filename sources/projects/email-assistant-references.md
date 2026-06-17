---
name: Email Assistant References
description: 开发历程、技术细节、QQ 邮箱特性、项目路线图 — Email Assistant 的补充参考资料
type: concept
updated: 2026-05-14
tags: email, python, imap, smtp, debugging, roadmap, strategy
source: [[sources/projects/email-assistant-journey.md]], [[sources/projects/project-roadmap.md]]
aliases: [email-assistant-references]
---

# Email Assistant References

> 这是 [[email-assistant]] 的补充文档，包含开发历程、技术细节和项目路线图。

## 开发时间线

### 2026-05-09 基础功能

- **邮件拉取**：IMAP FETCH 获取未读邮件，用 `email` 标准库解析
- **踩坑：BODY vs BODY.PEEK**：FETCH BODY[] 会自动标记已读，必须用 BODY.PEEK[]
- **踩坑：aioimaplib 响应解析**：FETCH 返回混合 bytes 块（UID 行 + 邮件正文），需手动用 `b"From:"` / `b"Received:"` 识别正文块
- **踩坑：multipart 解析**：邮件可能是 multipart/mixed、multipart/alternative 或纯文本，需要递归遍历
- **mark-read 命令**：用 UID STORE +FLAGS \Seen 标记已读
- **发送验证**：SMTP send 成功不代表邮件到达，需检查 IMAP Sent Messages 确认

← [[sources/projects/email-assistant-journey.md]] L20-27

### 2026-05-09 目录重构

- 将 `skills/email-assistant/` 扁平化到项目根目录
- 去除 `{{PROJECT_DIR}}` 占位符，安装改为显式文件列表
- **双目录问题出现**：项目目录（开发）和 `~/.claude/skills/`（运行）是两份独立副本

← [[sources/projects/email-assistant-journey.md]] L29-33

### 2026-05-09 邮件筛选功能

- `fetch` 新增 `--since`/`--before`/`--days`/`--from`/`--subject` 参数
- **设计决策：客户端过滤**：QQ IMAP 不完全支持 IMAP SEARCH 的所有条件（如日期范围），改为 Python 侧过滤
- 日期解析用 `parsedate_to_datetime`，无效日期静默跳过

← [[sources/projects/email-assistant-journey.md]] L35-39

### 2026-05-10 HTML 转发功能

- **需求**：转发 HTML 邮件时保留原始格式，而不是 strip 成纯文本
- **实现**：
  - 新增 `extract_html_body()` 函数，递归提取 HTML 部分
  - 转发时发送 `multipart/alternative`（纯文本 + HTML 双版本）
  - HTML 转发头用 `div border-left` 包裹，模拟邮件客户端的引用样式
- **实测**：Nature Briefing 富 HTML Newsletter 转发到 Gmail，排版正常

← [[sources/projects/email-assistant-journey.md]] L41-48

### 2026-05-11 .env 加载修复

- **问题**：Skill 的 `_load_dotenv()` 只读脚本自身目录的 `.env`，而用户凭证分散在三个文件中
- 三个 .env 文件：
  - `~/.claude/skills/email-assistant/.env`：QQ 邮箱凭证 + DAKANG_EMAIL
  - `/home/app/project/.env`：QQ 邮箱 + Gmail + GitHub + DAKANG_EMAIL（最全）
  - `/home/app/project/docker/.env`：API keys + proxy + Obsidian 路径
- **修复**：`_load_dotenv()` 用 fallback 列表依次加载两个 .env，`os.environ.setdefault` 保证优先级不覆盖

← [[sources/projects/email-assistant-journey.md]] L50-57

## 技术细节

### IMAP 关键模式

- **BODY.PEEK vs BODY**：FETCH BODY[] 自动标记已读，必须用 BODY.PEEK[] 避免副作用
- **aioimaplib 响应解析**：FETCH 返回混合 bytes 块（UID 行 + 邮件正文），需用 `b"From:"`/`b"Received:"` 识别正文块，不能用固定偏移
- **multipart 递归**：邮件结构可能是 multipart/mixed → multipart/alternative → text/plain + text/html 的多层嵌套，必须递归遍历
- **客户端过滤**：QQ IMAP 不完全支持 IMAP SEARCH 条件，日期范围等建议 Python 侧用 `parsedate_to_datetime` 过滤

← [[sources/projects/email-assistant-journey.md]] L45-59

### SMTP 关键模式

- **multipart/alternative 发送 HTML**：同时包含 plain text 和 HTML 两个 part，邮件客户端优先渲染 HTML
- **发送后验证**：SMTP send 成功不代表邮件到达，检查 IMAP Sent Messages 确认

← [[sources/projects/email-assistant-journey.md]] L65-68

### HTML 邮件处理

- **正文双层提取**：同时保留 `extract_body()`（strip HTML → 终端展示）和 `extract_html_body()`（原始 HTML → 转发保留）
- **HTML 转发样式**：原始 HTML 套 `<div style="border-left: 2px solid #ccc; padding-left: 8px">` 容器，模拟邮件客户端引用样式
- **邮件线程**：Reply 用 `In-Reply-To`（指向 Message-ID）+ `References`（追加历史链），客户端据此分组

← [[sources/projects/email-assistant-journey.md]] L65-68

## QQ 邮箱特性

| 特性 | 详情 |
|------|------|
| IMAP 文件夹命名 | 英文：`Sent Messages`、`Drafts`、`Junk`、`Deleted Messages`，非中文 |
| 归档文件夹 | 默认 `Archives`，QQ 邮箱可能不存在需手动创建 |
| 发件箱同步延迟 | SMTP 发送成功后 IMAP Sent Messages 不会立即出现，需等待 |
| IMAP SEARCH 限制 | 不完全支持所有 IMAP SEARCH 条件，日期范围等建议客户端过滤 |
| UID MOVE 支持 | 部分版本不支持 UID MOVE，需 fallback 到 COPY + STORE \Deleted |

← [[sources/projects/email-assistant-journey.md]] L59-67

## .env 加载策略

- 多目录 fallback 列表：先加载脚本自身目录 `.env`，再加载项目根目录 `.env`
- 用 `os.environ.setdefault()` 保证先加载的值优先，不覆盖
- 适用场景：部署目录和开发目录有不同凭证，需合并加载

← [[sources/projects/email-assistant-journey.md]] L50-57

## 调试方法论

- 先用 `fetch --limit 1` 验证 IMAP 连接和凭证
- 用 `sent --limit 5` 验证发送结果
- 用 `folders` 确认 IMAP 文件夹名称
- 修改代码后必须 `cp` 到 skill 目录再测试

← [[sources/projects/email-assistant-journey.md]] L69-74

## 项目路线图

### 三阶段战略

| 阶段 | 内容 | 信号 |
|------|------|------|
| A：开源 + Build in Public | 打磨开源项目，多平台内容输出 | 当前阶段 |
| B：AI Skill 开发框架 | 从 email_cli.py 抽象通用 Skill 骨架 | 社区有人问"我也想做一个类似的" |
| C：AI 邮件助手 SaaS | Web 产品，多用户，订阅制 | A+B 发现明确市场信号 |

← [[sources/projects/project-roadmap.md]] L22-72

### Build in Public 内容策略

**核心定位**：展示「一个人用 AI 能做出什么」，主角是 AI + 开发过程，邮件助手只是载体。

| 平台 | 内容 | 目的 |
|------|------|------|
| GitHub | 项目、Wiki、Issue | 技术 credibility 根基 |
| 即刻 / X | 短帖：进展、踩坑 | 圈子曝光、互动 |
| 抖音 / B站 | 2-5 分钟录屏+讲解 | 触达更广技术受众 |
| 公众号 / 博客 | 长文：设计决策解析 | SEO 沉淀、深度信任 |

← [[sources/projects/project-roadmap.md]] L76-90

### 内容节奏（前 2 个月）

- 每周 1-2 条短帖（star 里程碑、bug 解决、AI code review 对比）
- 每 2 周 1 个视频（功能演示、安装教程、从零实现录屏）
- 每月 1 篇长文（如「我用 Claude Code 做了一个邮件助手，以下是学到的事」）

← [[sources/projects/project-roadmap.md]] L92-97

### 应用场景洞察

**出海贸易沟通（核心场景）**：
- **时差兼容：** 跨时区沟通天然容忍 AI 思考/处理延迟
- **语言无障碍：** AI 自动翻译，实现与全球任何语言客户的实时邮件沟通
- **陌生拜访拓客：** AI 撰写个性化 cold email，批量但非群发

← [[sources/projects/project-roadmap.md]] L55-60

## 引用索引

| 来源 | 行号 | 内容 |
|------|------|------|
| [[sources/projects/email-assistant-journey.md]] | L20-27 | 基础功能开发 |
| [[sources/projects/email-assistant-journey.md]] | L29-33 | 目录重构 |
| [[sources/projects/email-assistant-journey.md]] | L35-39 | 邮件筛选功能 |
| [[sources/projects/email-assistant-journey.md]] | L41-48 | HTML 转发功能 |
| [[sources/projects/email-assistant-journey.md]] | L50-57 | .env 加载修复 |
| [[sources/projects/email-assistant-journey.md]] | L59-67 | QQ 邮箱特性 |
| [[sources/projects/email-assistant-journey.md]] | L65-68 | SMTP 关键模式 |
| [[sources/projects/email-assistant-journey.md]] | L69-74 | 调试方法论 |
| [[sources/projects/project-roadmap.md]] | L22-72 | 三阶段路线图 |
| [[sources/projects/project-roadmap.md]] | L55-60 | 应用场景洞察 |
| [[sources/projects/project-roadmap.md]] | L76-90 | Build in Public 策略 |
| [[sources/projects/project-roadmap.md]] | L92-97 | 内容节奏 |
