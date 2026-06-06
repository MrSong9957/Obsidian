---
name: 个人 Claude Code 配置
description: 个人 Claude Code 配置偏好 — 记录用户主动选择的设置、推荐配置理由、配置历史
type: concept
updated: 2026-05-30
tags: claude-code, settings, personal, config
source: https://example.com
---

# 个人 Claude Code 配置

> 关联文档：[[wiki/claude-code-setup.md|Claude Code 当前配置清单]] | [[sources/claude-code/settings-reference.md|完整设置清单]] | [[wiki/claude-code-p1.md|Claude Code 配置]]

本文档记录个人的 Claude Code 配置偏好，作为配置的统一入口。

**重要：** 所有配置修改都应使用 `update-config` 技能，避免手动编辑出错。

## 当前已启用的设置

### 核心功能

| 设置 | 值 | 启用日期 | 说明 |
|------|-----|----------|------|
| `autoMemoryEnabled` | `true` | 2026-05-30 | 启用自动记忆，跨会话记住项目上下文 |
| `fileCheckpointingEnabled` | `true` | 2026-05-30 | 编辑前自动快照，支持 `/rewind` 撤销 |
| `autoCompactEnabled` | `true` | 2026-05-30 | 上下文满时自动压缩，避免会话中断 |
| `todoFeatureEnabled` | `true` | 2026-05-30 | 启用任务追踪面板，多步骤任务更清晰 |

### UI 与交互

| 设置 | 值 | 启用日期 | 说明 |
|------|-----|----------|------|
| `showClearContextOnPlanAccept` | `true` | 2026-05-30 | 计划审批时显示"清除上下文"选项 |
| `showTurnDuration` | `true` | 2026-05-30 | 显示每轮对话耗时 |
| `showMessageTimestamps` | `true` | 2026-05-30 | 显示消息时间戳 |
| `terminalProgressBarEnabled` | `true` | 2026-05-30 | 启用终端进度条 |
| `spinnerTipsEnabled` | `true` | 2026-05-30 | 启用加载提示 |
| `statusLine` | 综合版 | 2026-06-01 | 模型/目录/Git分支/进度条/当前任务/成本指标/安全网 |

### 模型与性能

| 设置 | 值 | 启用日期 | 说明 |
|------|-----|----------|------|
| `alwaysThinkingEnabled` | `true` | - | 启用扩展思考 |
| `effortLevel` | `max` | - | 思考深度设为最高（通过环境变量 `CLAUDE_CODE_EFFORT_LEVEL=max`） |
| `showThinkingSummaries` | `true` | 2026-05-30 | 显示思考摘要 |
| 上下文窗口 | `默认` | 2026-05-30 | 使用默认上下文，不指定 [200k] 或 [1m] |

### 权限与安全

| 设置 | 值 | 启用日期 | 说明 |
|------|-----|----------|------|
| `permissions.defaultMode` | `auto` | 2026-05-30 | 默认权限模式，大部分操作自动批准 |
| `skipDangerousModePermissionPrompt` | `true` | - | 跳过危险模式确认 |
| `skipAutoPermissionPrompt` | `true` | - | 跳过自动权限确认 |
| `disableRemoteControl` | `true` | 2026-05-30 | 禁用远程控制，提高安全性 |

### 语言与通知

| 设置 | 值 | 启用日期 | 说明 |
|------|-----|----------|------|
| `language` | `chinese` | 2026-05-30 | 首选响应语言为中文 |
| `preferredNotifChannel` | `auto` | 2026-05-30 | 通知渠道自动选择 |

## 推荐配置理由

### 自动记忆（`autoMemoryEnabled`）
- **原因：** 让 Claude 跨会话记住项目上下文，避免每次会话都要重新解释
- **效果：** Claude 会自动记住项目的关键信息，下次会话不用重新解释

### 文件快照（`fileCheckpointingEnabled`）
- **原因：** 编辑前自动备份，可以用 `/rewind` 恢复
- **效果：** 每次编辑文件前自动创建快照，支持撤销操作

### 自动压缩（`autoCompactEnabled`）
- **原因：** 上下文快满时自动压缩，避免会话突然中断
- **效果：** 上下文快满时自动压缩，保持会话连续性

