---
name: settings-reference
description: Claude Code 完整设置清单 — 100+ 配置项按类别组织，包含类型、说明、默认值、可选值
type: reference
updated: 2026-05-30
tags: [claude-code, settings, configuration, reference]
source: update-config skill Settings JSON Schema
---

# Claude Code 完整设置清单

> 来源：`update-config` 技能加载时附带的 Settings JSON Schema

本文档列出 Claude Code `settings.json` 中所有可用的配置项，按类别组织。

## 1. 认证与 API（Authentication & API）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `apiKeyHelper` | string | 输出认证值的脚本路径 | - |
| `proxyAuthHelper` | string | 输出代理认证 header 的命令 | - |
| `awsCredentialExport` | string | AWS 凭证导出脚本路径 | - |
| `awsAuthRefresh` | string | AWS 认证刷新脚本路径 | - |
| `gcpAuthRefresh` | string | GCP 认证刷新命令 | - |
| `forceLoginMethod` | string | 强制登录方式 | 可选：`claudeai`、`console` |
| `forceLoginOrgUUID` | string \| string[] | 强制要求的组织 UUID | - |

## 2. 策略与管理（Policy & Managed）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `policyHelper` | object | 启动时计算托管设置的可执行文件 | - |
| `policyHelper.path` | string | 可执行文件的绝对路径（必填） | - |
| `policyHelper.timeoutMs` | integer | 超时时间（1000-9007199254740991） | - |
| `policyHelper.refreshIntervalMs` | number | 刷新间隔（0 或 60000-9007199254740991） | - |
| `parentSettingsBehavior` | string | 父设置层行为 | 可选：`first-wins`、`merge` |
| `forceRemoteSettingsRefresh` | boolean | 启动时强制刷新远程托管设置 | false |
| `claudeMd` | string | 组织管理的 CLAUDE.md 指令 | - |
| `claudeMdExcludes` | string[] | 排除的 CLAUDE.md 文件模式 | - |
| `companyAnnouncements` | string[] | 启动时显示的公司公告 | - |

## 3. 文件与 Git（File & Git）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `fileSuggestion` | object | @ 提及的自定义文件建议配置 | - |
| `fileSuggestion.type` | string | 类型（必须为 `command`） | - |
| `fileSuggestion.command` | string | 命令（必填） | - |
| `respectGitignore` | boolean | 文件选择器是否遵守 .gitignore | true |
| `cleanupPeriodDays` | integer | 会话记录保留天数（最小 1） | 30 |

## 4. UI 与显示（UI & Display）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `theme` | string | 颜色主题 | `auto` |
| `editorMode` | string | 键盘模式 | `normal` |
| `verbose` | boolean | 显示完整工具输出 | false |
| `syntaxHighlightingDisabled` | boolean | 禁用 diff 语法高亮 | false |
| `showTurnDuration` | boolean | 显示每轮对话耗时 | false |
| `showMessageTimestamps` | boolean | 显示消息时间戳 | false |
| `terminalProgressBarEnabled` | boolean | 启用终端进度条 | - |
| `spinnerTipsEnabled` | boolean | 加载时显示提示 | - |
| `spinnerVerbs` | object | 自定义加载动词 | - |
| `spinnerTipsOverride` | object | 覆盖加载提示 | - |
| `tui` | string | 终端 UI 渲染器 | 可选：`default`、`fullscreen` |
| `viewMode` | string | 默认转录视图模式 | 可选：`default`、`verbose`、`focus` |
| `defaultView` | string | 默认视图 | 可选：`chat`、`transcript` |
| `terminalTitleFromRename` | boolean | /rename 是否更新终端标题 | true |
| `prefersReducedMotion` | boolean | 减少动画以提高可访问性 | false |
| `showClearContextOnPlanAccept` | boolean | 计划审批时显示"清除上下文"选项 | false |

**theme 可选值：**
- `auto` — 跟随系统
- `dark` — 深色主题
- `light` — 浅色主题
- `light-daltonized` — 浅色（色盲友好）
- `dark-daltonized` — 深色（色盲友好）
- `light-ansi` — 浅色 ANSI
- `dark-ansi` — 深色 ANSI
- `custom:*` — 自定义主题

