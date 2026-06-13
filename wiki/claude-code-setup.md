---
name: Claude Code 当前配置清单
description: 本机 Claude Code 完整配置清单 + 自检清单（插件/Hooks/状态栏/MCP/技能/代理/规则/团队/经验教训）— 定期巡检、排查配置问题、新环境迁移时参考
type: concept
updated: 2026-06-10
tags: claude-code, setup, plugins, hooks, mcp, skills, agents, config, checklist
source: https://example.com
---

# Claude Code 当前配置清单

> 关联概念：[[wiki/claude-code-p1.md|Claude Code 配置]] | [[wiki/claude-code-p2.md|Claude Code 工具]] | [[wiki/claude-code-p3.md|Claude Code 进阶工作流]] | [[wiki/opencode-bridge.md|OpenCode 桥接]]

本文档记录本机 Claude Code 的实际安装配置，作为排查和迁移的参考。配置变更时需同步更新本文档。

## 全局设置概要

配置文件路径：`~/.claude/settings.json`（335 行）

| 配置项 | 值 | 说明 |
|--------|-----|------|
| `defaultMode` | `"auto"` | 自动批准所有工具调用 |
| `alwaysThinkingEnabled` | `true` | 始终启用扩展思考 |
| `effortLevel` | `"max"` | 最高推理强度（env: `CLAUDE_CODE_EFFORT_LEVEL="max"`） |
| `skipDangerousModePermissionPrompt` | `true` | 跳过危险模式确认 |
| `skipAutoPermissionPrompt` | `true` | 跳过自动权限确认 |
| `includeCoAuthoredBy` | `false` | 禁用 co-authored-by 署名 |
| `autoMemoryEnabled` | `true` | 自动记忆功能 |
| `todoFeatureEnabled` | `true` | Todo 任务追踪 |
| `language` | `"chinese"` | 中文输出 |
| `disableRemoteControl` | `true` | 禁用远程控制 |
| 自动更新 | 已禁用 | `DISABLE_AUTOUPDATER=1` |
| Tool Search | 已启用 | `ENABLE_TOOL_SEARCH=true` |
| Agent Teams | 已启用 | `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` |

### 经验教训

- **持久化优先级**：Hooks（硬约束）> CLAUDE.md > 无条件 rules > scoped rules > MEMORY.md 前 200 行 > 主题文件 > 口头约定。安全关键规则必须用 Hook 实现，CLAUDE.md 的"软"规则无法替代 ← [[wiki/claude-code-p1.md]]
- **上下文窗口管理**：配置 20-30 个 MCP 没问题，但**同时启用 <10 个**，**活跃工具 <80 个**。用 `disabledMcpServers` 按项目禁用不用的 MCP ← [[wiki/claude-code-p3.md]]
- **settings.json 关键隐藏字段**：`showClearContextOnPlanAccept`（Plan 审批时释放窗口）、`autoCompactEnabled`（上下文满时自动压缩）、`fileCheckpointingEnabled`（编辑前快照支持 `/rewind`）← [[wiki/claude-code-p1.md]]

### API 配置

| 配置项 | 值 |
|--------|-----|
| 自定义端点 | `https://token-plan-cn.xiaomimimo.com/anthropic` |
| 模型映射 | `mimo-v2.5-pro`（Opus/Reasoning） / `mimo-v2.5`（Sonnet/Haiku） |
| 认证方式 | `ANTHROPIC_AUTH_TOKEN`（非 `ANTHROPIC_API_KEY`） |
| 缓存优化 | `CLAUDE_CODE_ATTRIBUTION_HEADER=0`（禁用归因头，提升缓存命中率） |

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
| safety-net | cc-marketplace（`kenryu42/cc-marketplace`） | 1.0.1 | 2026-05-21 | 安全网，拦截危险命令（rm -rf、git push --force 等） |
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

### 手动安装 ECC（命令行方式）

如果 `/plugin` 命令失败，可手动安装：

```bash
# 1. 添加市场
claude plugin marketplace add https://github.com/affaan-m/everything-claude-code

# 2. 查看可用插件
claude plugin search ecc

# 3. 安装
claude plugin install ecc@ecc
```

**注意**：手动安装后需重启 Claude Code，且 `~/.claude/plugins/cache/` 路径可能与自动安装不同。安装后验证 `CLAUDE_PLUGIN_ROOT` 指向正确路径。


### 经验教训

- **安装前备份**：任何插件安装前必须 `cp -r ~/.claude/rules ~/.claude/rules.backup`，Rules 和 Hooks 都是追加合并，绝不能直接覆盖 ← [[wiki/claude-code-p2.md]]
- **Safety Net Windows 配置**：Windows 需手动配 Hook 和 `SAFETY_NET_PLUGIN_ROOT` 环境变量，不会自动配置 ← [[wiki/claude-code-p2.md]]
- **Safety Net 状态栏 ❌ 根因**：`isPluginEnabled()` 读取 `CLAUDE_SETTINGS_PATH`，该变量在 `cc switch` 后可能指向不存在的项目级文件。修复：statusLine 命令中显式覆盖 `CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json`。详见「状态栏 → 快速诊断」← [[wiki/claude-code-setup.md]]（2026-06-10 复盘）
- **Skill 双目录部署**：项目目录（开发）和 `~/.claude/skills/`（运行）是两份独立副本，改代码后必须 `cp` 同步，否则 Skill 调用仍用旧版 ← [[wiki/claude-code-p1.md]]
- **技能链调用**：Skill 内引用其他 Skill 时必须用 Skill 工具加载完整内容，不能跳过 ← [[wiki/claude-code-p1.md]]
- **子代理无法调用 Skill**：子代理和团队成员都不能使用 Skill 工具，父代理必须预加载技能内容并用 `---SKILL_NAME START/END---` 标记注入 prompt ← [[wiki/claude-code-p2.md]]

