---
name: Agent 核心循环：上下文组装与工具执行
description: 理解如何组装请求、提取模型消息、执行工具，并将结果交回模型。
type: tech-concept
updated: 2026-09-13
tags: [ai-agent, context-engineering, tool-calling, python]
source: self
---

模型需要外部信息时，框架负责执行工具、回传结果，再让模型继续判断。**模型决定调用什么，框架负责实际执行并维护消息历史。**

**1. 准备上下文**

请求包含两部分：

* `messages`：系统规则、用户问题，以及后续追加的模型回复和工具结果。
* `tools`：工具的名称、用途、参数格式，告诉模型有哪些工具可以调用。

```python
messages = [
    {
        "role": "system",
        "content": "你是一个有帮助的助手。需要实时信息时使用工具。",
    },
    {
        "role": "user",
        "content": "温哥华现在几点，天气怎样？",
    },
]
```

原文的 `tools` 包含两个工具：

| 工具                 | 用途     | 参数                                        |
| ------------------ | ------ | ----------------------------------------- |
| `get_current_time` | 查询当前时间 | `timezone`：时区，例如 `America/Vancouver`      |
| `get_weather`      | 查询当前天气 | `city`：城市；`unit`：`celsius` 或 `fahrenheit` |

工具定义只是说明书，执行逻辑由另外编写的 `execute_tool` 负责。

**2. 核心循环**

以下保留原文的真实 API 调用。前提：OpenAI SDK 客户端 `client` 已配置好对应模型服务，`tools` 和 `execute_tool` 已准备好。

```python
while True:
    # 1. 发送请求，得到完整响应
    response = client.chat.completions.create(
        model="Qwen3-0.6B",
        messages=messages,
        tools=tools,
    )

    # 2. 从完整响应中，取出第一条候选回复里的消息
    # API 设计上允许“一次给你多个答案”，所以统一放进 choices 列表里。即使实际上只给了一个，也仍然要用 [0] 把第一个取出来。
    assistant_message = response.choices[0].message

    # 3. 保存模型消息，包括文本或工具调用请求
    messages.append(assistant_message)

    # 4. 没有工具调用，就输出正文并结束
    if not assistant_message.tool_calls:
        print(assistant_message.content)
        break

    # 5. 取出工具名称和参数，交给执行函数
    for tool_call in assistant_message.tool_calls:
        result = execute_tool(
            tool_call.function.name,
            tool_call.function.arguments,
        )

        # 6. 把执行结果与对应调用配对，写入历史
        messages.append({
            "role": "tool",
            "tool_call_id": tool_call.id,
            "content": result,
        })

    # 下一轮重新发送 messages，模型才能看到工具结果
```

**3. 数据怎样取出来**

| 字段                             | 内容                |
| ------------------------------ | ----------------- |
| `response`                     | 完整响应，包含候选回复、用量等信息 |
| `response.choices`             | 候选回复列表；`[0]` 取第一条 |
| `.message`                     | 这条候选中的模型消息        |
| `.message.content`             | 模型回复的正文           |
| `.message.tool_calls`          | 模型要求执行的工具调用列表     |
| `tool_call.function.name`      | 要执行哪个工具           |
| `tool_call.function.arguments` | 工具参数，形式是 JSON 字符串 |
| `tool_call.id`                 | 本次调用的标识，用来匹配执行结果  |

**SDK 已把完整响应解析成 Python 对象**，所以可以用 `.choices`、`.message` 取字段；但工具的 `arguments` 仍是 JSON 字符串。

例如：

```python
# 模型给出的参数：字符串
arguments = '{"timezone": "America/Vancouver"}'

# 真实执行函数需要先解析成字典，才能取出参数
args = json.loads(arguments)  # 需要 import json
timezone = args["timezone"]
```

原文的 `execute_tool` 只返回固定数据，没有真正解析参数或查询时间、天气。

**4. 消息怎样增长**

以“温哥华现在几点、天气怎样”为例：

| 时刻           | 新增消息                 | 总条数 |
| ------------ | -------------------- | --- |
| 首次请求前        | 系统规则、用户问题            | 2   |
| 模型提出调用，框架执行后 | 一条包含两个调用的模型消息、两条工具结果 | 5   |
| 模型根据结果回答后    | 最终回复                 | 6   |

**先保存模型的调用请求，再保存工具结果。** 每条结果通过 `tool_call_id` 对应到一次调用，避免混淆。

工具结果写入列表后，模型并不会立刻看到；框架必须发起下一次请求，将更新后的 `messages` 重新交给模型。

**5. 容易搞错的地方**

* `for` 依次执行工具；同一轮提出多个调用，不等于并行执行。
* `while True` 需要最大轮数限制，防止模型反复调用工具。
* 没有工具调用只是示例的退出条件，不保证任务成功。
* 上下文不只有 `messages`，还包括单独传入的 `tools` 等信息。
