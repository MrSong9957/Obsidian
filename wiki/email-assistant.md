---
name: Email Assistant
description: Python 邮件助手 — Claude Code Skill，通过 IMAP/SMTP 连接 QQ 邮箱，AI 摘要分类后终端展示
type: concept
updated: 2026-05-14
tags: email, python, imap, smtp, skill, claude-code
source: [[sources/projects/email-assistant-journey.md]], [[sources/projects/project-roadmap.md]]
aliases: [邮件助手, email-assistant]
---

# Email Assistant

## 定位

"AI × 终端工作流" 的实战案例，而非"又一个邮件客户端"。

## 项目概述

Python 邮件助手，作为 Claude Code Skill 运行。通过 IMAP 连接 QQ 邮箱，AI 摘要分类后终端展示；用户口头指示回复/转发/归档，助手通过 SMTP 执行。

## 技术栈

| 组件 | 技术 |
|------|------|
| 语言 | Python 3.13+ asyncio |
| IMAP | aioimaplib |
| SMTP | aiosmtplib |
| 邮箱 | QQ 邮箱 IMAP/SMTP |
| 运行方式 | Claude Code Skill |

## 核心功能

### 邮件拉取（IMAP）

- IMAP FETCH 获取未读邮件
- `email` 标准库解析邮件结构
- 支持 `--since`/`--before`/`--days`/`--from`/`--subject` 筛选
- 客户端过滤（QQ IMAP SEARCH 限制）

### 邮件发送（SMTP）

- SMTP 发送邮件
- `multipart/alternative` 支持 HTML 邮件
- 发送后验证（检查 IMAP Sent Messages）

### HTML 邮件处理

- 双层提取：`extract_body()`（终端展示）+ `extract_html_body()`（转发保留）
- HTML 转发样式：`div border-left` 模拟引用样式
- 邮件线程：`In-Reply-To` + `References` 头

### 邮件管理

- `mark-read`：UID STORE +FLAGS \Seen
- `folders`：列出 IMAP 文件夹
- `sent`：查看已发送邮件

## 系统架构

```
┌─────────────────┐
│   用户终端       │
│  (Claude Code)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Email Assistant │
│    (Skill)      │
├─────────────────┤
│  - IMAP Client  │
│  - SMTP Client  │
│  - AI 摘要分类   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   QQ 邮箱       │
│  IMAP/SMTP      │
└─────────────────┘
```

## 快速开始

### 安装

```bash
# 克隆项目
git clone <repo-url>

# 安装依赖
pip install -r requirements.txt

# 配置 .env
cp .env.example .env
# 编辑 .env 填入 QQ 邮箱凭证
```

### 配置

```env
# QQ 邮箱凭证
QQ_EMAIL=your@qq.com
QQ_IMAP_HOST=imap.qq.com
QQ_SMTP_HOST=smtp.qq.com
QQ_PASSWORD=your-app-password  # QQ 邮箱授权码
```

### 使用

```bash
# 拉取未读邮件
email fetch --unseen

# 拉取最近 3 天邮件
email fetch --days 3

# 查看已发送邮件
email sent --limit 5

# 列出文件夹
email folders
```

## 开发状态

**当前阶段**：A — 开源项目 + Build in Public

- 邮件助手本身不卖钱，开发过程有价值
- 打磨高质量开源项目 + 多平台内容输出
- 变现路径：品牌 → 咨询/合作/工作机会

## 相关文档

- [[email-assistant-references]] — 开发历程、技术细节、项目路线图
- [[sources/projects/email-assistant-journey.md]] — 完整开发过程记录
- [[sources/projects/project-roadmap.md]] — 项目发展规划原始资料
