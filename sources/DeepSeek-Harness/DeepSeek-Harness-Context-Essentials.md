---
name: DeepSeek Harness Context Essentials
description: 用 5 个关键机制讲解 DeepSeek Harness 如何保存、组装和压缩模型上下文，全部结论经 deepseek-ai/deepseek-harness 源码核对（master，2026-08-19）
type: reference
updated: 2026-08-25
tags: [deepseek, harness, context, llm]
source: https://github.com/deepseek-ai/deepseek-harness
---

# DeepSeek Harness Context Essentials

> 用最关键的 5 个机制理解 DeepSeek Harness 如何保存、组装和压缩模型上下文。
>
> **源码核对基准：** `deepseek-ai/deepseek-harness` → `master`，2026-08-19。
>
> 本文只写当前 GitHub 源码能够直接支持的结论。代码块中的代码均来自对应源码；为了保持简短，只引用最关键的完整行，不用伪造的 `...` 冒充原始代码。

## 0. 先建立正确的主链路

```text
用户 / Agent / Tool
        ↓
Session Event Log
完整记录发生过的事件
        ↓
Surface
维护“当前模型可见”的历史节点
        ↓
deriveMessages()
生成真正的 Message[]
        ↓
        ├──────────── System Prompt
        ├──────────── Tool Schemas
        ↓
LLM Request
        ↓
Token Meter
        ↓
上下文压力过高时 Compaction
        ↓
用 summary checkpoint replace 旧 Surface
原始 Event Log 仍然保留
```

还有一个容易画错的位置：**PromptContext 最终不是独立于 `Message[]` 的请求字段。**

它先被组装成 runtime-context snapshot，再被做成 `user/message` 写进 Session，因此最后会通过 `deriveMessages()` 进入 `messages`。

最简单地记：

```text
System Prompt ────────────────────────┐
Tools ────────────────────────────────┼→ LLM Request
PromptContext → user/message → Surface│
普通历史 ───────────────→ Surface     │
                       ↓              │
                deriveMessages() ─────┘
```

本文只学 5 件事：

1. Session Event Log：完整历史保存在哪里
2. Surface：模型现在能看到什么
3. `deriveMessages()`：怎么生成模型消息
4. PromptContext：动态环境怎么真正进入模型历史
5. Token Meter + Compaction：上下文太长怎么办

---

# 1. Session Event Log：完整流水账

源码在 `packages/core/session/src/types.ts` 对 `SessionEventMap` 的说明非常直接：它是一次 agent interaction 的 **append-only source of truth**，message history 从这个 log 派生。

源码：[`packages/core/session/src/types.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/session/src/types.ts)

**大白话：**

Harness 不是维护一份随时被改写的“最终聊天数组”，而是先把发生的事情按事件记下来。

例如 Log 中可能同时存在：

```text
user/message
assistant/chunk
assistant/message
tool/call
tool/result
request/header
turn/start
step/end
...
```

这些事件用途不同：有的用于模型历史，有的用于流式回放，有的用于请求重建或生命周期记录。

所以第一个关键区别是：

> **写进 Session Log，不等于一定会发给模型。**

**只记一句：Session Log = 完整事件账本。**

---

# 2. Surface：模型当前可见的历史

`Session Log` 很完整，但模型不需要直接看到全部事件。

当前源码明确限定了能够进入 ordered surface 的基础事件类型：

```ts
export type SurfaceEventType =
  | 'user/message'
  | 'assistant/message'
  | 'tool/result'
```

源码：[`packages/core/session/src/types.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/session/src/types.ts)

**大白话：**

```text
Log     = 完整录像
Surface = 当前给模型看的剪辑版
```

因此：

```text
user/message       → Surface 候选
assistant/message  → Surface 候选
tool/result        → Surface 候选

assistant/chunk    → Log-only
turn/start         → Log-only
request/header     → Log-only，不是聊天消息
```

注意：模型产生的 tool call 可以存在于 `assistant/message` 的 content 中，因此没有单独的 `tool/call` SurfaceEventType 并不等于模型看不到之前的 tool call。

## append 与 replace

源码定义两类 Surface 操作：

```ts
export type SurfaceOp = 'append' | { op: 'replace'; start: number; end: number }
```