---

## 状态栏（StatusLine）

### 配置（综合版，当前使用）

```json
"statusLine": {
  "type": "command",
  "command": "py ~/.claude/scripts/statusline.py | CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json node \"${SAFETY_NET_PLUGIN_ROOT}/dist/bin/cc-safety-net.js\" --statusline"
}
```

> **关键**：`CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json` 必须覆盖，否则 cc-safety-net 会读取项目级设置（可能不存在或缺少 `enabledPlugins`），导致始终显示 ❌。

### 快速诊断（状态栏异常时按顺序检查）

状态栏异常通常由 4 个原因导致，按排查优先级排序：

| 步骤 | 检查项 | 命令 | 正常结果 | 异常修复 |
|------|--------|------|----------|----------|
| 1 | `CLAUDE_SETTINGS_PATH` 指向有效文件 | `cat $CLAUDE_SETTINGS_PATH` | 包含 `enabledPlugins["safety-net@cc-marketplace"]: true` | statusLine 命令中添加 `CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json` 覆盖 |
| 2 | `SAFETY_NET_PLUGIN_ROOT` 路径存在 | `ls $SAFETY_NET_PLUGIN_ROOT/dist/bin/cc-safety-net.js` | 文件存在 | 更新 settings.json 的 env 中版本号，或 `find ~/.claude/plugins/cache -name "cc-safety-net.js"` 找正确路径 |
| 3 | `CLAUDE_PLUGIN_ROOT` 路径存在且无多余前缀 | `ls $CLAUDE_PLUGIN_ROOT/hooks/hooks.json` | 文件存在 | 更新 env 路径，去掉多余的 `everything-claude-code/` 前缀 |
| 4 | Python 命令可用 | `py --version` | 返回 Python 3.x | Windows 必须用 `py`，不能用 `python`（exit code 49） |

**典型故障模式**：`cc switch` 切换 API KEY 后，`CLAUDE_SETTINGS_PATH` 可能指向不同的项目级路径，该路径不存在或不含 `enabledPlugins`，导致 `isPluginEnabled()` 返回 false，状态栏显示 `🛡️ CC Safety Net ❌`。

**验证修复**：
```bash
# 测试完整管道（替换 SAFETY_NET_PLUGIN_ROOT 为实际路径）
echo '{"model":{"display_name":"test"},"workspace":{"current_dir":"E:/Files/Projects/rust-my-project"},"context_window":{"remaining_percentage":80},"session_id":"test"}' | py ~/.claude/scripts/statusline.py | CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json node "${SAFETY_NET_PLUGIN_ROOT}/dist/bin/cc-safety-net.js" --statusline
```

期望输出包含所有分段，安全网显示 ✅。

### 显示效果

```
mimo-v2.5-pro | PycharmProjects/test | main | ████░░░░░░ 47% | 测试状态栏显示 | $10.44 288t 4f 2h23m | 🛡️ Safety Net ✅
   模型名         目录路径(2级)       Git分支   进度条(4色)      当前任务(中文)     成本/工具/文件/时长   安全网插件cc-safety-net
```

### 管道结构

`statusline.py`（综合版）→ `cc-safety-net --statusline`（附加安全网状态指示器）

### 综合版特性

| 元素 | 来源 | 说明 |
|------|------|------|
| 模型名 | stdin `model.display_name` | 当前使用的模型 |
| 目录路径 | stdin `workspace.current_dir` | 压缩为最后 2 级，~ 表示 home |
| Git 分支 | `git rev-parse --abbrev-ref HEAD` | 自动向上搜索 .git 目录 |
| 上下文进度条 | stdin `context_window.remaining_percentage` | ECC 4 色阈值：<50% 绿、<65% 黄、<80% 橙、≥80% 红 |
| 当前任务 | `~/.claude/todos/*.json` | 读取 `in_progress` 状态的 `activeForm` 字段 |
| 成本/工具/文件/时长 | `/tmp/ecc-metrics-{session}.json` | 由 `ecc-metrics-bridge.js` hook 写入 |
| 安全网 | `cc-safety-net --statusline` | 管道追加，由 safety-net 插件提供 |

### 依赖配置

状态栏依赖以下 hook 写入数据（已在 settings.json 中配置）：

```json
// PostToolUse hook — 写入会话指标到 bridge 文件
{
  "matcher": "*",
  "hooks": [{
    "type": "command",
    "command": "node \"<ECC_ROOT>/scripts/hooks/run-with-flags.js\" \"post:metrics-bridge\" \"scripts/hooks/ecc-metrics-bridge.js\" \"standard,strict\"",
    "timeout": 5,
    "async": true
  }]
}
```

