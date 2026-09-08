```yaml
name: Agent 的 5 个通用设计模式与第一章总框架
description: 用 5 个可复用模式理解 Agent 如何审查、加载信息、保存状态、验证修改和安全演进。
type: methodology
updated: 2026-09-09
tags: [agent, harness, design-patterns, react, guardrails, context-engineering]
source: [[sources/1.3-贯穿全书的设计模式]]
```

# 五个最重要的设计模式

## 1. 提议者—审核者

解决的问题：

> **自己检查自己不可靠。**

基本结构：

```text
Proposer
负责产出
    ↓
产物
    ↓
Reviewer
独立检查
```

关键不是简单地“再问一次模型”，而是：

> **审核者不要共享产出者的完整思考上下文，只检查最终产物和客观证据。**

例如 Coding Agent：

```text
Agent A：修改代码
        ↓
Agent B：只看 diff + 测试结果
        ↓
判断修改是否正确
```

因为同一个 Agent 很容易：

* 延续自己最初的错误假设；
* 看不到自己的认知盲区；
* 被提示注入后连“自审”一起失效。

典型应用：

* 知识更新审核；
* 工具调用审批；
* 代码 Review；
* UI / PPT / 视频结果评估；
* Prompt 更新审核。

**Sidecar** 可以理解成一种只读 Reviewer：它观察、检查，但不直接修改主任务。

---

## 2. 渐进式披露

解决的问题：

> **上下文装不下所有信息，而且信息越多不一定越容易选对。**

不要：

```text
把 100 个工具
+ 所有 Skill
+ 所有文档
一次性塞给模型
```

而是：

```text
先给目录 / 元数据
        ↓
模型判断需要什么
        ↓
再加载具体内容
```

例如 Agent Skills：

```text
常驻：
skill 名称 + 简介

需要时：
完整 SKILL.md
```

好处有两个：

1. 节省 Context；
2. 减少无关内容干扰，提高选择准确率。

典型应用：

* Agent Skills；
* 分层 RAG；
* 动态工具发现；
* 分页读取；
* Multi-Agent 中的 Agent 发现。

---

## 3. 只增不改

解决的问题：

> **状态不断变化时，怎样保证可缓存、可回放、可审计？**

核心纪律：

```text
旧状态
不修改

新信息
追加到后面
```

例如 ReAct：

```text
user
assistant
tool_call
tool_result
assistant
tool_call
tool_result
...
```

轨迹一直往后长，而不是不断重写前面的历史。

这样可以获得：

* **可缓存**：旧前缀保持稳定；
* **可重放**：可以从头恢复执行轨迹；
* **可审计**：知道每一步发生了什么。

这也解释了为什么 KV Cache 喜欢稳定前缀：

> **改得越靠前，后面的缓存作废得越多。**

---

## 4. 边界集 + 保留集

这是非常重要的测试思想。

任何修改都应该同时测试：

```text
Boundary Set
修改应该改变的东西

+

Retention Set
修改绝对不应该破坏的东西
```

例如修一个 Bug：

```text
Boundary：
原来失败的输入现在必须成功

Retention：
以前正常工作的其他输入继续成功
```

如果只测 Boundary：

> 很容易为了修一个案例，把系统改坏。

如果只测 Retention：

> 又可能什么都没真正修好。

这其实就是：

```text
新功能测试
+
回归测试
```

更一般化后的表达。

---

## 5. 最小 Diff + 可回滚

解决的问题：

> **系统更新以后出了问题，能不能知道是哪一次修改造成的？**

原则：

```text
一次只改尽可能少的东西
+
记录修改来源
+
修改可以单独撤销
```

这样出了问题：

```text
版本 A
↓
修改 1
↓
修改 2
↓
修改 3 ← 出问题

可以定位并只撤销修改 3
```

而不是一次重写整个系统，最后根本不知道哪里错了。

典型应用：

* Git 小提交；
* Prompt 局部修改；
* 知识库增量更新；
* 代码 Patch；
* Agent 程序更新。

它也是为什么 Agent 开发里通常强调：

> **只改任务需要的东西，不要顺手重构。**

# 五个模式其实是一套完整闭环

可以把它们连起来理解：