### 任务追踪面板（`todoFeatureEnabled`）
- **原因：** 复杂任务时显示进度条，清晰看到完成了什么
- **效果：** 多步骤任务时显示任务面板，便于跟踪进度

### 清除上下文选项（`showClearContextOnPlanAccept`）
- **原因：** 计划审批时可以清除上下文，释放窗口空间
- **效果：** 计划审批对话框多出一个"清除上下文"按钮

### 显示对话耗时（`showTurnDuration`）
- **原因：** 了解 Claude 处理时间，优化 prompt
- **效果：** 每轮对话后显示处理时间

### 显示消息时间戳（`showMessageTimestamps`）
- **原因：** 追踪会话时间线
- **效果：** 每条消息显示时间戳

### 终端进度条（`terminalProgressBarEnabled`）
- **原因：** 长时间操作有进度提示
- **效果：** 长时间操作显示进度条

### 加载提示（`spinnerTipsEnabled`）
- **原因：** 等待时显示有用提示
- **效果：** 等待时显示提示信息

### 思考摘要（`showThinkingSummaries`）
- **原因：** 了解 Claude 的推理过程
- **效果：** 在对话中显示 Claude 的思考摘要

### 自动权限模式（`permissions.defaultMode: auto`）
- **原因：** 大部分操作自动批准，减少确认提示
- **效果：** 提高工作效率，仍然拦截危险操作

### 禁用远程控制（`disableRemoteControl`）
- **原因：** 提高安全性，防止未经授权的远程访问
- **效果：** 禁用远程控制功能

### 中文语言（`language: chinese`）
- **原因：** 确保所有输出都是中文
- **效果：** Claude 默认用中文回复

### 上下文窗口（200k）
- **原因：** 1M 上下文对 token 消耗太大，日常任务 200k 完全够用，性价比更高
- **效果：** 默认使用 200k 上下文，需要时可通过 `/model` 切换到 1M（第三方 API 下仍可选）
- **后备：** 遇到上下文不够用时，优先考虑压缩/摘要而非直接切 1M

## 配置历史

| 日期 | 操作 | 说明 |
|------|------|------|
| 2026-05-30 | 启用 4 个推荐设置 | autoMemoryEnabled、fileCheckpointingEnabled、autoCompactEnabled、todoFeatureEnabled |
| 2026-05-30 | 启用 showClearContextOnPlanAccept | 计划审批时显示"清除上下文"选项 |
| 2026-05-30 | 设置 permissions.defaultMode 为 auto | 默认权限模式，大部分操作自动批准 |
| 2026-05-30 | 启用 disableRemoteControl | 禁用远程控制，提高安全性 |
| 2026-05-30 | 设置 language 为 chinese | 首选响应语言为中文 |
| 2026-05-30 | 启用 UI 显示设置 | showTurnDuration、showMessageTimestamps、terminalProgressBarEnabled、spinnerTipsEnabled |
| 2026-05-30 | 启用 showThinkingSummaries | 显示思考摘要 |
| 2026-05-30 | 设置上下文窗口为默认 | 默认上下文，1M 太贵非必要，可通过 /model 切换 |
| 2026-06-07 | 更新 effortLevel 文档值 | 从 `xhigh` 更正为 `max`，与实际环境变量一致 |
| 2026-05-30 | 修改 MiMo 模型声明 | 移除所有上下文后缀，使用默认值 |
| 2026-06-01 | 升级状态栏为综合版 | 融合旧版目录/分支 + ECC 指标/任务，添加 ecc-metrics-bridge hook |

## 如何修改配置

**使用 `update-config` 技能：**
1. 告诉 Claude 你想开启/关闭哪个功能
2. Claude 会使用 `update-config` 技能自动修改 `~/.claude/settings.json`
3. 修改会自动合并，不会覆盖现有配置

**示例：**
- "帮我开启自动记忆"
- "关闭任务追踪面板"
- "配置一个 hook，让 Claude 在写文件后自动格式化"

## 相关文档

- [[wiki/claude-code-setup.md|Claude Code 当前配置清单]] — 本机实际安装配置（插件/Hooks/MCP/技能/代理）
- [[sources/claude-code/settings-reference.md|完整设置清单]] — 100+ 配置项的完整参考
- [[wiki/claude-code-p1.md|Claude Code 配置]] — settings.json/CLAUDE.md/rules/记忆系统