Bridge 文件路径：`/tmp/ecc-metrics-{session_id}.json`（Node.js `os.tmpdir()`）

### 脚本位置

- **主脚本**：`~/.claude/scripts/statusline.py`（综合版，融合旧版目录/分支 + ECC 指标/任务）
- 备选脚本：`~/.claude/scripts/statusline.sh`（Bash 版，无 jq 依赖）
- 调试脚本：`~/.claude/scripts/debug_path.py`

### stdin JSON 关键字段

- `model.display_name` — 模型名称
- `workspace.current_dir` — 当前目录
- `context_window.remaining_percentage` — 上下文窗口剩余百分比
- `session_id` — 会话 ID（用于查找 bridge 文件和 todo 文件）

### 已知问题

1. **Windows `python` 命令**：解析为 Windows Store 存根，exit code 49，必须用 `py` 或 `py -3` 替代。
2. **`CLAUDE_SETTINGS_PATH` 指向无效文件**：`cc switch` 切换 API KEY 后，该变量可能指向不存在的项目级设置文件，导致 `isPluginEnabled()` 返回 false。修复：statusLine 命令中添加 `CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json` 覆盖。
3. **`SAFETY_NET_PLUGIN_ROOT` 版本过时**：插件升级后 env 中的版本号未同步，导致路径不存在。修复：`find ~/.claude/plugins/cache -name "cc-safety-net.js"` 找正确路径后更新 env。
4. **`cc switch` 重置 settings.json**：切换 API KEY 时可能使用过时模板重写整个配置，导致 env 路径、statusLine、hooks 全部回到旧值。修复后需验证未被再次重置。详见「Hooks 系统 → 快速诊断」。

### 经验教训

- **`CLAUDE_SETTINGS_PATH` 是状态栏失败的首要原因**：`cc-safety-net --statusline` 的 `isPluginEnabled()` 读取此路径，文件不存在或不含 `enabledPlugins` 即返回 false。必须在 statusLine 命令中显式覆盖为 `$HOME/.claude/settings.json` ← [[wiki/claude-code-setup.md]]（2026-06-10 复盘）
- **Windows `py` vs `python`**：`python` 命令返回 exit code 49（不存在），必须用 `py` 或 `py -3` ← [[wiki/claude-code-p2.md]]
- **Worktree `.git` 是文件**：worktree 中 `.git` 是文件（指向主仓库），不是目录。`get_git_branch()` 原先用 `isdir` 会跳过 worktree 找到错误分支，修复为 `isdir(...) or isfile(...)` ← [[wiki/claude-code-p2.md]]
- **`read_bridge()` 路径**：必须用 `os.environ.get("TEMP")` 而非 `tempfile.gettempdir()`，因为 Python 的 `tempfile` 在 Windows 上会追加 `\claude` 子目录，与 Node.js 的 `os.tmpdir()` 路径不一致 ← [[wiki/claude-code-setup.md]]
- **插件升级后需同步 env 版本号**：`SAFETY_NET_PLUGIN_ROOT` 硬编码版本号，插件更新后必须手动同步 ← [[wiki/claude-code-setup.md]]（2026-06-10 复盘）

---

### 备选：简易版状态栏

如不需要当前任务和会话指标，可使用简易版：

```json
"statusLine": {
  "type": "command",
  "command": "py -3 \"C:/Users/<user>/.claude/scripts/statusline-simple.py\" | npx -y cc-safety-net --statusline"
}
```

简易版只显示：模型名 | 目录路径 | Git 分支 | 进度条 | 安全网，无当前任务和成本指标。

---

### 备选：ECC 原版状态栏

ECC 插件自带的状态栏脚本：

```json
"statusLine": {
  "type": "command",
  "command": "node \"<ECC_ROOT>/scripts/hooks/ecc-statusline.js\""
}
```

显示：模型名 | 当前任务 | $成本 工具数 文件数 时长 | 目录名 | 进度条

特点：目录仅显示最后一级，无 Git 分支。

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

### 经验教训

- **GitHub MCP 认证**：插件 `.mcp.json` 用 `${GITHUB_PERSONAL_ACCESS_TOKEN}` header 认证，变量缺失时回退到 OAuth 动态注册（Copilot 端点不支持），报错 `Incompatible auth server`。必须在 settings.json 的 `env` 中添加该变量 ← [[wiki/claude-code-p2.md]]
- **切换 API KEY 后 MCP 断连**：99% 是 token 问题。智谱 HTTP MCP 和 zai-mcp-server 已配置为 `${ANTHROPIC_AUTH_TOKEN}` 环境变量引用，切换后无需手动改。验证：`claude mcp list` ← [[wiki/claude-code-p2.md]]
- **HTTP MCP 无需本地安装**：Cloudflare/ClickHouse 等 HTTP 类型 MCP 直接 URL 连接，`npx -y` 类型首次调用自动安装 ← [[wiki/claude-code-p3.md]]

---

## Hooks 系统

