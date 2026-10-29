---
name: Agent 架构（上）- 详细资料
description: Agent循环/多智能体框架/系统提示词/教学脚本的完整详细内容
type: concept
updated: 2026-10-04
tags: agent, architecture, multi-agent, system-prompts
source: https://example.com
---
# Agent 架构（上）- 详细资料

> 入口文档：[[wiki/agent-arch.md|Agent 架构核心]] | 索引：[[sources/agent/agent-arch-references.md|Agent 架构详细参考]]

---

## Claude Code Agent 架构 - 详细说明

#### 1. Agent Loop：循环就是一切

教学版（s01）用约 20 行 Python 实现了核心循环：

```python
while True:
    response = client.messages.create(messages=messages, tools=TOOLS)
    if response.stop_reason != "tool_use":
        return
    # 执行工具，结果喂回 messages
```

生产版对应 System Prompts 中的多个片段：
- **Doing Tasks 系列**（约 15 个片段）定义了循环内的行为约束
- **Tool Usage 系列**（约 12 个片段）定义了工具调用的策略
- **Output Efficiency** 片段控制循环的输出节奏

交叉验证结论：生产版没有改变循环本身，而是通过提示词片段控制循环内的决策质量。

#### 2. 工具注册表：可扩展的分发机制

教学版（s02）用 `TOOL_HANDLERS` 字典实现分发，新增工具只需注册一个 handler：

```python
TOOL_HANDLERS = {"bash": run_bash, "read_file": run_read, ...}
handler = TOOL_HANDLERS.get(block.name)
output = handler(**block.input)
```

生产版的工具注册表更复杂——Tool Descriptions 约 75 个片段，包括：
- 核心工具声明（Bash/Read/Write/Edit/Glob/Grep）
- 每个工具有独立的描述片段和多个使用规则片段
- Bash 工具的 sandbox 策略单独拆成约 20 个片段
- 子代理通过 `disallowedTools` 限制可用工具集

#### 3. 上下文管理：有策略地遗忘

教学版（s06）设计了三层压缩：

| 层级 | 机制 | 触发条件 | 成本 |
|------|------|----------|------|
| micro_compact | 替换旧 tool_result 为占位符 | 每轮 | 零 |
| auto_compact | 保存+LLM摘要+替换 | token > 50000 | 一次 API 调用 |
| compact 工具 | 同 auto_compact | 模型主动触发 | 一次 API 调用 |

生产版对应 `system-prompt-context-compaction-summary` 和 `agent-prompt-conversation-summarization`：
- 生产版的压缩摘要要求更结构化：Task Overview / Current State / Discoveries / Next Steps / Context to Preserve
- 额外有 `agent-prompt-recent-message-summarization` 做近期消息摘要
- 压缩后的摘要用 `<summary>` 标签包裹

#### 4. 子代理：隔离的独立循环

教学版（s04）的子代理设计原则：

1. **全新上下文**：`sub_messages = [prompt]`，父代理的历史不可见
2. **受限工具集**：`CHILD_TOOLS` 没有 `task`，防止递归派生
3. **只返回摘要**：子代理的整个历史被丢弃，父代理只收到一句话

生产版的子代理通过 `agentMetadata` 管理隔离：

```yaml
agentMetadata:
  agentType: 'Explore'
  model: 'haiku'
  disallowedTools: [Agent, ExitPlanMode, Edit, Write, NotebookEdit]
```

- Explore 代理：只读，禁止任何文件修改
- Plan 代理：规划专用，不能执行
- Verification 代理：测试专用，独立沙箱

#### 5. 技能按需加载：知识与执行的分离

教学版（s05）的两层设计：Layer 1 放 name+description（约 100 tokens），Layer 2 在调用时加载完整内容。

生产版对应 `data-*` 系列（27 个）和 `skill-*` 系列（13 个）：
- SDK 参考、API 参考、Tool Use 概念等按需注入
- 技能 prompt 只在用户触发对应 slash 命令时加载

### 相关概念

- [[openai-harness-engineering]] — "代码是 harness" 的核心方法论
- [[agent-arch]] — 多智能体协作的四角色框架
- [[learn-shareai.md]] — 19 章教学笔记（Learn ShareAI）
- 250 个生产提示词的分类摘要（详见 sources/claude-code/claude-code-system-prompts/）
- 自然语言定义 Agent 行为（对应 System Prompts 的行为约束）

### 待解决的问题

- 250 个提示词片段的精确运行时组装顺序尚未完全逆向（README 只列出了分类，未描述拼接逻辑）
- 教学版的 s07-s12（任务持久化、后台任务、团队协作）在生产版中的对应关系需要进一步追踪
- sandbox 策略的 20 个 Bash 子片段之间的组合条件需要更细致的分析

### 引用索引

[^1]: [[sources/agent/learn-claude-code/s01-Agent循环]] `L9-19` — Agent Loop 核心模式定义
[^2]: [[sources/claude-code/claude-code-system-prompts/README]] `L46-57` — 多片段组合的设计理由
[^3]: [[sources/agent/learn-claude-code/s01-Agent循环]] `L124-155` — agent_loop 完整代码
[^4]: [[sources/claude-code/claude-code-system-prompts/README]] `L154-220` — System Prompt 片段列表
[^5]: [[sources/agent/learn-claude-code/s02-工具调用]] `L10-20` — TOOL_HANDLERS 分发机制
[^6]: [[sources/claude-code/claude-code-system-prompts/README]] `L265-343` — Tool Descriptions 完整列表
[^7]: [[sources/agent/learn-claude-code/s06-上下文压缩]] `L12-40` — 三层压缩架构图
[^8]: [[sources/claude-code/claude-code-system-prompts/system-prompts/system-prompt-context-compaction-summary]] `L1-28` — 生产版压缩摘要 prompt
[^9]: [[sources/agent/learn-claude-code/s04-子代理]] `L10-29` — 子代理的独立上下文设计
[^10]: [[sources/claude-code/claude-code-system-prompts/system-prompts/agent-prompt-explore]] `L12-26` — Explore 代理的元数据和工具限制
[^11]: [[sources/agent/learn-claude-code/s05-技能系统]] `L9-18` — 两层按需加载设计
[^12]: [[sources/claude-code/claude-code-system-prompts/README]] `L123-153` — Data 系列参考文档列表

