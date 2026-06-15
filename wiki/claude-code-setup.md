---
name: Claude Code 当前配置清单
description: 本机 Claude Code 完整配置清单 + 自检清单（插件/Hooks/状态栏/MCP/技能/代理/规则/团队/经验教训）— 定期巡检、排查配置问题、新环境迁移时参考
type: concept
updated: 2026-06-14
tags: claude-code, setup, plugins, hooks, mcp, skills, agents, config, checklist
source: https://example.com
---

# Claude Code 当前配置清单

> 关联概念：[[wiki/claude-code-p1.md|Claude Code 配置]] | [[wiki/claude-code-p2.md|Claude Code 工具]] | [[wiki/claude-code-p3.md|Claude Code 进阶工作流]] | [[wiki/opencode-bridge.md|OpenCode 桥接]]

本文档记录本机 Claude Code 的实际安装配置，作为排查和迁移的参考。配置变更时需同步更新本文档。

> 可执行资产（状态栏脚本、cc switch 模板）已外移至 `sources/claude-code/`，本文档只留指针与约束，避免脚本更新导致文档过时。

## 全局设置概要

配置文件路径：`~/.claude/settings.json`

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
| 缓存优化 | `CLAUDE_CODE_ATTRIBUTION_HEADER=0`（禁用归因头，提升缓存命中率，详见下方「缓存优化」） |

---

## `cc switch` 后的常见失效与修复

> 状态栏 ❌、Hooks 报错、安全网失效等问题，绝大多数源于 `cc switch` 切换 API KEY 时的三类根因。本节合并原散落于「状态栏」「Hooks 系统」的重复诊断，后续章节不再重复。

### 三大根因

| 根因 | 症状 | 修复 |
|------|------|------|
| **`CLAUDE_SETTINGS_PATH` 指向无效文件** | `cc-safety-net --statusline` 的 `isPluginEnabled()` 读不到 `enabledPlugins`，状态栏显示 `🛡️ CC Safety Net ❌` | statusLine 命令中显式覆盖 `CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json` |
| **`cc switch` 重置 settings.json** | 切换 API KEY 时用过时模板重写配置，`CLAUDE_PLUGIN_ROOT`/`SAFETY_NET_PLUGIN_ROOT` 回旧路径、ECC hooks 被重新注入、statusLine 回旧值 | 按 [[sources/claude-code/cc-switch-template.json]] 修复后，验证未被再次重置 |
| **`SAFETY_NET_PLUGIN_ROOT` 版本过时** | 插件升级后 env 中版本号未同步，`dist/bin/cc-safety-net.js` 路径不存在 | `find ~/.claude/plugins/cache -name "cc-safety-net.js"` 找正确路径，更新 env 版本号（cc switch 模板也同步） |

### 诊断流程（异常时按顺序检查）

| 步骤 | 检查项 | 命令 | 正常结果 | 异常修复 |
|------|--------|------|----------|----------|
| 1 | `CLAUDE_SETTINGS_PATH` 指向有效文件 | `cat $CLAUDE_SETTINGS_PATH` | 含 `enabledPlugins["safety-net@cc-marketplace"]: true` | statusLine 命令加 `CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json` |
| 2 | `SAFETY_NET_PLUGIN_ROOT` 路径存在 | `ls $SAFETY_NET_PLUGIN_ROOT/dist/bin/cc-safety-net.js` | 文件存在 | find 正确路径后更新 env（见根因 3） |
| 3 | `CLAUDE_PLUGIN_ROOT` 路径存在且无多余前缀 | `ls $CLAUDE_PLUGIN_ROOT/hooks/hooks.json` | 文件存在 | 更新 env 路径，去掉多余的 `everything-claude-code/` 前缀 |
| 4 | settings.json 不含 ECC hooks | `grep -c CLAUDE_PLUGIN_ROOT ~/.claude/settings.json` | 返回 `0` | 删除引用 `${CLAUDE_PLUGIN_ROOT}` 的 hooks，只保留 Safety Net + block-no-verify |
| 5 | Python 命令可用 | `py --version`（Windows） | 返回 Python 3.x | Windows 必须用 `py`，不能用 `python`（exit code 49） |

### 验证修复

