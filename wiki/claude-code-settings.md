---
name: Claude Code 官方配置速查
description: Claude Code settings.json 配置项集中记录，含键名、默认值、说明
type: reference
updated: 2026-04-17
tags: [claude-code, settings, configuration]
originSessionId: 5dd80a21-b6a0-4125-a3a2-6b822be29288
status: deprecated
---
# Claude Code 官方配置速查

> 内容已迁移至 [[wiki/claude-code-p1.md|claude-code-p1]]。

配置位置：`~/.claude/settings.json`（全局）/ `.claude/settings.json`（项目）/ `.claude/settings.local.json`（本地覆盖）

---

## showClearContextOnPlanAccept

- **默认值：** `false`（v2.1.81 起隐藏）
- **说明：** Plan 模式审批对话框中显示"clear context"选项。开启后退出 plan 时可选择清空上下文再执行，释放上下文窗口让执行更聚焦。计划文件保存在 `~/.claude/plans/` 不受 clear 影响。
- **配置：**

```json
{ "showClearContextOnPlanAccept": true }
```

- **相关 Issue：** [#39665](https://github.com/anthropics/claude-code/issues/39665) [#38071](https://github.com/anthropics/claude-code/issues/38071) [#25734](https://github.com/anthropics/claude-code/issues/25734) [#19426](https://github.com/anthropics/claude-code/issues/19426)
