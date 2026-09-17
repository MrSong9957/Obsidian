---
name: 智谱MCP
description: 智谱MCP服务器配置速查/Web抓取缓存与JS渲染问题 — 安装或排查智谱MCP、遇到页面抓取问题时参考
type: concept
updated: 2026-05-07
tags: zhipu, mcp, web-scraping, caching
source: "[[sources/claude-code/zhipu-mcp]]"
---
# 智谱 MCP

## Web Reader 对 JS 渲染页面可能遗漏内容

- Web reader 只解析服务端 HTML，不执行 JS，SPA/Next.js 页面内容可能丢失
- 禁止：对 JS 渲染页面重复使用 web reader
  - 原因：reader 不执行 JS，客户端内容丢失，实测遗漏两次才发现
  - 替代：首次失败后直接 `curl -sL URL | grep 关键词` 搜索原始 HTML

> 详细内容见 sources/claude-code/zhipu-mcp.md

## Web Reader 缓存可能返回过时内容

- Web reader 有缓存机制，对同一 URL 可能返回旧版本内容而非最新版本
- 禁止：首次抓取内容与用户描述不符时，直接信任抓取结果
  - 原因：缓存导致返回过时页面，实测 learn.shareai.run/zh/s07/ 第一次抓到旧版（任务图系统），用户实际看到的是新版（权限系统）
  - 替代：用 `no_cache=true` 参数重新抓取，或与用户交叉验证内容是否匹配

## 智谱 MCP 服务器速查

- zai-mcp-server：视觉理解（截图转代码、OCR、图表分析），stdio 本地
- web-search-prime：联网搜索，HTTP 云端
- web-reader：网页读取，HTTP 云端
- zread：GitHub 仓库文档/目录/文件查询，HTTP 云端
- 安装：`claude mcp add` 一键安装或编辑 `~/.claude.json` 手动配置
- 验证：`claude mcp list` 确认 Connected
- 实测（2026-04-08）：4 个 MCP 均成功；视觉 MCP 对 Wikipedia 缩略图报 400，CDN 直链正常
- .env 坑：Key 可能叫 `ANTHROPIC_API_KEY` 而非 `ZHIPU_API_KEY`；文件可能有 `\r\n` 换行

> 详细内容见 sources/claude-code/zhipu-mcp.md
