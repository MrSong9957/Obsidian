---
name: ECC Superpowers 补强配置指南
description: ECC 插件中纳入 Superpowers 补强的 8 项功能——配置方式、使用时机、行为规则，作为今后开发和 CLAUDE.md 编写的指导
type: concept
updated: 2026-05-31
tags: ecc, superpowers, hooks, rules, context-management, session-memory, token-optimization, subagents
source: [sources/claude-code/ecc.md]
---

# ECC Superpowers 补强配置指南

> 原始资料：[[sources/claude-code/ecc.md|ECC 完整参考]]
>
> 本文档覆盖 ECC 中与 Superpowers 互补的 8 项功能。除 Rules 需手动安装外，其余功能安装 ECC 插件后自动可用。

---

## 定位

ECC 是独立于 Superpowers 的插件，提供**基础设施层**，补充 Superpowers 不覆盖的领域。

## 与 Superpowers 分工

- **Superpowers 管**（流程层）：TDD、规划、代码审查、Git 工作流、调试、验证
- **ECC 管**（基础设施层）：编码规范、上下文管理、语言模式、安全、持续学习
- **重叠时优先用 Superpowers**：TDD 流程、任务规划、代码审查、分支管理（Superpowers 更严格）

---

## 1. Hooks — 自动质量门禁

### 配置

**零配置。** ECC 安装为插件后，Claude Code v2.1+ 自动加载 `hooks/hooks.json`，共 33 个 hooks 跨 7 个事件类型。

**禁止**在 `plugin.json` 或 `settings.json` 中重复声明 hooks 字段（会触发 "Duplicate hooks file detected" 错误）。

### 关键 hooks 职能

| 事件类型 | 数量 | 核心功能 |
|----------|------|---------|
| PreToolUse | 8 | Bash 预检、文档写入警告、配置保护、治理捕获 |
| PostToolUse | 8 | 质量门禁、设计质量检查、console.log 残留检测 |
| PreCompact | 1 | 压缩策略建议，避免自动压缩丢关键上下文 |
| Stop | 6 | 格式化/类型检查、会话评估、成本追踪、桌面通知 |
| SessionStart | 1 | 会话启动引导 |
| SessionEnd | 1 | 会话结束标记 |
| PostToolUseFailure | 1 | 工具调用失败时 MCP 健康检查 |

### 使用时机

始终生效，无需手动触发。Claude 应：
- **信任 hooks 的拦截**：hooks 触发的警告（配置保护、console.log 残留）是硬性约束，不是建议
- **配合 PreCompact hook**：当 hook 建议压缩时，评估当前是否处于逻辑节点，配合 Strategic Compacting 使用