> **架构说明**：settings.json 只保留 2 个用户级 hooks（Safety Net + block-no-verify）。其余 hooks 由 ECC 插件自带的 `hooks/hooks.json` 自动注入，无需手动维护。以下列表为完整生效的 hooks。

### 快速诊断（Hooks 报错时按顺序检查）

Hooks 异常通常由 2 个原因导致，按排查优先级排序：

| 步骤 | 检查项 | 命令 | 正常结果 | 异常修复 |
|------|--------|------|----------|----------|
| 1 | `CLAUDE_PLUGIN_ROOT` 路径存在 | `ls $CLAUDE_PLUGIN_ROOT` | 目录存在 | 更新 settings.json 的 env 中路径：`find ~/.claude/plugins/cache -name "hooks.json"` 找正确版本 |
| 2 | settings.json 不含 ECC hooks | `grep CLAUDE_PLUGIN_ROOT ~/.claude/settings.json` | 无匹配 | 删除 settings.json 中引用 `${CLAUDE_PLUGIN_ROOT}` 的 hooks，只保留 Safety Net + block-no-verify |

**典型故障模式**：
1. **`cc switch` 重置 settings.json**：切换 API KEY 时可能使用过时模板重写配置，导致 `CLAUDE_PLUGIN_ROOT` 回到旧路径、hooks 被重新注入。修复后需检查是否再次被重置。
2. **ECC 插件升级**：`plugins/cache/` 路径变化，env 未同步更新。

**验证修复**：
```bash
# 检查 ECC hooks.json 存在且可被插件系统发现
ls $CLAUDE_PLUGIN_ROOT/hooks/hooks.json

# 检查 settings.json 中无 ECC hooks
grep -c "CLAUDE_PLUGIN_ROOT" ~/.claude/settings.json  # 应为 0
```

### settings.json 中的 Hooks（2 个）

| 匹配器 | 脚本 | 功能 | 超时 |
|--------|------|------|------|
| `Bash` | `cc-safety-net.js --claude-code` | Safety Net 危险命令拦截 | 15s |
| `Bash` | `npx block-no-verify@1.1.2` | 阻止未验证的 git 操作 | - |

### ECC 插件注入的 Hooks

以下 hooks 由 ECC v2.0.0-rc.1 自动注册，不在 settings.json 中：

#### PreToolUse（9 个）

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

### PostToolUse（9 个）

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
| `*` | `ecc-metrics-bridge.js` | 会话指标桥接（状态栏依赖） | 5s | async |

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
- `CLAUDE_PLUGIN_ROOT` → ECC 插件根目录（v2.0.0-rc.1）
- `SAFETY_NET_PLUGIN_ROOT` → Safety Net 插件根目录（v1.0.1）

### 经验教训

- **`cc switch` 重置 settings.json**：切换 API KEY 时可能使用过时模板重写配置，导致 `CLAUDE_PLUGIN_ROOT` 回到旧路径、`SAFETY_NET_PLUGIN_ROOT` 版本过时、ECC hooks 被重新注入。修复后需验证未被再次重置 ← [[wiki/claude-code-setup.md]]（2026-06-10 复盘）
- **settings.json 只保留 2 个用户级 hooks**：Safety Net（`${SAFETY_NET_PLUGIN_ROOT}`）+ block-no-verify（`npx`）。所有引用 `${CLAUDE_PLUGIN_ROOT}` 的 hooks 必须删除，由 ECC 插件 `hooks/hooks.json` 自动注册 ← [[wiki/claude-code-setup.md]]（2026-06-10 复盘）
- **Hook 类型选择**：prompt hook 返回 `{"ok": false, "reason": "..."}` 可让主 Claude 继续工作（拥有完整上下文和工具权限），command hook + `claude -p` 子进程需读 transcript 重建上下文，有冷启动和终端乱码风险 ← [[wiki/claude-code-p1.md]]
- **Stop Hook 演进 V1→V4**：command+claude-p（V1）→ prompt hook ok:false（V2）→ prompt hook+后台 agent（V3，过度触发）→ 关键词触发+后台 agent（V4，当前方案）。教训：自动触发的 hook 评估器难以准确判断对话是否有复盘价值，用户主动触发更可靠 ← [[wiki/claude-code-p1.md]]
- **ECC 升级后 Hooks 报错**：ECC 插件升级后 `plugins/cache/` 路径变化，settings.json 中旧 hooks 失效报 "not associated with a plugin"。ECC v2+ 自带 `hooks/hooks.json` 自解析路径，settings.json 只保留 Safety Net 和 block-no-verify ← [[wiki/claude-code-p2.md]]
- **Hooks 范式**：PreToolUse 用于长命令提醒用 tmux、写 .md 阻止；PostToolUse 用于自动格式化/类型检查/console.log 检查；Stop 用于审计残留 ← [[wiki/claude-code-p3.md]]

---

## 技能（Skills，3 个）

路径：`~/.claude/skills/`

