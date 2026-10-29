---
name: Claude Code 首次安装配置指南
description: 首次安装或初始化 Claude Code 后的完整配置流程 — 新环境首次配置时按此文档执行
type: concept
updated: 2026-05-07
tags: claude-code, setup, plugins, first-time
source: https://example.com
---

# Claude Code 首次安装配置指南

> 本文档覆盖首次安装 Claude Code 后需要完成的所有配置。
> 标记 🤖 的步骤可由 Claude Code 自动完成，标记 👤 的步骤需要用户手动操作。

---

## 1. 安装官方插件市场

👤 **需要用户手动操作**（Claude Code 的 `/plugin` 命令是交互式 CLI 指令，无法通过 bash 执行）

在 Claude Code 中输入：

```
/plugin
```

进入插件市场 → **Marketplaces** → **+ Add Marketplace** → 输入：

```
https://github.com/anthropics/claude-plugins-official.git
```

## 2. 安装官方插件

👤 **需要用户手动操作**

继续在 `/plugin` 界面，从官方市场安装以下插件：

| 插件 | 用途 |
|------|------|
| **GitHub** | GitHub PR/Issue 操作 |
| **context7** | 实时查询库/框架最新文档 |
| **skill-creator** | 创建和管理自定义技能 |
| **learning-output-style** | 学习模式输出风格 |
| **superpowers** | 七阶段开发工作流（TDD、brainstorming、代码审查等） |
| **frontend-design** | 高质量前端界面生成 |

## 3. 安全网插件（Safety Net）

### 3a. 添加第三方市场并安装

👤 **需要用户手动操作**（交互式 `/plugin` 命令）

```
/plugin marketplace add kenryu42/cc-marketplace
/plugin install safety-net@cc-marketplace
```

Safety Net 是被动式拦截插件，防止执行危险命令（如 `rm -rf`、`git push --force`、`git reset --hard`）。

### 3b. 配置 Hook

🤖 **Claude Code 可自动完成** — 安装后需确认 `~/.claude/settings.json` 的 `hooks` 中有 Safety Net 的 PreToolUse Hook。如果缺失，参考以下配置添加：

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "npx cc-safety-net --hook \"$TOOL_INPUT\""
          }
        ]
      }
    ]
  }
}
```

配置后执行 `/reload-plugins` 使 Hook 生效。

## 4. 状态栏插件（StatusLine）

📋 **参考独立文档**：`sources/claude-code-statusline-setup.md`

状态栏配置包括：
- Python 脚本显示模型名、工作目录、上下文使用率
- 管道接入 Safety Net 状态栏显示安全网状态

配置目标（参考值）：

```json
{
  "statusLine": {
    "type": "command",
    "command": "python3 ~/.claude/scripts/statusline.py | npx cc-safety-net --statusline"
  }
}
```

## 5. 智谱 GLM MCP 服务器

📋 **参考独立文档**：`wiki/zhipu-mcp.md`（速查）/ `sources/claude-code/zhipu-mcp.md`（完整安装指南）

包含 4 个 MCP 服务器：视觉理解、联网搜索、网页读取、开源仓库查询。

## 6. 验证清单

全部配置完成后，逐项验证：

```
□ /plugin → 确认 7 个插件全部显示已安装
□ /mcp → 确认 4 个智谱 MCP 显示 Connected
□ 输入任意 bash 命令 → StatusLine 正常显示模型信息和上下文百分比
□ 执行危险命令（如 git push --force）→ Safety Net 拦截并提示
```

---

## 配置参考：完整的 settings.json 示例

以下是所有配置完成后的 `~/.claude/settings.json` 关键字段（环境变量和 MCP 服务器配置因环境而异，此处不包含）：

```json
{
  "statusLine": {
    "type": "command",
    "command": "python3 ~/.claude/scripts/statusline.py | npx cc-safety-net --statusline"
  },
  "enabledPlugins": {
    "frontend-design@claude-plugins-official": true,
    "superpowers@claude-plugins-official": true,
    "context7@claude-plugins-official": true,
    "github@claude-plugins-official": true,
    "skill-creator@claude-plugins-official": true,
    "learning-output-style@claude-plugins-official": true,
    "safety-net@cc-marketplace": true
  },
  "extraKnownMarketplaces": {
    "cc-marketplace": {
      "source": {
        "source": "github",
        "repo": "kenryu42/cc-marketplace"
      }
    }
  }
}
```
