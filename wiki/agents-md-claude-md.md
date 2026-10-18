---
name: AGENTS.md 与 CLAUDE.md 配置关系
description: Claude Code 指令文件四层级放置规则、AGENTS.md 默认替补逻辑、四档"项目说明"配置与 import 共存方案 — 配置指令文件或排查其未加载时查阅
type: reference
updated: 2026-10-18
tags: [claude-code, claude-md, agents-md, memory]
source: https://code.claude.com/docs/zh-CN/memory
---

# AGENTS.md 与 CLAUDE.md 配置关系

### 核心思想

CLAUDE.md 是 Claude Code 的原生指令文件，四个层级都认；AGENTS.md 是跨工具通用标准，Claude Code（v2.1.277 起）只在**项目层**认它，且默认只是"没有 CLAUDE.md 时的替补"。全局想用 AGENTS.md 内容，唯一官方通道是在 CLAUDE.md 里写 `@AGENTS.md` 导入。[^1]

### 详细说明

#### 层级地图：谁能放什么

| 层级 | 位置 | AGENTS.md | 给谁看 |
|---|---|---|---|
| 托管策略（公司） | `/etc/claude-code/CLAUDE.md`（或 managed settings 的 `claudeMd` 键） | ❌ | 全组织，IT 管，个人无法排除 |
| 用户指令（全局） | `~/.claude/CLAUDE.md` ＋ `~/.claude/rules/*.md` | ❌ | 本机所有项目 |
| 项目指令 | `./CLAUDE.md`、`./.claude/CLAUDE.md`、`./AGENTS.md`、`./.claude/AGENTS.md` | ✅ 唯一认 AGENTS.md 的层 | 跟仓库走，团队共享 |
| 本地指令 | `./CLAUDE.local.md`（gitignore） | ❌ | 个人＋本项目 |
| 子目录 | 两种文件都认 | ✅ | 按需加载（Read/Edit 触碰该子目录时） |

永不读取：`AGENTS.local.md`、`AGENTS.override.md`、`.agents/` 目录。[^1]

#### 项目层默认规则（"抢座位"）

从当前目录一路向上查到根，看有没有 `CLAUDE.md` / `.claude/CLAUDE.md` / `CLAUDE.local.md` 任一：

- 有 → 只读 CLAUDE.md，AGENTS.md 不读
- 都没有 → 读路径上的 AGENTS.md（会话开头显示 `no CLAUDE.md found; AGENTS.md loaded: …`）

三个"不参赛、照常并行加载"的旁观者：`~/.claude/CLAUDE.md`、托管 CLAUDE.md、`.claude/rules/`。[^1]

> [!warning] 常见坑
> 在依赖 AGENTS.md 的仓库里加 `CLAUDE.local.md` 会把 AGENTS.md 顶掉；想都要就改用下面的 `claude-md-and-agents-md` 档。

#### 四档配置（`/config` → 项目说明）

| 值 | 行为 |
|---|---|
| `claude-md-or-agents-md`（默认） | 有 CLAUDE 读 CLAUDE，没有读 AGENTS |
| `claude-md-and-agents-md` | 都读；同目录 CLAUDE 在前、AGENTS 在后；已 import 的不重复读 |
| `claude-md` | 只读 CLAUDE.md |
| `managed-only` | 只留托管 CLAUDE.md ＋ 自动记忆 |

写入配置文件只认 `~/.claude/settings.json`、`--settings`、托管设置里的 `pluginConfigs["cc-plugin-agents-md@builtin"].options.instructionFiles`（v2.1.285 前插件 ID 为 `agents-md@builtin`）；写在项目/本地设置里无效。[^1]

#### 两全方案（官方三种）

1. **只放 AGENTS.md** → 自动直读，要求 ≥ v2.1.277。
2. **CLAUDE.md 首行写 `@AGENTS.md`** → 任何会话都稳；Claude 专属指令写导入行下方（先读导入、后读其余）；官方保证无论哪档配置都不会读两遍。
3. **symlink**（`ln -s AGENTS.md CLAUDE.md`）→ 有两坑：Edit/Write 拒绝穿透链接写文件；Windows 克隆把链接检出成纯文本。Windows 场景官方建议用方案 2。[^1]

#### 直读 AGENTS.md 与 import 方式的差异

| | CLAUDE.md / import | 直读 AGENTS.md |
|---|---|---|
| `InstructionsLoaded` hook | 触发 | 不触发 |
| `--add-dir` 附加目录 | 加载其 CLAUDE.md | 其 AGENTS.md 不加载 |
| 工作目录外的 `@import` | 弹审批框 | 不弹；仅该项目已批准外部导入才加载 |

#### 直读不可用的会话

版本 < v2.1.277；`/plugin` 停用内置插件；升级后首个会话（偶尔）；< v2.1.281 的 Bedrock / 关遥测会话。兜底统一是 `@AGENTS.md` import。[^1]

#### 加载顺序与合并

- 所有文件拼接进上下文，互不覆盖；顺序：托管 → 用户 → 项目上层 → 项目（越靠近启动目录越晚读）；同目录内 `CLAUDE.local.md` 排在 `CLAUDE.md` 后。
- 用户级 rules 先于项目级 rules；两边冲突时 Claude 可能任听一边，保持一致才是正解。[^2]

#### 健康线与验证

- 单文件建议 < 200 行（超限降遵守度），> 4 MiB 直接跳过；`claudeMdExcludes` 可按 glob 排除指定文件（托管层除外）。
- `/context` 看实际加载清单（Memory files）；`/memory` 看文件位置与自动记忆开关；`/doctor prompt-audit` 体检过时/冲突指令。[^2]

### 相关概念

- [[wiki/claude-code.md|Claude Code 配置与工作流]] — settings/rules/记忆系统聚合参考
- [[wiki/claude-md-base.md|CLAUDE.md 双层架构模板]] — 全局（Karpathy 准则）＋项目级双层落地方案，是本页 import 方案的具体应用
- [[wiki/how-anthropic-teams-use-claude-code.md|Anthropic 团队如何使用 Claude Code]] — 团队实践视角

### 待解决的问题

暂无。

## 引用索引

[^1]: https://code.claude.com/docs/zh-CN/memory#agents-md — 官方 Memory 文档 AGENTS.md 节：默认读取规则、四档配置、共享方案、差异表、不可用会话。
[^2]: https://code.claude.com/docs/zh-CN/memory#claude-md-files — 官方 Memory 文档 CLAUDE.md 文件：层级表、加载顺序、导入语法、大小限制。
[^3]: https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md — 版本时间线：v2.1.277 引入 AGENTS.md，v2.1.281 扩展到 Bedrock/关遥测会话。