← [[sources/claude-code/ecc.md#Hooks 完整清单|ECC Hooks 完整清单]]

---

## 2. Rules — 语言编码规范

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

# 注意！！！请根据你的项目实际需求按需配置，以上仅为示例参考
```

若需要项目本地生效的规范，在项目根目录下使用相同目录结构：
```bash
mkdir -p .claude/rules/ecc
cp -r rules/common .claude/rules/ecc/
cp -r rules/typescript .claude/rules/ecc/
```

加载机制确认：ECC SessionStart hook 不碰 rules、plugin.json 不声明 rules 目录 → rules 纯靠 Claude Code 原生读 ~/.claude/rules/。复制即加载，无双重注入风险。带 paths frontmatter 的做 scoped、不带的做无条件——与文档 L40 的分层吻合。

## 规范（Rules）与实操手册（Skills）的区别
- **规范 Rules**：定义通用开发标准、编码约定与检查清单，适用范围广（例如：测试覆盖率不低于80%、禁止硬编码密钥）。
- **实操手册 Skills**（存放于 `skills/` 目录）：针对特定开发任务提供详尽、可直接落地的参考资料（例如：`python-patterns` Python 编码范式、`golang-testing`
 Go 测试实操）。

各语言专属规范文件会按需引用对应的实操手册。
规范告诉你**要做什么**；实操手册告诉你**具体怎么做**。

ECC 源目录提供 18 个规则集：common + angular, arkts, cpp, csharp, dart, fsharp, golang, java, kotlin, perl, php, python, ruby, rust, swift, typescript, web。每种语言含 5 个维度：
- `coding-style` — 编码风格（不可变性、文件组织、大小限制）
- `security` — 安全检查（禁止硬编码密钥、验证输入）
- `testing` — 测试规范（覆盖率、测试模式）
- `patterns` — 语言特定模式
- `hooks` — Hook 文档

← [[sources/claude-code/2026-05-13-safety-net-hook-fix.md#ECC 规则安装|ECC 规则安装实践]]

← [[sources/claude-code/ecc.md|ECC 完整参考]] §入门-4

---

## 3. Strategic Compacting — 战略性上下文压缩

### 配置

ECC 的 PreCompact hook + `suggest-compact` 脚本自动介入，在编辑/写入操作达到阈值时提醒压缩。

### 行为规则

**核心原则**：在逻辑节点手动 `/compact`，不让自动压缩在任务中途触发。

压缩时机：
1. **探索阶段完成 → 执行前**：已理解问题、确认方案，可以丢弃探索过程
2. **里程碑完成 → 下一个前**：一个完整功能或修复已完成，开始新任务前
3. **Hook 提醒时**：PreCompact hook 建议压缩时评估是否适合

**禁止压缩**的时机：
- 任务执行到一半，关键中间状态尚未落地
- 刚收到用户反馈，尚未消化

### 使用方式

```
/compact                          # 手动触发
/strategic-compact                # ECC 的结构化压缩命令
```

← [[sources/claude-code/ecc.md|ECC 完整参考]] §进阶-2

---

## 4. Continuous Learning — 持续学习

### 配置

ECC 的 Stop hook 包含 `evaluate-session` 脚本，会话结束时自动运行。同时提供 `/learn` 斜杠命令供手动触发。

### 行为规则

**自动模式**（Stop hook）：
- 会话结束时自动评估，提取值得保存的模式
- 保存到 `~/.claude/skills/learned/`

**主动模式**（手动 `/learn`）：
- 发现非平凡模式时应主动运行，不必等到会话结束
- 适合提取的模式类型：
  - 错误解决方案（怎么发现、怎么修的）
  - 调试技巧（特定工具的用法、踩过的坑）
  - 变通方案（绕过已知限制的方法）
  - 项目特定模式（代码库中的约定和偏好）

### 使用方式

```
/learn                            # 会话中途随时运行
# 会话结束时 Stop hook 自动触发 evaluate-session
```

← [[sources/claude-code/ecc.md|ECC 完整参考]] §进阶-5

---

## 5. Session Memory — 跨会话上下文

### 配置

ECC 的 SessionStart + PreCompact + Stop 三个 hook 协作，自动管理会话状态文件。

状态文件路径：`~/.claude/sessions/YYYY-MM-DD-topic.tmp`

### 行为规则

**保存时机**：
- 撞上下文限制前主动保存进度（不等到被迫压缩）
- 复杂任务的每个阶段完成后
- 一天结束时保存当日进度

**恢复时机**：
- 跨天继续工作时，提供上次会话文件路径
- 上下文被压缩后，需要恢复关键状态时

**状态文件应包含**：
- 哪些方法有效（附验证证据）
- 哪些方法尝试过但失败了
- 哪些还没尝试，还剩什么要做

### 使用方式

```
/save-session                     # 保存当前会话状态
/resume-session                   # 恢复最近的会话状态
# 或直接提供文件路径：~/.claude/sessions/2026-05-13-feature-xyz.tmp
```

← [[sources/claude-code/ecc.md|ECC 完整参考]] §进阶-1

---

## 6. Token 经济学 — 子代理模型选择

### 决策规则

| 场景 | 模型 | 原因 |
|------|------|------|
| 重复性任务、指令非常清晰、多代理 worker | **Haiku** | $0.25/$1.25 per M tokens，5x 便宜于 Opus |
| 90% 的编码任务 | **Sonnet**（默认） | $3/$15 per M tokens，性价比最优 |
| 首次尝试失败、跨 5+ 文件、架构决策、安全关键代码 | **Opus** | $5/$15 per M tokens，最强能力 |

**核心原则**：Haiku + Opus 组合最划算（5x 价差），Sonnet + Opus 只有 1.67x 价差不值得分层。

### 配置

在子代理定义中指定模型：

```yaml
---
name: quick-search
description: Fast file search
tools: Glob, Grep
model: haiku        # 指定模型
---
```

← [[sources/claude-code/ecc.md|ECC 完整参考]] §进阶-6

---

## 7. Sub-Agent 上下文传递

### 行为规则

**核心问题**：子代理只收到查询字面意思，缺乏背后的目的和推理，返回的摘要常遗漏关键细节。

**三步原则**：
1. **传目标，不只传查询**：告诉子代理"为什么需要这个信息"，帮它判断摘要中该包含什么
2. **评估返回值**：收到子代理结果后，检查是否包含决策所需的所有信息
3. **不充分则追问**：最多 3 轮迭代，防止死循环

```
主代理 → 传递 query + objective → 子代理
子代理 → 返回摘要 → 主代理
主代理 → 评估：充分？
  ├─ 是 → 接受结果
  └─ 否 → 追问（最多 3 轮）→ 子代理补充
```

← [[sources/claude-code/ecc.md|ECC 完整参考]] §进阶-7

---

## 8. Context Budget — 上下文窗口审计

### 配置

ECC 提供 `/context-budget` 斜杠命令，分析当前上下文使用情况并给出优化建议。

### 行为规则

- 上下文使用超过 **70%** 时主动运行 `/context-budget` 审计
- 审计结果中重点关注：
  - 哪些 MCP/插件占用了最多上下文
  - 是否有可以禁用的服务
  - 是否到了 Strategic Compact 的时机
- **经验阈值**：同时启用的 MCP < 10 个，活跃工具 < 80 个

### 使用方式

```
/context-budget                   # 审计当前上下文使用情况
```

← [[sources/claude-code/ecc.md|ECC 完整参考]] §入门-5

---

## 9. Agents — 全局代理注册

### 配置（手动安装）

插件机制无法自动分理 agents 文件到全局目录，必须手动复制。

```bash
cp -r ~/.claude/plugins/marketplaces/ecc/agents/*.md ~/.claude/agents/
```

### 说明

- ECC 提供 63 个语言/框架专用代理（python-reviewer、rust-reviewer、go-reviewer 等）
- 全局安装后，所有项目可通过 `Agent` 工具的 `subagent_type` 参数调用
- ECC 更新后需重新执行复制命令以同步最新代理定义

### 与 Rules 的区别

| 项目 | Rules | Agents |
|------|-------|--------|
| 全局安装 | `cp -r rules/common ~/.claude/rules/ecc/` | `cp -r agents/*.md ~/.claude/agents/` |
| 项目级安装 | 按技术栈选择性复制 | 不需要（agents 全局即可） |
| 加载方式 | CLAUDE.md 拼接或按需加载 | `Agent` 工具按名称调用 |

### 注意事项

- 新环境迁移时必须执行此步骤，否则 `Agent` 工具找不到对应代理
- ECC 更新后需重新执行复制命令以同步最新代理定义
- 已存在的同名文件会被覆盖，无需手动清理

---

## 10. CLAUDE.md 模板库

### 模板位置

ECC 插件提供 7 个 CLAUDE.md 模板，位于：

```
~/.claude/plugins/marketplaces/ecc/docs/zh-CN/examples/
```

### 使用层级说明

| 层级 | 是否使用 ECC 模板 | 说明 |
|------|-------------------|------|
| **用户级** `~/.claude/CLAUDE.md` | 否 | 使用 [[wiki/claude-md-base.md\|双层架构模板]] 中的 Karpathy 行为准则，不使用 ECC 的 user-CLAUDE.md |
| **项目级** `./CLAUDE.md` | 参考 | 从下方模板中选择最接近的作为起点，按需裁剪 |

### 项目级模板清单

| 模板文件 | 适用场景 | 技术栈 |
|----------|----------|--------|
| `CLAUDE.md` | 通用项目模板（基础） | 通用，可适配任何技术栈 |
| `django-api-CLAUDE.md` | Django REST API | Python 3.12+, Django 5.x, DRF, PostgreSQL, Celery |
| `go-microservice-CLAUDE.md` | Go 微服务 | Go 1.22+, PostgreSQL, gRPC, sqlc, Wire |
| `laravel-api-CLAUDE.md` | Laravel API | PHP 8.2+, Laravel 11.x, PostgreSQL, Redis |
| `rust-api-CLAUDE.md` | Rust API 服务 | Rust 1.78+, Axum, SQLx, PostgreSQL, Tokio |
| `saas-nextjs-CLAUDE.md` | Next.js SaaS 应用 | Next.js 15, Supabase, Stripe, Tailwind, Playwright |

> `user-CLAUDE.md` 是 ECC 提供的用户级模板示例，仅供参考，实际使用 [[wiki/claude-md-base.md#第一层：全局 CLAUDE.md|Karpathy 行为准则]] 作为用户级配置。

### 模板共性结构

所有项目级模板遵循统一结构：

1. **项目概述** — 技术栈、架构描述
2. **关键规则** — 语言约定、数据库、认证、错误处理、代码风格
3. **文件结构** — 目录树及各目录职责
4. **关键模式** — 常用代码模式（含示例代码）
5. **环境变量** — 必需和可选配置
6. **测试策略** — 测试命令和覆盖率要求
7. **ECC 工作流** — 可用的 ECC 斜杠命令
8. **Git 工作流** — 提交格式、CI/CD 流程

### 使用方式

1. **新建项目时**：选择最接近的模板作为起点，复制到项目根目录
2. **自定义**：根据实际技术栈和项目需求修改
3. **增量添加**：可从多个模板中提取特定模式组合

### 模板亮点

| 模板 | 特色模式 |
|------|----------|
| Django | 服务层模式、序列化器分离读写、Factory Boy 测试 |
| Go | 仓库接口模式、表格驱动测试、哨兵错误 |
| Laravel | Action 模式、Form Request + DTO、Policy 授权 |
| Rust | thiserror 错误枚举、IntoResponse 映射、sqlx 编译时检查 |
| Next.js SaaS | Server Action 模式、RLS 安全、Stripe Webhook 处理 |
| 通用基础 | Prompt Defense 防御基线、API 响应格式、错误处理 |

### 参考

模板位于 ECC 插件的 `docs/zh-CN/examples/` 目录，可直接查看完整内容：

```bash
# 查看所有模板
ls ~/.claude/plugins/marketplaces/ecc/docs/zh-CN/examples/

# 查看特定模板
cat ~/.claude/plugins/marketplaces/ecc/docs/zh-CN/examples/django-api-CLAUDE.md
```

---

## 11. 高价值隐藏组件

以下组件位于 ECC 插件目录中，直接影响日常使用效率。

### 11.1 项目技术栈自动检测配置

**文件**：`config/project-stack-mappings.json`

`/project-init` 命令读取此文件，自动检测项目技术栈并推荐对应的 skills、rules、commands。

**覆盖 18 种技术栈**：TypeScript、React、Next.js、Go、Python、Rust、Java、Spring Boot、Kotlin、Swift、Flutter、Laravel、Ruby、C#、C++、Perl、Django、Android、Docker。

**每种技术栈定义了**：
- 指示文件（用于自动检测）
- 推荐 rules
- 推荐 skills
- 构建/测试/lint/格式化命令
- 权限白名单/黑名单

### 11.2 Plan-PRD 模式

**文件**：`docs/PLAN-PRD-PATTERN.md`

结构化规划流程：

```
/plan-prd 写需求文档 → /plan 写实现计划 → TDD 实现 → /pr 提交
```

每个阶段产出 `.md` 文件，可跨会话恢复、可审计、可版本控制。

### 11.3 上下文模式文件

**文件**：`contexts/dev.md`、`contexts/research.md`、`contexts/review.md`

三种工作模式的行为规范：

| 模式 | 文件 | 行为 |
|------|------|------|
| 开发模式 | `dev.md` | 先写代码后解释 |
| 研究模式 | `research.md` | 先理解后行动 |
| 审查模式 | `review.md` | 按严重性分级 |

### 11.4 状态栏配置

**文件**：`examples/statusline.json`

显示信息布局：

```
模型名 | 当前任务 | $成本 工具数 文件数 时长 | 目录 | 上下文进度条
```

上下文进度条颜色阈值：
- <50%：绿色
- <65%：黄色
- <80%：橙色
- ≥80%：红色闪烁

---

## 12. 中价值架构参考

以下组件适用于架构决策和运维参考。

### 12.1 能力表面选择指南

**文件**：`docs/capability-surface-selection.md`

决定能力放哪里的决策树：

```
rule（确定性约束）→ skill（按需工作流）→ MCP（结构化交互）→ CLI（简单本地操作）→ API（窄远程调用）
```

### 12.2 故障排除指南

**文件**：`docs/TROUBLESHOOTING.md`

Claude Code 已知 bug 的社区解决方案：

| 问题 | 说明 |
|------|------|
| Hook Error 误标 | Hook 正常运行但显示红色错误 |
| 提前压缩 | 上下文未满时意外触发压缩 |
| MCP 认证失效 | MCP 服务器认证过期不提示 |
| Hook 不热重载 | 修改 hook 后需重启才生效 |
| 529 过载 | 服务器过载时的重试策略 |

### 12.3 命令-代理映射

**文件**：`docs/COMMAND-AGENT-MAP.md`

79 个斜杠命令与代理/skill 的对应关系：

| 命令 | 代理/Skill |
|------|-----------|
| `/plan` | planner |
| `/tdd` | tdd-guide |
| `/code-review` | code-reviewer |
| `/build-fix` | build-error-resolver |
| `/security-review` | security-reviewer |
| `/e2e` | e2e-runner |
| `/refactor` | refactor-cleaner |
| `/docs-update` | doc-updater |
