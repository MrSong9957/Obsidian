---
name: Agent 架构核心
description: Agent 循环、多智能体框架、Harness 工程方法论的核心概念与关键要点
type: reference
updated: 2026-06-17
tags: agent, architecture, multi-agent, harness, system-prompts
source: https://openai.com/index/harness-engineering/
---

# Agent 架构核心

> 详细参考：[[sources/agent/agent-arch-references.md|Agent 架构详细参考]]

## Agent Loop：循环就是一切

Agent 的核心是一个 while 循环：调用模型 → 判断是否需要工具 → 执行工具 → 结果喂回 → 继续。

```python
while True:
    response = client.messages.create(messages=messages, tools=TOOLS)
    if response.stop_reason != "tool_use":
        return
    # 执行工具，结果喂回 messages
```

生产版通过 System Prompts 中约 15 个片段定义循环内的行为约束。

## 多智能体架构

四个角色的迭代流程：
- **Planner**：拆解任务，制定计划
- **Implementer**：执行编码
- **Reviewer**：代码审查
- **Verifier**：验证结果

核心原则：动态发现（运行时决定调用哪个 agent），非静态编排。

## Harness Engineering

OpenAI 团队的实验：用 0 行人工编写代码构建和发布软件产品。

关键结论：
- Agent-first 开发需要新的软件工程方法论
- 人话编程（Natural Language Programming）：用自然语言描述意图，Agent 执行
- 工程师角色从"写代码"转向"编排 Agent"

## 人话编程规范

用自然语言编写可执行的规范，Agent 将其翻译为代码：
- 基本规则：声明式描述，不指定实现细节
- 基础语法：意图、约束、验收标准
- 高级语法：条件分支、循环、异常处理

## 关键数据

- s_full.py：~600 行 Python 实现完整 Agent Loop
- System Prompts：~15 个片段定义行为约束
- 实验周期：5 个月，0 行人工代码
