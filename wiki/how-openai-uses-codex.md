---
name: How OpenAI uses Codex
description: OpenAI 内部怎么把 Codex 用进日常工程 — 7 类用法 + 6 条最佳实践入口（细节见 sources 全量中译）
type: concept
updated: 2026-10-20
tags: [codex, openai, coding-agent, workflow, best-practices, agents-md]
source: "[[sources/agent/how-openai-uses-codex]]"
---

# How OpenAI uses Codex

## 核心思想

OpenAI 各技术团队把 Codex（编程 Agent）当成**日常工程加速器**，而不是只会补全的编辑器插件。手册按**用法类型**（不是按团队）组织 7 类场景；前提是给它结构化任务、可运行环境、持久上下文。[^1]

本页是入口；**正文说明、工程师原话、全部示例 prompt、Best Practices 完整段落**见 [[sources/agent/how-openai-uses-codex|详文中译]]（保原意精简翻译，不是摘要）。

## 怎么读详文

| 你想干什么 | 对应用例 |
|------------|----------|
| 上手陌生代码 / 值班分诊 | 代码理解 |
| 跨文件改 API / 换依赖 | 重构与迁移 |
| 找慢路径、DB 热点 | 性能优化 |
| 补边界测试、睡前丢任务 | 补测试 |
| scaffold / 发布冲刺 | 加快开发 |
| 日程被打碎、顺手修 | 保持心流 |
| 多方案、找同类 bug | 探索与构思 |

## 七类用法（导航，细节见详文）

1. **代码理解** — 定位逻辑、画依赖、追数据流；贴 stack trace 跳文件。[^2]
2. **重构与迁移** — 几十文件一致改；正则搞不定的结构感知修改。[^3]
3. **性能优化** — 慢路径/内存/昂贵查询；清危险旧模式。[^4]
4. **补测试** — 边界与失败路径；空输入、最大长度。[^5]
5. **加快交付** — 启动 scaffold；冲刺期分诊、最后一公里、telemetry。[^6]
6. **抗打断** — 半成品/Slack/值班丢进任务队列。[^7]
7. **探索构思** — 多解法暴露 tradeoff；已知 bug 找全库同类。[^8]

## 六条最佳实践（导航）

| 实践 | 一句话 |
|------|--------|
| 先 Ask 后 Code | 大改动先出计划再动手；粒度约「一小时 / 几百行」 |
| 迭代环境 | 启动脚本、环境变量、网络；盯构建错误 |
| Issue 式 prompt | 路径、组件名、diff、文档片段 |
| 任务队列当 backlog | 不必一次逼出完整 PR |
| AGENTS.md | 命名、业务逻辑、已知坑、推不出来的依赖 |
| Best-of-N | 多份结果挑最好，或拼装 |

完整解释见详文 Best Practices 节。[^9]

## 诚实限制

- 手册是访谈 + 内部数据整理，**不是受控对照实验**。
- 写明 Codex 仍在 **research preview**。[^10]
- 任务粒度约一小时/几百行是**当前建议**；模型变强后上限会上移。[^9]
- AGENTS.md 结构细节手册未展开，只指向 docs。

## 相关概念

- [[how-anthropic-teams-use-claude-code]] — 姊妹篇：Anthropic 按**团队**组织案例
- [[claude-code]] — Claude Code 配置与工作流手册
- [[AGENTS-md-base]] — AGENTS.md / CLAUDE.md 双层模板
- [[Harness-工程]] — 环境与反馈回路
- [[上下文工程]] — 给 Agent 喂什么、喂多少
- [[Agent核心循环]] — 编码 Agent 通用循环骨架

## 待解决的问题

无内部矛盾。手册未给出统一效率基线；工程师原话是主观评价。

## 引用索引

[^1]: [[sources/agent/how-openai-uses-codex]] L10-22 — 手册定位与开篇导语（原 PDF P3）
[^2]: [[sources/agent/how-openai-uses-codex]] L25-48 — 代码理解（原 PDF P4）
[^3]: [[sources/agent/how-openai-uses-codex]] L51-70 — 重构与迁移（原 PDF P5）
[^4]: [[sources/agent/how-openai-uses-codex]] L73-91 — 性能优化（原 PDF P6）
[^5]: [[sources/agent/how-openai-uses-codex]] L94-112 — 补测试（原 PDF P7）
[^6]: [[sources/agent/how-openai-uses-codex]] L115-135 — 加快交付（原 PDF P8）
[^7]: [[sources/agent/how-openai-uses-codex]] L138-159 — 保持心流（原 PDF P9）
[^8]: [[sources/agent/how-openai-uses-codex]] L162-180 — 探索构思（原 PDF P10）
[^9]: [[sources/agent/how-openai-uses-codex]] L183-234 — 六条 Best Practices 全文（原 PDF P11-12）
[^10]: [[sources/agent/how-openai-uses-codex]] L237-242 — Looking Ahead：research preview（原 PDF P12）