---

## s_full.py 底层逻辑全景

### 1. 概述

`s_full.py` 是整个教学系列的集成参考实现（capstone），把 s01 到 s11 的所有机制组装成一个可运行的完整 Agent。它不是一个教学脚本，而是一个"把所有东西放在一起"的工作参考。

核心理念：**模型本身就是 Agent，代码是 harness**——代码不决定"做什么"，只提供工具、观测和动作接口，让模型自主决策。

整合了哪些机制：

| 来源 | 机制 | 一句话 |
|------|------|--------|
| s02 | 工具分发 | LLM 返回 tool_use → handler 映射执行 |
| s03 | TodoManager | 内存中的待办列表 + 催促提醒 |
| s04 | Subagent | 一次性子 agent，跑完返回摘要 |
| s05 | SkillLoader | 从文件系统加载技能包 |
| s06 | 压缩管线 | microcompact + auto_compact 防上下文溢出 |
| s07 | TaskManager | 文件持久化任务板（含依赖和认领） |
| s08 | BackgroundManager | 后台线程跑命令，异步通知 |
| s09 | TeammateManager + MessageBus | 持久自主 agent + 进程间消息 |
| s10 | Shutdown 协议 + Plan 审批 | request_id 握手关机、计划提交审批 |
| s11 | Auto-claim | teammate 空闲时自动认领无人任务 |

---

### 2. 启动初始化

按代码加载顺序，Python 模块级代码在 `import` 时立即执行：

```
import 阶段
  ├── 标准库：json, os, re, subprocess, threading, time, uuid, pathlib, queue
  ├── 第三方：anthropic, dotenv
  └── load_dotenv() → 加载 .env 到环境变量

常量阶段
  ├── WORKDIR = Path.cwd()              # 工作目录锚点
  ├── client = Anthropic(...)           # API 客户端（支持 BASE_URL 代理）
  ├── MODEL = os.environ["MODEL_ID"]    # 模型 ID
  ├── TEAM_DIR, INBOX_DIR, TASKS_DIR   # 目录路径常量
  ├── TOKEN_THRESHOLD = 100000          # 压缩触发阈值
  ├── POLL_INTERVAL = 5                 # teammate 轮询间隔（秒）
  └── IDLE_TIMEOUT = 60                 # teammate 空闲超时（秒）

全局单例阶段（按定义顺序）
  ├── TODO = TodoManager()              # 内存待办，无持久化
  ├── SKILLS = SkillLoader(SKILLS_DIR)  # 启动时扫描 skills/ 目录
  ├── TASK_MGR = TaskManager()          # 自动创建 .tasks/ 目录
  ├── BG = BackgroundManager()          # 后台任务管理器
  ├── BUS = MessageBus()                # 自动创建 .team/inbox/ 目录
  └── TEAM = TeammateManager(BUS, TASK_MGR)  # 加载 .team/config.json

System Prompt 构建
  └── SYSTEM = f"...{SKILLS.descriptions()}..."
      # 把所有已加载技能的描述注入 system prompt
      # 告诉模型优先用 task_create/update/list 做多步工作
      # 用 TodoWrite 做短期清单
```

**关键设计**：SkillLoader 在启动时 eagerly 扫描整个 `skills/` 目录，找到所有 `SKILL.md` 文件并解析 YAML frontmatter。这意味着添加新技能后需要重启才能生效。

---

### 3. REPL 主循环

入口在 `if __name__ == "__main__"` 块（L711-736）：

```
用户输入
  │
  ├─ "q"/"exit"/空行 → 退出程序
  ├─ "/compact"     → 手动压缩历史，continue
  ├─ "/tasks"       → 打印任务板，continue
  ├─ "/team"        → 打印队友状态，continue
  ├─ "/inbox"       → 读取并清空 lead 收件箱，continue
  │
  └─ 其他 → 追加到 history，调用 agent_loop(history)
```

REPL 命令是用户侧的快捷操作，不经过 LLM。`/compact` 和 `/inbox` 会**消费性读取**（读完清空），这是有意的——避免重复处理。

---

### 4. agent_loop 内部流程

`agent_loop(messages)` 是核心驱动循环（L655-707），每次用户输入触发一轮：

