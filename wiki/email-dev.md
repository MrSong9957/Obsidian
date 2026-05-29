---
name: Python 邮件开发
description: IMAP/SMTP asyncio 编程实战经验、QQ 邮箱特性、HTML 邮件处理 — 遇到 Python 邮件开发问题时参考
type: concept
updated: 2026-05-11
tags: [python, email, imap, smtp, asyncio]
sources: [[sources/projects/email-assistant-journey.md]]
---
# Python 邮件开发

## IMAP 关键模式

- **BODY.PEEK vs BODY**：FETCH BODY[] 自动标记已读，必须用 BODY.PEEK[] 避免副作用 ← [[sources/projects/email-assistant-journey.md]] L45
- **aioimaplib 响应解析**：FETCH 返回混合 bytes 块（UID 行 + 邮件正文），需用 `b"From:"`/`b"Received:"` 识别正文块，不能用固定偏移 ← [[sources/projects/email-assistant-journey.md]] L46
- **multipart 递归**：邮件结构可能是 multipart/mixed → multipart/alternative → text/plain + text/html 的多层嵌套，必须递归遍历 ← [[sources/projects/email-assistant-journey.md]] L47
- **客户端过滤**：QQ IMAP 不完全支持 IMAP SEARCH 条件，日期范围等建议 Python 侧用 `parsedate_to_datetime` 过滤 ← [[sources/projects/email-assistant-journey.md]] L59

## SMTP 关键模式

- **multipart/alternative 发送 HTML**：同时包含 plain text 和 HTML 两个 part，邮件客户端优先渲染 HTML ← [[sources/projects/email-assistant-journey.md]] L65-68
- **发送后验证**：SMTP send 成功不代表邮件到达，检查 IMAP Sent Messages 确认 ← [[sources/projects/email-assistant-journey.md]] L50

## HTML 邮件处理

- **正文双层提取**：同时保留 `extract_body()`（strip HTML → 终端展示）和 `extract_html_body()`（原始 HTML → 转发保留） ← [[sources/projects/email-assistant-journey.md]] L65-68
- **HTML 转发样式**：原始 HTML 套 `<div style="border-left: 2px solid #ccc; padding-left: 8px">` 容器，模拟邮件客户端引用样式
- **邮件线程**：Reply 用 `In-Reply-To`（指向 Message-ID）+ `References`（追加历史链），客户端据此分组

## QQ 邮箱特性

- IMAP 文件夹用英文名：`Sent Messages`（非"已发送"）、`Archives`（非"归档"）
- SMTP 发送后 IMAP Sent Messages 同步有延迟，不会立即出现
- UID MOVE 可能不支持，fallback 到 COPY + STORE \\Deleted + EXPUNGE

## .env 加载策略

- 多目录 fallback 列表：先加载脚本自身目录 `.env`，再加载项目根目录 `.env`
- 用 `os.environ.setdefault()` 保证先加载的值优先，不覆盖
- 适用场景：部署目录和开发目录有不同凭证，需合并加载
