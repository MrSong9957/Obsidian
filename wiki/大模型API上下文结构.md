---
name: Agent 调用大模型时的 API 上下文结构
description: 理解 Agent 如何把系统规则、对话历史、工具和工具结果一起发送给模型。
type: tech-concept
updated: 2026-09-12
tags: [ai-agent, context-engineering, llm-api, messages, tool-calling]
source: self
---

## 一句话理解

Agent 调用大模型时，本质上是在**把模型这一次需要看到的全部信息重新打包成一个 API 请求**。

模型本身不记得上一次调用，因此历史信息要由 Agent 再次传进去。

## API 里有什么

以 Chat Completions 风格为例，上下文主要是：

```text
messages
├─ system      开发者规则
├─ user        用户说了什么
├─ assistant   模型之前说了什么 / 发起过什么工具调用
└─ tool        工具执行后返回了什么

tools           模型可以使用哪些工具
```

### 四种消息角色

| role        | 大白话          | 作用            |
| ----------- | ------------ | ------------- |
| `system`    | Agent 的岗位说明书 | 身份、规则、限制      |
| `user`      | 用户当前说的话      | 告诉模型要做什么      |
| `assistant` | 模型以前说过的话     | 保留对话和工具调用历史   |
| `tool`      | 工具执行结果       | 告诉模型工具实际返回了什么 |

`tools` **不是第五种消息角色**，而是 API 请求中的独立字段，用来描述：

> 有哪些工具，以及每个工具怎么调用。

所以可以理解成：

```text
上下文
= 四种消息角色
+ tools 工具定义
```

这正好对应上下文的五部分：

```text
system      → 系统提示词
user        → 用户消息
assistant   → 模型回复
tool        → 工具执行结果
tools       → 工具定义
```

## 最简单的一次调用

```json
{
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful coding assistant."
    },
    {
      "role": "user",
      "content": "Hello, who are you?"
    }
  ]
}
```

模型返回：

```json
{
  "role": "assistant",
  "content": "Hi! I'm a coding assistant."
}
```

数据流就是：

```text
Agent 构造上下文
      ↓
发送 API 请求
      ↓
模型读取 messages
      ↓
生成 assistant 消息
      ↓
Agent 保存回复
```

## 最重要的原理：API 调用是无状态的

大模型 API 不会因为你刚刚调用过一次，就自动记住之前发生了什么。

例如第一轮：

```text
user: 我叫小明
assistant: 你好，小明
```

第二次调用如果只发送：

```text
user: 我叫什么？
```

模型并不知道答案。

Agent 必须重新发送历史：

```text
user: 我叫小明
assistant: 你好，小明
user: 我叫什么？
```

模型才能回答：

```text
小明
```

所以所谓“模型有对话记忆”，通常并不是模型真的保存了记忆，而是 **Agent/Harness 把历史消息重新放进下一次请求的上下文里**。

## 为什么这很重要

后面的上下文工程，本质上都在解决同一个问题：

> **下一次调用模型时，到底应该让模型看到什么？**

信息太少，模型不知道发生过什么；信息太多，又会浪费 Context Window、增加成本，还可能干扰模型判断。

因此 Harness 的一个核心职责就是：

```text
保存状态
→ 选择有用信息
→ 组织成 API 上下文
→ 调用模型
```

这就是后续历史压缩、工具结果管理、记忆、Context Engineering 等技术的基础。