```
agent_loop(messages)
  │
  │  ┌─────────────────────────────────────────────┐
  │  │ 前处理（每次循环开始）                         │
  │  │                                               │
  │  │  1. microcompact(messages)                    │
  │  │     - 找所有 tool_result，保留最近 3 个        │
  │  │     - 其余长内容替换为 "[cleared]"            │
  │  │     - 在原位修改，不产生新消息                 │
  │  │                                               │
  │  │  2. 检查 token 数 > TOKEN_THRESHOLD?          │
  │  │     - 是 → auto_compact(messages)             │
  │  │       ① 把完整历史存入 .transcripts/           │
  │  │       ② 调 LLM 生成摘要                       │
  │  │       ③ 用 2 条消息替换整个历史                │
  │  │                                               │
  │  │  3. BG.drain() — 排空后台通知队列              │
  │  │     - 每条通知注入为 user+assistant 消息对     │
  │  │                                               │
  │  │  4. BUS.read_inbox("lead") — 检查 lead 收件箱 │
  │  │     - 有消息则注入为 user+assistant 消息对     │
  │  └─────────────────────────────────────────────┘
  │
  │  ┌─────────────────────────────────────────────┐
  │  │ LLM 调用                                     │
  │  │                                               │
  │  │  response = client.messages.create(           │
  │  │    model=MODEL,                               │
  │  │    system=SYSTEM,   # 技能描述 + 行为指引      │
  │  │    messages=messages,                         │
  │  │    tools=TOOLS,     # 25 个工具定义            │
  │  │    max_tokens=8000                            │
  │  │  )                                            │
  │  │                                               │
  │  │  stop_reason == "end_turn" → return（结束）    │
  │  │  stop_reason == "tool_use" → 继续工具执行     │
  │  └─────────────────────────────────────────────┘
  │
  │  ┌─────────────────────────────────────────────┐
  │  │ 工具分发执行                                  │
  │  │                                               │
  │  │  遍历 response.content 中的 block:            │
  │  │    - block.type == "tool_use"                 │
  │  │    - 查 TOOL_HANDLERS 字典找到 handler        │
  │  │    - handler(**block.input) 执行              │
  │  │    - 异常捕获：输出 "Error: {e}"              │
  │  │    - 结果收集到 results 列表                   │
  │  └─────────────────────────────────────────────┘
  │
  │  ┌─────────────────────────────────────────────┐
  │  │ 后处理                                       │
  │  │                                               │
  │  │  5. Todo 催促机制                             │
  │  │     - 每轮检查是否调用了 TodoWrite             │
  │  │     - 如果连续 3 轮没更新 + 还有未完成项       │
  │  │     - 在 results 开头插入 <reminder>           │
  │  │                                               │
  │  │  6. 手动压缩                                  │
  │  │     - 如果 LLM 调了 compress 工具              │
  │  │     - 执行 auto_compact 压缩                   │
  │  └─────────────────────────────────────────────┘
  │
  └── 回到循环顶部，继续前处理 → LLM → 工具 → 后处理
```

**压缩策略的设计意图**：
- `microcompact` 是轻量级清理，只清除旧的 tool_result 大内容，不调 LLM
- `auto_compact` 是重量级操作，需要额外一次 LLM 调用生成摘要
- 两者配合：先用 microcompact 减少日常膨胀，不够时才触发 auto_compact
- 手动 `compress` 给模型主动控制上下文的能力

---

### 5. 六大支撑机制

#### 5.1 base_tools（基础工具）

四个原子操作，构成 agent 操作文件系统的全部能力：

```
safe_path(p) → 安全检查，防止路径逃逸
  │
  ├── run_bash(command)     → shell 命令执行
  │   - 危险命令黑名单：rm -rf /, sudo, shutdown, reboot
  │   - 超时 120 秒
  │   - 输出截断 50000 字符
  │
  ├── run_read(path, limit) → 读文件
  │   - 路径安全检查
  │   - limit 参数截断，超长显示 "... (N more)"
  │   - 输出截断 50000 字符
  │
  ├── run_write(path, content) → 写文件
  │   - 自动创建父目录
  │   - 路径安全检查
  │
  └── run_edit(path, old, new) → 替换编辑
      - 精确匹配 old_text，替换第一个出现
      - 找不到返回错误
```

**设计意图**：`safe_path` 确保 agent 不能读写工作目录外的文件。所有输出都有 50000 字符上限，防止单次工具结果撑爆上下文。

#### 5.2 Subagent（子 agent 闭环）

`run_subagent(prompt, agent_type)` 启动一个独立的 agent_loop（L160-195）：

```
调用方传入 prompt
  │
  └── 创建独立的 messages 列表
      │
      └── 循环（最多 30 轮）
          │
          ├── LLM 调用（独立上下文，不共享主循环消息）
          │
          ├── 工具执行：
          │   ├── Explore 类型：只有 bash + read_file（只读）
          │   └── 其他类型：bash + read + write + edit（全功能）
          │
          ├── stop_reason != "tool_use" → 提取文本摘要返回
          │
          └── 30 轮用完 → 返回已有文本或 "(subagent failed)"
```

**与 teammate 的区别**：
- Subagent 是**一次性**的：调用 → 执行 → 返回摘要 → 销毁
- Teammate 是**持久**的：spawn → work → idle → auto-claim 循环
- Subagent 没有收件箱、没有任务板、没有身份

#### 5.3 TodoManager（内存待办列表）

```
TodoManager
  ├── items: list[dict]  # 纯内存，不持久化
  │
  ├── update(items) → 验证 + 替换整个列表
  │   - 每个 item 必须有 content + status + activeForm
  │   - status 只能是 pending / in_progress / completed
  │   - 最多 20 个 item
  │   - 最多 1 个 in_progress（聚焦原则）
  │   - 返回渲染后的文本
  │
  ├── render() → 文本渲染
  │   [x] 已完成项
  │   [>] 进行中项 <- activeForm
  │   [ ] 待办项
  │   (N/M completed)
  │
  └── has_open_items() → 是否还有未完成项（催促用）
```

**催促机制**：agent_loop 中跟踪 `rounds_without_todo` 计数器。如果连续 3 轮没调用 TodoWrite 且还有未完成项，就在 tool_result 前插入 `<reminder>Update your todos.</reminder>`。这是一个**软提醒**，不是强制——模型可以选择忽略。

#### 5.4 TaskManager（文件持久化任务板）

```
TaskManager
  ├── 存储：.tasks/task_{id}.json 文件
  │
  ├── create(subject, description)
  │   └── 自增 ID，初始状态 pending，无 owner
  │
  ├── get(tid) → 读取单个任务 JSON
  │
  ├── update(tid, status?, add_blocked_by?, add_blocks?)
  │   ├── status="completed" → 自动解除其他任务的 blockedBy 引用
  │   └── status="deleted" → 删除文件
  │
  ├── list_all() → 渲染任务板
  │   [ ] #1: 任务标题 @owner (blocked by: [2,3])
  │
  └── claim(tid, owner) → 设置 owner + status=in_progress
```

