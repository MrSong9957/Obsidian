---
name: 网络与 API
description: Web抓取JS渲染问题/MCP服务器配置/API调用经验 — 遇到网络请求、页面抓取、MCP配置问题时参考
type: concept
updated: 2026-04-17
tags: [network, api, web-scraping, mcp]
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# 网络与 API

## Web Reader 对 JS 渲染页面可能遗漏内容

- Web reader 只解析服务端 HTML，不执行 JS，SPA/Next.js 页面内容可能丢失
- 禁止：对 JS 渲染页面重复使用 web reader
  - 原因：reader 不执行 JS，客户端内容丢失，实测遗漏两次才发现
  - 替代：首次失败后直接 `curl -sL URL | grep 关键词` 搜索原始 HTML

> 详细内容见 sources/network-api.md

## 智谱 MCP 服务器速查

- zai-mcp-server：视觉理解（截图转代码、OCR、图表分析），stdio 本地
- web-search-prime：联网搜索，HTTP 云端
- web-reader：网页读取，HTTP 云端
- zread：GitHub 仓库文档/目录/文件查询，HTTP 云端
- 安装：`claude mcp add` 一键安装或编辑 `~/.claude.json` 手动配置
- 验证：`claude mcp list` 确认 Connected
- 实测（2026-04-08）：4 个 MCP 均成功；视觉 MCP 对 Wikipedia 缩略图报 400，CDN 直链正常
- .env 坑：Key 可能叫 `ANTHROPIC_API_KEY` 而非 `ZHIPU_API_KEY`；文件可能有 `\r\n` 换行

> 详细内容见 sources/network-api.md