```bash
# 测试状态栏完整管道（替换 SAFETY_NET_PLUGIN_ROOT 为实际路径）
echo '{"model":{"display_name":"test"},"workspace":{"current_dir":"E:/Files/Projects/rust-my-project"},"context_window":{"remaining_percentage":80},"session_id":"test"}' | py ~/.claude/scripts/statusline.py | CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json node "${SAFETY_NET_PLUGIN_ROOT}/dist/bin/cc-safety-net.js" --statusline

# 检查 ECC hooks 未被重新注入
grep -c "CLAUDE_PLUGIN_ROOT" ~/.claude/settings.json  # 应为 0
```

期望状态栏输出包含所有分段，安全网显示 ✅。

---

## 已安装插件（8 个）

主要使用两个核心插件，其余为辅助：

### 核心插件

| 插件 | 安装渠道 | 版本 | 安装日期 | 用途 |
|------|----------|------|----------|------|
| **superpowers** | Claude Code 插件市场（`claude-plugins-official`） | 5.1.0 | 2026-03-20 | 技能调度框架，所有任务的起点 |
| **ecc**（Everything Claude Code） | GitHub 仓库（`affaan-m/everything-claude-code`） | 2.0.0 | 2026-05-21 | 基础设施层：编码规范、上下文管理、安全审查、持续学习，补充 Superpowers 不覆盖的领域 |

> Superpowers 提供工作流骨架（七阶段开发流程），ECC 提供底层能力（rules、hooks、MCP、session memory）。两者互补，非替代关系。详见 [[wiki/ecc-superpowers-supplement.md|ECC 补强指南]]。

### 辅助插件

| 插件 | 安装渠道 | 版本 | 安装日期 | 用途 |
|------|----------|------|----------|------|
| safety-net | cc-marketplace（`kenryu42/cc-marketplace`） | 1.0.6 | 2026-05-21 | 安全网，拦截危险命令（rm -rf、git push --force 等） |
| github | claude-plugins-official | 9e150cfd48b8 | 2026-03-29 | GitHub 集成 |
| context7 | claude-plugins-official | 9e150cfd48b8 | 2026-03-20 | 实时文档查询 |
| frontend-design | claude-plugins-official | 9e150cfd48b8 | 2026-03-20 | 前端设计辅助 |
| skill-creator | claude-plugins-official | 9e150cfd48b8 | 2026-03-20 | 技能创建工具 |
| learning-output-style | claude-plugins-official | 1.0.0 | 2026-03-20 | 交互式学习输出风格 |

### 安装渠道

| 渠道 | 命令 | 适用 |
|------|------|------|
| Claude Code 插件市场 | `/plugin` → 从 `claude-plugins-official` 安装 | Superpowers 等 6 个插件 |
| GitHub 仓库 | `/plugin marketplace add <repo>` → `/plugin install` | ECC（需先添加仓库为自定义市场） |
| cc-marketplace | `/plugin marketplace add kenryu42/cc-marketplace` | Safety Net 专用 |

### 手动安装 ECC（`/plugin` 命令失败时）

```bash
claude plugin marketplace add https://github.com/affaan-m/everything-claude-code
claude plugin search ecc
claude plugin install ecc@ecc
```

手动安装后需重启 Claude Code，且 `~/.claude/plugins/cache/` 路径可能与自动安装不同。验证 `CLAUDE_PLUGIN_ROOT` 指向正确路径。

### 经验教训

- **安装前备份**：任何插件安装前必须 `cp -r ~/.claude/rules ~/.claude/rules.backup`，Rules 和 Hooks 都是追加合并，绝不能直接覆盖 ← [[wiki/claude-code-p2.md]]
- **Safety Net Windows 配置**：Windows 需手动配 Hook 和 `SAFETY_NET_PLUGIN_ROOT` 环境变量，不会自动配置 ← [[wiki/claude-code-p2.md]]
- **Skill 双目录部署**：项目目录（开发）和 `~/.claude/skills/`（运行）是两份独立副本，改代码后必须 `cp` 同步，否则 Skill 调用仍用旧版 ← [[wiki/claude-code-p1.md]]
- **技能链调用**：Skill 内引用其他 Skill 时必须用 Skill 工具加载完整内容，不能跳过 ← [[wiki/claude-code-p1.md]]
- **子代理无法调用 Skill**：子代理和团队成员都不能使用 Skill 工具，父代理必须预加载技能内容并用 `---SKILL_NAME START/END---` 标记注入 prompt ← [[wiki/claude-code-p2.md]]

