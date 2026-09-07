```yaml
name: ReAct 循环：Agent 如何通过轨迹持续思考和行动
description: 理解 Agent 如何把模型、上下文和工具串成“想→做→看”的执行闭环。
type: hybrid
updated: 2026-09-06
tags: [agent, react, trajectory, harness, tool-calling, context]
source: 用户提供原文
```

## 一句话理解

**ReAct 就是让 Agent 反复执行“想一下 → 调工具 → 看结果 → 再想”，直到任务完成。**

它解决的问题是：

> LLM 本身只会生成下一步，怎么让它真正连续做事？

答案就是用 Harness 把模型、工具和环境串成循环。

## 最小运行骨架

```python
trajectory = [user_request]

repeat:
    context = stable_prefix + trajectory
    decision = Model(context)
    trajectory.append(decision)

    if decision has no tool call:
        return decision.answer

    for call in decision.tool_calls:
        call = Harness.validate(call)
        observation = Environment.execute(call)
        trajectory.append(observation)
```

整个机制只有三步：

```text
Reasoning
想：下一步该干什么？
    ↓
Acting
做：调用工具
    ↓
Observation
看：工具实际返回了什么
    ↓
继续下一轮
```

所以虽然叫 **ReAct = Reasoning + Acting**，实际理解时最好记成：

> **想 → 做 → 看 → 想 → 做 → 看**

---

## Trajectory：Agent 的执行轨迹

**Trajectory（轨迹）就是 Agent 做任务过程中不断积累的历史记录。**

里面主要有：

```text
用户消息
↓
模型决定 / reasoning / tool_calls
↓
工具执行结果
↓
模型下一次决定
↓
...
```

因此：

```text
Agent 每次调用 LLM 的上下文
=
静态前缀
+
轨迹
```

其中：

```text
静态前缀
├─ System Prompt
└─ Tool Definitions

轨迹
├─ User Messages
├─ Assistant Messages
│  ├─ reasoning
│  └─ tool_calls / content
└─ Tool Results
```

静态前缀通常不写进 trajectory，而是在**每次调用模型时重新拼到前面**。

---

## Model、Harness、Environment 分工

这是理解 ReAct 最重要的一层：

| 部分          | 负责什么              |
| ----------- | ----------------- |
| Model       | 决定下一步做什么          |
| Harness     | 组装上下文、校验工具调用、维护循环 |
| Environment | 真正执行操作并产生结果       |

例如：

```text
Model
“我要查欧元汇率”
      ↓
Harness
检查参数是否合法
      ↓
Environment
真正查询汇率
      ↓
Tool Result
“1 EUR = ... USD”
      ↓
Model
根据结果继续计算
```

**模型负责决策，不负责让现实世界真的发生变化。**

---

## 为什么轨迹很重要

因为每一轮的新结果都会追加进去：

```text
第 1 轮：查汇率
↓
得到汇率

第 2 轮：
模型能看到第 1 轮结果
↓
开始计算

第 3 轮：
模型能看到前面的查询和计算
↓
输出答案
```

没有 trajectory，Agent 很容易不知道：

* 自己已经做过什么；
* 哪些方案失败过；
* 工具刚刚返回了什么；
* 任务现在进行到哪一步。

所以可以把 trajectory 理解成：

> **Agent 一次任务中的工作记忆 + 操作日志。**

它同时也很适合调试、分析和训练。

---

## 一个重要误区：原生 Agent ≠ 模型里面有工具

现代模型可以通过强化学习学会：

```text
什么时候调用工具
调用哪个工具
传什么参数
看到结果后下一步做什么
```

但真正的：

```text
web_search
code_runner
文件系统
浏览器
代码沙盒
```

仍然存在于模型外部。

也就是：

> **RL 内化的是“工具使用策略”，不是工具执行能力。**

例如 Kimi K3 即使非常擅长连续调用几百次工具，真正执行搜索、代码的仍然是外部服务。

原文特别提到 GitHub Issue #30 对这个区别进行了纠正。

---

## “模型即 Agent”真正改变了什么

传统 Agent：

```text
客户端自己写 Harness

while:
    调模型
    检查 tool_calls
    执行工具
    把结果传回模型
```

现在一些 API：

```text
客户端
↓
服务端 Harness
├─ LLM
├─ web_search
├─ code_interpreter
└─ ReAct 循环
↓
最终结果
```

也就是说：

> **ReAct 并没有消失，只是 Harness 从客户端搬到了服务端。**

模型越来越负责“怎么决策”，平台越来越负责“怎么执行”。

---

## Kimi K3 与 GPT-5.6 两个案例

### Kimi K3

原文用它说明：

**强化学习可以把长链工具调用策略训练进模型。**

特点包括：

* 模型自己判断何时搜索；
* 根据工具结果动态调整下一步；
* 面向长周期 Agent 工作负载；
* 原文称其可以稳定维持约 200～300 次连续工具调用。

重点不是参数量，而是：

> **模型开始越来越擅长自己掌控 ReAct 的决策过程。**

### GPT-5.6

原文重点是两件事。

第一，**服务端托管 Agent 循环**：

```text
搜索
↓
阅读
↓
代码分析
↓
发现信息不足
↓
再次搜索
```

客户端不必自己实现完整 Deep Research Harness。

第二，**Freeform Tool Calling**：

传统：

```json
{
  "code": "print(...)"
}
```

自由格式工具可以直接传：

```python
print(...)
```

它只是改变了**工具参数的表达方式**，没有改变 ReAct 的基本机制。

---

## 最值得记住的关系

```text
Agent
│
├─ Model：决定下一步
│
├─ Harness：维护循环
│
└─ Environment：真正执行
        │
        ↓
      Result
        │
        ↓
trajectory 持续增长
        │
        ↓
下一次 Model 决策
```

所以 ReAct 的本质不是某段神奇算法，而是：

> **让模型的每一次决定，都建立在前一次真实行动结果之上。**

这才让一个“一次只生成一个回复”的 LLM，变成能够连续完成复杂任务的 Agent。
