---
name: 观察空间与动作空间决定 Agent 能看什么、能做什么
description: 理解 Agent 如何通过观察和行动接口连接外部世界，以及为什么扩展接口能直接提升任务能力。
type: tech-concept
updated: 2026-09-07
tags: [agent, observation-space, action-space, harness, tools, context]
source: self
---

## 一句话理解

**观察空间决定 Agent 能看到什么，动作空间决定 Agent 能做什么。**

很多“Agent 不够聪明”的问题，实际不是模型问题，而是：

> **它没看到关键数据，或者根本没有对应工具可以执行。**

---

## 最核心的关系

```text
Environment
    ↓ 观察
Harness
    ↓ 转成 Context
Model
    ↓ 选择行动
Harness
    ↓ 转成 Tool Call
Environment
```

所以 Harness 做了两次“翻译”：

1. **环境 → 模型**：把观察整理成上下文；
2. **模型 → 环境**：把模型决定转换成工具调用。

---

## 观察空间：Agent 能看到什么

例如 Coding Agent 能看到：

* 用户需求；
* 代码文件；
* 目录结构；
* Git 状态；
* 编译错误；
* 测试结果。

如果某条信息没有进入 Context：

> **对模型来说，它基本等于不存在。**

例如：

```text
真实数据库里有订单状态
↓
但 Agent 没有查询数据库的工具
↓
Context 里没有订单状态
↓
模型只能猜
```

所以扩展观察空间，本质上就是：

> **让更多有用的环境信息进入上下文。**

---

## 动作空间：Agent 能做什么

动作空间就是 Harness 暴露给模型的能力。

例如：

```text
read_file
write_file
search_code
run_tests
search_web
click
send_email
```

模型可能知道“应该发邮件”，但如果没有邮件工具：

```text
知道怎么做
≠
能够真的做
```

它最多只能告诉用户：

> “你应该发送一封邮件。”

所以扩展动作空间，本质上就是：

> **把完成任务需要的操作封装成工具。**

---

## 为什么这比单纯换模型重要

底层模型固定时，Agent 工程常见的提升路径是：

```text
任务失败
↓
先检查：是不是看不到关键东西？
↓
扩大观察空间

或者

任务失败
↓
检查：是不是没有执行能力？
↓
扩大动作空间
```

例如：

```text
Agent 不会分析项目错误
```

不一定要换模型。

可能只是：

```text
之前只能看代码

加入：
- terminal 输出
- test result
- git diff

↓
Agent suddenly 能调试了
```

所以一个重要判断是：

> **先检查接口，再怀疑智力。**

---

# Manus：把原本分开的能力合并

早期 Agent 大致分成三类：

```text
Deep Research
Coding Agent
Computer Use
```

它们原本拥有不同的观察和动作能力。

Manus 的重要变化是把这些空间合并：

```text
浏览器
+
文件系统
+
代码执行
+
命令行
```

因此：

```text
观察空间扩大
+
动作空间扩大
↓
同一个 Agent 能完成更多类型任务
```

它的通用性不只是来自“模型更强”，而是：

> **把几类 Agent 的接口能力取了并集。**

---

# OpenClaw：继续扩大到用户数字生活

OpenClaw 进一步把接口连接到用户已有的数字环境。

观察入口包括：

* WhatsApp
* Telegram
* Slack
* Discord
* iMessage

数据来源包括：

* Google Drive
* Notion
* 本地文件系统

核心变化：

```text
原来：
Agent 主要活动在隔离沙箱

后来：
Agent 可以连接用户真实的数字资产
```

也就是：

> **观察空间从“Agent 自己的环境”，扩大到“用户已经在使用的数字世界”。**

Manus 后来也增加 Google Drive 和本地访问能力，这再次说明：

> 产品能力的升级，很多时候就是观察空间和动作空间的升级。

---

## 不同 Agent，本质上差在哪

| Agent         | 观察空间          | 动作空间         |
| ------------- | ------------- | ------------ |
| Coding Agent  | 代码、目录、终端、测试结果 | 搜索、编辑、执行命令   |
| Deep Research | 搜索结果、网页、论文    | 搜索、阅读、生成报告   |
| Computer Use  | 截图、DOM、界面状态   | 点击、输入、滚动     |
| 手机助手          | App 界面、系统反馈   | 点击、滑动、打开 App |
| 个人办事 Agent    | 账单、账户、服务商资料   | 打电话、发邮件、填表   |

它们底层结构其实非常像：

```text
看什么
+
能做什么
+
怎么决定下一步
```

即：

```text
Context
+
Tools
+
Model
```

---

## 开放式动作空间

现代 Agent 的一个重要特征是：

> **动作不是几个固定按钮，而是可以动态生成参数、文本、代码或操作序列。**

例如不是：

```text
A / B / C 三选一
```

而是：

```text
search_code("payment retry logic")
```

甚至：

```text
生成一段代码
→
执行
→
看结果
→
继续修改
```

因此动作空间虽然由工具限制，但工具内部可以非常开放。

---

## 最值得记住的判断框架

Agent 做不好任务时，先问三个问题：

```text
1. 它看到了需要的信息吗？
   → Observation / Context

2. 它拥有完成任务需要的能力吗？
   → Action / Tools

3. 它知道该怎么选择下一步吗？
   → Model / Policy
```

这三个问题分别对应：

```text
眼睛
手脚
大脑
```

最核心的一句：

> **Agent 的能力，不只取决于模型有多聪明，还取决于你给了它多大的“视野”和多大的“行动范围”。**
