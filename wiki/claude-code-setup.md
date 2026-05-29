---
name: Claude Code 当前配置清单
description: 本机 Claude Code 完整配置清单（插件/Hooks/状态栏/MCP/技能/代理/规则/团队）— 排查配置问题、新环境迁移时参考
type: concept
updated: 2026-05-29
tags: [claude-code, setup, plugins, hooks, mcp, skills, agents, config]
---

# Claude Code 当前配置清单

> 关联概念：[[wiki/claude-code-p1.md|Claude Code 配置]] | [[wiki/claude-code-p2.md|Claude Code 工具]] | [[wiki/claude-code-p3.md|Claude Code 进阶工作流]]

本文档记录本机 Claude Code 的实际安装配置，作为排查和迁移的参考。配置变更时需同步更新本文档。

## 全局设置概要

配置文件路径：`~/.claude/settings.json`（335 行）

| 配置项 | 值 | 说明 |
|--------|-----|------|
| `defaultMode` | `"auto"` | 自动批准所有工具调用 |
| `alwaysThinkingEnabled` | `true` | 始终启用扩展思考 |
| `effortLevel` | `"high"` | 高推理强度 |
| `skipDangerousModePermissionPrompt` | `true` | 跳过危险模式确认 |
| `skipAutoPermissionPrompt` | `true` | 跳过自动权限确认 |
| `includeCoAuthoredBy` | `false` | 禁用 co-authored-by 署名 |
| 自动更新 | 已禁用 | `CLAUDE_CODE_AUTO_UPDATE=0` |
| Tool Search | 已启用 | `CLAUDE_CODE_TOOL_SEARCH_ENABLED=1` |
| Agent Teams | 已启用 | `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` |

### API 配置

| 配置项 | 值 |
|--------|-----|
| 自定义端点 | `https://token-plan-cn.xiaomimimo.com/anthropic` |
| 模型映射 | 所有模型 → `mimo-v2.5` / `mimo-v2.5-pro` |

---

## 已安装插件（8 个）

主要使用两个核心插件，其余为辅助：

### 核心插件

| 插件 | 安装渠道 | 版本 | 安装日期 | 用途 |
|------|----------|------|----------|------|
| **superpowers** | Claude Code 插件市场（`claude-plugins-official`） | 5.1.0 | 2026-03-20 | 技能调度框架，所有任务的起点 |
| **ecc**（Everything Claude Code） | GitHub 仓库（`affaan-m/everything-claude-code`） | 2.0.0-rc.1 | 2026-05-21 | 基础设施层：编码规范、上下文管理、安全审查、持续学习，补充 Superpowers 不覆盖的领域 |

> Superpowers 提供工作流骨架（七阶段开发流程），ECC 提供底层能力（rules、hooks、MCP、session memory）。两者互补，非替代关系。详见 [[wiki/ecc-superpowers-supplement.md|ECC 补强指南]]。

### 辅助插件

| 插件 | 安装渠道 | 版本 | 安装日期 | 用途 |
|------|----------|------|----------|------|
| safety-net | cc-marketplace（`kenryu42/cc-marketplace`） | 0.9.0 | 2026-05-21 | 安全网，拦截危险命令（rm -rf、git push --force 等） |
| github | claude-plugins-official | 9e150cfd48b8 | 2026-03-29 | GitHub 集成 |
| context7 | claude-plugins-official | 9e150cfd48b8 | 2026-03-20 | 实时文档查询 |
| frontend-design | claude-plugins-official | 9e150cfd48b8 | 2026-03-20 | 前端设计辅助 |
| skill-creator | claude-plugins-official | 9e150cfd48b8 | 2026-03-20 | 技能创建工具 |
| learning-output-style | claude-plugins-official | 1.0.0 | 2026-03-20 | 交互式学习输出风格 |

### 安装渠道说明

| 渠道 | 命令 | 说明 |
|------|------|------|
| Claude Code 插件市场 | `/plugin` → 从 `claude-plugins-official` 安装 | 官方市场，Superpowers 等 6 个插件通过此渠道安装 |
| GitHub 仓库 | `/plugin marketplace add <repo>` → `/plugin install` | ECC 通过此渠道安装，需先添加仓库为自定义市场 |
| cc-marketplace | `/plugin marketplace add kenryu42/cc-marketplace` | Safety Net 专用市场 |

### 插件黑名单

- `code-review@claude-plugins-official`
- `fizz@testmkt-marketplace`

---

## 状态栏（StatusLine）

### 配置

```json
"statusLine": {
  "type": "command",
  "command": "python ~/.claude/scripts/statusline.py | npx cc-safety-net --statusline"
}
```

### 管道结构

`statusline.py`（模型/目录/分支/上下文进度条）→ `cc-safety-net --statusline`（附加安全网状态指示器）

### 脚本