## 5. 模型与性能（Model & Performance）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `model` | string | 默认模型 | - |
| `availableModels` | string[] | 允许的模型白名单 | - |
| `modelOverrides` | object | 模型 ID 覆盖映射 | - |
| `alwaysThinkingEnabled` | boolean | 启用扩展思考 | true |
| `effortLevel` | string | 思考深度 | 可选：`low`、`medium`、`high`、`xhigh`、`max` |
| `fastMode` | boolean | 快速模式 | false |
| `fastModePerSessionOptIn` | boolean | 快速模式不跨会话持久化 | false |
| `autoCompactEnabled` | boolean | 上下文满时自动压缩 | - |
| `autoCompactWindow` | integer | 自动压缩窗口大小（100000-1000000） | - |
| `showThinkingSummaries` | boolean | 显示思考摘要 | - |
| `advisorModel` | string | 顾问模型 | - |

## 6. 权限与安全（Permissions & Security）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `permissions.defaultMode` | string | 默认权限模式 | `default` |
| `permissions.allow` | string[] | 允许的工具规则列表 | [] |
| `permissions.deny` | string[] | 拒绝的工具规则列表 | [] |
| `permissions.ask` | string[] | 需要确认的工具规则列表 | [] |
| `permissions.disableBypassPermissionsMode` | string | 禁用绕过权限模式 | 可选：`disable` |
| `permissions.disableAutoMode` | string | 禁用自动模式 | 可选：`disable` |
| `permissions.additionalDirectories` | string[] | 额外权限目录 | [] |
| `skipDangerousModePermissionPrompt` | boolean | 跳过危险模式确认 | false |
| `skipAutoPermissionPrompt` | boolean | 跳过自动模式确认 | false |

**defaultMode 可选值：**
- `default` — 默认模式（每次操作都询问）
- `plan` — 计划模式（只读操作自动批准）
- `acceptEdits` — 接受编辑模式（编辑操作自动批准）
- `auto` — 自动模式（大部分操作自动批准）
- `dontAsk` — 不询问模式（所有操作自动批准）
- `bypassPermissions` — 绕过权限模式（危险）

## 7. 记忆与持久化（Memory & Persistence）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `autoMemoryEnabled` | boolean | 启用自动记忆 | - |
| `autoMemoryDirectory` | string | 自定义记忆存储路径 | - |
| `autoDreamEnabled` | boolean | 启用后台记忆整合 | - |
| `fileCheckpointingEnabled` | boolean | 编辑前快照文件（支持 /rewind） | - |

## 8. MCP 服务器（MCP Servers）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `enableAllProjectMcpServers` | boolean | 自动批准项目所有 MCP 服务器 | false |
| `enabledMcpjsonServers` | string[] | 已批准的 MCP 服务器列表 | [] |
| `disabledMcpjsonServers` | string[] | 已拒绝的 MCP 服务器列表 | [] |
| `allowedMcpServers` | object[] | 企业允许的 MCP 服务器白名单 | - |
| `deniedMcpServers` | object[] | 企业拒绝的 MCP 服务器黑名单 | - |

## 9. 插件（Plugins）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `enabledPlugins` | object | 启用的插件（plugin@marketplace 格式） | - |
| `extraKnownMarketplaces` | object | 额外的市场源 | - |
| `strictKnownMarketplaces` | array | 严格的市场源白名单 | - |
| `blockedMarketplaces` | array | 阻止的市场源 | - |
| `pluginConfigs` | object | 每个插件的配置 | - |
| `pluginTrustMessage` | string | 插件信任警告的自定义消息 | - |
| `skillOverrides` | object | 技能覆盖配置 | - |

**enabledPlugins 格式：**
```json
{
  "plugin-name@marketplace-id": true
}
```

**skillOverrides 可选值：**
- `on` — 启用
- `name-only` — 只显示名称（无描述）
- `user-invocable-only` — 只对用户调用可见
- `off` — 禁用

## 10. Hooks（自动化钩子）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `hooks.PreToolUse` | array | 工具执行前 | [] |
| `hooks.PostToolUse` | array | 工具执行后 | [] |
| `hooks.PostToolUseFailure` | array | 工具执行失败后 | [] |
| `hooks.Stop` | array | 会话停止时 | [] |
| `hooks.SessionStart` | array | 会话开始时 | [] |
| `hooks.SessionEnd` | array | 会话结束时 | [] |
| `hooks.PreCompact` | array | 压缩前 | [] |
| `hooks.PostCompact` | array | 压缩后 | [] |
| `hooks.Notification` | array | 通知时 | [] |
| `hooks.UserPromptSubmit` | array | 用户提交提示时 | [] |
| `hooks.PermissionRequest` | array | 权限请求时 | [] |
| `disableAllHooks` | boolean | 禁用所有钩子 | false |

