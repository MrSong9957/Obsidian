---
name: Obsidian
description: Obsidian CLI工具使用/知识仓库管理 — 遇到Obsidian操作或知识管理问题时参考
type: concept
updated: 2026-04-17
tags: [obsidian, knowledge-management]
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# Obsidian

> Obsidian 1.12+ CLI，需 Catalyst Insider 许可证，桌面版需正在运行

## 常用命令

- `obsidian daily` — 打开今日日记
- `obsidian daily:append content="..."` — 向日记追加内容
- `obsidian search query="关键词"` — 全库搜索
- `obsidian tasks daily` — 列出日记中的任务
- `obsidian tags counts` / `tags list` — 标签统计/列表
- `obsidian files sort=modified limit=5` — 最近修改文件
- `obsidian create name="xxx" template=Tpl` — 从模板创建
- `obsidian sync --headless` — 无 GUI 同步（CI/自动化）
- `obsidian` — 无参数进入 TUI 交互模式

> 详细内容见 sources/tools/obsidian.md