源码：[`packages/core/session/src/types.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/session/src/types.ts)

### append

普通对话不断向尾部追加：

```text
U1 → A1 → ToolResult1 → A2 → U2
```

### replace

当旧上下文需要压缩时，Surface 可以从：

```text
U1 A1 U2 A2 U3 A3
```

变成：

```text
Summary1 U3 A3
```

这里发生的是：**旧 Surface 节点被新的 checkpoint 遮蔽。**

不是把原始 Log 中的 `U1/A1/U2/A2` 删除。

最简单类比：

```text
Session Log = 完整会议录像
Surface     = 当前工作材料
Compaction  = 把工作材料中的前两小时换成一页纪要
              原录像仍保留
```

**只记一句：Surface = 模型当前应该记得什么。**

---

# 3. deriveMessages()：Surface → Message[]

真正调用模型时，需要把 Surface 节点变成 `Message[]`。

`Session.deriveMessages()` 会遍历当前 Surface 节点，并调用单节点投影函数：

```ts
const msg = this.deriveEventMessage(this.log[seq]!)
if (msg) this.derived.push(msg)
```

源码：[`packages/core/session/src/index.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/session/src/index.ts)

真正决定“一个事件变成什么 Message”的是 `deriveEventMessage()`。

例如 `tool/result` 的真实代码就是：

```ts
case 'tool/result': {
  return event.data.message
}
```

源码：[`packages/core/session/src/surface.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/session/src/surface.ts)

这里还有一个之前容易漏掉的重要边界：空内容的 `assistant/message` 不进入历史；源码会直接返回 `null`。

**大白话：**

```text
Surface          = 决定哪些 Event 现在有效
deriveEventMessage = 决定一个 Event 对应什么 Message
deriveMessages() = 把当前所有有效节点汇总成 Message[]
```

因此更准确的完整关系是：

```text
Session Log
   ↓ fold surfaceOp
Surface.nodes
   ↓ deriveEventMessage()
Message[]
```

**只记一句：`deriveMessages()` = 把当前 Surface 投影成 LLM 真正使用的历史消息。**

---

# 4. PromptContext：动态环境到底怎么进入请求

这一部分之前最容易理解错。

## 4.1 System Prompt 和 Tools 是请求级输入

`SystemPrompt` 服务会组装：

- prompt sections
- dynamic contexts
- tool schemas

但最终 agent loop 构建 LLM request 时，源码传入的是：

```text
messages
system（可选）
tools（可选）
```

也就是说，**PromptContext 并不是最终 `GenerateOptions` 中一个独立的 `context` 字段。**

源码：[`packages/core/agent-loop/src/agent.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/agent-loop/src/agent.ts)

## 4.2 PromptContext 是动态 runtime snapshot

`PromptContext` 的源码定义将它描述成一个 durable user-role snapshot。

