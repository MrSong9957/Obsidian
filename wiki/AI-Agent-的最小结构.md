```yaml
name: AI Agent 的最小结构：Model、Context、Tools 与 Environment
description: 理解现代 Agent 由什么组成，以及 Agent 与外部环境之间如何形成行动闭环。
type: tech-concept
updated: 2026-09-07
tags: [agent, llm, context, tools, harness, environment]
source: 用户提供原文
```

## 一句话理解

**Agent 就是：模型负责决定，上下文负责提供信息，工具负责行动；Environment 则是它真正要观察和改变的外部世界。**

最小公式：

```text
Agent = LLM + Context + Tools
```

更工程化地写：

```text
Agent = Model + Harness
```

其中最小 Harness 就是：

```text
Harness = Context + Tools
```

生产环境还会继续加入权限、验证、纠错等能力。

---

## Agent 的三个核心组件

| 直觉 | 组件          | 负责什么                |
| -- | ----------- | ------------------- |
| 大脑 | LLM / Model | 理解任务、推理、决定下一步       |
| 眼睛 | Context     | 把当前观察、历史、记忆、状态组织给模型 |
| 手脚 | Tools       | 让 Agent 能读取或改变外部世界  |

所以可以直接记成：

```text
Agent = 大脑 + 眼睛 + 手脚
```

### Model：决定“下一步做什么”

Model 不只是模型参数。

它的能力来自两部分：

* 预训练获得的知识和语言能力；
* 后训练形成的决策策略。

在 Agent 中，它主要负责：

```text
看到当前信息
↓
判断现在是什么情况
↓
选择下一步行动
```

---

### Context：Agent 当前能看到什么

Context 不只是用户刚输入的一句话，还包括：

* 环境观察；
* 历史对话；
* 用户记忆；
* 外部知识；
* 当前任务状态；
* 已完成和未完成的步骤。

核心原则：

> **Agent 只能根据当前上下文里存在的信息做决定。**

---

### Tools：Agent 能对世界做什么

工具不仅是一个函数，还包括：

```text
工具定义
+
调用协议
+
真正连接环境的适配器
```

例如：

* 搜索网页；
* 读写文件；
* 调用 API；
* 执行代码；
* 操作浏览器；
* 调用其他 Agent；
* 向用户提问。

工具既可以用来**观察环境**，也可以用来**改变环境**。

---

## Agent 和 Environment 必须分开

这是这一节最重要的边界。

```text
Agent
│
├─ Model
└─ Harness
     ├─ Context
     ├─ Tools
     ├─ Loop
     ├─ State Management
     ├─ Permission
     ├─ Verify
     └─ Correct

        ↕ 观察 / 行动

Environment
├─ 文件系统
├─ 数据库
├─ 网页
├─ API / 应用
├─ 用户
├─ 其他 Agent
└─ 物理或仿真世界
```

**Environment 不是 Agent 的组成部分。**

例如文件系统：

```text
文件读取工具
→ Harness

文件真正的内容和状态
→ Environment
```

即使 Harness 创建了一个沙箱：

```text
创建沙箱、限制权限、重置沙箱
→ Harness

沙箱里的文件、进程、当前状态
→ Environment
```

物理上运行在哪里，不决定概念边界。

---

## Agent 为什么能连续完成任务

因为 Agent 和 Environment 会形成闭环：

```text
Environment
产生观察
   ↓
Agent
根据 Context 做决定
   ↓
Tools
执行行动
   ↓
Environment
状态发生变化
   ↓
产生新的观察
   ↓
Agent 再决定
```

这就是最基础的 Agent 运行模式。

大白话就是：

> **看情况 → 做决定 → 动手 → 看结果 → 再决定。**

---

## Model 和 Harness 怎么分工

### Model

负责：

> **该做什么？**

例如：

```text
“测试失败了，我应该先查看报错。”
```

### Harness

负责：

> **怎么让这次决定安全、可靠地执行？**

例如：

```text
构造模型上下文
↓
把可用工具告诉模型
↓
接收工具调用
↓
检查权限和参数
↓
执行
↓
收集结果
↓
再传回模型
```

所以：

> **Model 是决策核心，Harness 是围绕 Model 的运行与治理层。**

---

## 与强化学习概念的对应

可以粗略这样理解：

| Agent 工程    | RL 概念                          | 大白话             |
| ----------- | ------------------------------ | --------------- |
| Model       | Policy                         | 根据当前情况决定下一步     |
| Context     | Observation + History 的内部表示    | Agent 当前掌握的信息   |
| Tools       | Observation / Action Interface | Agent 能看什么、能做什么 |
| Environment | Environment                    | 真正保存状态并发生变化的世界  |

但不要机械地一一对应。

尤其要注意：

> **Context 不是整个 Observation Space，Tools 也不是 Environment。**

工具只是 Agent 与环境之间的接口。

---

## 最重要的结构

以后看到任何 Agent，都可以先用这张图判断：

```text
               Agent
        ┌─────────────────┐
        │ Model           │
        │ 决定下一步       │
        │        ↕        │
        │ Harness         │
        │ Context + Tools │
        └────────┬────────┘
                 │
          观察 ↕ 行动
                 │
        ┌────────▼────────┐
        │ Environment     │
        │ 保存真实状态     │
        └─────────────────┘
```

如果只记一句：

> **Model 决策，Harness 帮它运行，Environment 承载真实世界。**
