---
name: 工具调用闭环：Codex复用与参考范围
description: Codex 工具定义机制（Rust 规格 → JSON 声明，自然语言写在 description）与工具调用闭环的源码复用范围——固定快照 2fdf047c 的文件级参考表、测试夹具与两处接口适配点
type: concept
updated: 2026-10-10
tags: [codex, agent, tool-calling, rust, harness]
source: "https://github.com/openai/codex/tree/2fdf047c9631c9ed01a31b62efb7891718a931a8"
---

# 工具调用闭环：Codex复用与参考范围

## 核心思想

Codex 的初始工具定义是「用途说明＋参数规格」：用 Rust 代码创建，再序列化成 JSON 发给模型；提示词中的自然语言主要写在工具的 `description` 和各参数的 `description` 里。[^1]

在自研 Agent（[[从零实现Agent]] 项目）中复用 Codex 的工具调用闭环时，范围锁定为：**工具声明 → 模型提出调用 → 执行现有工具 → 回传结果 → 继续请求模型**，首版完成两个工具（`exec_command`、`apply_patch`）的串行调用闭环。

## 详细说明

### 工具如何声明给模型

普通函数工具的结构定义在 `codex-rs/tools/src/responses_api.rs`，核心字段：[^1]

| 字段 | 作用 |
|---|---|
| `name` | 工具叫什么 |
| `description` | 告诉模型：这个工具做什么、怎么使用 |
| `parameters` | 参数规格：名称、类型、说明、哪些必填 |
| `strict` | 是否启用 API 的严格参数约束 |

以执行命令的 `exec_command` 为例，它在 `codex-rs/core/src/tools/handlers/shell_spec.rs` 中创建。按源码整理成精简 JSON（说明文字译为中文，只保留 `cmd` 参数）：[^2]

```json
{
  "type": "function",
  "name": "exec_command",
  "description": "执行命令，返回输出或用于后续交互的会话 ID。",
  "strict": false,
  "parameters": {
    "type": "object",
    "properties": {
      "cmd": {
        "type": "string",
        "description": "要执行的 Shell 命令。"
      }
    },
    "required": ["cmd"],
    "additionalProperties": false
  }
}
```

模型据此知道：

- 有一个叫 `exec_command` 的工具。
- 调用时要提供字符串参数 `cmd`。
- `cmd` 必填，参数名必须符合定义。

完整定义还有工作目录、Shell、等待时间、输出上限、权限审批等参数；部分字段会根据环境和配置加入。

### 工具列表组装与序列化

初始工具列表由代码组装：从已注册工具中选出当前模型可见的工具，再合并命名空间。[^3] 这些 Rust 工具规格随后被序列化成 JSON：普通请求放进 `tools`，Responses Lite 路径放进 `AdditionalTools`。[^4][^5]

### 复用参考范围（固定快照 2fdf047c）

