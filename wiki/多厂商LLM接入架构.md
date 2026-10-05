---
name: 多厂商LLM接入架构
description: Rust CLI 多厂商 LLM 接入方案——Harness 厂商适配器 + Codex 请求分层，五类职责划分、一次对话数据流、错误/重试/断流/取消规则与验收
type: concept
updated: 2026-10-05
tags: [rust, llm, provider, adapter, retry, streaming, codex, deepseek-harness]
source: self
---

# 多厂商LLM接入架构

## 核心思想

接入多家模型厂商时，采用 **DeepSeek Harness 的厂商适配方式 + Codex 的 Rust 请求分层**：主循环只面向统一消息与统一输出事件，厂商差异由适配器翻译吸收。先接一个厂商，把配置、流式、取消、错误和重试做完整，再横向扩展。

## 参考实现的核对结论

- **DeepSeek Harness**：内部统一消息和输出事件，由适配器翻译成各厂商的 API 格式。主循环通过 `provider` 找到对应适配器。[Harness llm 入口 index.ts](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/llm/llm/src/index.ts)
- **Codex**：配置、网络请求、协议解析分层。当前标准 `wire_api` 使用 **Responses**，已经移除 `chat`；不能直接把它当作支持所有厂商原生协议的通用客户端。[ModelProviderInfo 源码](https://raw.githubusercontent.com/openai/codex/main/codex-rs/model-provider-info/src/lib.rs)

## 职责划分

| 部分 | 具体负责什么 | 源码参考 |
|---|---|---|
| 配置 | 保存厂商地址、协议、密钥环境变量名；另外选择模型和它支持的参数 | [Codex ModelProviderInfo](https://github.com/openai/codex/blob/main/codex-rs/model-provider-info/src/lib.rs) |
| 统一数据 | 定义消息、请求、文本增量、用量、结束原因和错误，供主循环使用 | [Harness types.ts](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/llm/llm/src/types.ts) |
| 协议适配器 | 将统一请求翻译成厂商 JSON，将厂商响应翻译成统一事件 | [Harness 适配器说明](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/user/develop/practice/llm-adapter.md) |
| 请求执行 | 复用 HTTP 客户端，负责鉴权、超时、取消和网络传输 | [Codex 请求分层](https://github.com/openai/codex/blob/main/codex-rs/codex-api/README.md) |
| 会话 | 保存聊天历史和本轮状态，控制什么时候加入模型回答 | [Harness 重试与历史规则](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/llm/llm-retry/README.md) |

## 配置设计

沿用 Codex 的方式：**结构体规定每个厂商有哪些字段，`HashMap` 按厂商名字查找**，两者一起使用。密钥从环境变量读取，配置文件保存环境变量名。[ModelProviderInfo 源码](https://raw.githubusercontent.com/openai/codex/main/codex-rs/model-provider-info/src/lib.rs)

## 一次对话的数据流

1. **读取输入**：生成一条 `user` 消息。上下键历史用于找回输入；聊天历史用于让模型记住对话，两者分别管理。
2. **确定路由**：根据厂商名字找到配置和适配器，检查密钥、模型和参数；本次调用固定使用这份配置。
3. **组装请求**：把系统提示词、聊天历史和当前输入交给适配器。它决定使用 `messages`，还是 Responses 的 `instructions/input` 等字段。
4. **处理响应**：先把网络字节组装成完整的 SSE 事件，再解析 JSON，转换成统一的文本增量、用量和结束事件。SSE 就是服务端持续分段发送结果。[Codex 客户端 README](https://github.com/openai/codex/blob/main/codex-rs/codex-client/README.md)
5. **完成本轮**：终端显示增量文字；确认正常结束后，才将完整回答加入聊天历史。

## 生产级接入规则

- **错误分类**：缺密钥、鉴权失败、参数错误、额度耗尽，直接说明原因；限流和可恢复的网络、服务错误才有限重试。
- **重试只有一个负责人**：次数和等待时间设上限，遵循有效的 `Retry-After`。重试沿用同一次输入，避免重复添加消息。Harness 将重试放在 Agent 步骤边界，并关闭适配 SDK 的重试，避免多层重复执行。[Harness llm-retry README](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/llm/llm-retry/README.md)
- **断流算失败**：HTTP 200、已经打印了一些文字，都不能证明回答完成。Codex 明确将“未收到完成事件就断开”判为错误。失败的半截回答应标记为中断，不作为成功回答进入下一轮上下文。[Codex SSE responses.rs](https://github.com/openai/codex/blob/main/codex-rs/codex-api/src/sse/responses.rs)
- **请求可以取消**：连接超时、流长时间没数据都要处理；请求阶段的 Ctrl+C 也要接入取消，不能只依赖输入阶段的 `rustyline`。
- **参数按模型能力发送**：输出上限、推理参数不能所有模型照搬；上下文超限要明确处理，不能靠原样重发解决。
- **能够定位问题**：记录厂商、模型、请求 ID、耗时、用量和重试次数；密钥必须脱敏。

## 落地顺序与验收

**配置与统一数据 → 单轮真实调用 → 多轮历史、流式和取消 → 错误、重试和故障验收。**

验收要检查**实际发送的 JSON、最终文字、请求次数和历史变化**。至少覆盖：两轮聊天、错 Key、限流后成功、中文与 JSON 被拆包、提前断流、Ctrl+C。全部通过，才算这一阶段完成。

## 相关概念

- [[从零实现Agent]] — 本页是路线图第 28 步“多模型供应商”的落地方案
- [[终端UI方案：Codex复用与首版规格]] — 同项目第 27 步“终端界面”的落地方案
- [[Agent驱动Rust学习]] — 以 Codex 源码为地图的学习路径，请求分层的取用参考
- [[大模型API上下文结构]] — 统一消息结构与 API 无状态原理，“统一数据”层的背景知识
