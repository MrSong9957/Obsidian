---
name: Chat Template：API 消息如何变成模型输入
description: 理解消息格式转换，以及它对工具调用、思考内容保留和缓存复用的影响。
type: tech-concept
updated: 2026-09-15
tags: [llm, chat-template, tokenization, tool-calling, kv-cache]
source: self
---


**Chat Template（聊天模板）负责把结构化消息，排成模型训练时熟悉的格式。** 它标明谁在说话、消息在哪里结束、接下来轮到谁回答。

补充一个精确区别：**模板负责排列内容，Tokenizer（分词器）负责把内容编码成 token ID，也就是数字编号。** 模型实际接收这些编号。[Hugging Face 文档](https://huggingface.co/docs/transformers/en/chat_templating)

例如，请求中的 `messages` 数组：

```json
[
  {"role": "system", "content": "你是助手"},
  {"role": "user", "content": "你好"}
]
```

使用 Qwen3 的模板，并添加助手回复的起始标记后：

```text
<|im_start|>system
你是助手<|im_end|>
<|im_start|>user
你好<|im_end|>
<|im_start|>assistant
```

* `<|im_start|>`：一条消息开始，后面标明角色。
* `<|im_end|>`：这条消息结束。
* 最后的 `<|im_start|>assistant`：提示模型从这里开始生成助手回复。

上面展示的是 token 序列的可读形式，还需要编码成数字编号。Qwen、Llama、Gemma 等模型使用的格式可能不同；通过正确配置的 vLLM、Ollama 等聊天 API 调用时，通常由服务端完成转换，无需手动拼接。

**消息角色写错，可能改变模型收到的历史内容。**

以 Qwen3-0.6B 的模板为例，它会找出最后一次真正的用户输入：

* 这次输入之后的工具调用过程中，保留助手的思考内容，即 `<think>...</think>`。
* 更早的助手消息，只保留回答等内容，不再拼入旧思考。

因此，把工具结果错写成普通 `user` 消息，可能让模板把它当成新一轮用户输入，提前移除当前工具循环里的思考内容。

原文的“判断用户换了话题”只是类比。**实际是模板按消息结构执行规则，不是在理解用户有没有换话题，也不代表删除了客户端保存的历史。** [Qwen3 模板源码](https://huggingface.co/Qwen/Qwen3-0.6B/blob/main/tokenizer_config.json)

正确做法是遵守对应 API 的工具结果格式：

| API 格式                      | 工具结果怎么回传                                   |
| --------------------------- | ------------------------------------------ |
| OpenAI 兼容的 Chat Completions | 使用 `role: "tool"`，通过 `tool_call_id` 对应工具调用 |
| Claude Messages             | 放在 `user` 消息的 `tool_result` 内容块中           |

所以，关键是**保留协议规定的工具结果结构**，不能把“用了 `user` 角色”一概视为错误。Claude 本来就这样承载工具结果。[Claude 文档](https://platform.claude.com/docs/en/build-with-claude/thinking)

**思考内容是否回传，要区分模型、版本和使用场景。**

思考内容可能记录“为什么调用这个工具、排除了哪些可能”。在连续工具调用中保留它，可以帮助模型接着之前的推理继续工作。但不能由开发者统一决定全部保留或全部删除。

下面结合官方资料修正原文中的版本概括：

| 模型／接口             | 思考内容处理规则                                                                                                                                                            |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| DeepSeek R1 历史接口  | 后续请求回传回答 `content`，不回传 `reasoning_content`。[旧版文档](https://deepseek.apidog.io/reasoning-model-deepseek-reasoner-835841m0)                                            |
| DeepSeek 现行思考 API | 请求携带 `tools` 时，要求完整回传历史 `reasoning_content`，包括没有实际调用工具的轮次；缺失会返回 `400`。不带 `tools` 时，历史思考会被忽略。[当前文档](https://api-docs.deepseek.com/guides/thinking_mode/)             |
| Kimi K2 Thinking  | 官方工具循环示例原样追加完整助手消息，随后追加工具结果，保留其中的思考字段。[官方示例](https://huggingface.co/moonshotai/Kimi-K2-Thinking)                                                                    |
| GLM-5             | 工具循环中保留并回传思考；跨轮保留可通过 `thinking.clear_thinking: false` 开启。Coding Plan 接口默认开启，标准 API 默认关闭。[官方文档](https://docs.z.ai/guides/capabilities/thinking-mode)                 |
| Claude            | 工具循环中，完整回传 `thinking` 块，包括 `signature`，不要修改。跨用户轮次是否保留取决于版本；Opus 4.5 起、Sonnet 4.6 起默认保留全部历史思考。[官方文档](https://platform.claude.com/docs/en/build-with-claude/thinking) |

原文有两点需要明确纠正：

* **DeepSeek 并非到 V4 才开始在工具调用中回传思考**，V3.2 已引入这项能力。[官方发布说明](https://api-docs.deepseek.com/zh-cn/news/news251201/)
* **移除历史思考不等于完全从零开始**：模型仍可看到回传的用户消息、助手回答和工具结果，只是失去了思考内容中的部分状态。

因此，这些差异会影响推理连续性、上下文用量、缓存和请求是否合法。把执行到一半的轨迹交给另一家模型时，也需要适配消息格式，不能只替换模型名称。

**KV Cache 复用看的是最终 token 前缀是否一致。**

KV Cache 保存模型处理前文时产生的中间计算结果。Chat Template 本身不管理缓存，但它决定最终输入的排列，因此会影响缓存命中。

在模型和相关配置相同、采用常规前缀缓存的前提下：

* 首个不同 token **之前**的缓存仍有复用条件。
* 从首个不同 token **开始及其之后**，不能直接复用原有缓存，因为后面的计算依赖前面的内容。
* 实际复用还受缓存是否仍存在、缓存块大小等条件限制。[vLLM 文档](https://docs.vllm.ai/en/latest/design/prefix_caching/)

这意味着：系统提示词多一个空格、工具定义顺序变化，或者模板移除了旧思考，都可能改变最终 token 序列。只有在变化确实影响编码结果时，才会影响对应位置之后的前缀复用。

开发时应尽量保持前部的系统提示词和工具定义稳定，把新增信息追加到后面；具体排列仍以模型模板为准。**即使 API 消息只做了追加，模板重新渲染后的 token 前缀，也不一定完全不变。**