参考版本锁定为 [Codex 快照 `2fdf047c9631`](https://github.com/openai/codex/tree/2fdf047c9631c9ed01a31b62efb7891718a931a8)，与项目现有 `apply_patch` 规格使用同一版本；下列路径均相对 `codex-rs/`，已核验存在。

| 研究环节 | 文件和路径 | 重点参考 |
|---|---|---|
| 对话执行循环 | `core/src/session/turn.rs`、`core/src/stream_events_utils.rs` | 接收工具调用、等待执行、写入结果、继续请求模型、结束本轮 |
| 工具声明与注册 | `core/src/tools/spec_plan.rs`、`core/src/tools/handlers/shell_spec.rs`、`core/src/tools/handlers/apply_patch_spec.rs` | 工具名称、描述、参数格式，以及声明与实际执行器的对应关系 |
| 调用识别与分发 | `core/src/tools/router.rs`、`core/src/tools/registry.rs` | 提取工具名、调用编号、参数；找到执行器；处理未知工具 |
| 接入现有工具 | `core/src/tools/handlers/unified_exec.rs`、`core/src/tools/handlers/unified_exec/exec_command.rs`、`core/src/tools/handlers/apply_patch.rs` | 参数解析、工作目录传递、调用工具模块、包装执行结果 |
| 结果与错误回传 | `core/src/tools/context.rs`、`core/src/tools/parallel.rs`、`tools/src/function_call_error.rs` | 结果关联原调用编号；区分返回模型的工具错误与终止流程的程序错误；取消处理 |
| 对话历史 | `core/src/context_manager/history.rs`、`core/src/context_manager/normalize.rs` | 同时保存模型的调用和工具结果，供下一次模型请求使用 |
| 协议与请求构造 | `protocol/src/models.rs`、`tools/src/tool_spec.rs`、`tools/src/responses_api.rs`、`core/src/client_common.rs`、`core/src/client.rs`、`codex-api/src/common.rs`、`codex-api/src/endpoint/responses.rs` | 工具声明如何进入请求，调用与结果如何表示 |
| 流式解析，按需参考 | `codex-api/src/sse/responses.rs` | 项目使用流式响应时，怎样接收完整的工具参数 |

其中 `ToolRouter::build_tool_call` 把模型输出转换为工具名、调用编号和参数；注册表负责查找并执行对应工具。[^6]

参考测试也应纳入范围：

| 路径 | 借用的验证方法 |
|---|---|
| `core/tests/common/responses.rs` | 构造模型响应，捕获下一次请求并检查工具结果 |
| `core/tests/suite/tool_harness.rs` | 验证命令执行、补丁修改，以及结果真正返回模型 |
| `core/tests/suite/apply_patch_serialization.rs` | 检查工具调用后的真实文件内容和结果格式 |
| `core/tests/suite/tools.rs`、`core/src/tools/router_tests.rs`、`core/src/tools/context_tests.rs` | 检查未知工具、调用编号、路由和结果转换 |

### 接口适配点

复用时有两个需要适配的接口差异：

- **Chat Completions vs Responses API**：本项目当前接入层为 Chat Completions，Codex 此快照使用 Responses API。应借用执行循环和分发逻辑，按当前接口适配请求与回传——Chat Completions 需要保留 `assistant.tool_calls`，再添加带对应 `tool_call_id` 的工具结果。[^7]
- **`apply_patch` 参数格式**：Codex 原版接收补丁原文；本项目规格采用 JSON `{"patch":"…"}`。工具声明和执行入口必须一起适配，结果继续保留现有的变更清单及 `state_known`。

### 复用顺序与边界

复用顺序：**沿用现有工具模块，优先搬用或裁剪上述解析、分发、回传函数和测试夹具，再补无法复用的协议连接部分。**

本轮不扩展 MCP、Code Mode、多 Agent、并行调度、后台终端会话或完整 TUI；源码移植同时核对根目录 `LICENSE`、`NOTICE`（Apache-2.0，保留版权与 NOTICE 并注明修改）。

## 待解决的问题

- 闭环首版尚未落码，本页为范围与参考锁定；实现时按上表逐环节核对固定快照路径。
- 两处接口适配（Chat Completions 请求构造、`apply_patch` JSON 格式）需在实现时落地并配套测试。

## 相关概念

- [[从零实现Agent]] — 本页服务该项目的工具调用闭环里程碑
- [[终端UI方案：Codex复用与首版规格]] — 同项目第 27 步终端界面的 Codex 复用边界；该页首版明确不引入工具执行
- [[多厂商LLM接入架构]] — 同项目第 28 步模型接入层，工具结果回传走该层请求构造
- [[Agent工具调用的API循环]] — 调用闭环的通用概念：职责边界与 `tool_call_id` 配对
- [[Agent-工具设计]] — 工具 description 写法与工具设计的通用原则

## 引用索引

[^1]: https://github.com/openai/codex/blob/2fdf047c9631c9ed01a31b62efb7891718a931a8/codex-rs/tools/src/responses_api.rs#L32-L45 — 普通函数工具的结构定义（name / description / parameters / strict）
[^2]: https://github.com/openai/codex/blob/2fdf047c9631c9ed01a31b62efb7891718a931a8/codex-rs/core/src/tools/handlers/shell_spec.rs#L24-L114 — `exec_command` 工具规格的创建
[^3]: https://github.com/openai/codex/blob/2fdf047c9631c9ed01a31b62efb7891718a931a8/codex-rs/core/src/tools/spec_plan.rs#L647-L675 — 初始工具列表的组装与命名空间合并
[^4]: https://github.com/openai/codex/blob/2fdf047c9631c9ed01a31b62efb7891718a931a8/codex-rs/tools/src/tool_spec.rs#L80-L149 — Rust 工具规格序列化为 JSON
[^5]: https://github.com/openai/codex/blob/2fdf047c9631c9ed01a31b62efb7891718a931a8/codex-rs/core/src/client.rs#L971-L998 — 请求构造：`tools` 与 Responses Lite 的 `AdditionalTools`
[^6]: https://github.com/openai/codex/blob/2fdf047c9631c9ed01a31b62efb7891718a931a8/codex-rs/core/src/tools/router.rs — `ToolRouter::build_tool_call` 调用识别与注册表分发
[^7]: https://developers.openai.com/api/docs/guides/function-calling — Chat Completions 的工具调用与 `tool_call_id` 结果回传