**依赖关系机制**：
- `blockedBy`：该任务被哪些任务阻塞（需要它们先完成）
- `blocks`：该任务阻塞了哪些任务
- 当 status 设为 completed 时，**自动清除**所有依赖此任务的其他任务的 blockedBy 引用
- 这实现了一个简单的 DAG 依赖图

#### 5.5 BackgroundManager（后台线程执行）

```
BackgroundManager
  ├── tasks: dict  # {task_id: {status, command, result}}
  ├── notifications: Queue  # 线程安全通知队列
  │
  ├── run(command, timeout=120)
  │   ├── 生成 UUID[:8] 作为 task_id
  │   ├── 启动 daemon 线程执行命令
  │   └── 立即返回 "Background task {tid} started"
  │
  ├── _exec(tid, command, timeout) [线程内]
  │   ├── subprocess.run 执行命令
  │   ├── 更新 tasks[tid] 的 status 和 result
  │   └── 往 notifications 队列放一条通知
  │
  ├── check(tid=None) → 查询状态
  │
  └── drain() → 排空通知队列，返回所有通知
      # agent_loop 每轮调用，把通知注入主对话
```

**设计意图**：长时间运行的命令（构建、测试）不阻塞主循环。agent 发起后台任务后继续工作，下一轮循环时自动收到结果通知。

#### 5.6 TeammateManager + MessageBus（持久自主 agent）

这是最复杂的机制，涉及两个类协作：

##### MessageBus（消息总线）

```
MessageBus
  ├── 存储：.team/inbox/{name}.jsonl（追加写入）
  │
  ├── send(sender, to, content, msg_type, extra)
  │   └── 追加一行 JSON 到收件人的文件
  │
  ├── read_inbox(name) → 读取并清空
  │   └── 读全部行 → 解析 JSON → 清空文件 → 返回消息列表
  │
  └── broadcast(sender, content, names)
      └── 给除 sender 外的所有人发 broadcast
```

消息类型（VALID_MSG_TYPES）：
- `message` — 普通消息
- `broadcast` — 广播
- `shutdown_request` / `shutdown_response` — 关机握手
- `plan_approval_response` — 计划审批回复

**关键**：`read_inbox` 是**消费性**的——读一次就清空，确保消息不被重复处理。

##### TeammateManager（队友管理器）

每个 teammate 运行在独立 daemon 线程中，有自己的 LLM 上下文：

```
spawn(name, role, prompt)
  │
  └── 启动 daemon 线程 → _loop(name, role, prompt)
      │
      │  ═══════ WORK PHASE ═══════
      │  循环（最多 50 轮）：
      │    1. 检查收件箱
      │       - shutdown_request → 设置状态并退出线程
      │       - 其他消息 → 注入对话
      │    2. LLM 调用（独立 system prompt，7 个工具）
      │    3. 工具执行：
      │       - idle → 设置 idle_requested，退出 work phase
      │       - claim_task → 通过 TaskManager 认领任务
      │       - send_message → 通过 MessageBus 发消息
      │       - bash/read/write/edit → 基础工具
      │    4. stop_reason != tool_use → 自然结束
      │
      │  ═══════ IDLE PHASE ═══════
      │  设置状态为 idle
      │  循环（IDLE_TIMEOUT / POLL_INTERVAL 次）：
      │    sleep(POLL_INTERVAL)
      │    1. 检查收件箱
      │       - 有消息 → resume=True, 回到 WORK PHASE
      │    2. 扫描 .tasks/ 找无人认领的任务
      │       - pending + 无 owner + 无 blockedBy
      │       - 找到 → claim + 注入对话 + resume=True
      │       - 身份重注入：如果上下文太短（≤3条），插入 identity 标签
      │
      │  如果 IDLE 超时无 resume → 设置状态 shutdown，退出线程
      │  否则 → 回到 WORK PHASE
```

**teammate 的工具集**（比 lead 少得多）：
- bash, read_file, write_file, edit_file — 基础操作
- send_message — 发消息给 lead 或其他 teammate
- idle — 信号：当前工作做完
- claim_task — 认领任务板上的任务

**身份重注入**（L520-523）：当 teammate 的上下文被压缩到很短时（≤3 条消息），自动在开头插入 `<identity>` 标签，确保模型知道自己是谁。

---

### 6. 协作模式

#### Lead Agent（主 agent）

- 拥有全部 25 个工具
- 运行在 REPL 主线程
- 负责：接收用户指令、分解任务、分配工作、审批计划

#### Teammate（队友 agent）

- 只有 7 个工具
- 运行在独立 daemon 线程
- 生命周期：spawn → work → idle → auto-claim → work → ... → shutdown
- 通过 MessageBus 与 lead 通信
- 空闲时自动扫描任务板认领任务

#### 通信拓扑

```
                    MessageBus (.team/inbox/)
                    ┌──────────────────────┐
                    │                      │
    lead ◄──────────┤─────── inbox ────────┤──────► teammate_A
      │             │                      │              │
      │             │                      │              │
      ├─ spawn ─────┤─── thread start ─────┤              │
      │             │                      │              │
      ├─ send ──────┤─── A.inbox.jsonl ◄───┤──────────────┘
      │             │                      │
      ├─ broadcast ─┤─── all inboxes ──────┤──► teammate_B
      │             │                      │
      └─ shutdown ──┤── shutdown_request ──┤──► target
                    └──────────────────────┘

TaskManager (.tasks/)  ←  共享任务板，lead 和 teammates 都可读写
```

#### Shutdown 协议