**Hook 结构：**
```json
{
  "matcher": "ToolName|OtherTool",
  "hooks": [
    {
      "type": "command",
      "command": "your-command-here",
      "timeout": 60,
      "statusMessage": "Running..."
    }
  ]
}
```

## 11. 工作树（Worktree）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `worktree.symlinkDirectories` | string[] | 符号链接目录列表 | [] |
| `worktree.sparsePaths` | string[] | 稀疏检出路径 | [] |
| `worktree.baseRef` | string | 新工作树的基础引用 | `fresh` |
| `worktree.bgIsolation` | string | 后台会话隔离模式 | `worktree` |

**baseRef 可选值：**
- `fresh` — 从 origin/<默认分支> 分支（干净树）
- `head` — 从当前本地 HEAD 分支（包含未推送的提交）

**bgIsolation 可选值：**
- `worktree` — 阻止在主 checkout 中编辑
- `none` — 允许后台作业直接编辑工作副本

## 12. 状态栏（Status Line）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `statusLine.type` | string | 类型（必须为 `command`） | - |
| `statusLine.command` | string | 命令 | - |
| `statusLine.padding` | number | 填充 | - |
| `statusLine.refreshInterval` | number | 刷新间隔（秒） | - |
| `statusLine.hideVimModeIndicator` | boolean | 隐藏 Vim 模式指示器 | false |
| `subagentStatusLine` | object | 子代理状态栏配置 | - |

## 13. 沙箱（Sandbox）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `sandbox.enabled` | boolean | 启用沙箱 | false |
| `sandbox.failIfUnavailable` | boolean | 沙箱不可用时失败 | false |
| `sandbox.autoAllowBashIfSandboxed` | boolean | 沙箱中自动允许 Bash | - |
| `sandbox.allowUnsandboxedCommands` | boolean | 允许非沙箱命令 | true |
| `sandbox.network.allowedDomains` | string[] | 允许的网络域名 | [] |
| `sandbox.network.deniedDomains` | string[] | 拒绝的网络域名 | [] |
| `sandbox.network.allowLocalBinding` | boolean | 允许本地绑定 | - |
| `sandbox.network.allowAllUnixSockets` | boolean | 允许所有 Unix 套接字 | false |
| `sandbox.filesystem.allowWrite` | string[] | 允许写入的路径 | [] |
| `sandbox.filesystem.denyWrite` | string[] | 拒绝写入的路径 | [] |
| `sandbox.filesystem.denyRead` | string[] | 拒绝读取的路径 | [] |
| `sandbox.filesystem.allowRead` | string[] | 允许读取的路径 | [] |
| `sandbox.excludedCommands` | string[] | 排除的命令 | [] |

## 14. 通知（Notifications）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `preferredNotifChannel` | string | 首选通知渠道 | `auto` |
| `inputNeededNotifEnabled` | boolean | 权限提示时推送通知 | - |
| `agentPushNotifEnabled` | boolean | 允许 Claude 主动推送通知 | - |

**preferredNotifChannel 可选值：**
- `auto` — 自动选择
- `iterm2` — iTerm2
- `iterm2_with_bell` — iTerm2（带铃声）
- `terminal_bell` — 终端铃声
- `kitty` — Kitty
- `ghostty` — Ghostty
- `notifications_disabled` — 禁用通知

## 15. 语音（Voice）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `voiceEnabled` | boolean | 启用语音模式 | false |
| `voice.mode` | string | 模式 | `hold` |
| `voice.autoSubmit` | boolean | 松开时自动提交 | false |

**voice.mode 可选值：**
- `hold` — 按住说话
- `tap` — 点击开始，点击停止

## 16. 自动模式（Auto Mode）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `autoMode.allow` | string[] | 允许规则 | - |
| `autoMode.soft_deny` | string[] | 软拒绝规则 | - |
| `autoMode.hard_deny` | string[] | 硬拒绝规则 | - |
| `autoMode.environment` | string[] | 环境规则 | - |
| `useAutoModeDuringPlan` | boolean | 计划模式使用自动模式语义 | true |

**规则中可使用 `$defaults` 继承内置规则。**

