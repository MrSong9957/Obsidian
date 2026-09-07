```yaml
name: Harness 工程：让 Agent 从“能做”变成“可靠地做”
description: 理解 Harness 如何通过上下文、工具、约束、验证和纠正，让 Agent 在生产环境中可靠运行。
type: tech-concept
updated: 2026-09-07
tags: [agent, harness, context, tools, reliability, verification]
source: 用户提供原文
```

## 一句话理解

**Harness 是包在模型外面的“运行与治理层”——负责让模型安全、可靠地使用工具完成任务。**

最小 Demo 只要求 Agent **能做事**；生产系统还必须保证它：

* 不乱做；
* 做完能检查；
* 做错能恢复。

---

## Agent、Harness、Environment 的关系

```text
Agent = Model + Harness

Harness =
上下文管理
+ 工具接口
+ 约束
+ 验证
+ 纠正

Agent ↔ Environment
```

三者分工：

| 部分          | 负责什么             |
| ----------- | ---------------- |
| Model       | 判断下一步该做什么        |
| Harness     | 管理模型怎么运行、怎么和环境交互 |
| Environment | 真正保存和改变外部状态      |

关键边界：

> **Harness 在 Agent 内部，Environment 在 Agent 外部。**

例如代码沙箱：

```text
Harness
├─ 沙箱权限
├─ 工具适配器
└─ 重置机制

Environment
├─ 沙箱里的文件
├─ 正在运行的进程
└─ 文件修改后的真实状态
```

所以 Harness 不是“模型之外的一切”。

---

## Harness 的 5 个核心功能

### 1. Context：让模型看见足够的信息

解决：

> 模型做决定时到底知道什么？

包括：

* System Prompt
* 历史消息
* 知识库
* Agent 当前状态
* 动态检索结果

原则：

> **信息不够，模型再聪明也只能猜。**

---

### 2. Tools：让模型能观察和行动

解决：

> 模型怎么真正做事？

例如：

* 搜索
* 文件读写
* Shell
* MCP
* Code Interpreter

好的工具接口应该：

* 名字直观；
* 参数清楚；
* 给出示例；
* 明确边界。

---

### 3. Constrain：限制它不能乱做

解决：

> 模型想做危险操作怎么办？

核心原则：

**默认拒绝，明确授权后才开放。**

例如：

```text
读文件 → 可以直接执行
删除文件 → 需要确认
转账 10 万元 → 更严格权限
```

类似手机 App 权限。

---

### 4. Verify：检查它到底做对没有

解决：

> 模型说“完成了”，真的完成了吗？

例如：

```text
修改代码
↓
运行 tests
↓
运行 linter
↓
检查数据库状态
↓
确认结果
```

一个重要原则：

> **安全验证优先看机器产生的结构化结果，不相信模型自己的描述。**

例如应该检查：

```json
{"status": "success"}
```

而不是相信模型说：

```text
“操作已经成功完成。”
```

因为模型文本可能出错，也可能被提示注入影响。

---

### 5. Correct：失败后自动恢复

解决：

> 出错以后怎么办？

常见方法：

* 自动重试；
* 换方案；
* 回滚；
* 接续生成；
* 恢复稳定状态；
* 多次失败后交给人类。

例如：

```text
API 调用失败
↓
静默重试
↓
仍然失败
↓
换备用方案
↓
连续失败
↓
熔断，交给用户
```

---

## Harness 的基本闭环

```python
observation = Environment.observe()
trajectory = [observation]

while true:
    actions = Model(Harness.build_context(trajectory))

    if no actions:
        break

    actions = Harness.constrain(actions)

    observation = Environment.apply(actions)

    if not Harness.verify(Environment):
        observation = Harness.correct(Environment)

    trajectory.append(actions, observation)
```

大白话：

```text
看环境
↓
模型决定
↓
检查能不能做
↓
真正执行
↓
检查做对没有
↓
错了就修
↓
继续下一轮
```

---

## 最关键的理解

Harness 的 5 个部分其实分成两组：

```text
让 Agent 能做事
├─ Context
└─ Tools

让 Agent 可靠做事
├─ Constrain
├─ Verify
└─ Correct
```

早期 Agent 框架主要解决：

> 怎么让模型调用工具？

现在生产系统更关心：

> **怎么确保模型调用工具以后不会把事情搞坏？**

这就是 Harness 工程的重心变化。

---

## 为什么 Harness 会成为核心竞争力

工具本身通常并不复杂：

```text
读文件
写文件
跑命令
搜索
请求 API
```

真正困难的是围绕工具建立：

* 权限控制；
* 状态管理；
* 上下文压缩；
* 错误恢复；
* 自动验证；
* 重试；
* 回滚；
* 熔断。

以 Claude Code 这类 Coding Agent 为例，真正复杂的往往不是“怎么执行 Shell”，而是：

> **什么时候允许执行、失败后怎么办、怎么判断成功、什么时候必须停下来。**

因此 Agent 工程正在从：

```text
能不能做
```

转向：

```text
能不能长期、安全、可恢复地做
```

这就是 Harness 的价值。