```text
渐进式披露
↓
只给 Agent 当前真正需要的信息

Proposer
↓
提出修改

最小 Diff
↓
只修改必要部分

Boundary + Retention
↓
验证既修好了问题，也没有制造回归

Reviewer
↓
独立检查结果

Append-only
↓
把本次执行和结果留下来
```

这已经很接近一个成熟 Harness 的基本工作方式。

# 第一章的总框架

## Agent 的三个基本组成

```text
Agent = LLM + Context + Tools
```

大白话：

```text
LLM      = 大脑
Context  = 眼睛
Tools    = 手脚
```

其中模型负责：

> **决定下一步做什么。**

Harness 负责：

> **让它看得到正确的信息、调用得到正确的工具，而且不能乱来。**

---

# Context 和 Tools 是主要能力杠杆

模型不变时，很多 Agent 能力提升并不需要训练模型。

只需要扩大：

```text
Observation Space
模型能看到什么

+

Action Space
模型能做什么
```

例如给 Coding Agent 增加：

```text
Git
Shell
浏览器
测试工具
代码搜索
文件系统
```

很多以前做不了的任务马上就变得可解。

所以 Agent 工程一个核心思想是：

> **模型能力只是上限之一，接口边界同样决定系统能力。**

---

# Context 是 Agent 的工作记忆

每次模型调用大致看到：

```text
静态前缀
├─ System Prompt
└─ Tool Definitions

动态轨迹
├─ 用户消息
├─ Agent 回复
├─ Tool Call
└─ Tool Result
```

ReAct 就是在不断向这条轨迹追加：

```text
Reason
→ Act
→ Observe
→ Reason
→ ...
```

因此 Agent 能连续工作，不是因为模型内部一直保存着状态，而是因为：

> **Harness 把过去发生的事情重新放进下一次 Context。**

---

# Harness 才是生产系统真正的工程量

可以把生产级 Agent 理解成：

```text
Model
+
Harness
```

Harness 主要负责：

```text
Context Management      # 上下文管理
Tool Interface          # 工具接口
Constraints             # 约束
Validation              # 验证
Correction              # 纠错
```

也就是：

> **不是只让模型“会做事”，而是保证它可靠地做事。**

随着模型逐渐商品化，不同 Agent 产品之间的差距会越来越来自：

* 上下文如何组织；
* 工具如何设计；
* 权限如何控制；
* 怎么验证结果；
* 出错以后怎么恢复。

---

# 编排复杂度应该逐级增加

正确顺序：

```text
单次 LLM
    ↓ 不够
Workflow
    ↓ 不够
Autonomous Agent    # 自主 Agent
```

不要因为 Agent 更“高级”就直接使用自主循环。

判断标准：

```text
一次调用能解决？
→ 单次调用

步骤可以提前确定？
→ Workflow

步骤无法提前确定？
→ Autonomous Agent
```

实际生产系统往往：

```text
Workflow
+
Agent
```

高风险部分固定，开放式部分自主。

---

# 安全是架构问题

不要把安全理解成：

> 上线前加几个 Prompt。

应该从系统第一天就设计三层护栏：

```text
上下文层
模型能看到什么

执行层
模型能做什么

数据层
世界最终允许被改成什么
```

越重要的限制，越应该往下沉。

例如：

```text
Prompt：
“不要修改其他用户的数据”

不如

数据库：
“你的凭证物理上只能修改自己的数据”
```

后者即使 Agent 被注入也很难绕过。

# 第一章最终应该留下的模型

```text
                Agent
                  │
      ┌───────────┼───────────┐
      │           │           │
     LLM       Context       Tools
     大脑         眼睛          手脚
                  │
                  ▼
               Harness
                  │
    ┌─────────────┼─────────────┐
 Context       Tools       Guardrails
 Management   Interface    Validation
                  │
                  ▼
               ReAct Loop
                  │
                  ▼
          Reliable Execution
```

而构建 Harness 时，反复使用五条纪律：

```text
独立审核
渐进加载
状态追加
双集合验证
最小修改 + 可回滚
```

一句话总结：

> **好的 Agent 不是单纯让模型更聪明，而是通过 Harness 给模型合适的信息、合适的工具、明确的边界和可靠的反馈，让一个会犯错的模型仍然能够稳定完成复杂任务。**
