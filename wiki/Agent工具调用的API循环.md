---
name: Agent 工具调用的 API 循环
description: 理解模型如何决定调用工具，以及 Harness 如何执行工具并把结果送回模型。
type: tech-concept
updated: 2026-09-12
tags: [ai-agent, tool-calling, react, context-engineering, llm-api]
source: self
---

## 一句话理解

Agent 的核心循环就是：

> **模型决定做什么，Harness 真正去做，再把结果交回模型继续判断。**

```text
模型决策
  ↓
tool_calls
  ↓
Harness 执行工具
  ↓
tool 结果
  ↓
再次调用模型
  ↓
继续调用工具 / 最终回答
```

## 最小例子

用户问：

> 现在温哥华几点？天气怎么样？

第一次调用模型：

```text
messages:
  system
  user

tools:
  get_current_time
  get_weather
```

模型不直接回答，而是返回：

```text
assistant
└─ tool_calls
   ├─ get_current_time("America/Vancouver")
   └─ get_weather("Vancouver", "celsius")
```

这里最重要的是：

> **模型只决定“调用哪个工具、传什么参数”，并不真正执行工具。**

真正执行的是 Agent 的 Harness。

---

## 工具执行后发生什么

Harness 执行两个工具，得到：

```text
get_current_time → 当前时间
get_weather      → 当前天气
```

然后再次调用模型：

```text
system
user
assistant   ← 上一次模型发出的 tool_calls
tool        ← 时间结果
tool        ← 天气结果
```

模型看到结果后，生成最终回答：

```text
现在是……
天气是……
```

没有新的 `tool_calls`，循环结束。

---

## 为什么第二次还要把前面的消息全部带上

因为模型 API 是**无状态的**。

第二次调用时，模型不会自动知道：

* 用户之前问了什么；
* 自己刚才调用了什么工具；
* 哪些工具结果对应哪些调用。

所以 Harness 必须重新发送：

```text
system
↓
user
↓
assistant(tool_calls)
↓
tool result
↓
tool result
```

也就是：

> **对话状态存在 Harness 里，不存在模型 API 里。**

---

## `assistant` 的工具调用也必须保留

第一次模型返回：

```text
assistant:
  tool_calls:
    - id: call_abc123
      get_current_time(...)
```

第二次请求不能只发送工具结果，还要把这条 `assistant` 消息原样放回来。

原因很简单：

> 模型需要看到“这是我刚才请求执行的工具”。

否则工具结果就失去了上下文。

---

## `tool_call_id` 是干什么的

例如：

```text
assistant:
  call_abc123 → get_current_time
  call_def456 → get_weather

tool:
  tool_call_id: call_abc123 → 时间结果

tool:
  tool_call_id: call_def456 → 天气结果
```

`tool_call_id` 就像订单号：

> **把某个工具结果和之前那次工具调用一一对应起来。**

当同时调用多个工具时尤其重要。

---

## 并行还是串行，由依赖关系决定

这个例子中的：

```text
get_current_time(...)
get_weather(...)
```

互相不依赖，因此 Harness 可以：

```text
        ┌─ 查时间 ─┐
模型 ───┤          ├─ 汇总结果 → 模型
        └─ 查天气 ─┘
```

并行执行。

但如果第二个工具需要第一个工具的结果：

```text
工具 A
  ↓
得到参数
  ↓
工具 B
```

就不能同时调用。

此时通常要：

```text
模型 → tool A
     ↓
Harness 执行 A
     ↓
模型读取 A 的结果
     ↓
模型 → tool B
```

所以：

> **能不能并行，不看工具数量，而看它们之间有没有数据依赖。**

---

## `tools` 是谁准备的

一般情况下，工具的名称、描述和参数 Schema 是开发者提前注册好的，例如：

```text
get_weather
- city: string
- unit: celsius | fahrenheit
```

模型拿到这些工具说明后，自己判断：

```text
用户要什么
    ↓
需不需要工具
    ↓
调用哪个工具
    ↓
参数填什么
```

不是 Harness 先把用户问题拆成：

```text
查时间
查天气
```

然后临时生成两个工具定义。

**任务拆解和工具选择主要发生在模型这一侧。**

---

## Harness 和模型的职责边界

| 组件      | 负责什么                        |
| ------- | --------------------------- |
| Model   | 判断下一步做什么、选工具、生成参数、决定是否结束    |
| Harness | 保存上下文、执行工具、收集结果、再次调用模型、控制循环 |

可以记成：

> **Model 是决策者，Harness 是执行和运行环境。**

---

## 这就是 ReAct 在 API 层的样子

抽象的 ReAct：

```text
思考 → 行动 → 观察 → 再思考
```

落到 API 上就是：

```text
模型调用
↓
assistant.tool_calls
↓
Harness 执行
↓
tool messages
↓
再次调用模型
```

如此循环，直到模型不再返回 `tool_calls`，而是给出最终回复。

## 核心记忆

```text
用户请求
   ↓
Model：决定调用什么工具
   ↓
Harness：执行工具
   ↓
Harness：把结果写成 tool 消息
   ↓
Model：继续判断
   ↓
没有 tool_calls
   ↓
最终回答
```

真正理解 Agent 工具调用，抓住一句话就够：

> **模型不会自己执行工具；它只发出工具调用请求。Harness 才负责真正执行，并把结果送回模型。**