| 技能 | 功能 | 触发方式 |
|------|------|----------|
| `docker-dev-container/` | 创建 Docker 开发环境（Claude Code + OpenCode） | `/docker-dev-container` |
| `obsidian-link/` | Obsidian 仓库与项目记忆 symlink 桥接 | `/obsidian-link` |
| `obsidian-remember/` | Obsidian 知识仓库管理（三层架构：sources/wiki/MEMORY.md） | `/obsidian-remember` |

### 技能依赖链

`obsidian-remember` → `obsidian-link`（remember 依赖 link）

### 经验教训

- **SKILL.md 结构**：纯自然语言指令，定义触发条件 → 操作流程（含 bash 命令模板）→ 输出格式 → 安全约束 ← [[wiki/claude-code-p1.md]]
- **.env 加载策略**：Skill 的 `_load_dotenv()` 用 fallback 列表加载多个 .env，`os.environ.setdefault` 保证先加载的优先，解决部署目录和开发目录凭证分散问题 ← [[wiki/claude-code-p1.md]]

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

## 自定义代理（Agents，10 + 52 个）

路径：`~/.claude/agents/`

### 用户自定义代理（10 个）

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

### ECC 插件提供的代理（52 个）

ECC v2.0.0-rc.1 自动安装了大量领域专用代理，包括但不限于：`a11y-architect`、`code-explorer`、`cpp-reviewer`、`dart-build-resolver`、`django-reviewer`、`fastapi-reviewer`、`flutter-reviewer`、`go-reviewer`、`java-reviewer`、`kotlin-reviewer`、`network-architect`、`python-reviewer`、`react-reviewer`、`rust-build-resolver`、`swift-reviewer`、`typescript-reviewer` 等。完整列表见 `~/.claude/agents/` 目录。

### 经验教训

- **限制工具 = 聚焦执行**：给子代理的工具越少，越不容易跑偏。子代理 + 技能 = 自动化组合 ← [[wiki/claude-code-p3.md]]
- **传递目标上下文**：给子代理传递目标上下文而不只是查询——帮其判断该包含什么。迭代检索模式：评估返回值 → 不充分则追问 → 最多 3 轮 ← [[wiki/claude-code-p3.md]]
- **`description` 最关键**：Claude 据此决定何时自动委托任务，写不好会导致从不触发或误触发 ← [[wiki/claude-code-p2.md]]

---

## 团队（Teams，1 个）

路径：`~/.claude/teams/`

| 团队 | 成员 | 用途 |
|------|------|------|
| realtime-push-research | team-lead（glm-5）、performance-expert（claude-opus-4-6） | 实时推送系统架构研究 |

---

## OpenCode 桥接

OpenCode 是另一个 AI 编码工具，可通过桥接与 Claude Code 共享配置。

### 配置位置

`~/.opencode/config.json`

### 与 Claude Code 共享的内容

- 环境变量（`ANTHROPIC_AUTH_TOKEN`、`ANTHROPIC_BASE_URL` 等）
- 插件路径（`CLAUDE_PLUGIN_ROOT`、`SAFETY_NET_PLUGIN_ROOT`）
- Hooks（需单独配置，不自动继承）

### 经验教训

- OpenCode 的 hooks 配置格式与 Claude Code 不同，需单独维护
- 桥接脚本位于 `~/.claude/scripts/opencode-bridge.js`
- 详见 [[wiki/opencode-bridge.md|OpenCode 桥接文档]]

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

## 工作流约束

### Hooks 持久化（硬约束）

必须用 Hook 实现的规则：
- 安全关键：`rm -rf`、`git push --force` 等危险命令拦截
- 流程强制：TDD 验证、代码审查、提交前检查
- 环境一致性：开发环境配置、依赖检查

### ECC 软规则覆盖

ECC 插件的 rules 是软约束，可被项目级 CLAUDE.md 覆盖：
- 项目级规则优先于 ECC 全局规则
- 但不能覆盖 Hooks（硬约束）

### 子代理派发规则

派发子代理时，将对应技能要求注入 prompt：
- 实现任务 → `先调用 Skill 工具加载 test-driven-development 技能`
- 代码质量审查 → `先调用 Skill 工具加载 requesting-code-review 技能`
- 完成分支 → `先调用 Skill 工具加载 finishing-a-development-branch 技能`

### 铁律

1. **TDD 铁律**：没有失败测试，不写生产代码
2. **调试铁律**：没有根因调查，不实施修复
3. **验证铁律**：没有运行证据，不声称完成

---

## 自检概览

定期巡检时按此表逐项验证：

| 自检项 | 验证方法 | 期望结果 |
|--------|----------|----------|
| 插件状态 | `/plugins` | 8 个插件全部 loaded |
| MCP 连接 | `claude mcp list` | 全部 Connected，无 hook 错误 |
| 状态栏 | 看底部状态栏 | 显示模型/目录/分支/进度/安全网 |
| Hooks 生效 | 执行 `rm -rf /tmp/test` | Safety Net 拦截 |
| 技能可用 | `/docker-dev-container` | 技能内容加载 |
| 规则加载 | 问 Claude "你加载了哪些 rules" | 列出 9 个 rules 文件 |
| API 连接 | 任意对话 | 正常回复，无 auth 错误 |
| 上下文余量 | 看进度条 | <50% 绿色 |
| 插件路径 | Glob `plugins/cache/` | env 变量指向实际版本路径 |