```
lead 调用 handle_shutdown_request(teammate)
  │
  ├── 生成 request_id
  ├── 记录到 shutdown_requests 字典
  ├── 发送 shutdown_request 消息到 teammate 的收件箱
  │
  └── teammate 在 work/idle phase 检查收件箱时
      ├── 发现 shutdown_request
      └── 设置状态为 shutdown，退出线程
```

#### Plan 审批协议

```
teammate 发送 plan_approval 消息给 lead
  │
  └── lead 调用 plan_approval(request_id, approve, feedback)
      ├── 更新 plan_requests 状态
      └── 发送 plan_approval_response 消息给 teammate
```

---

### 7. s_full vs s12 对比

| 维度 | s_full | s12（概念） |
|------|--------|-------------|
| 隔离级别 | 线程级（共享进程、文件系统、Python 全局状态） | 进程级（独立 worktree、独立文件系统视图） |
| 通信机制 | MessageBus（文件系统 JSONL） | 跨进程通信（需要 IPC 机制） |
| 任务板 | 共享 .tasks/ 目录 | 需要协调机制防冲突 |
| 资源竞争 | 多线程操作同一文件可能冲突 | 物理隔离天然避免冲突 |
| 启动成本 | threading.Thread（毫秒级） | 新进程 + git worktree（秒级） |
| 适用场景 | 轻量协作、快速原型 | 安全隔离、正式多 agent 系统 |
| 崩溃影响 | 一个 teammate 异常可能影响全局 | 物理隔离，崩溃不扩散 |

**s_full 的局限**：所有 teammate 共享同一个工作目录和 Python 进程。虽然线程是并发的，但文件操作没有锁——两个 teammate 同时写同一个文件会导致数据丢失。这是 s12 要解决的问题（git worktree 物理隔离）。

---

### 附录：工具清单速查

| 工具名 | handler | 来源 | 用途 |
|--------|---------|------|------|
| bash | run_bash | s02 | shell 命令 |
| read_file | run_read | s02 | 读文件 |
| write_file | run_write | s02 | 写文件 |
| edit_file | run_edit | s02 | 替换编辑 |
| TodoWrite | TODO.update | s03 | 内存待办 |
| task | run_subagent | s04 | 一次性子 agent |
| load_skill | SKILLS.load | s05 | 加载技能包 |
| compress | auto_compact | s06 | 手动压缩 |
| background_run | BG.run | s08 | 后台命令 |
| check_background | BG.check | s08 | 查询后台任务 |
| task_create | TASK_MGR.create | s07 | 创建持久任务 |
| task_get | TASK_MGR.get | s07 | 查询任务详情 |
| task_update | TASK_MGR.update | s07 | 更新任务状态/依赖 |
| task_list | TASK_MGR.list_all | s07 | 列出所有任务 |
| spawn_teammate | TEAM.spawn | s09 | 创建队友 |
| list_teammates | TEAM.list_all | s09 | 列出队友 |
| send_message | BUS.send | s09 | 发消息 |
| read_inbox | BUS.read_inbox | s09 | 读收件箱 |
| broadcast | BUS.broadcast | s09 | 广播 |
| shutdown_request | handle_shutdown_request | s10 | 请求关机 |
| plan_approval | handle_plan_review | s10 | 审批计划 |
| idle | "Lead does not idle" | s09 | lead 不进入空闲 |
| claim_task | TASK_MGR.claim | s11 | 认领任务 |

---

## 多智能体迭代框架精要

**触发词：** 用户说"多智能体框架"、"生成器+评估器"、"迭代开发"等

---

### 执行约束（强制）

#### 协调器禁止事项
- 禁止直接使用 Edit/Write 工具修改代码
- 禁止跳过 Agent 工具直接执行任务
- 禁止在评估通过前结束迭代

#### 协调器强制行为
- 必须使用 Agent 工具启动子代理
- 必须等待子代理返回结果
- 必须基于反馈进行下一轮迭代

#### Claude Code 工具映射

| 框架概念 | Claude Code 工具 |
|----------|------------------|
| `subagent(prompt)` | `Agent` 工具 |
| **研究者** | `Agent(subagent_type="Explore")` |
| **实现者** | `Agent(subagent_type="general-purpose")` |
| **测试者** | `Agent(subagent_type="general-purpose")` |
| **评估者** | `Agent(subagent_type="general-purpose")` |

**测试者 vs 评估者：**
- 测试者：主动执行测试，输出报告
- 评估者：阅读报告，决定 PASS/CONTINUE

---

### 核心原则：动态发现

框架**不硬编码**智能体/技能，而是**动态读取 CLAUDE.md 索引**：

1. 读取索引区域，发现可用资源
2. 根据任务需求匹配智能体/技能
3. 优先使用索引中的现成资源
4. 仅在索引无匹配时才自定义提示词

**索引资源示例：**
- `Superpowers` - 核心工作流技能
- 其他插件技能

---

### 目录结构

```
docs/agents/
├── README.md           # 框架说明
├── workflows/          # 工作流配置 (YAML)
└── prompts/            # 提示词模板 (MD)
```

---

### 工作流配置 (YAML)

**必填字段：**
```yaml
name: "工作流名称"
max_iterations: 12

agents:
  - id: generator
    role: "生成器"
    trigger: "odd"           # "odd" | "even" | [0,3,6]
    prompt_template: |
      任务: {{task_description}}
      反馈: {{last_feedback}}

  - id: evaluator
    role: "评估器"
    trigger: "even"
    criteria:
      - "功能性"
      - "代码质量"
    prompt_template: |
      标准: {{criteria}}
      改动: {{changes}}
      通过输出: <promise>PASS</promise>

completion:
  signal: "<promise>PASS</promise>"
  max_iterations_reached: "report"
```

