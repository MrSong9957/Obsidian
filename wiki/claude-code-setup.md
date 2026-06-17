---
name: Claude Code 当前配置清单
description: 本机 Claude Code 完整配置清单 + 自检清单（插件/Hooks/状态栏/MCP/技能/代理/规则/团队/经验教训）— 定期巡检、排查配置问题、新环境迁移时参考
type: concept
updated: 2026-06-17
tags: claude-code, setup, plugins, hooks, mcp, skills, agents, config, checklist
source: https://example.com
---

# Claude Code 当前配置清单

> 关联概念：[[wiki/claude-code.md|Claude Code 配置与工作流]] | [[wiki/opencode-bridge.md|OpenCode 桥接]]
>
> 详细参考：[[sources/claude-code/claude-code-setup-references.md|配置清单详细参考]]（cc switch 诊断、状态栏源码、Hooks 机制、Rules 安装、缓存优化等）

本文档记录本机 Claude Code 的实际安装配置，作为排查和迁移的参考。配置变更时需同步更新本文档。

## 全局设置概要

配置文件路径：`~/.claude/settings.json`

| 配置项 | 值 | 说明 |
|--------|-----|------|
| `defaultMode` | `"auto"` | 自动批准所有工具调用 |
| `alwaysThinkingEnabled` | `true` | 始终启用扩展思考 |
| `skipDangerousModePermissionPrompt` | `true` | 跳过危险模式确认 |
| `autoMemoryEnabled` | `true` | 自动记忆功能 |
| `todoFeatureEnabled` | `true` | Todo 任务追踪 |
| `showClearContextOnPlanAccept` | `true` | Plan 模式审批时显示清空上下文选项，释放窗口空间 |
| `autoCompactEnabled` | `true` | 上下文满时自动压缩，避免会话中断 |
| `fileCheckpointingEnabled` | `true` | 编辑前自动快照，支持 `/rewind` 撤销 |
| `language` | `"chinese"` | 中文输出 |
| 自动更新 | 已开启 | — |
| Tool Search | 已启用 | `ENABLE_TOOL_SEARCH=true` |
| Agent Teams | 已启用 | `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` |

> 完整权威配置以 [[sources/claude-code/cc-switch-template.json]] 为准（SSOT）。

### API 配置

| 配置项 | 值 |
|--------|-----|
| 自定义端点 | `https://token-plan-cn.xiaomimimo.com/anthropic` |
| 模型映射 | `mimo-v2.5-pro`（Opus） / `mimo-v2.5`（Sonnet/Haiku） |
| 认证方式 | `ANTHROPIC_AUTH_TOKEN` |
| 缓存优化 | `CLAUDE_CODE_ATTRIBUTION_HEADER=0` |

### 经验教训

- **持久化优先级**：Hooks > CLAUDE.md > rules > MEMORY.md。安全关键规则必须用 Hook 实现 ← [[wiki/claude-code.md]]
- **上下文窗口管理**：同时启用 <10 个 MCP，活跃工具 <80 个 ← [[wiki/claude-code.md]]
- **settings.json 关键隐藏字段**：`showClearContextOnPlanAccept`、`autoCompactEnabled`、`fileCheckpointingEnabled` ← [[wiki/claude-code.md]]