- 主脚本：`~/.claude/scripts/statusline.py`（Python 版，推荐）
- 备选脚本：`~/.claude/scripts/statusline.sh`（Bash 版，无 jq 依赖）
- 调试脚本：`~/.claude/scripts/debug_path.py`

### stdin JSON 关键字段

- `model.display_name` — 模型名称
- `workspace.current_dir` — 当前目录（压缩为最后 2 层）
- `context_window.used_percentage` — 上下文窗口使用百分比（带颜色进度条：<50% 绿、<80% 黄、>=80% 红）

### 已知问题

Windows 上 `python` 可能解析为 Windows Store 存根，应使用 `py` 替代。修复计划记录在 `~/.claude/plans/git-goofy-mountain.md`。

---

## MCP 服务器（6 个）

来源：ECC 插件 `.mcp.json`

| 服务器 | 传输方式 | 命令 | 用途 |
|--------|----------|------|------|
| github | stdio | `npx -y @modelcontextprotocol/server-github@2025.4.8` | GitHub API |
| context7 | stdio | `npx -y @upstash/context7-mcp@2.1.4` | 实时文档查询 |
| exa | HTTP | `https://mcp.exa.ai/mcp` | 网页搜索 |
| memory | stdio | `npx -y @modelcontextprotocol/server-memory@2026.1.26` | 知识图谱记忆 |
| playwright | stdio | `npx -y @playwright/mcp@0.0.69 --extension` | 浏览器自动化 |
| sequential-thinking | stdio | `npx -y @modelcontextprotocol/server-sequential-thinking@2025.12.18` | 序列化思考 |

---

## Hooks 系统

### PreToolUse（11 个）

| 匹配器 | 脚本 | 功能 | 超时 | 异步 |
|--------|------|------|------|------|
| `Bash` | `cc-safety-net.js --claude-code` | Safety Net 危险命令拦截 | 15s | - |
| `Bash` | `npx block-no-verify@1.1.2` | 阻止未验证的 git 操作 | - | - |
| `Bash` | `auto-tmux-dev.js` | 自动 tmux 开发环境 | - | - |
| `Bash` | `pre-bash-tmux-reminder.js` | tmux 提醒（strict 模式） | - | - |
| `Bash` | `pre-bash-git-push-reminder.js` | git push 提醒（strict 模式） | - | - |
| `Write` | `doc-file-warning.js` | 写文档文件前警告 | - | - |
| `Edit\|Write` | `suggest-compact.js` | 建议上下文压缩 | - | - |
| `*` | `observe.sh` | 持续学习观察器 | - | async |
| `Bash\|Write\|Edit\|MultiEdit` | `insaits-security-wrapper.js` | 安全检查 | 15s | - |
| `Bash\|Write\|Edit\|MultiEdit` | `governance-capture.js` | 治理日志 | 10s | - |
| `*` | `mcp-health-check.js` | MCP 服务器健康检查 | - | - |

### PostToolUse（8 个）

| 匹配器 | 脚本 | 功能 | 超时 | 异步 |
|--------|------|------|------|------|
| `Bash` | `post-bash-pr-created.js` | PR 创建检测 | - | - |
| `Bash` | `post-bash-build-complete.js` | 构建完成检测 | 30s | async |
| `Edit\|Write\|MultiEdit` | `quality-gate.js` | 代码质量检查 | 30s | async |
| `Edit` | `post-edit-format.js` | 编辑后自动格式化 | - | - |
| `Edit` | `post-edit-typecheck.js` | 编辑后类型检查 | - | - |
| `Edit` | `post-edit-console-warn.js` | console.log 残留检查 | - | - |
| `Bash\|Write\|Edit\|MultiEdit` | `governance-capture.js` | 治理日志 | - | - |
| `*` | `observe.sh` | 持续学习观察器 | - | async |

### 其他事件

| 事件 | 匹配器 | 脚本 | 功能 |
|------|--------|------|------|
| PostToolUseFailure | `*` | `mcp-health-check.js` | MCP 健康检查 |
| PreCompact | `*` | `pre-compact.js` | 压缩前准备 |
| SessionStart | `*` | ECC `session-start.js` | 会话启动初始化 |
| SessionEnd | `*` | `session-end-marker.js` | 会话结束标记（async） |
| Stop | `*` | `check-console-log.js` | console 日志检查 |
| Stop | `*` | `session-end.js` | 会话清理（async） |
| Stop | `*` | `evaluate-session.js` | 会话评估（async） |
| Stop | `*` | `cost-tracker.js` | 成本追踪（async） |

### Hook 脚本来源

Hook 脚本通过环境变量引用插件目录：
- `CLAUDE_PLUGIN_ROOT` → ECC 插件根目录（v1.8.0）
- `SAFETY_NET_PLUGIN_ROOT` → Safety Net 插件根目录（v0.7.1）

---

## 技能（Skills，5 个）