> Safety Net 状态栏 ❌ 的根因（`CLAUDE_SETTINGS_PATH` / `SAFETY_NET_PLUGIN_ROOT` 版本过时）见上文「`cc switch` 后的常见失效与修复」。

---

## 状态栏（StatusLine）

### 配置（综合版，当前使用）

```json
"statusLine": {
  "type": "command",
  "command": "py ~/.claude/scripts/statusline.py | CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json node \"${SAFETY_NET_PLUGIN_ROOT}/dist/bin/cc-safety-net.js\" --statusline"
}
```

> `CLAUDE_SETTINGS_PATH` 覆盖是状态栏正常的关键，根因与诊断见上文「`cc switch` 后的常见失效与修复」。

### 显示效果

```
glm-5.2 | app/project | feat/improve-output-formatting | ██░░░░░░░░ 24% | $6.96 0t 0f 2h41m | 🛡️ CC Safety Net ✅
 模型名(白)  目录(蓝)     Git分支(品红)                     进度条(语义色)    计费(青)               安全网
```

### 配色（4 主元素互异）

| 元素 | 颜色 | ANSI |
|------|------|------|
| 模型名 | 亮白 | `\033[97m` |
| 目录 | 亮蓝 | `\033[94m` |
| Git 分支 | 亮品红 | `\033[95m` |
| 计费/指标 | 亮青 | `\033[96m` |
| 进度条 | 语义（绿<50%/黄<65%/橙<80%/红≥80%） | 32/33/38;5;208/31 |
| 当前任务 | 黄 | `\033[33m` |

> 换色改 `_c(code, t)` 的码即可。详见 [[sources/claude-code/claude-code-statusline-setup|安装指南]]「配色」。

### 管道结构

`statusline.py`（综合版）→ `cc-safety-net --statusline`（管道追加安全网状态指示器）

### 综合版特性

| 元素 | 来源 | 说明 |
|------|------|------|
| 模型名 | stdin `model.display_name` | 当前模型（自动剥离 ANSI 残片，如 `[1m]`） |
| 目录路径 | stdin `cwd`（顶层，权威）/ `workspace.current_dir` | 压缩为最后 2 级 |
| Git 分支 | `git rev-parse --abbrev-ref HEAD` | 自动向上搜索 .git 目录 |
| 上下文进度条 | stdin `context_window.remaining_percentage` | 4 色阈值：<50% 绿、<65% 黄、<80% 橙、≥80% 红 |
| 当前任务 | `~/.claude/todos/*.json` | 读取 `in_progress` 的 `activeForm` |
| 成本/时长 | **stdin `cost` 对象（权威）** | `total_cost_usd` / `total_duration_ms`，类型稳定 |
| 工具/文件数 | `/tmp/ecc-metrics-{session}.json` | ECC bridge，best-effort（值可能为字符串，已强转） |
| 安全网 | `cc-safety-net --statusline` | 管道追加，由 safety-net 插件提供 |

> **健壮性**：每段独立 try/except（per-segment 隔离）+ null 防御 + 顶层兜底——单字段崩溃不会清空整行。详见 [[sources/claude-code/claude-code-statusline-setup|安装指南]]「健壮性与常见崩溃」。

### 依赖配置

状态栏依赖 PostToolUse hook 写入会话指标到 bridge 文件（已在 settings.json 中配置）：

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

### 脚本位置与完整源码

- **主脚本**：`~/.claude/scripts/statusline.py`（综合版，融合旧版目录/分支 + ECC 指标/任务）
- **完整源码**：[[sources/claude-code/statusline.py]]（外移至此，文档不再嵌副本）
- 备选脚本：`~/.claude/scripts/statusline.sh`（Bash 版，无 jq 依赖）
- 调试脚本：`~/.claude/scripts/debug_path.py`

