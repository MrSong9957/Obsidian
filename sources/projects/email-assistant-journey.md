---
name: Email Assistant 开发历程
description: Email Assistant 项目从零到一的完整开发过程、调试故事、踩坑记录
type: process
updated: 2026-05-11
tags: email, python, imap, smtp, skill, debugging
source: https://example.com
---

# Email Assistant 开发历程

## 项目概述

Python 邮件助手，作为 Claude Code Skill 运行。通过 IMAP 连接 QQ 邮箱，AI 摘要分类后终端展示；用户口头指示回复/转发/归档，助手通过 SMTP 执行。

技术栈：Python 3.13+ asyncio，aioimaplib + aiosmtplib，QQ 邮箱 IMAP/SMTP。

## 开发时间线

### 2026-05-09 基础功能

- **邮件拉取**：IMAP FETCH 获取未读邮件，用 `email` 标准库解析
- **踩坑：BODY vs BODY.PEEK**：FETCH BODY[] 会自动标记已读，必须用 BODY.PEEK[]
- **踩坑：aioimaplib 响应解析**：FETCH 返回混合 bytes 块（UID 行 + 邮件正文），需手动用 `b"From:"` / `b"Received:"` 识别正文块
- **踩坑：multipart 解析**：邮件可能是 multipart/mixed、multipart/alternative 或纯文本，需要递归遍历
- **mark-read 命令**：用 UID STORE +FLAGS \\Seen 标记已读
- **发送验证**：SMTP send 成功不代表邮件到达，需检查 IMAP Sent Messages 确认

### 2026-05-09 目录重构

- 将 `skills/email-assistant/` 扁平化到项目根目录
- 去除 `{{PROJECT_DIR}}` 占位符，安装改为显式文件列表
- **双目录问题出现**：项目目录（开发）和 `~/.claude/skills/`（运行）是两份独立副本

### 2026-05-09 邮件筛选功能

- `fetch` 新增 `--since`/`--before`/`--days`/`--from`/`--subject` 参数
- **设计决策：客户端过滤**：QQ IMAP 不完全支持 IMAP SEARCH 的所有条件（如日期范围），改为 Python 侧过滤
- 日期解析用 `parsedate_to_datetime`，无效日期静默跳过

### 2026-05-10 HTML 转发功能

- **需求**：转发 HTML 邮件时保留原始格式，而不是 strip 成纯文本
- **实现**：
  - 新增 `extract_html_body()` 函数，递归提取 HTML 部分
  - 转发时发送 `multipart/alternative`（纯文本 + HTML 双版本）
  - HTML 转发头用 `div border-left` 包裹，模拟邮件客户端的引用样式
- **实测**：Nature Briefing 富 HTML Newsletter 转发到 Gmail，排版正常

### 2026-05-11 .env 加载修复

- **问题**：Skill 的 `_load_dotenv()` 只读脚本自身目录的 `.env`，而用户凭证分散在三个文件中
- 三个 .env 文件：
  - `~/.claude/skills/email-assistant/.env`：QQ 邮箱凭证 + DAKANG_EMAIL
  - `/home/app/project/.env`：QQ 邮箱 + Gmail + GitHub + DAKANG_EMAIL（最全）
  - `/home/app/project/docker/.env`：API keys + proxy + Obsidian 路径
- **修复**：`_load_dotenv()` 用 fallback 列表依次加载两个 .env，`os.environ.setdefault` 保证优先级不覆盖

## QQ 邮箱特性记录

| 特性 | 详情 |
|------|------|
| IMAP 文件夹命名 | 英文：`Sent Messages`、`Drafts`、`Junk`、`Deleted Messages`，非中文 |
| 归档文件夹 | 默认 `Archives`，QQ 邮箱可能不存在需手动创建 |
| 发件箱同步延迟 | SMTP 发送成功后 IMAP Sent Messages 不会立即出现，需等待 |
| IMAP SEARCH 限制 | 不完全支持所有 IMAP SEARCH 条件，日期范围等建议客户端过滤 |
| UID MOVE 支持 | 部分版本不支持 UID MOVE，需 fallback 到 COPY + STORE \\Deleted |

## 调试方法论记录

- 先用 `fetch --limit 1` 验证 IMAP 连接和凭证
- 用 `sent --limit 5` 验证发送结果
- 用 `folders` 确认 IMAP 文件夹名称
- 修改代码后必须 `cp` 到 skill 目录再测试
