---
name: Email Assistant 架构决策
description: Email Assistant 项目的关键设计决策及理由 — 遇到类似邮件助手项目时参考架构选型
type: concept
updated: 2026-05-11
tags: [email, architecture, decision, claude-code, skill]
sources: [[sources/email-assistant-journey.md]]
---
# Email Assistant 架构决策

## 部署形态：Claude Code Skill

**决策**：纯 CLI + SKILL.md 自然语言指令，部署为 Claude Code Skill。

**理由**：用户通过自然语言操作邮件，AI 解析意图并调用 CLI。无需 Web/API 层，终端输出由 AI 格式化为表格。

**替代方案**：Web 界面需要前端+后端，对单用户场景过度工程。

## 正文双层提取

**决策**：同时保留 `extract_body()`（纯文本）和 `extract_html_body()`（原始 HTML）。

**理由**：终端展示需要纯文本，转发需要保留 HTML 格式。只存一种会丢失信息。

**替代方案**：只存 HTML，展示时 strip——但 strip 会丢失结构，转发纯文本邮件无 HTML 可用。

## 客户端过滤

**决策**：IMAP FETCH 全量拉取，Python 侧按日期/发件人/主题过滤。

**理由**：QQ IMAP SEARCH 不完全支持所有条件（尤其是日期范围和中文匹配），客户端过滤更可控。

**替代方案**：依赖 IMAP SEARCH 条件——无法保证跨邮箱兼容。

## Newsletter 检测与拆解

**决策**：由 AI 在 SKILL.md 指令中完成 Newsletter 识别和文章提取，不在 Python 中硬编码。

**理由**：Newsletter 格式各异（Nature Briefing、AI Weekly、PyCoder's 结构完全不同），Python 正则解析太脆弱。AI 天然擅长非结构化文本理解。

**替代方案**：Python 预处理——需要为每种 Newsletter 维护解析规则，不可扩展。

## 发送后验证

**决策**：SMTP 发送成功后，检查 IMAP Sent Messages 确认邮件出现。

**理由**：SMTP 成功只表示服务器接收了，不代表邮件进入发件箱。双重确认更可靠。

**替代方案**：只信任 SMTP 返回码——QQ 邮箱偶尔 SMTP 成功但 Sent Messages 不显示。

## 邮件分类简化

**决策**：只保留"需要关注"和"订阅/通知"两级分类，删除"待处理"。

**理由**：三级分类增加认知负担，实际使用中"待处理"和"需要关注"难以区分，AI 二分法足够。

## HTML 转发保留格式

**决策**：转发 HTML 邮件时发送 multipart/alternative（纯文本 + 原始 HTML），HTML 部分用 div 容器包裹。

**理由**：Gmail 等客户端优先渲染 HTML 版本，原始 HTML 直接嵌入可保留完整排版（图片、链接、样式）。

**替代方案**：转发时 strip HTML 为纯文本——丢失所有排版和图片。