**关键实现细节**（踩坑结晶）：
- `read_bridge()` 用 `os.environ.get("TEMP")` 而非 `tempfile.gettempdir()`——Python 的 `tempfile` 在 Windows 上会追加 `\claude` 子目录，与 Node.js 的 `os.tmpdir()` 路径不一致
- `build_context_bar()` 的 `used` 值通过 auto-compact buffer 公式计算，而非直接使用 stdin 的 `used_percentage`
- `read_current_task()` 按 `session_id` 前缀匹配 todo 文件，取首个 `status == "in_progress"` 的 `activeForm`

**回归测试**：`bash ~/.claude/scripts/test-statusline.sh`

### stdin JSON 关键字段

- `model.display_name` — 模型名称
- `workspace.current_dir` — 当前目录
- `context_window.remaining_percentage` — 上下文窗口剩余百分比
- `session_id` — 会话 ID（用于查找 bridge 文件和 todo 文件）

### 已知问题

1. **Windows `python` 命令**：解析为 Windows Store 存根，exit code 49，必须用 `py` 或 `py -3` 替代。
2. **stdin 叶子字段 null 导致 `render error`**：Claude Code 在会话启动/上下文压缩等过渡态会发出 key 存在但值为 `null` 的字段（`context_window.remaining_percentage`、`model.id`、`cwd`）。`.get(key, default)` 对"存在但 null"返回 `None` 而非 default，导致 `_render()` 抛 `TypeError` → `main()` fallback 显示 `statusline render error`。症状**间歇性**——过渡态瞬时，过后自愈，极具迷惑性。修复：叶子字段做 null 防御（`model.get("id") or "unknown"`、`context_window.get("remaining_percentage")` + `is None` 检查、`data.get("cwd") or os.getcwd()`），让 null 与"缺失"行为一致。回归测试：`bash ~/.claude/scripts/test-statusline.sh`。← [[wiki/claude-code-setup.md]]（2026-06-14 复盘）

3. **ECC bridge `first_timestamp` 为字符串致计费段崩溃**（2026-06-15 复盘）：`read_bridge()` 取到的 `first_timestamp` 可能是字符串，`first_ts / 1000` → `TypeError`，单 try/except 兜底清空整行（只剩模型名 + 🛡️）。修复：**计费/时长改取 stdin `cost.total_cost_usd` / `total_duration_ms`**（原生、类型稳定），bridge 值一律 `_num()` 强转，并改 per-segment 隔离。← [[wiki/claude-code-setup.md]]
4. **（2026-06-15）源码已实装上述防御**：#2 的 null 防御此前**仅文档描述、源码未改（漂移）**，本轮补进 `sources/claude-code/statusline.py`。教训：文档须与源码同步，勿信"已修"描述。← [[wiki/claude-code-setup.md]]

> `CLAUDE_SETTINGS_PATH` / `SAFETY_NET_PLUGIN_ROOT` 版本过时 / `cc switch` 重置相关问题，见上文「`cc switch` 后的常见失效与修复」。

### 经验教训（状态栏专属）

- **Worktree `.git` 是文件**：worktree 中 `.git` 是文件（指向主仓库），不是目录。`get_git_branch()` 原先用 `isdir` 会跳过 worktree 找到错误分支，修复为 `isdir(...) or isfile(...)` ← [[wiki/claude-code-p2.md]]
- **`read_bridge()` 路径**：必须用 `os.environ.get("TEMP")` 而非 `tempfile.gettempdir()`（详见关键实现细节）← [[wiki/claude-code-setup.md]]
- **stdin 叶子字段 null 是间歇崩溃根因**：`.get(key, default)` 对"存在但 null"返回 `None`，过渡态发出的 null 字段触发 `TypeError` → render error。容器级 `or {}` 不够，叶子字段也要防 null。详见已知问题 #2 ← [[wiki/claude-code-setup.md]]（2026-06-14 复盘）
- **计费/时长取 stdin `cost`，勿依赖 ECC bridge**：bridge `first_timestamp` 为字符串 → `str/1000` `TypeError` 致计费段崩、单 try 兜底清空整行（只剩模型名 + 🛡️）。改取 stdin `cost.total_cost_usd` / `total_duration_ms`（原生、类型稳定） ← [[wiki/claude-code-setup.md]]（2026-06-15 复盘）
- **per-segment 隔离**：单 try/except 兜底会让任一字段崩溃清空整行。每段独立 try/except，一段崩只丢一段 ← [[wiki/claude-code-setup.md]]（2026-06-15 复盘）
- **真实 stdin schema**：顶层有 `cwd`（权威）与 `cost` 对象，`context_window` 有 `remaining_percentage`；会话 ID 是顶层 `session_id`（非 `session.id`）。完整字段见 [[sources/claude-code/claude-code-statusline-setup|安装指南]]「可用变量」 ← [[wiki/claude-code-setup.md]]
- **状态栏只在 idle 刷新**：agent 执行期间不刷新。调试「改脚本→等→读日志」永远抓不到，须让用户发条消息触发 idle 再读 ← [[wiki/claude-code-setup.md]]（2026-06-15 复盘）
- **文档须与源码同步**：#2 曾仅文档描述「已修 null」、源码未改（漂移），浪费一轮调试。源码是 source-of-truth，文档只描述 ← [[wiki/claude-code-setup.md]]（2026-06-15 复盘）

