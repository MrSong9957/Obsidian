---
name: Claude Code 当前配置清单
description: 本机 Claude Code 完整配置清单 + 自检清单（插件/Hooks/状态栏/MCP/技能/代理/规则/团队/经验教训）— 定期巡检、排查配置问题、新环境迁移时参考
type: concept
updated: 2026-06-07
tags: claude-code, setup, plugins, hooks, mcp, skills, agents, config, checklist
source: https://example.com
---

# Claude Code 当前配置清单

> 关联概念：[[wiki/claude-code-p1.md|Claude Code 配置]] | [[wiki/claude-code-p2.md|Claude Code 工具]] | [[wiki/claude-code-p3.md|Claude Code 进阶工作流]] | [[wiki/claude-code-personal-config.md|个人 Claude Code 配置]]

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
| 自定义端点 | `https://open.bigmodel.cn/api/anthropic` |
| 模型映射 | `glm-5.1`（Opus/默认） / `glm-5-turbo`（Sonnet/Haiku/Reasoning） |

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


### 经验教训

- **安装前备份**：任何插件安装前必须 `cp -r ~/.claude/rules ~/.claude/rules.backup`，Rules 和 Hooks 都是追加合并，绝不能直接覆盖 ← [[wiki/claude-code-p2.md]]
- **Safety Net Windows 配置**：Windows 需手动配 Hook 和 `SAFETY_NET_PLUGIN_ROOT` 环境变量，不会自动配置 ← [[wiki/claude-code-p2.md]]
- **Safety Net 状态栏 fallback**：`isPluginEnabled()` 只读 `CLAUDE_SETTINGS_PATH`（IDE 设为项目级路径），文件不存在或不包含 `enabledPlugins` 时直接返回 false。修复：增加 fallback 回退 `~/.claude/settings.json` 全局配置；状态栏命令改用 `node "${SAFETY_NET_PLUGIN_ROOT}/dist/bin/cc-safety-net.js" --statusline` 而非 `npx -y`（npx 从 npm 拉缓存，不走本地修改）。插件升级后需重新打补丁，长期方案向上游提 PR ← [[wiki/claude-code-setup.md]]
- **Skill 双目录部署**：项目目录（开发）和 `~/.claude/skills/`（运行）是两份独立副本，改代码后必须 `cp` 同步，否则 Skill 调用仍用旧版 ← [[wiki/claude-code-p1.md]]
- **技能链调用**：Skill 内引用其他 Skill 时必须用 Skill 工具加载完整内容，不能跳过 ← [[wiki/claude-code-p1.md]]
- **子代理无法调用 Skill**：子代理和团队成员都不能使用 Skill 工具，父代理必须预加载技能内容并用 `---SKILL_NAME START/END---` 标记注入 prompt ← [[wiki/claude-code-p2.md]]

---

## 状态栏（StatusLine）

### 配置（综合版，当前使用）

```json
"statusLine": {
  "type": "command",
  "command": "py ~/.claude/scripts/statusline.py | node \"${SAFETY_NET_PLUGIN_ROOT}/dist/bin/cc-safety-net.js\" --statusline"
}
```

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

Windows 上 `python` 可能解析为 Windows Store 存根，应使用 `py -3` 替代。

### 经验教训

- **Windows `py` vs `python`**：`python` 命令返回 exit code 49（不存在），必须用 `py` 或 `py -3` ← [[wiki/claude-code-p2.md]]
- **Worktree `.git` 是文件**：worktree 中 `.git` 是文件（指向主仓库），不是目录。`get_git_branch()` 原先用 `isdir` 会跳过 worktree 找到错误分支，修复为 `isdir(...) or isfile(...)` ← [[wiki/claude-code-p2.md]]
- **`read_bridge()` 路径**：必须用 `os.environ.get("TEMP")` 而非 `tempfile.gettempdir()`，因为 Python 的 `tempfile` 在 Windows 上会追加 `\claude` 子目录，与 Node.js 的 `os.tmpdir()` 路径不一致 ← [[wiki/claude-code-setup.md]]
- **切换 API KEY 后 StatusLine 失效**：先检查 `py` 命令是否正常，再用 `echo '...' | py statusline.py` 验证脚本本身 ← [[wiki/claude-code-p2.md]]

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
- `SAFETY_NET_PLUGIN_ROOT` → Safety Net 插件根目录（v0.9.0）

### 经验教训

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

> 来源：`~/.claude/settings.json`、`~/.claude/plugins/installed_plugins.json`、`~/.claude/skills/`、`~/.claude/rules/`、`~/.claude/agents/`、`~/.claude/teams/`、`E:/Files/Obsidian/.claude/`
