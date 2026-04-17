---
type: wiki
name: s_full_architecture
description: s_full.py 完整底层逻辑中文文档，按执行顺序组织，整合 s01-s11 所有机制
updated: 2026-04-10
tags: [agent, harness, python, architecture]
status: deprecated
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# s_full.py 底层逻辑全景

> 内容已迁移至 [[wiki/agent-arch-p1.md|agent-arch-p1]]。

## 1. 概述

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

## 2. 启动初始化

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

## 3. REPL 主循环

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

## 4. agent_loop 内部流程

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

## 5. 六大支撑机制

### 5.1 base_tools（基础工具）

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

### 5.2 Subagent（子 agent 闭环）

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

### 5.3 TodoManager（内存待办列表）

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

### 5.4 TaskManager（文件持久化任务板）

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

### 5.5 BackgroundManager（后台线程执行）

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

### 5.6 TeammateManager + MessageBus（持久自主 agent）

这是最复杂的机制，涉及两个类协作：

#### MessageBus（消息总线）

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

#### TeammateManager（队友管理器）

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

## 6. 协作模式

### Lead Agent（主 agent）

- 拥有全部 25 个工具
- 运行在 REPL 主线程
- 负责：接收用户指令、分解任务、分配工作、审批计划

### Teammate（队友 agent）

- 只有 7 个工具
- 运行在独立 daemon 线程
- 生命周期：spawn → work → idle → auto-claim → work → ... → shutdown
- 通过 MessageBus 与 lead 通信
- 空闲时自动扫描任务板认领任务

### 通信拓扑

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

### Shutdown 协议

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

### Plan 审批协议

```
teammate 发送 plan_approval 消息给 lead
  │
  └── lead 调用 plan_approval(request_id, approve, feedback)
      ├── 更新 plan_requests 状态
      └── 发送 plan_approval_response 消息给 teammate
```

---

## 7. s_full vs s12 对比

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

## 附录：工具清单速查

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