### 备选状态栏

如不需要当前任务和会话指标，用**简易版**（只显示模型名 | 目录路径 | Git 分支 | 进度条 | 安全网）：

```json
"statusLine": {
  "type": "command",
  "command": "py -3 \"C:/Users/<user>/.claude/scripts/statusline-simple.py\" | npx -y cc-safety-net --statusline"
}
```

或用 **ECC 原版**（目录仅显示最后一级，无 Git 分支）：`node \"<ECC_ROOT>/scripts/hooks/ecc-statusline.js\"`

---

## MCP 服务器（6 个）

来源：各插件自带 `.mcp.json`。

> ⚠️ **注意（2026-06-15 核实）**：ECC 2.0.0 的 Claude Code `mcpServers`（`.claude-plugin/plugin.json`）为空 `{}`；其 `.mcp.json`（Codex 用）里只有 `chrome-devtools`。所以 plugin MCP 实际是 **`chrome-devtools`(ECC) + `context7`(context7 插件)**，勿误以为都来自 ECC。下表 6 个为手动添加的目标清单。用 `enabledMcpjsonServers` 批准、`disabledMcpjsonServers` 拒绝。

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

> **架构说明**：settings.json 只保留 2 个用户级 hooks（Safety Net + block-no-verify）。其余 hooks 由 ECC 插件自带的 `hooks/hooks.json` 自动注入，无需手动维护。诊断见上文「`cc switch` 后的常见失效与修复」。

### settings.json 中的 Hooks（2 个）

| 匹配器 | 脚本 | 功能 | 超时 |
|--------|------|------|------|
| `Bash` | `cc-safety-net.js --claude-code` | Safety Net 危险命令拦截 | 15s |
| `Bash` | `npx block-no-verify@1.1.2` | 阻止未验证的 git 操作 | - |

### ECC 插件注入的 Hooks

ECC v2.0.0 自动注册约 30 个 hooks，覆盖 PreToolUse / PostToolUse / PostToolUseFailure / PreCompact / SessionStart / SessionEnd / Stop 等事件，功能包括：危险命令拦截、tmux/git-push 提醒、文档写入警告、上下文压缩建议、自动格式化、类型检查、console.log 残留检查、代码质量门禁、治理日志、MCP 健康检查、会话指标桥接、持续学习观察器、成本追踪等。

完整列表见 `$CLAUDE_PLUGIN_ROOT/hooks/hooks.json`，**无需在 settings.json 维护**（settings.json 中引用 `${CLAUDE_PLUGIN_ROOT}` 的 hooks 必须删除，否则升级后报 "not associated with a plugin"）。

### Hook 脚本来源

Hook 脚本通过环境变量引用插件目录：
- `CLAUDE_PLUGIN_ROOT` → ECC 插件根目录（v2.0.0）
- `SAFETY_NET_PLUGIN_ROOT` → Safety Net 插件根目录（v1.0.6）

### ECC Hooks 开关机制

ECC 的 33 个 hooks 默认**零配置全开**（安装插件即生效）。通过 `~/.claude/settings.json` 的 `env` 块注入两个环境变量控制开关，判定逻辑见 `scripts/lib/hook-flags.js`：