**触发规则：**
- `odd` → 奇数轮（第 1,3,5...）
- `even` → 偶数轮（第 2,4,6...）
- `[0,3,6]` → 指定轮次

---

### 预置工作流

| 名称 | 智能体 | 场景 |
|------|--------|------|
| **researcher-implementer-tester-evaluator** | 研究者 + 实现者 + 测试者 + 评估者 | **标准四角色迭代（推荐）** |
| generator-evaluator | 生成器 + 评估器 | 简单代码 + 快速审核 |
| writer-editor | 作者 + 编辑 | 文档 + 审核 |
| coder-reviewer-tester | 程序员 + 审核者 + 测试者 | TDD（无独立评估者） |

---

### 四角色迭代流程

#### 标准角色

| 角色 | 职责 | 权限 | 输出 |
|------|------|------|------|
| **研究者** | 分析需求、探索代码库、**产出验收标准** | 只读 | 分析报告 + 验收标准 |
| **实现者** | 编写/修改代码 | 可编辑 | 代码改动 |
| **测试者** | 写 .tape 脚本、跑 build、运行测试、**产出报告+截图** | 只读+运行 | 测试报告 + 截图 |
| **评估者** | 审查报告和截图（**必须多模态**）、决定结果 | 只读 | PASS/CONTINUE |

#### 流程图

```
迭代 N：
  1. 研究者 → 分析需求 + 产出验收标准（只读）
  2. 实现者 → 编写代码（可编辑）
  3. 测试者 → 写 .tape 脚本、跑 build、截图、填报告
  4. 评估者 → 审查报告和截图（多模态），决定 PASS/CONTINUE
       ↑
   评估者是守门员，在最后
```

#### 循环规则

当评估者返回 **CONTINUE** 时：

| 问题类型 | 循环起点 | 说明 |
|----------|----------|------|
| 代码 Bug | 实现者 | 小问题，直接修复 |
| 需求理解错误 | 研究者 | 大问题，重新分析 |
| **默认** | **实现者** | 大多数情况 |

**评估者职责强调（最严格模式）：**
- 评估者是**守门员**，在流程最后
- 必须**基于测试者的报告**判断，不能跳过测试
- **最严格原则：有任何问题都必须 CONTINUE，不能放过**
- 只有**零问题**才能输出 PASS
- CONTINUE 时必须说明问题类型，决定循环起点

---

### 产出物契约

#### 测试产出物目录

```
docs/test-artifacts/
├── templates/
│   ├── acceptance-criteria.md   # 研究者产出（功能点、交互路径、UI 预期、边界情况）
│   └── report.md                # 测试者产出（构建状态、截图清单、验收检查）
├── screenshots/                 # VHS 截图存放处
├── test-{N}.tape                # 每轮迭代的 .tape 脚本
└── report-{N}.md                # 每轮迭代的测试报告
```

#### 信息流

```
研究者(验收标准) → 测试者(.tape + 截图 + 报告) → 评估者(多模态判断)
```

#### Git 工作流

| 阶段 | Git 操作 |
|------|----------|
| 实现者修改前 | **必须先 commit**（保存当前状态，方便回溯） |
| 评估者 PASS | **必须 commit**（放行，保存通过状态） |
| 评估者 CONTINUE | **不 commit**，直接进入下一轮迭代 |
| 全部通过后 | 最终 commit 包含完整改动 |

#### 评估者多模态要求

- 评估者必须能**查看截图**（Read 工具读取 PNG）
- 判断依据：测试报告文字 + VHS 截图视觉证据
- UI 相关验收标准**必须**对照截图判断，不能仅看文字报告

#### VHS 集成（测试者角色）

- 测试者使用 VHS 执行 .tape 脚本，生成终端截图
- 截图路径约定：`docs/test-artifacts/screenshots/{name}-{iteration}.png`
- 容器环境必须设置 `VHS_NO_SANDBOX=true`

---

### CLAUDE.md 索引配置

```markdown
- [Multi-Agent Framework](./docs/agents/README.md) - 多智能体迭代框架（四角色迭代：研究者→实现者→测试者→评估者）
```

---

### 使用方式

**直接描述：**
```
我需要一个生成器和评估器，迭代10次实现xxx功能
```

**引用配置：**
```
用 generator-evaluator.yaml 实现 xxx 功能
```

---

### 重建清单

创建框架时需要创建的文件：
1. `docs/agents/README.md` - 框架说明
2. `docs/agents/workflows/generator-evaluator.yaml`
3. `docs/agents/workflows/writer-editor.yaml`
4. `docs/agents/workflows/coder-reviewer-tester.yaml`
5. `docs/agents/prompts/generator.md`
6. `docs/agents/prompts/evaluator.md`
7. `docs/agents/prompts/writer.md`
8. `docs/agents/prompts/editor.md`
9. `docs/agents/prompts/coder.md`
10. `docs/agents/prompts/reviewer.md`
11. `docs/agents/prompts/tester.md`

---

### 实战验证（2026-03-27）

**任务：** 修复 ai-terminal 中 Thinking/工具调用/文件编辑不可见问题

**迭代记录：**
```
研究者 (分析 6 个根因 + 验收标准)
  → 实现者 (第一轮：启用 Thinking + 工具进度 + 中间持久化)
    → 测试者 (PASS：705/739 通过，34 失败均为预已存在)
      → 评估者 (CONTINUE：发现 tool_result 角色错误 + redacted_thinking 缺失)
        → 实现者 (第二轮：toAnthropicMessageParams 拆分 + RedactedThinkingPart)
          → 评估者 (PASS)
```

**关键发现：**
- 评估者捕获了测试者和实现者都遗漏的 **API 协议合规性问题**（`tool_result` 必须属于 `user` 角色）
- 证实了"评估者做守门员"的价值——协议层面的错误只有全局审查才能发现
- 工具循环中 `resetStreaming()` 导致 UI 空白是最核心的根因