---

## 维护说明

配置变更时更新本文档对应章节：

- 安装/卸载插件 → 更新「已安装插件」
- 修改 hooks → 更新「Hooks 系统」
- 更换状态栏脚本 → 更新「状态栏」
- 增减 MCP 服务器 → 更新「MCP 服务器」
- 增减技能/命令/规则/代理 → 更新对应章节

---

---

## 附录：综合版 statusline.py 完整脚本

> 路径：`~/.claude/scripts/statusline.py`，通过 `py -3 "C:/Users/<user>/.claude/scripts/statusline.py" | npx -y cc-safety-net --statusline` 调用。

```python
import sys
import json
import os
import re
import subprocess
import tempfile
from pathlib import Path

# ============================================================
# Context bar thresholds (ECC standard)
# ============================================================
AUTO_COMPACT_BUFFER = 16.5  # Reserve for auto-compact overhead

# ============================================================
# ANSI color helpers
# ============================================================
def _c(code: int, text: str) -> str:
    return f"\033[{code}m{text}\033[0m"

def green(t):  return _c(32, t)
def yellow(t): return _c(33, t)
def red(t):    return _c(31, t)
def orange(t): return _c(38;5;208", t)
def dim(t):    return _c(2, t)
def bold(t):   return _c(1, t)

# ============================================================
# Git branch detection
# ============================================================
def get_git_branch() -> str:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            capture_output=True, text=True, timeout=3,
            cwd=os.environ.get("CLAUDE_WORKING_DIR", "."),
        )
        if result.returncode == 0:
            return result.stdout.strip()
    except Exception:
        pass
    return ""

# ============================================================
# Path shortening
# ============================================================
def shorten_path(path: str, n: int = 2) -> str:
    parts = Path(path).parts
    if len(parts) <= n:
        return path
    return os.sep.join(parts[-n:])

# ============================================================
# Context bar (4-color: green < 50%, yellow < 65%, orange < 80%, red >= 80%)
# ============================================================
def build_context_bar(context_used_pct: float, width: int = 10) -> str:
    if context_used_pct < 0:
        return dim("N/A")
    used = max(0, min(100, round(context_used_pct)))
    filled = round(used / 100 * width)
    empty = width - filled
    bar = "█" * filled + "░" * empty

    if used < 50:
        colored_bar = green(bar)
    elif used < 65:
        colored_bar = yellow(bar)
    elif used < 80:
        colored_bar = orange(bar)
    else:
        colored_bar = red(bar)

    return f"{colored_bar} {used}%"

# ============================================================
# Duration formatting
# ============================================================
def format_duration(seconds: float) -> str:
    if seconds < 60:
        return f"{int(seconds)}s"
    minutes = int(seconds // 60)
    if minutes < 60:
        return f"{minutes}m"
    hours = minutes // 60
    mins = minutes % 60
    return f"{hours}h{mins}m"

# ============================================================
# Current task from todo files
# ============================================================
def read_current_task(session_id: str) -> str:
    """Read active task from todo files."""
    if not session_id:
        return ""
    try:
        todo_dir = Path.home() / ".claude" / "todos"
        if not todo_dir.exists():
            return ""
        for f in todo_dir.glob(f"{session_id[:8]}*-agent-*.json"):
            try:
                data = json.loads(f.read_text(encoding="utf-8"))
                if data.get("status") == "in_progress" and data.get("activeForm"):
                    return data["activeForm"]
            except Exception:
                continue
    except Exception:
        pass
    return ""

# ============================================================
# ECC metrics bridge reader
# ============================================================
def read_bridge(session_id: str) -> dict:
    """Read ECC metrics bridge file from temp directory."""
    if not session_id:
        return {}
    try:
        safe_id = "".join(c for c in session_id if c.isalnum() or c in "-_")
        tmpdir = os.environ.get("TEMP") or os.environ.get("TMP") or os.environ.get("TMPDIR") or "/tmp"
        bridge_path = os.path.join(tmpdir, f"ecc-metrics-{safe_id}.json")
        if os.path.exists(bridge_path):
            with open(bridge_path, "r", encoding="utf-8") as f:
                return json.load(f)
    except Exception:
        pass
    return {}

# ============================================================
# Main: read stdin JSON from Claude Code
# ============================================================
def main():
    try:
        data = json.loads(sys.stdin.read())
    except Exception:
        print(dim("statusline: no input"))
        return

    # Extract fields from stdin
    model = data.get("model", {})
    model_name = model.get("display_name") or model.get("id", "unknown")
    session_id = data.get("session_id", "")

    # Context usage
    context_window = data.get("context_window", {})
    context_used_pct = context_window.get("used_percentage", -1)
    context_remaining_pct = context_window.get("remaining_percentage", 100)

    # Calculate real usage accounting for auto-compact buffer
    # ECC logic: remaining_pct includes buffer, so subtract it to get usable
    usable = max(0, ((context_remaining_pct - AUTO_COMPACT_BUFFER) / (100 - AUTO_COMPACT_BUFFER)) * 100)
    used = max(0, min(100, round(100 - usable)))

    # Working directory
    cwd = data.get("cwd", os.getcwd())
    short_cwd = shorten_path(cwd)

    # Git branch
    branch = get_git_branch()

    # ECC bridge metrics
    bridge = read_bridge(session_id)
    tool_count = bridge.get("tool_count", 0)
    files_modified = bridge.get("files_modified_count", 0)
    cost = bridge.get("total_cost_usd", 0)

    # Duration from bridge
    first_ts = bridge.get("first_timestamp", 0)
    elapsed = 0
    if first_ts:
        import time
        elapsed = time.time() - first_ts / 1000  # bridge stores ms

    # Current task
    task = read_current_task(session_id)

    # Build status line parts
    parts = []

    # Model
    parts.append(bold(model_name))

    # Directory
    parts.append(dim(short_cwd))

    # Git branch
    if branch:
        parts.append(dim(branch))

    # Context bar
    parts.append(build_context_bar(used))

    # Current task (if any)
    if task:
        parts.append(yellow(task))

    # Metrics: cost | tools | files | duration (always show 4 items)
    metrics = []
    metrics.append(f"${cost:.2f}" if cost > 0 else "$0.00")
    metrics.append(f"{tool_count}t" if tool_count > 0 else "0t")
    metrics.append(f"{files_modified}f" if files_modified > 0 else "0f")
    metrics.append(format_duration(elapsed) if elapsed > 0 else "0s")
    parts.append(dim(" ".join(metrics)))

    # Output (cc-safety-net will append safety indicator via pipe)
    print(" | ".join(parts))

if __name__ == "__main__":
    main()
```