| 变量 | 作用 | 默认值 |
|------|------|--------|
| `ECC_HOOK_PROFILE` | 粗粒度档位：`minimal`（仅核心）/ `standard`（平衡）/ `strict`（最严） | `standard` |
| `ECC_DISABLED_HOOKS` | 细粒度黑名单，逗号分隔 hook ID | 无 |

**判定优先级**：黑名单 > 档位。每个 hook 在 `hooks/hooks.json` 声明所属档位（如 `standard,strict`），当前档位在允许列表内 → 启用。

#### 当前配置（settings.json env）

```json
"ECC_HOOK_PROFILE": "standard",
"ECC_DISABLED_HOOKS": "stop:desktop-notify,pre:edit-write:gateguard-fact-force,pre:observe:continuous-learning,post:observe:continuous-learning,stop:evaluate-session"
```

#### 禁用的 5 个 hooks 及理由

| Hook ID | 原功能 | 禁用理由 |
|---------|--------|----------|
| `stop:desktop-notify` | 响应结束后弹桌面通知 | WSL/容器无桌面 daemon，通知无效 |
| `pre:edit-write:gateguard-fact-force` | 首次编辑某文件硬阻断，强制先调查 | 每个新文件首次编辑都被拦，影响迭代速度。临时关用 `ECC_GATEGUARD=off` |
| `pre:observe:continuous-learning` | PreToolUse 持续学习观察器 | 每次工具调用都跑，开销最重 |
| `post:observe:continuous-learning` | PostToolUse 持续学习观察器 | 同上（Pre+Post 各跑一次） |
| `stop:evaluate-session` | 响应结束自动评估并提取模式 | 配合观察器关闭；不依赖自动 `/learn` 时无意义 |

#### 其他专项开关

| 变量 | 作用 |
|------|------|
| `ECC_GATEGUARD=off` | 单独关 GateGuard（保留其他） |
| `ECC_GOVERNANCE_CAPTURE=1` | **反向**：治理捕获默认关，需才开 |
| `ECC_SESSION_START_CONTEXT=off` | 关 SessionStart 额外上下文注入 |
| `ECC_CONTEXT_MONITOR_COST_WARNINGS=off` | 保留上下文/范围警告，只关成本估算 |

#### 验证生效

环境变量在进程启动时读取，**改完需重启 Claude Code 会话**。当前会话不受影响。

### 经验教训（Hooks 专属）

- **settings.json 只保留 2 个用户级 hooks**：Safety Net（`${SAFETY_NET_PLUGIN_ROOT}`）+ block-no-verify（`npx`）。所有引用 `${CLAUDE_PLUGIN_ROOT}` 的 hooks 必须删除，由 ECC 插件 `hooks/hooks.json` 自动注册 ← [[wiki/claude-code-setup.md]]（2026-06-10 复盘）
- **Hook 类型选择**：prompt hook 返回 `{"ok": false, "reason": "..."}` 可让主 Claude 继续工作（拥有完整上下文和工具权限），command hook + `claude -p` 子进程需读 transcript 重建上下文，有冷启动和终端乱码风险 ← [[wiki/claude-code-p1.md]]
- **Stop Hook 演进 V1→V4**：command+claude-p（V1）→ prompt hook ok:false（V2）→ prompt hook+后台 agent（V3，过度触发）→ 关键词触发+后台 agent（V4，当前方案）。教训：自动触发的 hook 评估器难以准确判断对话是否有复盘价值，用户主动触发更可靠 ← [[wiki/claude-code-p1.md]]
- **ECC 升级后 Hooks 报错**：ECC 插件升级后 `plugins/cache/` 路径变化，settings.json 中旧 hooks 失效报 "not associated with a plugin"。ECC v2+ 自带 `hooks/hooks.json` 自解析路径，settings.json 只保留 Safety Net 和 block-no-verify ← [[wiki/claude-code-p2.md]]
- **Hooks 范式**：PreToolUse 用于长命令提醒用 tmux、写 .md 阻止；PostToolUse 用于自动格式化/类型检查/console.log 检查；Stop 用于审计残留 ← [[wiki/claude-code-p3.md]]

