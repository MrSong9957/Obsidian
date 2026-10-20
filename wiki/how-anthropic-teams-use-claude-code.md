---
name: How Anthropic teams use Claude Code
description: Anthropic 内部 10 团队案例 — 开发者与非技术岗怎么用 Claude Code，含完整细节中译入口
type: concept
updated: 2026-10-20
tags: [claude-code, anthropic, case-study, workflow, non-technical]
source: "[[sources/claude-code/how-anthropic-teams-use-claude-code]]"
---

# How Anthropic teams use Claude Code

## 核心思想

Anthropic 用 10 个内部团队的访谈案例说明：Claude Code **不只是工程师工具**。它在数据基建、产品、安全、推理、数据科学、API、增长营销、产品设计、RL、法务里都能干活；非技术岗甚至一人团队也能借它自动化流程、抹平技能差。[^1]

本页是入口；**逐团队细节、量化数字和 Tips 全文**见 [[sources/claude-code/how-anthropic-teams-use-claude-code|详文中译]]（不丢细节的精简翻译，不是摘要）。

## 怎么读详文

| 你想看什么 | 对应团队 |
|------------|----------|
| 工程日常、自循环、异步/同步分工 | 产品开发 |
| 事故排查、Terraform 审查、slash command | 安全工程 |
| 新人上手、ML 概念、跨语言写测试 | 推理 |
| 不会前端也能做分析仪表盘 | 数据科学与可视化 |
| 任务第一站、少切界面 | API |
| 一人非技术岗自动化广告/MCP | 增长营销 |
| 设计师直接改代码、贴 mockup 出原型 | 产品设计 |
| 频繁回滚、首次成功率约 1/3 | RL 工程 |
| 非开发者做工具、MCP 合规顾虑 | 法务 |
| 纯文本工作流、多实例并行 | 数据基础设施 |

## 跨团队共性（细节见详文）

1. **写好 Claude.md / 记忆文件** — 仓库说明书越清楚，例行任务越稳。[^2]
2. **让 Claude 自验** — build / test / lint 进循环，才能更久放手。[^3]
3. **分清异步 vs 同步** — 边缘/原型可 auto-accept；核心业务要盯。[^3]
4. **先规划再动手** — 法务/营销都在 Claude.ai 想清再进 Code。[^4]
5. **子代理拆复杂流** — 标题代理 / 描述代理这类分工，好调试质量更好。[^4]
6. **状态存好再放手** — checkpoint /「老虎机」策略：收结果或重开，别硬修。[^5]

## 诚实限制（手册原话里的）

- 手册是**访谈自述**，不是对照实验。
- RL 团队承认：小/中 PR **首次约 1/3 成**，其余要指导或人工。[^5]
- 法务指出：深度 MCP 集成有安全含义；偏保守的安全姿态会形成壁垒。[^4]
- 非开发者上手：仓库/权限初始化仍要工程师帮；配好后才是转型级。[^6]

## 相关概念

- [[claude-code]] — 配置与工作流手册（settings / CLAUDE.md / Hooks / MCP）
- [[AGENTS-md-base]] — AGENTS.md / CLAUDE.md 双层模板
- [[how-openai-uses-codex]] — 姊妹篇：OpenAI 内部怎么用 Codex
- [[Harness-工程]] — 环境与反馈回路
- [[上下文工程]] — 给 Agent 喂什么、喂多少

## 待解决的问题

无内部矛盾。手册未给出统一的对照基线；部分数字（如 10×、2–4×）来自单团队自述，外推需谨慎。

## 引用索引

[^1]: [[sources/claude-code/how-anthropic-teams-use-claude-code]] L17-21、L25-439 — 开篇导语与 10 团队案例全文（原 PDF P1-22）
[^2]: [[sources/claude-code/how-anthropic-teams-use-claude-code]] L25-64、L314-355、L358-397 — 数据基建 / 产品设计 / RL 的 Claude.md Tips（原 PDF P4、P18、P20）
[^3]: [[sources/claude-code/how-anthropic-teams-use-claude-code]] L68-108 — 产品开发：自给自足循环、任务分类（原 PDF P5-6）
[^4]: [[sources/claude-code/how-anthropic-teams-use-claude-code]] L275-310、L400-439 — 增长营销规划；法务两步法与合规（原 PDF P16、P22）
[^5]: [[sources/claude-code/how-anthropic-teams-use-claude-code]] L197-233、L358-397 — 老虎机策略；RL 首次约 1/3（原 PDF P12、P20）
[^6]: [[sources/claude-code/how-anthropic-teams-use-claude-code]] L314-355 — 产品设计：初始化仍需工程师（原 PDF P18）