**关键实现细节：**
- `read_bridge()` 使用 `os.environ.get("TEMP")` 而非 `tempfile.gettempdir()`，因为 Python 的 `tempfile` 在 Windows 上会追加 `\claude` 子目录，与 Node.js 的 `os.tmpdir()` 路径不一致
- `build_context_bar()` 的 `used` 值通过 auto-compact buffer 公式计算，而非直接使用 stdin 的 `used_percentage`
- `read_current_task()` 按 session_id 前缀匹配 todo 文件，取第一个 `status == "in_progress"` 的任务

---

---

## 缓存优化（Prompt Caching）

### 问题描述

使用第三方 API（如 MiMo）时，缓存命中率可能远低于预期，导致 token 消耗增加。

### 根因分析

Claude Code 在系统提示中添加了 `x-anthropic-billing-header`，包含：

```
x-anthropic-billing-header: cc_version=2.1.113.244; cc_entrypoint=cli; cch=b29b8;
```

其中 `cch` 是基于用户提示计算的哈希值，**每次对话都不同**。这导致系统提示前缀变化，缓存失效。

**为什么第三方 API 受影响**：
- Anthropic 原生 API：可能在服务端忽略或处理了这个变化
- 第三方 API：严格按前缀匹配，`cch` 变化 → 缓存失效

### 解决方案

在 `settings.json` 的 `env` 中添加：

```json
"CLAUDE_CODE_ATTRIBUTION_HEADER": "0"
```

**效果**：移除系统提示中的 `cch=xxx` 哈希值，让系统提示前缀稳定。

### 局限性

这只是部分解决方案。还有其他因素导致缓存失效：

| 因素 | 位置 | 影响 |
|------|------|------|
| `git status` 输出 | 系统提示 | 每次文件变化都破坏缓存 |
| skills 内容 | messages[0] | 会话重启可能变化 |
| project CLAUDE.md | messages[0] | 内容变化破坏缓存 |
| 工具顺序 | 系统提示 | 可能随机排列 |

### 社区解决方案

社区开发了 `claude-code-cache-fix` 拦截器，能处理更多问题：

```bash
npm install -g claude-code-cache-fix@1.7.1
CACHE_FIX_DEBUG=1  # 查看缓存命中情况
```

### 参考

- GitHub issue #50085：`CLAUDE_CODE_ATTRIBUTION_HEADER` 文档缺失
- GitHub issue #47098：新会话永远无法命中完整缓存
- GitHub issue #64974：1M 上下文热修复后缓存失效

---

## cc switch 通用配置模板

`cc switch` 切换 API KEY 时使用此模板重写 settings.json。更新插件版本后需同步修改此模板。