> `cc switch` 重置 settings.json 导致 env 路径/hooks 回旧值的问题，见上文「`cc switch` 后的常见失效与修复」。

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

## Rules — 语言编码规范

### 目录结构

整套规范分为**通用层**与**各编程语言专属目录**两层架构：
```
rules/
├── common/          # 与编程语言无关的通用准则（必须安装）
│   ├── coding-style.md    # 代码风格规范
│   ├── git-workflow.md    # Git 工作流规范
│   ├── testing.md         # 测试规范
│   ├── performance.md     # 性能规范
│   ├── patterns.md        # 通用设计模式
│   ├── hooks.md           # 钩子配置规范
│   ├── agents.md          # AI 代理交互规范
│   └── security.md        # 安全开发规范
├── typescript/      # TypeScript / JavaScript 专属规范
├── angular/         # Angular 框架专属规范
├── python/          # Python 专属规范
├── golang/          # Go 语言专属规范
├── web/             # 网页/前端通用专属规范
├── swift/           # Swift 专属规范
├── php/             # PHP 专属规范
├── ruby/            # Ruby / Rails 框架专属规范
└── arkts/           # 鸿蒙系统 ArkTS 语言专属规范
```
- **common/ 通用目录**
：存放全语言通用准则，不含特定编程语言代码示例。
- **各语言专属目录**
：在通用规范基础上，补充对应框架专属实践、配套工具与代码示例；每个文件都会关联引用通用目录里对应的基础规范文档。

### 安装方式
#### 方案一：一键安装脚本（推荐）
```bash
# 安装通用规范 + 一套或多套编程语言专属规范
./install.sh typescript
./install.sh angular
./install.sh python
./install.sh golang
./install.sh web
./install.sh swift
./install.sh php
./install.sh ruby
./install.sh arkts

# 一次性安装多套语言规范
./install.sh typescript python
```

#### 方案二：手动安装
> **重要提醒**：必须完整复制整个目录，禁止使用 `/*`扁平化复制。
> 通用目录与各语言专属目录内存在同名文件，若扁平化合并到同一个文件夹，
> 语言专属文件会直接覆盖通用规范，同时语言文件中相对路径 `../common/`的引用会全部失效。
> 用户级 Claude 配置请统一使用下方 ECC 专属命名空间；平铺式包目录会和其他第三方规范包产生路径冲突，也与主 README 文档规范不符。

```bash
# 首次执行时创建 ECC 规范专属目录
mkdir
 -p ~/.claude/rules/ecc

# 安装通用规范（所有项目均必须依赖）
cp -r rules/common ~/.claude/rules/ecc/

# 根据项目技术栈安装对应语言专属规范
cp -r rules/typescript ~/.claude/rules/ecc/
cp -r rules/angular ~/.claude/rules/ecc/
cp -r rules/python ~/.claude/rules/ecc/
cp -r rules/golang ~/.claude/rules/ecc/
cp -r rules/web ~/.claude/rules/ecc/
cp -r rules/swift ~/.claude/rules/ecc/
cp -r rules/php ~/.claude/rules/ecc/
cp -r rules/ruby ~/.claude/rules/ecc/
cp -r rules/arkts ~/.claude/rules/ecc/

# 注意！！！请根据你的项目实际需求按需配置，以上仅为示例参考。全局安装通用规范，项目级根据技术栈安装对应语言专属规范。
```

若需要项目本地生效的规范，在项目根目录下使用相同目录结构：
```bash
mkdir -p .claude/rules/ecc
cp -r rules/common .claude/rules/ecc/
cp -r rules/typescript .claude/rules/ecc/
```

加载机制确认：ECC SessionStart hook 不碰 rules、plugin.json 不声明 rules 目录 → rules 纯靠 Claude Code 原生读 ~/.claude/rules/。复制即加载，无双重注入风险。带 paths frontmatter 的做 scoped、不带的做无条件——与文档 L40 的分层吻合。

ECC 源目录提供 18 个规则集：common + angular, arkts, cpp, csharp, dart, fsharp, golang, java, kotlin, perl, php, python, ruby, rust, swift, typescript, web。每种语言含 5 个维度：
- `coding-style` — 编码风格（不可变性、文件组织、大小限制）
- `security` — 安全检查（禁止硬编码密钥、验证输入）
- `testing` — 测试规范（覆盖率、测试模式）
- `patterns` — 语言特定模式
- `hooks` — Hook 文档