**改进方向：**
- 测试者应补充 API 协议合规性检查（不仅看 build/test）
- 评估者应增加"历史消息回传 API"场景的验证

---

## 摘要：Learn Claude Code（s01-s12）

### 四阶段递进关系

#### 阶段一：核心循环（s01-s02）

建立 Agent 的基本骨架——while 循环 + 工具分发。

- **s01 Agent 循环**：整个 Agent 的核心秘密就是 `while stop_reason == "tool_use"` 循环，把工具执行结果喂回模型直到模型自己决定停下来。
- **s02 工具调用**：不改循环本身，用 `TOOL_HANDLERS` 字典（dispatch map）路由多个工具（bash/read/write/edit），实现可扩展的工具注册。

#### 阶段二：状态与压缩（s03-s06）

让 Agent 能处理长任务——追踪进度、管理上下文、按需加载知识。

- **s03 进度追踪**：引入 `TodoWrite` 工具让模型自己维护任务列表，解决"忘了要做什么"的问题。
- **s04 子代理**：给子任务一个全新的 `messages=[]`，子代理干完只返回摘要，父代理的上下文保持干净。通过工具层面移除 `task` 工具防止无限递归。
- **s05 技能按需加载**：两层设计——系统提示词只放 name+description（约 100 tokens/技能），模型选择后再加载完整内容，避免全部塞进 prompt 又贵又慢。
- **s06 上下文压缩**：三层压缩策略——micro_compact（每轮静默替换旧结果）、auto_compact（超阈值时保存+LLM摘要+替换）、compact 工具（模型主动触发）。

#### 阶段三：持久化与协作（s07-s10）

让 Agent 能跨会话工作、后台运行、团队协作。

- **s07 任务系统**：任务持久化到 `.tasks/` 目录（JSON 文件），支持 blockedBy 依赖关系，Agent 崩溃后可恢复。
- **s08 后台任务**：`threading.Thread` 让子代理在后台运行，父代理通过 `task_output` 文件获取结果，实现并行执行。
- **s09 团队协作**：持久化的 teammate（非临时子代理），每个 teammate 有独立对话历史和双向消息通道。
- **s10 团队协议**：用自然语言定义团队规则（谁来做什么、什么顺序），模型按协议协调多个 teammate。

#### 阶段四：自治与隔离（s11-s12）

让 Agent 系统真正自主运行。

- **s11 自主代理**：任务看板 + 空闲轮询 + 自动认领，空闲 teammate 每隔几秒检查 `.tasks/` 目录，看到没人认领的任务就自己拿来做。
- **s12 工作树隔离**：用 `git worktree` 为每个 teammate 创建独立的工作目录，避免多人同时修改同一文件时的冲突。

### 关键数据

- 12 个脚本，总计 3194 行源材料
- s01 核心循环仅约 150 行 Python，之后每个脚本增加约 200 行
- 子代理上限 30 轮（安全防护）
- 上下文压缩阈值 50000 tokens
- 保留最近 3 条工具结果（micro_compact）

### 引用索引

[^0]: [[sources/agent/learn-claude-code/s01-Agent循环]] `L1-10` — 核心问题和解决方案概述
[^1]: [[sources/agent/learn-claude-code/s01-Agent循环]] `L9-19` — Agent Loop 的 while 循环模式
[^2]: [[sources/agent/learn-claude-code/s02-工具调用]] `L1-20` — 工具分发表的设计
[^3]: [[sources/agent/learn-claude-code/s03-进度追踪]] `L1-30` — TodoWrite 进度追踪机制
[^4]: [[sources/agent/learn-claude-code/s04-子代理]] `L1-30` — 子代理的独立上下文设计
[^5]: [[sources/agent/learn-claude-code/s05-技能系统]] `L1-20` — 两层按需加载架构
[^6]: [[sources/agent/learn-claude-code/s06-上下文压缩]] `L1-45` — 三层压缩策略
[^7]: [[sources/agent/learn-claude-code/s17-自主代理]] `L1-30` — 任务持久化设计（概念融入自主代理章节）
[^8]: [[sources/agent/learn-claude-code/s13-后台任务]] `L1-30` — 后台线程执行
[^9]: [[sources/agent/learn-claude-code/s15-Agent团队]] `L1-30` — 持久化 teammate 设计
[^10]: [[sources/agent/learn-claude-code/s16-团队协议]] `L1-30` — 自然语言团队协议
[^11]: [[sources/agent/learn-claude-code/s17-自主代理]] `L1-30` — 任务看板与自动认领
[^12]: [[sources/agent/learn-claude-code/s18-Worktree隔离]] `L1-30` — git worktree 隔离机制

---

## 摘要：Claude Code System Prompts

### 六大分类及职责

#### 1. System Prompts（约 101 个）

主系统提示词的各个片段，定义 Claude Code 的核心行为准则。

- **行为约束**：不做不必要的添加、不做过早抽象、不给时间估计、安全第一（防注入/XSS）
- **任务执行**：读代码再改、编辑优于创建、被阻塞时换思路、大胆接受复杂任务
- **子代理管理**：Fork 使用指南、子代理委派示例、子代理 prompt 编写指南
- **记忆系统**：Agent 记忆指令、记忆文件选择、会话记忆更新、记忆描述
- **模式控制**：Plan 模式（5 阶段）、Auto 模式、Minimal 模式、Learning 模式

#### 2. Tool Descriptions（约 75 个）

内置工具的声明和使用说明，是模型理解"我能做什么"的关键。

- **核心工具**：Bash、Read、Write、Edit、Glob、Grep
- **高级工具**：Agent（子代理）、AskUserQuestion、EnterPlanMode、TodoWrite
- **团队工具**：TeammateTool、SendMessage、TeamDelete
- **Bash 子规则**：sandbox 策略（约 20 个片段）、git 操作规范、sleep 使用规则