源码：[`packages/core/system-prompt/src/index.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/system-prompt/src/index.ts)

这句话的重点不在术语，而在 **user-role snapshot**。

### 实际调用链

agent 每个 step 前先 assemble：

```text
systemPrompt.assemble()
        ↓
renderContextSections()
        ↓
RuntimeContextProjection.project()
```

`RuntimeContextProjection.project()` 在需要更新时调用：

```ts
return createUserMessage({
```

源码：[`packages/core/agent-loop/src/runtime-context.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/agent-loop/src/runtime-context.ts)

随后 pre-step 得到的这些 messages 会真正写进 Session：

```ts
this.session.append('user/message', message, { surfaceOp: 'append' })
```

源码：[`packages/core/agent-loop/src/agent.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/agent-loop/src/agent.ts)

之后 `step()` 再调用：

```text
this.session.deriveMessages()
```

去构建最终 LLM 请求。

所以真正的路径是：

```text
PromptContext provider
        ↓
PromptAssembly.contexts
        ↓
render runtime context
        ↓
RuntimeContextProjection
        ↓
UserMessage
        ↓
session.append('user/message')
        ↓
Surface
        ↓
deriveMessages()
        ↓
LLM messages
```

### 为什么叫 Projection？

它不会每一步无脑重复插入同一个 runtime context。

`RuntimeContextProjection` 会记住当前 retained snapshot；如果新 snapshot 没变化，就不生成新的 UserMessage。若之前的 snapshot 被 Surface replacement（例如 compaction）遮掉，它也会更新自己的 retained 状态。

**大白话：**

> PromptContext 不是每次请求临时塞进去的一张便利贴，而是“运行环境发生变化时，生成一条可持久化的用户角色快照，让它成为模型历史的一部分”。

因此第一次学习时应该这样区分：

| 内容 | 实际位置 | 大白话 |
|---|---|---|
| System Prompt | request `system` | 稳定工作规则 |
| Tools | request `tools` | 当前能调用什么 |
| 普通对话 | request `messages` | 之前聊了什么 |
| PromptContext snapshot | 先写入 `user/message`，最终也在 `messages` | 当前运行环境 |

**只记一句：PromptContext 最终通过 `user/message → Surface → deriveMessages()` 进入模型历史。**

---

# 5. Token Meter + Compaction：上下文太长怎么办

## 5.1 Token Meter：测的是 request pressure，不只是聊天数组

核心 API：

```ts
measure(session: Session, requestHeader?: EpochHeader): TokenMeasurement
```

源码：[`packages/llm/token-meter/src/index.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/llm/token-meter/src/index.ts)

当前源码最终返回的两个关键量是：

```ts
totalTokens: Math.max(0, baseline.tokens + surfaceDeltaTokens),
surfaceTokens: state.surfaceTokens,
```

**大白话：**

```text
surfaceTokens = 当前模型可见 Surface 本身的 token 规模
totalTokens   = 当前请求整体的 token 压力
```

为什么两者不同？

因为 request pressure 还要考虑当前 request header 对应的 system、tools 等 envelope 内容，而不只是历史 `messages`。

当前实现能安全复用匹配请求的 provider usage 时会复用；否则会走 heuristic 重新估算。这个细节先不展开。

## 5.2 Compaction：旧 Surface 变成 summary checkpoint

当前 `compaction-basic` 默认策略中最值得先记的数字：

```text
thresholdRatio = 0.8
retainRatio    = 0.16
maxTokens      = 8192
auto           = true
```

含义：

- 压力阈值默认是 routed context window 的 80%。
- 默认保留最近约 16% context-window budget 对应的 Surface 原文。
- summarization call 的默认生成上限是 8192。
- 默认开启自动 pressure / overflow 监听。

来源：[`packages/compaction/compaction-basic/README.md`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/compaction/compaction-basic/README.md)

这些是当前 `compaction-basic` 默认值，不是 Harness 永久固定协议。

## 5.3 最关键的 compaction 提交逻辑

真正写入 summary 记录时，源码有：

```ts
const summaryEvent = session.append('compaction/summary', {
```

随后真正改变 Surface 的是另一个 `user/message` replacement：

```ts
surfaceOp: { op: 'replace', start, end },
```

源码：[`packages/compaction/compaction-basic/src/region.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/compaction/compaction-basic/src/region.ts)

完整语义是：

```text
compaction/summary
= Log 中保存此次压缩的 summary/provenance

user/message checkpoint + surfaceOp: replace
= 真正替换旧 Surface range 的模型可见节点
```

因此：

> **Compaction 改写的是模型可见 Surface，而不是直接删除原始 Session Log。**

## 5.4 Tool Result Pruning 是可选插件

`compaction-basic` README 明确说明，需要额外加载：

```text
dsh-compaction-tool-result-pruner
```

才会启用这个 model-free pruning pass。

所以正确流程是：

```text
Token pressure
      ↓
[可选] Tool Result Pruning
      ↓
重新测量
      ↓
如果仍然需要压缩
      ↓
Summary call
      ↓
checkpoint replace 旧 Surface
```

不要把 Tool Result Pruning 写成 basic compaction 永远必经的固定步骤。

**只记一句：Token Meter 判断压力，Compaction 通过 Surface replacement 缩短未来的模型历史。**

---

# 6. 一次真实 step 的最小调用链

把前面的知识合起来，一次典型 step 可以简化为：

```text
① inbox.claim()
   取本轮/本 step 要处理的新输入

② systemPrompt.assemble()
   得到 system sections、runtime contexts、tools

③ RuntimeContextProjection.project()
   runtime context 有变化时 → 生成 UserMessage

④ session.append('user/message', ..., surfaceOp='append')
   用户输入和新的 runtime snapshot 进入 Log + Surface

⑤ renderPrompt(assembly)
   得到本轮 system prompt

⑥ session.deriveMessages()
   从当前 Surface 得到 Message[]

⑦ buildRequest(... tools, system, messages ...)
   形成真正的 LLM request

⑧ llm.stream(request)
   调模型

⑨ assistant/chunk
   流式 chunk 写入 Log，但不是 Surface history

⑩ assistant/message + surfaceOp='append'
   最终完整回答进入 Surface
```

其中最能证明最终请求结构的一行调用链是：

```text
buildRequest(turn, step, assembly.tools, system, session.deriveMessages(), signal)
```

也就是说，最终需要抓住三个核心 request payload：

```text
system
+ tools
+ deriveMessages() 得到的 messages
```

而 PromptContext 已经在前面被物化为 `user/message`，包含在 `messages` 里面。

---

# 7. 用一个例子串起来

假设开发 Agent 正在修一个 bug：

```text
用户：帮我修 config.ts
模型：我先看文件
工具：返回文件内容
模型：发现问题
用户：继续修改
```

同时当前项目目录存在动态 workspace instructions。

### 第一步：动态环境先变成历史消息

```text
PromptContext(workspace instructions)
        ↓
RuntimeContextProjection
        ↓
user/message(runtime snapshot)
```

### 第二步：正常交互也写 Log / Surface

```text
user/message(runtime snapshot)
user/message(“帮我修 config.ts”)
assistant/message(包含 tool-call)
tool/result
assistant/message(“发现问题”)
user/message(“继续修改”)
```

### 第三步：下一次请求

```text
Surface
   ↓
deriveMessages()
   ↓
上述 model-visible messages

+ 当前 system prompt
+ 当前 tool schemas
   ↓
LLM Request
```

### 第四步：历史越来越长

Token Meter 检测到 request pressure 接近 context capacity。

### 第五步：Compaction

原 Surface：

```text
RuntimeContext1 U1 A1 ToolResult1 A2 U2 A3
```

压缩后可能变成：

```text
SummaryCheckpoint U2 A3
```

旧事件仍存在于 Session Log；只是当前 Surface 不再把被 shadow 的旧节点交给 `deriveMessages()`。

---

# 8. 只记住这 5 句话

1. **Session Log 是 append-only 的事实账本，模型历史从它派生。**
2. **Surface 是当前模型可见的历史节点集合；基础 Surface Event 是 `user/message`、`assistant/message`、`tool/result`。**
3. **`deriveMessages()` 根据当前 Surface 生成 LLM 的 `Message[]`。**
4. **PromptContext 不是最终请求的独立字段；它变化时会先物化成 `user/message`，再通过 Surface 进入 `messages`。**
5. **Token Meter 测 request pressure；Compaction 用 summary checkpoint replace 旧 Surface，而不是删除原 Log。**

再压成五个等式：

```text
Log            = 全部发生过什么
Surface        = 模型现在应该记得什么
deriveMessages = 把 Surface 变成模型历史
PromptContext  = 变化时写成 user/message 的运行环境快照
Compaction     = 用摘要节点替换旧工作记忆
```

---

# 9. 本次源码核对后修正了什么

| 原文表达 | 核对结果 | 修正 |
|---|---|---|
| `System Prompt + PromptContext + Tools + Message[]` | ❌ 不够准确 | PromptContext 会先物化成 `user/message`，最终属于 `messages` |
| `deriveEventMessage()` 的 assistant 分支只需 `return message` | ⚠️ 漏条件 | 空 content 的 `assistant/message` 会返回 `null` |
| compaction 代码块用 `...` 表示源码省略 | ⚠️ 容易误认为原始代码 | 改为只引用 GitHub 中真实存在的完整关键行 |
| `tool/result → event.data.message` | ✅ | 保留 |
| Surface 基础类型只有三类 | ✅ | 保留 |
| summary event 与 replace checkpoint 是两个事件 | ✅ | 保留 |
| Tool Result Pruning 是可选 sibling plugin | ✅ | 保留 |
| Token Meter 区分 request pressure 与 surface tokens | ✅ | 保留 |

---

# 10. 当前最值得看的源码

第一次学习只看下面 7 个文件即可：

1. [`packages/core/session/src/types.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/session/src/types.ts)  
   Event Log、SurfaceEventType、SurfaceOp。

2. [`packages/core/session/src/surface.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/session/src/surface.ts)  
   Surface fold 与 `deriveEventMessage()`。

3. [`packages/core/session/src/index.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/session/src/index.ts)  
   `Session.deriveMessages()`。

4. [`packages/core/system-prompt/src/index.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/system-prompt/src/index.ts)  
   PromptSection / PromptContext / PromptAssembly。

5. [`packages/core/agent-loop/src/runtime-context.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/agent-loop/src/runtime-context.ts)  
   PromptContext 如何变成 durable UserMessage。

6. [`packages/core/agent-loop/src/agent.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/agent-loop/src/agent.ts)  
   真正的 pre-step、写 Session、`deriveMessages()`、buildRequest、LLM 调用链。

7. [`packages/compaction/compaction-basic/src/region.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/compaction/compaction-basic/src/region.ts)  
   summary record 与 Surface replacement 的真实提交逻辑。

Token 计量再补看：

- [`packages/llm/token-meter/src/index.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/llm/token-meter/src/index.ts)
- [`packages/compaction/compaction-basic/README.md`](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/compaction/compaction-basic/README.md)

---

## 最终心智模型

以后遇到任何上下文问题，只问 5 个问题：

```text
1. 这个信息首先以什么 Event 写进 Session？
2. 它是否在当前 Surface 上？
3. deriveMessages() 会不会把它投影成 Message？
4. 它最终在 request 的 system、tools，还是 messages 中？
5. Context pressure / compaction 会怎样影响它？
```

能沿真实源码回答这 5 个问题，就能解释 DeepSeek Harness 大部分常见的上下文行为。
