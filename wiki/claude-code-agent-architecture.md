---
name: Claude Code Agent 架构
description: Claude Code 的 Agent 架构 = while 循环 + 工具分发 + 上下文管理 + 子代理，从教学版和生产版交叉验证
type: concept
sources:
  - "[[sources/learn-claude-code]]"
  - "[[sources/claude-code-system-prompts/README]]"
confidence: 0.92
updated: 2026-04-06
status: deprecated
tags: [Agent, 架构, Claude-Code, harness]
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# Claude Code Agent 架构

> 内容已迁移至 [[wiki/agent-arch-p1.md|agent-arch-p1]]。

## 核心思想

Claude Code 的 Agent 架构可以归纳为四个核心机制的组合：**while 循环 + 工具分发 + 上下文管理 + 子代理**。[^1]

这不是理论模型——learn-claude-code 的 12 个教学脚本从 s01 到 s12 逐步构建了每一个机制，而 claude-code-system-prompts 的 250 个生产提示词则展示了这些机制在真实产品中的成熟形态。[^2]

```
┌──────────────────────────────────────────────────┐
│  Agent Architecture                               │
│                                                    │
│  ┌────────────┐  ┌────────────┐                   │
│  │ Agent Loop │←→│ Tool       │  while + dispatch │
│  │ (while)    │  │ Registry   │                   │
│  └─────┬──────┘  └────────────┘                   │
│        │                                           │
│  ┌─────▼──────┐  ┌────────────┐                   │
│  │ Context    │←→│ Subagent   │  压缩 + 隔离      │
│  │ Manager    │  │ Spawner    │                   │
│  └────────────┘  └────────────┘                   │
└──────────────────────────────────────────────────┘
```

## 详细说明

### 1. Agent Loop：循环就是一切

教学版（s01）用约 20 行 Python 实现了核心循环：[^3]

```python
while True:
    response = client.messages.create(messages=messages, tools=TOOLS)
    if response.stop_reason != "tool_use":
        return
    # 执行工具，结果喂回 messages
```

生产版对应 System Prompts 中的多个片段：[^4]
- **Doing Tasks 系列**（约 15 个片段）定义了循环内的行为约束
- **Tool Usage 系列**（约 12 个片段）定义了工具调用的策略
- **Output Efficiency** 片段控制循环的输出节奏

交叉验证结论：生产版没有改变循环本身，而是通过提示词片段控制循环内的决策质量。

### 2. 工具注册表：可扩展的分发机制

教学版（s02）用 `TOOL_HANDLERS` 字典实现分发，新增工具只需注册一个 handler：[^5]

```python
TOOL_HANDLERS = {"bash": run_bash, "read_file": run_read, ...}
handler = TOOL_HANDLERS.get(block.name)
output = handler(**block.input)
```

生产版的工具注册表更复杂——Tool Descriptions 约 75 个片段，包括：[^6]
- 核心工具声明（Bash/Read/Write/Edit/Glob/Grep）
- 每个工具有独立的描述片段和多个使用规则片段
- Bash 工具的 sandbox 策略单独拆成约 20 个片段
- 子代理通过 `disallowedTools` 限制可用工具集

### 3. 上下文管理：有策略地遗忘

教学版（s06）设计了三层压缩：[^7]

| 层级 | 机制 | 触发条件 | 成本 |
|------|------|----------|------|
| micro_compact | 替换旧 tool_result 为占位符 | 每轮 | 零 |
| auto_compact | 保存+LLM摘要+替换 | token > 50000 | 一次 API 调用 |
| compact 工具 | 同 auto_compact | 模型主动触发 | 一次 API 调用 |

生产版对应 `system-prompt-context-compaction-summary` 和 `agent-prompt-conversation-summarization`：[^8]
- 生产版的压缩摘要要求更结构化：Task Overview / Current State / Discoveries / Next Steps / Context to Preserve
- 额外有 `agent-prompt-recent-message-summarization` 做近期消息摘要
- 压缩后的摘要用 `<summary>` 标签包裹

### 4. 子代理：隔离的独立循环

教学版（s04）的子代理设计原则：[^9]

1. **全新上下文**：`sub_messages = [prompt]`，父代理的历史不可见
2. **受限工具集**：`CHILD_TOOLS` 没有 `task`，防止递归派生
3. **只返回摘要**：子代理的整个历史被丢弃，父代理只收到一句话

生产版的子代理通过 `agentMetadata` 管理隔离：[^10]

```yaml
agentMetadata:
  agentType: 'Explore'
  model: 'haiku'
  disallowedTools: [Agent, ExitPlanMode, Edit, Write, NotebookEdit]
```

- Explore 代理：只读，禁止任何文件修改
- Plan 代理：规划专用，不能执行
- Verification 代理：测试专用，独立沙箱

### 5. 技能按需加载：知识与执行的分离

教学版（s05）的两层设计：Layer 1 放 name+description（约 100 tokens），Layer 2 在调用时加载完整内容。[^11]

生产版对应 `data-*` 系列（27 个）和 `skill-*` 系列（13 个）：[^12]
- SDK 参考、API 参考、Tool Use 概念等按需注入
- 技能 prompt 只在用户触发对应 slash 命令时加载

## 相关概念

- [[harness-engineering.md]] — "代码是 harness" 的核心方法论
- [[multi-agent-framework.md]] — 多智能体协作的四角色框架
- [[summary-learn-claude-code.md]] — 12 个教学脚本的完整摘要
- [[summary-claude-code-system-prompts.md]] — 250 个生产提示词的分类摘要
- [[Everything-Claude-Code.md]] — ECC 技能集，Skill 分类的第三方扩展
- [[natural-language-programming.md]] — 自然语言定义 Agent 行为（对应 System Prompts 的行为约束）

## 待解决的问题

- 250 个提示词片段的精确运行时组装顺序尚未完全逆向（README 只列出了分类，未描述拼接逻辑）
- 教学版的 s07-s12（任务持久化、后台任务、团队协作）在生产版中的对应关系需要进一步追踪
- sandbox 策略的 20 个 Bash 子片段之间的组合条件需要更细致的分析

## 引用索引

[^1]: [[sources/learn-claude-code/s01-Agent循环]] `L9-19` — Agent Loop 核心模式定义
[^2]: [[sources/claude-code-system-prompts/README]] `L46-57` — 多片段组合的设计理由
[^3]: [[sources/learn-claude-code/s01-Agent循环]] `L124-155` — agent_loop 完整代码
[^4]: [[sources/claude-code-system-prompts/README]] `L154-220` — System Prompt 片段列表
[^5]: [[sources/learn-claude-code/s02-工具调用]] `L10-20` — TOOL_HANDLERS 分发机制
[^6]: [[sources/claude-code-system-prompts/README]] `L265-343` — Tool Descriptions 完整列表
[^7]: [[sources/learn-claude-code/s06-上下文压缩]] `L12-40` — 三层压缩架构图
[^8]: [[sources/claude-code-system-prompts/system-prompts/system-prompt-context-compaction-summary]] `L1-28` — 生产版压缩摘要 prompt
[^9]: [[sources/learn-claude-code/s04-子代理]] `L10-29` — 子代理的独立上下文设计
[^10]: [[sources/claude-code-system-prompts/system-prompts/agent-prompt-explore]] `L12-26` — Explore 代理的元数据和工具限制
[^11]: [[sources/learn-claude-code/s05-技能按需加载]] `L9-18` — 两层按需加载设计
[^12]: [[sources/claude-code-system-prompts/README]] `L123-153` — Data 系列参考文档列表
