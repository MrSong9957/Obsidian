---
name: ce + last30days 技能组合
description: Compound Engineering 与 last30days 的安装、常用技能和日常流程 — 调研、规划、实现、调试、审查与经验沉淀
type: concept
updated: 2026-10-30
tags: [agent-skills, compound-engineering, last30days, installation, workflow]
source: https://github.com/EveryInc/compound-engineering-plugin
---

# ce + last30days 技能组合

## 核心思想

用 `last30days` 调研近期社区讨论，用 Compound Engineering（简称 ce）完成需求澄清、规划、实现、调试、审查和经验沉淀。[^ce][^last30days]

## 详细说明

### 一、安装（只做一次）

以下以 Claude Code 为例；`last30days` 需要 Python 3.12+。其他 Agent 按两个官方仓库的说明确认支持情况、安装方式和调用语法，不直接照抄 Claude Code 的插件命令。[^ce][^last30days][^last30days-runtime]

打开 Claude Code，逐行执行：

```text
/plugin marketplace add EveryInc/compound-engineering-plugin
/plugin install compound-engineering
/plugin marketplace add mvanhorn/last30days-skill
/plugin install last30days
```

安装后，在每个使用 ce 的项目运行一次 `/ce-setup`，按输出检查项目配置。[^ce]

仓库地址：

