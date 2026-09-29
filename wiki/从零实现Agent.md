---
name: 从零实现Agent
description: 按 34 步迭代表从零实现终端编程 Agent——能聊天、会用工具、能改代码、稳定持续工作、可扩展发布（Rust，主参考 Codex 与 DeepSeek Harness 源码）
type: learning-roadmap
updated: 2026-09-29
tags: [agent, coding-agent, rust, roadmap, tool-calling, codex, deepseek-harness]
source: self
---

# 从零实现 Agent

建议按 **“能聊天 → 会用工具 → 能改代码 → 能稳定持续工作 → 可扩展和发布”** 的顺序做，每次只实现一个能验收的小功能。

下面按**终端里的编程 Agent**规划。主参考用 [Codex](https://github.com/openai/codex) 的 Rust 源码，功能拆分参考 [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness)。Claude Code 可以参考交互和公开文档，其公开仓库主要提供插件、文档和问题反馈，不作为完整核心源码教材。
**表格中的每一行，就是一次独立迭代。** 先完成当前功能并验证，再进入下一项。下面是建议的学习顺序，不是照搬这些项目的架构。

第一阶段先做一个能连续聊天的程序。只接一家模型，先用普通输入输出。

| 顺序 | 实现功能 | 怎样算完成 |
|---|---|---|
| 1 | **程序外壳**：创建 Rust 项目，读取输入、配置和 API Key | 能输入、退出；缺少配置时明确提示 |
| 2 | **单次模型请求**：发送问题，解析并打印回复 | 输入一句话，得到真实模型回复 |
| 3 | **基础错误处理**：处理网络超时、鉴权失败、返回格式错误 | 出错能说明原因，不直接崩溃，也不打印密钥 |
| 4 | **多轮对话**：保存消息历史，支持清空 | 模型能接着上一轮聊；清空后不再带旧消息 |

参考：[reqwest 示例](https://github.com/seanmonstar/reqwest/tree/master/examples)，重点看 HTTP 请求与 JSON；再看 [Codex 模型客户端](https://github.com/openai/codex/blob/main/codex-rs/core/src/client.rs)。reqwest 同时提供同步和异步接口，起步可以先用同步请求。
第二阶段让模型能调用工具。核心流程是：**模型提出调用 → Rust 执行 → 结果写回历史 → 再请求模型**。

| 顺序 | 实现功能 | 怎样算完成 |
|---|---|---|
| 5 | **工具调用协议**：定义名称、参数、调用编号、结果；先做一个加法工具 | 模型发出调用，程序校验参数、计算并回传；非法参数返回错误 |
| 6 | **Agent 循环**：根据工具结果继续行动 | 能连续调用工具后回答；达到步数上限或用户取消时停止 |
| 7 | **读取文件**：支持按行读取，限定项目范围 | 能解释真实文件；文件不存在、路径越界时正确报错 |
| 8 | **项目检索**：查找文件、搜索文件内容 | 能先找到相关代码，再读取需要的片段 |
| 9 | **超长工具结果处理**：原文保存，返回预览和读取入口 | 上下文不会被大日志塞满；中间被省略的内容仍能搜索、读取 |

参考：[DeepSeek Harness 核心模块](https://github.com/deepseek-ai/deepseek-harness/tree/master/packages/core)中的 `tools`、`agent-loop`；超长输出参考 [spill 模块](https://github.com/deepseek-ai/deepseek-harness/tree/master/packages/spill)。后者把完整文本存到上下文之外，再提供找回原文的入口。
第三阶段补上记录、会话和项目规则，为后面的修改操作打基础。

| 顺序 | 实现功能 | 怎样算完成 |
|---|---|---|
| 10 | **事件记录**：记录模型回复、工具开始、结束和错误 | 能查清“执行了什么、结果是什么、哪里失败” |
| 11 | **会话保存与恢复**：保存完整消息和工具结果，支持新建、继续 | 程序关闭后重新打开，能接着原会话工作 |
| 12 | **项目规则**：读取 `AGENTS.md`，确定规则作用范围 | 使用当前项目的规则，不混入其他项目的规则 |

参考：[DeepSeek Harness 会话核心](https://github.com/deepseek-ai/deepseek-harness/tree/master/packages/core/session)、[Codex 会话存储](https://github.com/openai/codex/tree/main/codex-rs/thread-store)、[项目规则读取](https://github.com/openai/codex/blob/main/codex-rs/core/src/agents_md.rs)。初版可以用 JSONL——每行存一条记录，暂时不必引入数据库。Codex 的本地会话存储也使用 JSONL 保存历史。
第四阶段开放修改能力。**审批和执行范围必须先做好，再加入写文件和命令执行。**

| 顺序 | 实现功能 | 怎样算完成 |
|---|---|---|
| 13 | **权限与审批**：允许、拒绝、询问用户 | 用户拒绝后，操作确实没有执行 |
| 14 | **执行隔离**：限制可读写目录和网络；可先接现成隔离机制 | 即使模型要求越界，执行层也能阻止 |
| 15 | **创建文件**：写入新文件，处理已有同名文件 | 创建内容正确，不意外覆盖旧文件 |
| 16 | **修改文件**：精确替换或应用补丁，显示差异 | 只改目标位置；原文不匹配时拒绝硬改 |
| 17 | **执行命令**：收集输出、错误、退出码，支持超时和取消 | 能运行编译、测试；卡住的命令及其子进程能停止 |
| 18 | **修改与验证闭环**：查问题、修改、运行验证、根据失败修正 | 能解决一个真实小问题；反复失败时停止并说明阻碍 |

参考：[Codex 工具执行层](https://github.com/openai/codex/tree/main/codex-rs/core/src/tools)、[补丁实现](https://github.com/openai/codex/tree/main/codex-rs/apply-patch)、[隔离模块](https://github.com/openai/codex/tree/main/codex-rs/sandboxing)；审批交互参考 [DeepSeek Harness interaction](https://github.com/deepseek-ai/deepseek-harness/tree/master/packages/interaction)。
**做到第 18 步，就有了一个最小可用的编程 Agent。** 后面的重点是让它处理长任务时更稳。

第五阶段处理等待、长上下文和网络波动。

| 顺序 | 实现功能 | 怎样算完成 |
|---|---|---|
| 19 | **异步与流式输出**：边接收边显示，等待时仍能取消 | 回复逐步显示；工具参数完整接收并校验后才执行 |
| 20 | **上下文预算**：统计用量，为回复和工具结果预留空间 | 每次请求前检查容量，不随意截断工具调用与结果 |
| 21 | **上下文压缩**：先手动压缩，验证后再自动触发 | 保留目标、约束、已做事项和下一步；完整历史仍可查 |
| 22 | **有限重试**：处理临时断网、限流等问题 | 可恢复的请求会重试；超过上限会停；不盲目重放修改操作 |

参考：[DeepSeek Harness 模型接入与重试](https://github.com/deepseek-ai/deepseek-harness/tree/master/packages/llm)、[上下文压缩](https://github.com/deepseek-ai/deepseek-harness/tree/master/packages/compaction)，以及 [Codex 上下文管理](https://github.com/openai/codex/tree/main/codex-rs/core/src/context_manager)。这里要区分：**工具结果过长，先处理那一次结果；整段会话过长，再压缩历史。**
第六阶段完善恢复和人机协作。此时核心流程已经稳定，可以认真做界面。

| 顺序 | 实现功能 | 怎样算完成 |
|---|---|---|
| 23 | **异常中断恢复**：区分未开始、执行中、已完成 | 重启后不重复已完成操作；结果不确定时先核实 |
| 24 | **向用户提问**：需要信息时暂停，收到回答后继续 | 不靠猜测补齐关键需求，也不反复空转 |
| 25 | **计划与待办**：拆步骤、记录进度，支持只读规划 | 规划模式确实不能修改；执行进度有真实结果支撑 |
| 26 | **修改审阅与撤销**：查看差异，撤销本次修改 | 能撤销自己的改动，不覆盖用户后来修改的内容 |
| 27 | **终端界面**：输入区、历史、工具状态、审批、快捷命令 | 长输出和窗口缩放正常；界面通过事件读取核心状态 |

参考：[Codex 会话运行逻辑](https://github.com/openai/codex/tree/main/codex-rs/core/src/session)、[交互工具](https://github.com/openai/codex/tree/main/codex-rs/core/src/tools/handlers)、[终端界面](https://github.com/openai/codex/tree/main/codex-rs/tui)；计划功能也可看 [DeepSeek Harness plan](https://github.com/deepseek-ai/deepseek-harness/tree/master/packages/plan)。

第七阶段扩展能力。先把内置功能跑稳，再开放外部接入。

| 顺序 | 实现功能 | 怎样算完成 |
|---|---|---|
| 28 | **多模型供应商**：用统一内部消息对接不同接口 | 换供应商后，聊天、工具调用、流式和错误处理仍正确 |
| 29 | **Skills**：按需加载任务说明和配套资源 | 相关技能能被发现、读取；不会每次塞入所有技能全文 |
| 30 | **MCP 客户端**：连接外部工具服务 | 能发现、调用工具；外部工具也经过权限检查 |
| 31 | **长期记忆**：保存稳定偏好和项目知识，支持查询、修改、删除 | 新会话能找回相关知识；项目之间不串记忆 |

参考：[DeepSeek Harness llm](https://github.com/deepseek-ai/deepseek-harness/tree/master/packages/llm)、[skill](https://github.com/deepseek-ai/deepseek-harness/tree/master/packages/skill)、[官方 MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)、[Codex memories](https://github.com/openai/codex/tree/main/codex-rs/ext/memories)。MCP 可以直接使用官方 Rust SDK；长期记忆初版先用文件和关键词检索。
第八阶段把它变成能长期维护、交给别人使用的产品。

| 顺序 | 实现功能 | 怎样算完成 |
|---|---|---|
| 32 | **固定回归任务**：覆盖工具、权限、恢复、压缩和真实代码修改 | 每次更新都能重复验证；检查实际文件变化和测试结果 |
| 33 | **无交互运行**：支持命令行参数、结构化结果和退出码 | 脚本能调用 Agent，并判断成功、失败或需要人工介入 |
| 34 | **打包与发布**：配置目录、版本迁移、安装说明、自动构建 | 在声明支持的干净环境中能安装、运行、升级 |

参考：[Codex 集成测试](https://github.com/openai/codex/tree/main/codex-rs/core/tests/suite)、[非交互执行入口](https://github.com/openai/codex/tree/main/codex-rs/exec)、[构建与发布流程](https://github.com/openai/codex/tree/main/.github/workflows)。**测试从第一阶段就开始，第 32 步是把已有验证整理成稳定的回归体系。**

多 Agent 可以最后追加，顺序是：**无依赖工具并行 → 后台任务 → 独立子 Agent → 隔离工作目录与结果合并**。参考 DeepSeek Harness 的 [jobs](https://github.com/deepseek-ai/deepseek-harness/tree/master/packages/jobs)、[subagent](https://github.com/deepseek-ai/deepseek-harness/tree/master/packages/subagent)。单 Agent 的权限、取消、恢复没有做好之前，多 Agent 会放大这些问题。

实际动手时，每一步固定做四件事：

1. 用注释写清输入、处理、输出和失败情况。
2. 只读参考仓库里与当前功能有关的部分。
3. 写最小实现，并验证正常、边界和异常结果。
4. 能解释数据怎么流动、为什么这样写，再进入下一步。

**第一轮只完成第 1—4 步：Rust + reqwest + serde_json，做出一个能稳定多轮聊天的小程序。** 所有权、借用和错误处理跟着这四步学；异步、统一接口和插件机制到对应阶段再引入。