路径：`~/.claude/skills/`

| 技能 | 功能 | 触发方式 |
|------|------|----------|
| `bug-remember/` | 自学习 bug 修复系统，查询已知修复经验库 | 中英文重复 bug 关键词 |
| `docker-dev-container/` | 创建 Docker 开发环境（Claude Code + OpenCode） | `/docker-dev-container` |
| `learned/` | 自动学习技能占位目录（当前为空） | 自动 |
| `obsidian-link/` | Obsidian 仓库与项目记忆 symlink 桥接 | `/obsidian-link` |
| `obsidian-remember/` | Obsidian 知识仓库管理（三层架构：sources/wiki/MEMORY.md） | `/obsidian-remember` |

### 技能依赖链

`obsidian-remember` → `obsidian-link`（remember 依赖 link）

---

## 命令（Commands，1 个）

路径：`~/.claude/commands/`

| 命令 | 功能 |
|------|------|
| `/note-obsidian` | 从当前对话提取知识写入 Obsidian wiki（后台 agent，依赖 `obsidian-remember` 技能） |

---

## 规则（Rules，9 个）

路径：`~/.claude/rules/`

| 文件 | 内容 |
|------|------|
| `agents.md` | 代理编排指南（10 个代理列表、并行执行、多视角分析） |
| `coding-style.md` | 编码风格（不可变性、文件组织、错误处理、输入验证） |
| `development-workflow.md` | 开发工作流（研究→计划→TDD→代码审查→提交） |
| `git-workflow.md` | Git 工作流（conventional commits、PR 流程） |
| `hooks.md` | Hook 类型说明和最佳实践 |
| `patterns.md` | 设计模式（仓库模式、API 响应格式） |
| `performance.md` | 性能优化（模型选择、上下文管理、扩展思考） |
| `security.md` | 安全检查清单（密钥管理、注入防护、CSRF） |
| `testing.md` | 测试要求（80% 覆盖率、TDD 工作流） |

---

## 自定义代理（Agents）

路径：`~/.claude/agents/`

| 代理 | 用途 |
|------|------|
| planner | 功能实现规划 |
| architect | 系统设计和架构 |
| tdd-guide | 测试驱动开发 |
| code-reviewer | 代码审查 |
| security-reviewer | 安全漏洞分析 |
| build-error-resolver | 构建错误解决 |
| e2e-runner | Playwright E2E 测试 |
| refactor-cleaner | 死代码清理 |
| doc-updater | 文档更新 |
| rust-reviewer | Rust 代码审查 |

---

## 团队（Teams，1 个）

路径：`~/.claude/teams/`

| 团队 | 成员 | 用途 |
|------|------|------|
| realtime-push-research | team-lead（glm-5）、performance-expert（claude-opus-4-6） | 实时推送系统架构研究 |

---

## 项目级配置

### Obsidian 仓库（`E:/Files/Obsidian`）

| 文件 | 内容 |
|------|------|
| `.claude/CLAUDE.md` | LLM Wiki 管理协议（三层结构 sources/wiki/templates、权限矩阵、行号索引规则） |
| `.claude/settings.local.json` | 项目级权限（git、GitHub MCP、curl、powershell、status line 脚本） |

### 项目记忆目录

`~/.claude/projects/E--Files-Obsidian/memory/` — 当前为空，尚未配置 MEMORY.md 或 symlink。

---

## 其他目录

| 目录 | 用途 |
|------|------|
| `~/.claude/backups/` | `.claude.json` 备份文件 |
| `~/.claude/cache/` | 缓存（含 changelog.md） |
| `~/.claude/daemon/` | IPC pipe key（daemon 模式） |
| `~/.claude/file-history/` | 文件版本历史 |
| `~/.claude/homunculus/` | 进化学习系统（3 个已跟踪项目，evolved 目录当前为空） |
| `~/.claude/plans/` | 计划文件（含状态栏修复计划） |
| `~/.claude/projects/` | 4 个已跟踪项目 |
| `~/.claude/scripts/` | 状态栏脚本 |
| `~/.claude/sessions/` | 会话记录 |
| `~/.claude/tasks/` | 任务追踪 |
| `~/.claude/teams/` | 代理团队 |

---

## 维护说明

配置变更时更新本文档对应章节：

- 安装/卸载插件 → 更新「已安装插件」
- 修改 hooks → 更新「Hooks 系统」
- 更换状态栏脚本 → 更新「状态栏」
- 增减 MCP 服务器 → 更新「MCP 服务器」
- 增减技能/命令/规则/代理 → 更新对应章节

---

> 来源：`~/.claude/settings.json`、`~/.claude/plugins/installed_plugins.json`、`~/.claude/skills/`、`~/.claude/rules/`、`~/.claude/agents/`、`~/.claude/teams/`、`E:/Files/Obsidian/.claude/`