- 计划插件：[Compound Engineering](https://github.com/EveryInc/compound-engineering-plugin)
- 调研技能：[last30days](https://github.com/mvanhorn/last30days-skill)

本文统一使用当前官方的连字符命令，如 `/ce-plan`、`/ce-work`。[^ce]

**安装后验证**：

1. 新建会话，确认 ce 和 `last30days` 已被当前 Agent 识别。
2. 在目标项目运行 `/ce-setup`，检查配置结果；其他 Agent 使用其支持的技能调用方式。
3. 用一个明确的话题调用 `/last30days 话题或问题`，检查实际可用来源、报告及引用；首次运行按提示完成配置，缺少来源或运行条件时记录原因。[^last30days][^last30days-runtime]

### 二、8 个常用技能速查

#### 1. /last30days —— 先看大家怎么说

- **干啥**：调研过去 30 天的社区讨论（Reddit、HN、YouTube 等），生成一份简报。[^last30days]
- **敲**：`/last30days 话题或问题`
- **啥时候用**：动手前跑。技术选型、买大件、写方案、做调研都行。
- **备注**：Reddit、HN、Polymarket、GitHub 可先用；实际来源取决于网络、凭证和工具配置。[^last30days]

#### 2. /ce-brainstorm —— 模糊想法变清楚需求

- **干啥**：通过逐轮提问，把想法整理成仅含需求的计划文档。[^ce]
- **敲**：`/ce-brainstorm 想做的东西`
- **啥时候用**：脑子里只有个念头、说不清要啥的时候。想清楚了就跳过它。

#### 3. /ce-plan —— 需求变执行计划

- **干啥**：读代码库、翻旧笔记，按需查外部资料，把需求补充成可执行的计划。[^ce-plan]
- **敲**：`/ce-plan 功能描述`
- **产出**：范围、方案、涉及文件和验证要求；需要保存完整计划时，默认放在 `docs/plans/`。[^ce-plan]
- **用法要点**：拿到计划先自己读一遍，不满意就说“第 X 步改成 XX”，改到满意再执行。

#### 4. /ce-work —— 计划变代码

- **干啥**：照计划写代码、跑测试并进入提交 / PR 流程；计划正文保持不变，进度记录在任务列表和 Git 中。[^ce-work]
- **敲**：`/ce-work docs/plans/xxx.md`
- **断线续跑**：开新会话，再运行 `/ce-work docs/plans/xxx.md`。[^ce-work]
- **更换实现工具**：在 `/ce-work` 中明确指定可用的实现工具，宿主仍负责验证和提交。[^ce-work]

#### 5. /ce-debug —— 修 bug 专用

- **干啥**：复现 → 验证猜测 → 定位根因 → 先写测试再修。也可只诊断，不实施修复。[^ce-debug]
- **敲**：`/ce-debug 现象描述`，比如 `/ce-debug 支付回调有时候会创建重复的发票`
- **啥时候用**：遇到失败测试、报错、行为回归或可复现的性能问题时。[^ce-debug]

#### 6. /ce-simplify-code —— 审查前先瘦身

- **干啥**：对当前改动并行跑复用、质量、效率、测试价值、注释等专项审查，自动简化代码并清理无价值的测试和注释，行为保持不变，改完跑类型检查、lint 和测试。[^ce-simplify]
- **敲**：`/ce-simplify-code`，默认取当前分支改动；也可指定文件或描述，如 `/ce-simplify-code src/billing`
- **啥时候用**：实现或修复告一段落、进入审查之前；`/ce-work`、`/ce-debug` 也可能在完成节点自动调用它。[^ce-simplify]
- **备注**：格式化、依赖升级这类机械改动或只有几行的 diff 不用跑；它只做瘦身，不做深度审查——那是 `/ce-code-review` 的事。[^ce-simplify]

#### 7. /ce-code-review —— 提交前把关

- **干啥**：按改动选择不同审查角色，合并检查结果；默认只出报告，不改代码。[^ce-review]
- **敲**：`/ce-code-review`
- **啥时候用**：实现或 bug 修复完成后，检查最终改动；若当前流程已完成同范围审查，不重复运行。

#### 8. /ce-compound —— 记住这次的坑

- **干啥**：把这次的症状、根因、失败尝试和最终方案写进 `docs/solutions/`。[^ce]
- **敲**：`/ce-compound`
- **为啥重要**：后续 brainstorm 和 plan 会读取这些经验，减少重复踩坑。[^ce]

项目已规定经验保存路径时，先按项目规则对齐，避免同一结论重复维护。

### 三、三条日常流水线

```text
想法模糊：/ce-brainstorm → /ce-plan → /ce-work → /ce-simplify-code → /ce-code-review → /ce-compound
想法清楚：/ce-plan → /ce-work → /ce-simplify-code → /ce-code-review → /ce-compound
修 bug：/ce-debug → /ce-simplify-code → /ce-code-review → /ce-compound
```

日常约定：按需先用 `/last30days` 调研，再进入对应流程；实现或修复后完成审查，有可复用经验时再沉淀。项目要求的测试、安全检查和全量验证仍按项目规则执行。

### 四、偶尔用（遇到再敲）

以下技能按需使用。[^ce]

| 命令 | 啥时候用 |
|---|---|
| `/ce-explain` | 看不懂一段代码，让它写篇讲解文档存着 |
| `/ce-pov` | 拿不定主意做不做某功能，让它给“采用 / 搁置 / 拒绝”的裁决 |
| `/ce-prototype` | 想快速看效果，做个用完就扔的原型 |
| `/ce-commit-push-pr` | 需要规范提交、推送和创建 PR 时 |
| `/ce-doc-review` | 审查需求或计划文档，检查逻辑漏洞 |

`/ce-strategy`、`/ce-product-pulse` 偏产品管理，熟悉基本流程后再按需使用。

### 五、五条注意事项

1. 命令统一用连字符：`/ce-plan`、`/ce-work`、`/ce-compound`。[^ce]
2. 保留权限确认，按具体操作决定是否授权。
3. 计划是会话恢复的依据，保留 `docs/plans/` 中对应文件。[^ce-work]
4. `/last30days` 需要能访问对应来源；部分来源还需要凭证或额外工具。[^last30days]
5. 从上面两个官方仓库安装，并以实际安装版本的说明为准。

## 相关概念

- [[wiki/Agent工具初始化配置|Agent 工具初始化配置]] — 第 3 步使用本页安装技能
- [[wiki/AGENTS-md-base|AGENTS.md 双层架构模板]] — 约定项目的调试、审查和验证流程

## 引用索引

[^ce]: https://github.com/EveryInc/compound-engineering-plugin — 官方安装、项目设置、技能目录与工作流说明。
[^last30days]: https://github.com/mvanhorn/last30days-skill — 官方安装、调用方式与来源配置说明。
[^last30days-runtime]: https://github.com/mvanhorn/last30days-skill/blob/main/skills/last30days/SKILL.md — Python 3.12+ 运行条件、首次配置与实际来源检查要求。
[^ce-plan]: https://github.com/EveryInc/compound-engineering-plugin/blob/main/docs/guides/ce-plan.md — 计划内容、输出方式与默认保存目录。
[^ce-work]: https://github.com/EveryInc/compound-engineering-plugin/blob/main/docs/guides/ce-work.md — 执行计划、进度记录、恢复与更换实现工具。
[^ce-debug]: https://github.com/EveryInc/compound-engineering-plugin/blob/main/docs/guides/ce-debug.md — 根因诊断、测试先行修复及只诊断模式。
[^ce-simplify]: https://github.com/EveryInc/compound-engineering-plugin/blob/main/docs/guides/ce-simplify-code.md — 行为保持的代码简化与测试/注释清理，实现落定后、审查提交前运行。
[^ce-review]: https://github.com/EveryInc/compound-engineering-plugin/blob/main/docs/guides/ce-code-review.md — 按改动选择审查角色，默认仅报告。