```json
{
  "env": {
    "CLAUDE_PLUGIN_ROOT": "C:/Users/<user>/.claude/plugins/cache/ecc/ecc/2.0.0-rc.1",
    "SAFETY_NET_PLUGIN_ROOT": "C:/Users/<user>/.claude/plugins/cache/cc-marketplace/safety-net/1.0.1",
    "CLAUDE_CODE_ATTRIBUTION_HEADER": "0",
    "ENABLE_TOOL_SEARCH": "true",
    "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1",
    "CLAUDE_CODE_EFFORT_LEVEL": "max"
  },
  "theme": "dark",
  "includeCoAuthoredBy": false,
  "permissions": {
    "defaultMode": "auto"
  },
  "hooks": {
    "PreToolUse": [
      {
        "description": "Safety Net - Block destructive git and filesystem commands",
        "hooks": [
          {
            "command": "node \"${SAFETY_NET_PLUGIN_ROOT}/dist/bin/cc-safety-net.js\" --claude-code",
            "timeout": 15,
            "type": "command"
          }
        ],
        "matcher": "Bash"
      },
      {
        "hooks": [
          {
            "command": "npx block-no-verify@1.1.2",
            "type": "command"
          }
        ],
        "matcher": "Bash"
      }
    ]
  },
  "disableRemoteControl": true,
  "statusLine": {
    "type": "command",
    "command": "py ~/.claude/scripts/statusline.py | CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json node \"${SAFETY_NET_PLUGIN_ROOT}/dist/bin/cc-safety-net.js\" --statusline"
  },
  "enabledPlugins": {
    "context7@claude-plugins-official": true,
    "ecc@ecc": true,
    "frontend-design@claude-plugins-official": true,
    "github@claude-plugins-official": true,
    "learning-output-style@claude-plugins-official": true,
    "safety-net@cc-marketplace": true,
    "skill-creator@claude-plugins-official": true,
    "superpowers@claude-plugins-official": true
  },
  "extraKnownMarketplaces": {
    "cc-marketplace": {
      "source": {
        "source": "github",
        "repo": "kenryu42/cc-marketplace"
      }
    },
    "everything-claude-code": {
      "source": {
        "source": "github",
        "repo": "affaan-m/everything-claude-code"
      }
    }
  },
  "language": "chinese",
  "spinnerTipsEnabled": true,
  "alwaysThinkingEnabled": true,
  "effortLevel": "xhigh",
  "showClearContextOnPlanAccept": true,
  "autoMemoryEnabled": true,
  "showThinkingSummaries": true,
  "skipDangerousModePermissionPrompt": true,
  "preferredNotifChannel": "auto",
  "autoCompactEnabled": true,
  "fileCheckpointingEnabled": true,
  "showTurnDuration": true,
  "showMessageTimestamps": true,
  "terminalProgressBarEnabled": true,
  "todoFeatureEnabled": true,
  "skipAutoPermissionPrompt": true
}
```

### 模板说明

| 字段 | 规则 |
|------|------|
| `env.CLAUDE_PLUGIN_ROOT` | 必须指向实际存在的 ECC 版本目录，无多余前缀 |
| `env.SAFETY_NET_PLUGIN_ROOT` | 必须指向实际存在的 Safety Net 版本目录 |
| `env.CLAUDE_CODE_ATTRIBUTION_HEADER` | 设为 `"0"` 禁用归因头，提升第三方 API 缓存命中率 |
| `hooks` | **只保留 Safety Net + block-no-verify**，ECC hooks 由 `hooks/hooks.json` 自动注册 |
| `statusLine.command` | 必须用 `py`（非 `python`），必须加 `CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json` 覆盖 |
| `enabledPlugins` | 包含所有已安装插件，`cc switch` 会在此基础上追加 API KEY 相关配置 |
| 其他字段 | 保留用户偏好设置，不可删除 |

---

> 来源：`~/.claude/settings.json`、`~/.claude/plugins/installed_plugins.json`、`~/.claude/skills/`、`~/.claude/rules/`、`~/.claude/agents/`、`~/.claude/teams/`、`E:/Files/Obsidian/.claude/`

## MiMoCode Obsidian 链接方法

MiMoCode（OpenCode 分支）使用数据级记忆，与 Claude Code 的项目级记忆不同。

### 关键差异

| 项目 | Claude Code | MiMoCode |
|------|-------------|----------|
| 记忆位置 | `.claude/projects/<slug>/memory/` | `~/.local/share/mimocode/memory/projects/<pid>/` |
| 项目标识 | 路径派生的 slug | UUID（如 `f1252ca1-80af-4d8d-ab2c-1280de217e10`） |
| 链接方式 | Symlink/Junction | Junction（Windows）/ Symlink（Linux） |

### 操作步骤

**1. 查找 MiMoCode 记忆目录**

```bash
# Windows
MEMORY_ROOT="C:\Users\$USERNAME\.local\share\mimocode\memory"

# Linux/macOS
MEMORY_ROOT="$HOME/.local/share/mimocode/memory"
```

**2. 定位项目记忆目录**

```bash
ls "$MEMORY_ROOT/projects/"  # 查找项目 UUID
PROJECT_MEMORY_DIR="$MEMORY_ROOT/projects/<项目UUID>"
```

**3. 创建链接**

```bash
# Windows：Junction（无需管理员权限，支持跨盘符）
powershell -Command "New-Item -ItemType Junction -Path '$PROJECT_MEMORY_DIR' -Target 'E:\Files\Obsidian'"

# Linux/macOS：Symlink
ln -s "/path/to/obsidian" "$PROJECT_MEMORY_DIR"
```

**4. 验证**

```bash
ls "$PROJECT_MEMORY_DIR/MEMORY.md"  # 检查文件存在
head -3 "$PROJECT_MEMORY_DIR/MEMORY.md"  # 验证内容可读
```

### 注意事项

- 创建链接前务必备份原有记忆目录（`mv dir dir.bak`）
- 链接后 Obsidian 中的修改会实时反映到 MiMoCode
- 每个项目需要单独创建链接
