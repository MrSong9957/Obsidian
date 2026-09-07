---
name: 从 Prompt 到 Graph：Agent 工程关注范围如何逐层扩大
description: 理解 AI 工程如何从优化提示词，演进到管理 Agent 循环、长期自治和执行图。
type: insight
updated: 2026-09-07
tags: [agent, prompt-engineering, context-engineering, harness, loop-engineering, graph-engineering]
source: self
---

## 一句话理解

AI 工程的演进，本质上是在不断扩大一个问题的范围：

> **不再只问“模型怎么回答得更好”，而是逐步问“整个 Agent 系统怎么更可靠地运行”。**

## 五层演进

| 阶段                  | 主要解决什么                      |
| ------------------- | --------------------------- |
| Prompt Engineering  | 怎么把指令写清楚                    |
| Context Engineering | 怎么让模型看到正确、足够的信息             |
| Harness Engineering | 怎么让 Agent 安全、可靠地运行和使用工具     |
| Loop Engineering    | 怎么让 Agent 跨多轮持续找到下一步并真正完成任务 |
| Graph Engineering   | 怎么把 Agent、程序和人工审批组织成完整工作流   |

最简单的理解：

```text
Prompt：说什么
↓
Context：让模型看到什么
↓
Harness：怎么运行这一轮 Agent
↓
Loop：怎么持续跑很多轮
↓
Graph：怎么组织多个 Loop、程序和人工节点
```

## 它们不是替代，而是包含关系

```text
Graph Engineering
└─ Loop Engineering
   └─ Harness Engineering
      └─ Context Engineering
         └─ Prompt Engineering
```

也就是说：

* 提示词仍然重要；
* 只是它现在只是整个系统的一小部分。

单个 Agent 的 ReAct 循环，也可以只是一个更大执行图里的节点。

---

## 每一层到底多管了什么

### Prompt → Context

最开始只优化：

> “这句话怎么写，模型回答更好？”

后来发现真正影响模型的是整个上下文：

* System Prompt
* 工具定义
* 历史消息
* RAG 知识
* 当前状态

所以关注点从**一句提示词**扩大到**模型全部能看到的信息**。

### Context → Harness

再往后发现：

> 信息给对了，也不代表 Agent 会可靠执行。

于是开始处理：

* 工具怎么调用；
* 权限怎么限制；
* 怎么验证结果；
* 失败怎么重试；
* 怎么回滚；
* 什么时候熔断。

这就是 Harness 工程。

### Harness → Loop

Harness 主要关注一次 Agent 循环怎么可靠运行。

Loop Engineering 进一步关注：

> **这一轮结束后，下一件事谁来发现？什么时候继续？什么时候算真正完成？**

例如：

```text
执行任务
↓
检查结果
↓
发现还有缺口
↓
自动生成下一步任务
↓
继续执行
↓
直到满足完成条件
```

重点从“单轮可靠”变成“持续自治”。

### Loop → Graph

Graph Engineering 再往上一层：

> 不只是一个 Agent 自己循环，而是把多种执行单元组合起来。

节点可以是：

* Agent；
* 普通代码；
* 数据处理任务；
* 人工审批；
* 验证器。

边负责：

* 下一步去哪；
* 什么条件才能走；
* 数据怎么传；
* 哪里需要持久化。

所以 Graph 更像整个系统的**执行地图**。

---

## 为什么这条演进很重要

随着模型能力越来越接近，单纯换更强模型带来的优势会越来越有限。

真正拉开差距的是：

```text
同一个模型
+
更好的 Harness
+
更好的 Loop
+
更好的验证和恢复
=
明显更强的 Agent
```

原文给出的 LangChain Terminal Bench 2.0 案例就是这个意思：

```text
52.8%
→ 66.5%
```

模型没有换，主要改了 Harness，例如：

* 自动检查执行结果；
* 检测重复循环；
* 优化思考策略。

排名从 30 名开外提升到前 5。

## 最重要的认知变化

过去：

> **模型能力决定产品能力。**

现在越来越变成：

> **模型决定上限，模型之外的工程决定你能不能稳定达到这个上限。**

所以 Agent 工程的竞争重点，正在从：

```text
Prompt 写得好不好
```

逐渐转向：

```text
整个系统是否能
正确感知
→ 安全行动
→ 自动验证
→ 失败恢复
→ 持续推进
→ 正确结束
```

这就是从 Prompt Engineering 走向 Graph Engineering 的真正含义。