## 17. 远程控制（Remote Control）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `remoteControlAtStartup` | boolean | 启动时自动启动远程控制 | false |
| `isolatePeerMachines` | boolean | 需要显式批准跨机器通信 | false |
| `remote.defaultEnvironmentId` | string | 默认远程环境 ID | - |

## 18. SSH 配置（SSH Configs）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `sshConfigs` | object[] | SSH 连接配置列表 | [] |

**sshConfigs 元素结构：**
```json
{
  "id": "unique-id",
  "name": "Display Name",
  "sshHost": "user@hostname",
  "sshPort": 22,
  "sshIdentityFile": "~/.ssh/id_rsa",
  "startDirectory": "~/projects"
}
```

## 19. 会话管理（Session Management）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `autoUploadSessions` | boolean | 自动上传会话到 claude.ai | false |
| `daemonColdStart` | string | 后台服务启动方式 | 可选：`transient`、`ask` |

## 20. 其他（Other）

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `language` | string | 首选响应语言 | - |
| `attribution.commit` | string | Git 提交的自定义归属 | - |
| `attribution.pr` | string | PR 描述的自定义归属 | - |
| `includeCoAuthoredBy` | boolean | 是否包含 Claude 共同作者（已弃用） | true |
| `includeGitInstructions` | boolean | 是否包含 Git 工作流指令 | true |
| `prUrlTemplate` | string | PR 链接的 URL 模板 | - |
| `outputStyle` | string | 响应输出样式 | - |
| `plansDirectory` | string | 计划文件目录 | - |
| `feedbackSurveyRate` | number | 反馈调查出现概率（0-1） | - |
| `todoFeatureEnabled` | boolean | 启用任务追踪面板 | - |
| `teammateMode` | string | 团队成员执行方式 | 可选：`auto`、`tmux`、`in-process` |
| `channelsEnabled` | boolean | 启用频道通知 | - |
| `allowedChannelPlugins` | object[] | 允许的频道插件 | - |
| `doneMeansMerged` | boolean | 完成意味着可合并 | false |
| `skipWebFetchPreflight` | boolean | 跳过 WebFetch 预检 | false |
| `disableAgentView` | boolean | 禁用代理视图 | false |
| `disableRemoteControl` | boolean | 禁用远程控制 | false |
| `disableWorkflows` | boolean | 禁用工作流 | false |
| `disableSkillShellExecution` | boolean | 禁用技能中的 Shell 执行 | false |
| `defaultShell` | string | 默认 Shell | `bash` |
| `disableDeepLinkRegistration` | string | 禁用深度链接注册 | 可选：`disable` |
| `$schema` | string | JSON Schema 引用 | - |

## 常用配置场景

### 场景 1：开启自动记忆
```json
{
  "autoMemoryEnabled": true,
  "autoMemoryDirectory": "~/.claude/projects/my-project/memory"
}
```

### 场景 2：配置自动格式化 Hook
```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "prettier --write $FILE"
          }
        ]
      }
    ]
  }
}
```

### 场景 3：允许 npm 命令不弹确认
```json
{
  "permissions": {
    "allow": ["Bash(npm *)"]
  }
}
```

### 场景 4：开启快速模式和扩展思考
```json
{
  "fastMode": true,
  "alwaysThinkingEnabled": true,
  "effortLevel": "max"
}
```

### 场景 5：配置状态栏
```json
{
  "statusLine": {
    "type": "command",
    "command": "python ~/.claude/scripts/statusline.py"
  }
}
```

### 场景 6：推荐的默认配置
```json
{
  "autoMemoryEnabled": true,
  "fileCheckpointingEnabled": true,
  "autoCompactEnabled": true,
  "todoFeatureEnabled": true,
  "showClearContextOnPlanAccept": true,
  "alwaysThinkingEnabled": true
}
```

**说明：**
- `autoMemoryEnabled` — 启用自动记忆，让 Claude 跨会话记住项目上下文
- `fileCheckpointingEnabled` — 编辑前自动快照，支持 `/rewind` 撤销
- `autoCompactEnabled` — 上下文满时自动压缩，避免会话中断
- `todoFeatureEnabled` — 启用任务追踪面板，多步骤任务更清晰
- `showClearContextOnPlanAccept` — 计划审批时显示"清除上下文"选项
- `alwaysThinkingEnabled` — 启用扩展思考
