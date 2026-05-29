---
name: ECC Superpowers 补强配置指南
description: ECC 插件中纳入 Superpowers 补强的 8 项功能——配置方式、使用时机、行为规则，作为今后开发和 CLAUDE.md 编写的指导
type: concept
updated: 2026-05-14
tags: [ecc, superpowers, hooks, rules, context-management, session-memory, token-optimization, subagents]
sources: [[sources/claude-code/ecc.md]]
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

### 配置（手动安装）

插件机制无法自动分发 rules 文件，必须手动复制。

### 安装策略

**全局（~/.claude/rules/ecc/）**：只安装 `common` 规则集。Claude Code 会加载 `~/.claude/rules/` 下所有 `.md` 到每个会话 context，无按项目过滤，因此全局只保留通用规范。

**项目级（project/.claude/rules/ecc/）**：只安装项目实际技术栈。例如 Python 项目安装 common + python。

```bash
# 全局只装 common
cp -r ~/.claude/plugins/marketplaces/ecc/rules/common ~/.claude/rules/ecc/

# 项目级只装必要的
mkdir -p project/.claude/rules/ecc
cp -r ~/.claude/plugins/marketplaces/ecc/rules/common project/.claude/rules/ecc/
cp -r ~/.claude/plugins/marketplaces/ecc/rules/python project/.claude/rules/ecc/
```

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