> `cc switch` 后状态栏/Hooks/安全网失效的诊断流程 → [[sources/claude-code/claude-code-setup-references.md#cc switch 后的常见失效与修复]]

---

## 已安装插件（8 个）

### 核心插件

| 插件 | 版本 | 用途 |
|------|------|------|
| **superpowers** | 5.1.0 | 技能调度框架，所有任务的起点 |
| **ecc** | 2.0.0 | 基础设施层：编码规范、上下文管理、安全审查、持续学习 |

> Superpowers 提供工作流骨架，ECC 提供底层能力。两者互补，非替代关系。

### 辅助插件

| 插件 | 版本 | 用途 |
|------|------|------|
| safety-net | 1.0.6 | 安全网，拦截危险命令 |
| github | 9e150cfd48b8 | GitHub 集成 |
| context7 | 9e150cfd48b8 | 实时文档查询 |
| frontend-design | 9e150cfd48b8 | 前端设计辅助 |
| skill-creator | 9e150cfd48b8 | 技能创建工具 |
| learning-output-style | 1.0.0 | 交互式学习输出风格 |

### 安装渠道

| 渠道 | 命令 | 适用 |
|------|------|------|
| Claude Code 插件市场 | `/plugin` | Superpowers 等 6 个插件 |
| GitHub 仓库 | `/plugin marketplace add <repo>` | ECC |
| cc-marketplace | `/plugin marketplace add kenryu42/cc-marketplace` | Safety Net |

### 经验教训

- **安装前备份**：任何插件安装前必须 `cp -r ~/.claude/rules ~/.claude/rules.backup` ← [[wiki/claude-code.md]]
- **Safety Net Windows 配置**：Windows 需手动配 Hook 和 `SAFETY_NET_PLUGIN_ROOT` 环境变量 ← [[wiki/claude-code.md]]
- **Skill 双目录部署**：项目目录（开发）和 `~/.claude/skills/`（运行）是两份独立副本，改代码后必须 `cp` 同步 ← [[wiki/claude-code.md]]
- **子代理无法调用 Skill**：父代理必须预加载技能内容并注入 prompt ← [[wiki/claude-code.md]]

> 手动安装 ECC 步骤 → [[sources/claude-code/claude-code-setup-references.md#已安装插件]]

---

## 状态栏（StatusLine）

当前使用综合版：模型名 | 目录 | Git 分支 | 进度条 | 计费 | 安全网。

> 详细配置（管道结构、配色、已知问题、备选方案）→ [[sources/claude-code/claude-code-setup-references.md#状态栏（StatusLine）详细配置]]
>
> 完整源码：[[sources/claude-code/statusline.py]]

### 经验教训（状态栏专属）

- **Worktree `.git` 是文件**：`get_git_branch()` 修复为 `isdir(...) or isfile(...)`
- **stdin 叶子字段 null 是间歇崩溃根因**：需做 null 防御
- **计费/时长取 stdin `cost`**，勿依赖 ECC bridge
- **per-segment 隔离**：每段独立 try/except
- **状态栏只在 idle 刷新**
- **文档须与源码同步**

---

## MCP 服务器（6 个）

| 服务器 | 传输方式 | 用途 |
|--------|----------|------|
| github | stdio | GitHub API |
| context7 | stdio | 实时文档查询 |
| exa | HTTP | 网页搜索 |
| memory | stdio | 知识图谱记忆 |
| playwright | stdio | 浏览器自动化 |
| sequential-thinking | stdio | 序列化思考 |

### 经验教训

- **GitHub MCP 认证**：需在 settings.json 的 `env` 中添加 `GITHUB_PERSONAL_ACCESS_TOKEN` ← [[wiki/claude-code.md]]
- **切换 API KEY 后 MCP 断连**：99% 是 token 问题，验证：`claude mcp list` ← [[wiki/claude-code.md]]
- **HTTP MCP 无需本地安装** ← [[wiki/claude-code.md]]

---

## Hooks 系统

settings.json 只保留 2 个用户级 hooks（Safety Net + block-no-verify），其余由 ECC 插件 `hooks/hooks.json` 自动注入。

> 详细配置（ECC 开关机制、禁用列表、专项开关）→ [[sources/claude-code/claude-code-setup-references.md#Hooks 系统详细配置]]

### 经验教训（Hooks 专属）

- **settings.json 只保留 2 个用户级 hooks**：所有引用 `${CLAUDE_PLUGIN_ROOT}` 的 hooks 必须删除
- **Hook 类型选择**：prompt hook 优于 command hook（无冷启动风险）
- **Stop Hook 演进 V1→V4**：关键词触发+后台 agent（当前方案）
- **ECC 升级后 Hooks 报错**：ECC v2+ 自带 `hooks/hooks.json` 自解析路径

---

## 技能（Skills，3 个）

路径：`~/.claude/skills/`

| 技能 | 功能 | 触发方式 |
|------|------|----------|
| `docker-dev-container/` | 创建 Docker 开发环境 | `/docker-dev-container` |
| `obsidian-link/` | Obsidian 仓库 symlink 桥接 | `/obsidian-link` |
| `obsidian-remember/` | Obsidian 知识仓库管理 | `/obsidian-remember` |

技能依赖链：`obsidian-remember` → `obsidian-link`

### 经验教训

- **SKILL.md 结构**：触发条件 → 操作流程 → 输出格式 → 安全约束 ← [[wiki/claude-code.md]]
- **.env 加载策略**：`os.environ.setdefault` 保证先加载的优先 ← [[wiki/claude-code.md]]

---

## 命令（Commands，1 个）

路径：`~/.claude/commands/`

| 命令 | 功能 |
|------|------|
| `/note-obsidian` | 从当前对话提取知识写入 Obsidian wiki（后台 agent） |

---

## Rules — 语言编码规范

整套规范分为**通用层**与**各编程语言专属目录**两层架构。ECC 源目录提供 18 个规则集。

> 详细目录结构和安装方式 → [[sources/claude-code/claude-code-setup-references.md#Rules — 语言编码规范详细配置]]

---

## 自定义代理（Agents，10 + 67 个）

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

### ECC 插件提供的代理（67 个）

包括 `a11y-architect`、`code-explorer`、`cpp-reviewer`、`django-reviewer`、`go-reviewer`、`java-reviewer`、`python-reviewer`、`react-reviewer`、`rust-reviewer`、`typescript-reviewer` 等。完整列表见 `~/.claude/agents/`。

### 经验教训

- **限制工具 = 聚焦执行** ← [[wiki/claude-code.md]]
- **传递目标上下文**，不只是查询 ← [[wiki/claude-code.md]]
- **`description` 最关键**：Claude 据此决定何时自动委托任务 ← [[wiki/claude-code.md]]

---

## 团队（Teams，1 个）

路径：`~/.claude/teams/`

| 团队 | 成员 | 用途 |
|------|------|------|
| realtime-push-research | team-lead（glm-5）、performance-expert（claude-opus-4-6） | 实时推送系统架构研究 |

---

## 工作流约束

### Hooks 持久化（硬约束）

必须用 Hook 实现的规则：
- 安全关键：`rm -rf`、`git push --force` 等危险命令拦截
- 流程强制：TDD 验证、代码审查、提交前检查
- 环境一致性：开发环境配置、依赖检查

### ECC 软规则覆盖

ECC 插件的 rules 是软约束，可被项目级 CLAUDE.md 覆盖。但不能覆盖 Hooks（硬约束）。

### 子代理派发规则

派发子代理时，将对应技能要求注入 prompt：
- 实现任务 → 加载 `test-driven-development` 技能
- 代码审查 → 加载 `requesting-code-review` 技能
- 完成分支 → 加载 `finishing-a-development-branch` 技能

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
| MCP 连接 | `claude mcp list` | 全部 Connected |
| 状态栏 | 看底部状态栏 | 显示模型/目录/分支/进度/安全网 |
| Hooks 生效 | 执行 `rm -rf /tmp/test` | Safety Net 拦截 |
| 技能可用 | `/docker-dev-container` | 技能内容加载 |
| 规则加载 | 问 Claude "你加载了哪些 rules" | 列出 rules 文件 |
| API 连接 | 任意对话 | 正常回复 |

---

> 完整详细参考（cc switch 诊断、状态栏源码、Hooks 机制、Rules 安装、容器配置、缓存优化、cc switch 模板、MiMoCode 链接）→ [[sources/claude-code/claude-code-setup-references.md|配置清单详细参考]]
