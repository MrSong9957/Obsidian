---
name: Agent 上下文布局与本地模型工具调用
description: 理解上下文如何组装、压缩和复用缓存，以及本地模型如何参与工具调用。
type: tech-concept
updated: 2026-09-14
tags: [ai-agent, context-engineering, kv-cache, tool-calling, local-llm]
source: self
---

**Agent 每次调用模型前，都要组装模型这次能看到的信息。** 常见布局是“稳定前缀 + 动态轨迹”。

| 部分   | 内容                      | 处理方式                |
| ---- | ----------------------- | ------------------- |
| 稳定前缀 | 系统提示词、核心工具定义            | 尽量保持内容和顺序稳定，方便复用缓存  |
| 动态轨迹 | 用户消息、模型回复及工具调用请求、工具执行结果 | 随交互增长，接近上下文预算时压缩旧内容 |

这是逻辑上的划分。在 API 请求中，消息放入 `messages`，工具定义通常通过独立的 `tools` 字段传入。

还可以从轨迹中提炼**当前状态摘要**，放在消息末尾：已完成什么、还缺什么、必须遵守哪些要求。这样模型更容易了解当前进度。

**保持前缀稳定，是为了减少重复计算。**

KV Cache 保存模型处理过的 token 的中间计算结果。服务启用跨请求前缀缓存后，新请求可以复用相同开头对应的结果，减少重新处理输入的时间。

* 系统提示词、工具定义和保持一致的历史消息，都可能被复用。
* 修改前面的内容，会影响修改位置及后续部分的缓存复用。
* 压缩历史能节省上下文空间，但也会改变前缀，因此不宜每轮频繁改写历史。

前缀缓存主要缩短处理输入的时间，不会直接提高后续每个新 token 的生成速度。[vLLM 前缀缓存文档](https://docs.vllm.ai/en/latest/features/automatic_prefix_caching/)

**每次请求前，先整理状态，再检查预算，必要时压缩，最后发送。**

下面是 Python 风格伪代码；辅助函数表示需要自行实现的步骤。

```python
# 固定规则和工具说明，尽量保持不变
stable_prefix = system_message
stable_tools = core_tool_schemas

# 读取历史，包含本轮用户输入
trajectory = load_message_history(session)

# 整理“做到哪了、还缺什么”
status_message = make_status_message(
    derive_current_state(trajectory)
)

# 给模型接下来生成的内容留出空间
input_budget = context_limit - reserved_output_tokens

# 放不下时，压缩旧内容
if estimated_tokens(
    stable_prefix, stable_tools, trajectory, status_message
) > input_budget:
    trajectory = compress_old_evidence(
        trajectory,
        # 留下决定、限制、失败经验和来源
        preserve=["decisions", "constraints", "failures", "citations"],
    )

# 规则放前面，历史放中间，当前状态放最后
request.messages = [stable_prefix] + trajectory + [status_message]
request.tools = stable_tools

# 压缩后仍放不下，就先停止
if estimated_tokens(request.messages, request.tools) > input_budget:
    raise ValueError("上下文仍超出预算")

response = call_model(request)
```

压缩重点是旧工具输出等占空间的内容。保留决策和约束，避免模型忘记已确定的事情；保留失败经验，避免重复踩坑；保留来源引用，方便核查依据。

真实实现还要保持消息角色正确，以及工具调用与结果的 `tool_call_id` 对应关系，不能把这些结构当普通文本随意删改。

**本地部署改变的是模型运行的位置，工具调用仍由模型与框架分工完成。**

`local_llm_serving` 使用“帮我联系 Xfinity 砍价”作为示例任务，基本过程是：

1. **框架发送请求**：把消息历史和工具定义交给 vLLM、Ollama 等本地模型服务。
2. **服务转换格式**：按模型的聊天模板，将消息和工具说明转换成 token 序列。
3. **模型提出调用**：生成工具名称和参数，框架执行对应函数或外部 API。
4. **结果回传模型**：保存模型的工具调用请求和执行结果，再次请求模型。需要更多信息就继续调用工具；正常完成且没有工具调用时，展示回复并结束本轮循环。

这就是 ReAct 的“思考—行动—观察”循环。模型负责决定调用什么，真正的操作由接入的工具完成。

例如，同时查询温哥华的时间和天气时，模型可以在一次响应中提出两个调用。框架确认它们没有数据依赖、不会相互干扰后，可以并行执行。**并行的是工具执行，模型仍逐 token 生成调用内容。**

**流式输出让客户端边接收、边显示或处理结果。**

模型可能输出推理文本、正文和工具调用，具体格式与顺序取决于模型和服务。例如，Qwen3 在启用思考模式时可以输出 `<think>...</think>`；这是模型生成的推理文本，并不代表全部内部计算。[Qwen3 官方说明](https://huggingface.co/Qwen/Qwen3-0.6B)

API 也可以将这些内容分别返回，例如 Ollama 的 `thinking`、`content` 和 `tool_calls`。客户端应按协议处理，而不是假设所有内容都在一个正文字符串里。[Ollama 工具调用文档](https://docs.ollama.com/capabilities/tool-calling)

流式工具调用需要：

1. 按调用标识收集参数片段。
2. 确认该调用已接收完整，再解析和校验。
3. 满足权限和依赖条件后，开始执行。

协议支持单个调用的完成通知时，可以提前执行已完成的调用，无需等待其他调用全部生成。**当前片段能解析成 JSON，不等于参数已经接收完毕。**[流式函数调用文档](https://developers.openai.com/api/docs/guides/function-calling#streaming)

**小模型也能参与工具调用，但能力、速度和可靠性需要分别衡量。**

`local_llm_serving` 展示了 **0.6B（6 亿参数）模型**参与多轮工具调用，并报告在苹果 M2 上超过 **100 token/s** 的生成速度。该数据代表对应实验环境，不能直接套用到其他部署。

Token 是模型切分文本的单位，与中文字数或英文单词数没有固定换算关系。

模型大小并非工具调用能力的唯一决定因素，模型训练、提示词、工具设计和任务难度同样重要。小模型为手机、电脑等设备上的端侧 Agent 提供了可能，具体任务仍需实际测试。

**观察缓存效果，要测首 token 延迟，并控制其他条件。**

首 token 延迟，就是从发出请求到收到第一个输出 token 的时间，与“每秒生成多少 token”是不同指标。

实验方法：

1. 确认服务支持并启用了跨请求前缀缓存，完成模型加载和预热。
2. 固定模型、工具定义和其他配置，连续发送具有相同前缀的请求，记录首 token 延迟。
3. 修改系统提示词开头，再发送长度相近的请求，多次比较延迟；有缓存命中指标时一起记录。

缓存命中时，相同前缀可以减少输入计算。实际延迟还受输入长度和服务负载等因素影响，不能仅凭一次变快或变慢判断缓存是否生效。