#### 3. Agent Prompts（约 34 个）

子代理和 AI 驱动的工具函数，各自独立的系统提示词。

- **子代理**：Explore（只读搜索）、Plan（规划）、General-purpose（通用）
- **创建助手**：Agent 创建架构师、CLAUDE.md 创建、Statusline 配置
- **Slash 命令**：/batch、/pr-comments、/review-pr、/schedule、/security-review
- **AI 工具函数**：会话摘要、标题生成、安全监控、验证专家、WebFetch 摘要

#### 4. Data（约 27 个）

嵌入的参考文档模板，按需注入提供 API 知识。

- **SDK 参考**：Python/TypeScript Agent SDK、Claude API（6 种语言 + cURL）
- **概念文档**：Tool Use 概念、Streaming 参考、Prompt Caching 设计

#### 5. Skills（约 13 个）

内置技能的完整 prompt，用于专业化任务。

- **核心技能**：/init（CLAUDE.md 创建）、/loop、/stuck、/simplify
- **API 构建技能**：Claude API 构建指南 + 语言参考模板
- **验证技能**：Verify skill + CLI/Server 验证示例
- **配置技能**：update-config 7 步验证流

#### 6. System Reminders（约 40 个）

运行时动态注入的通知片段，报告环境状态变化。

- **文件事件**：文件被外部修改、文件为空、文件被截断
- **模式切换**：Plan 模式激活/退出、Session 延续、技能调用
- **团队协调**：Team Coordination、Team Shutdown
- **工具提醒**：Task tools reminder、Token usage、USD budget

### 提示词格式

每个提示词文件采用统一的 YAML frontmatter + Markdown body 格式：

```
<!--
name: '提示词名称'
description: 一行描述
ccVersion: 2.1.84
variables:                    ← 模板变量（运行时替换）
  - TOOL_NAME
agentMetadata:                ← 子代理元数据（仅 agent 类型有）
  agentType: 'Explore'
  model: 'haiku'
  disallowedTools: [...]      ← 禁用的工具列表
-->
（Markdown 格式的提示词正文，含 ${VARIABLE} 模板变量）
```

关键设计特点：
- **条件组合**：不是单一长字符串，而是运行时按环境/配置动态拼接
- **模板变量**：`${GLOB_TOOL_NAME}` 等占位符在运行时替换为实际工具名
- **版本追踪**：每个片段标注 `ccVersion`，可通过 CHANGELOG 追踪变更

### 引用索引

[^1]: [[sources/claude-code/claude-code-system-prompts/README]] `L30-57` — 多系统提示词的设计理由和构成
[^2]: [[sources/claude-code/claude-code-system-prompts/README]] `L169-181` — 行为约束类 system prompt
[^3]: [[sources/claude-code/claude-code-system-prompts/README]] `L169-181` — 任务执行类 system prompt
[^4]: [[sources/claude-code/claude-code-system-prompts/README]] `L182-184` — Fork 和子代理管理
[^5]: [[sources/claude-code/claude-code-system-prompts/README]] `L160-164` — 记忆系统相关 prompt
[^6]: [[sources/claude-code/claude-code-system-prompts/README]] `L192-194` — 模式控制相关 prompt
[^7]: [[sources/claude-code/claude-code-system-prompts/README]] `L265-289` — 核心工具描述
[^8]: [[sources/claude-code/claude-code-system-prompts/README]] `L271-287` — 高级工具描述
[^9]: [[sources/claude-code/claude-code-system-prompts/README]] `L280-286` — 团队工具描述
[^10]: [[sources/claude-code/claude-code-system-prompts/README]] `L269-339` — Bash sandbox 子规则
[^11]: [[sources/claude-code/claude-code-system-prompts/README]] `L78-79` — Explore 和 Plan 子代理
[^12]: [[sources/claude-code/claude-code-system-prompts/README]] `L83-86` — 创建助手类
[^13]: [[sources/claude-code/claude-code-system-prompts/README]] `L89-94` — Slash 命令类
[^14]: [[sources/claude-code/claude-code-system-prompts/README]] `L97-119` — AI 工具函数类
[^15]: [[sources/claude-code/claude-code-system-prompts/README]] `L127-136` — SDK 参考类
[^16]: [[sources/claude-code/claude-code-system-prompts/README]] `L147-153` — 概念文档类
[^17]: [[sources/claude-code/claude-code-system-prompts/README]] `L350-353` — 核心技能
[^18]: [[sources/claude-code/claude-code-system-prompts/README]] `L353-354` — API 构建技能
[^19]: [[sources/claude-code/claude-code-system-prompts/README]] `L355-358` — 验证技能
[^20]: [[sources/claude-code/claude-code-system-prompts/README]] `L362-363` — 配置技能
[^21]: [[sources/claude-code/claude-code-system-prompts/README]] `L229-235` — 文件事件类
[^22]: [[sources/claude-code/claude-code-system-prompts/README]] `L250-254` — 模式切换类
[^23]: [[sources/claude-code/claude-code-system-prompts/README]] `L257-258` — 团队协调类
[^24]: [[sources/claude-code/claude-code-system-prompts/README]] `L256-262` — 工具提醒类
[^25]: [[sources/claude-code/claude-code-system-prompts/system-prompts/agent-prompt-explore]] `L1-27` — frontmatter 格式示例
[^26]: [[sources/claude-code/claude-code-system-prompts/README]] `L49-55` — 条件组合设计
[^27]: [[sources/claude-code/claude-code-system-prompts/system-prompts/agent-prompt-explore]] `L48-49` — 模板变量示例
[^28]: [[sources/claude-code/claude-code-system-prompts/README]] `L37-39` — 版本追踪说明