← [[sources/claude-code/2026-05-13-safety-net-hook-fix.md#ECC 规则安装|ECC 规则安装实践]]

← [[sources/claude-code/ecc.md|ECC 完整参考]] §入门-4

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

ECC v2.0.0 自动安装了大量领域专用代理，包括但不限于：`a11y-architect`、`code-explorer`、`cpp-reviewer`、`dart-build-resolver`、`django-reviewer`、`fastapi-reviewer`、`flutter-reviewer`、`go-reviewer`、`java-reviewer`、`kotlin-reviewer`、`network-architect`、`python-reviewer`、`react-reviewer`、`rust-build-resolver`、`swift-reviewer`、`typescript-reviewer` 等。完整列表见 `~/.claude/agents/` 目录。

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

## 容器 / WSL 一步到位配置

目标：在 Linux 容器/WSL 里复刻本配置，三步跑通，无需反复调试。

```bash
# 1. 状态栏脚本（canonical 源：4 色 + per-segment 隔离 + null 防御 + 计费取 stdin cost）
cp sources/claude-code/statusline.py ~/.claude/scripts/statusline.py && chmod +x ~/.claude/scripts/statusline.py

# 2. settings.json（ECC 2.0.0 / safety-net 1.0.6 路径 + ECC 档位 + 2 hooks + python3 statusLine + effortLevel:max + enabledMcpjsonServers）
#    classifier 禁止 agent 直写 ~/.claude/settings.json → stage + 用户 !cp
cp sources/claude-code/container-settings.json /tmp/cc-target-settings.json
!cp /tmp/cc-target-settings.json ~/.claude/settings.json   # 用户执行（! 前缀在本会话跑）

# 3. 重载
/reload-plugins
```

关键差异（相对 Windows 宿主）：

| 项 | 宿主 (Windows) | 容器 (Linux) |
|----|------|------|
| Python | `py` / `py -3` | **`python3`**（无 `py`） |
| statusLine 命令 | `py ~/.claude/scripts/statusline.py \| ...` | `python3 "$HOME/.claude/scripts/statusline.py" \| ...` |
| ECC 版本/路径 | `.../ecc/ecc/2.0.0-rc.1` | `.../ecc/ecc/2.0.0` |
| safety-net 版本/路径 | `.../safety-net/1.0.1` | `.../safety-net/1.0.6` |
| settings.json 写入 | 直写 / cc switch | **classifier 禁 agent 直写** → stage `/tmp` + 用户 `!cp` |
| plugin MCP | （见 MCP 节） | `enabledMcpjsonServers: [chrome-devtools, context7]` |

> - 版本随插件升级变动，落地前 `find ~/.claude/plugins/cache -name cc-safety-net.js` 核实实际路径，回填 `container-settings.json` 的 `env`。
> - `container-settings.json` 不含 API base URL/token（由 `cc switch` 或环境另行注入），套用不扰动现有提供商。
> - 状态栏只在 idle 刷新：配置后发条消息触发刷新即可见整行。

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
| `~/.claude/plans/` | 计划文件 |
| `~/.claude/projects/` | 4 个已跟踪项目 |
| `~/.claude/scripts/` | 状态栏脚本 |
| `~/.claude/sessions/` / `tasks/` / `teams/` | 会话记录 / 任务追踪 / 代理团队 |

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
- 更换状态栏脚本 → 更新「状态栏」+ 同步 [[sources/claude-code/statusline.py]]
- 增减 MCP 服务器 → 更新「MCP 服务器」
- 增减技能/命令/规则/代理 → 更新对应章节
- 更新 cc switch 模板 → 同步 [[sources/claude-code/cc-switch-template.json]]

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

**完整模板**：[[sources/claude-code/cc-switch-template.json]]

### 模板说明（字段规则）

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

---

> 来源：`~/.claude/settings.json`、`~/.claude/plugins/installed_plugins.json`、`~/.claude/skills/`、`~/.claude/rules/`、`~/.claude/agents/`、`~/.claude/teams/`、`E:/Files/Obsidian/.claude/`
