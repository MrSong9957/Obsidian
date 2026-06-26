---
name: s15 Agent 团队
description: 子 agent 适合一次性委派；团队系统解决长期在线、能继续接活、能互相协作
type: concept
updated: 2026-05-07
tags: agent, learn-shareai, s15
source: https://learn.shareai.run/zh/s15/
---

> _子 agent 适合一次性委派；团队系统解决的是"有人长期在线、能继续接活、能互相协作"。_

## 这一章要解决什么问题

`s04` 的 subagent 已经能帮主 agent 拆小任务。但 subagent 有一个很明显的边界：

```
创建 -> 执行 -> 返回摘要 -> 消失
```

可如果你想让一个测试 agent 长期待命、让两个 agent 长期分工，就不够用了。

系统现在缺的是：

**一批有身份、能长期存在、能反复协作的队友。**

## 最小心智模型

```
lead
|
+-- spawn alice (coder)
+-- spawn bob (tester)
|
+-- send message --> alice inbox
+-- send message --> bob inbox

alice
|
+-- 自己的 messages
+-- 自己的 inbox
+-- 自己的 agent loop

bob
|
+-- 自己的 messages
+-- 自己的 inbox
+-- 自己的 agent loop
```

和 `s04` 的最大区别是：**subagent 是一次性执行单元，teammate 是长期存在的协作成员。**

## 关键数据结构

### 1. TeamMember

```python
member = {
    "name": "alice",
    "role": "coder",
    "status": "working",
}
```

### 2. TeamConfig

```python
config = {
    "team_name": "default",
    "members": [member1, member2],
}
# 通常放在 .team/config.json
```

### 3. MessageEnvelope

```python
message = {
    "type": "message",
    "from": "lead",
    "content": "Please review auth module.",
    "timestamp": 1710000000.0,
}
```

## 最小实现

### 第一步：先有一份队伍名册

```python
class TeammateManager:
    def __init__(self, team_dir: Path):
        self.team_dir = team_dir
        self.config_path = team_dir / "config.json"
        self.config = self._load_config()
```

### 第二步：spawn 一个持久队友

```python
def spawn(self, name: str, role: str, prompt: str):
    member = {"name": name, "role": role, "status": "working"}
    self.config["members"].append(member)
    self._save_config()
    thread = threading.Thread(
        target=self._teammate_loop,
        args=(name, role, prompt),
        daemon=True,
    )
    thread.start()
```

### 第三步：给每个队友一个邮箱

JSONL 文件，发消息时追加一行，收消息时读取全部并清空。

### 第四步：队友每轮先看邮箱，再继续工作

**队友不是靠"被重新创建"来获得新任务，而是靠"下一轮先检查邮箱"来接收新工作。**

## Teammate、Subagent、Runtime Task 怎么区分

| 机制 | 更像什么 | 生命周期 | 关键边界 |
|---|---|---|---|
| subagent | 一次性外包助手 | 干完就结束 | 隔离一小段探索性上下文 |
| runtime task | 正在运行的后台执行槽位 | 跑完或取消就结束 | 慢任务稍后回来 |
| teammate | 长期在线队友 | 可以反复接任务 | 有名字、有邮箱、有独立循环 |

## 一句话记住

**teammate 的核心不是"多一个模型调用"，而是"多一个长期存在的执行者"。**

---

# 新版 Agent 团队

在 AI Agent 的架构演进中，**Agent Teams（多 Agent 团队协作）** 是为了解决“单个 Agent 注意力与上下文窗口有限，无法独自处理大型复杂项目”而设计的核心机制。

它将单一的大型任务拆分，由多个各司其职、相互协作的专用 Agent 共同完成。

---

## 核心痛点：为什么单个 Agent 不够用？

当面对类似“重构整个后端系统”这样的大型任务时，单个 Agent 会遭遇两个瓶颈：

1. **注意力分散：** 任务涉及认证模块、数据库、API 路由和测试，Agent 在修改 API 路由时，认证模块的细节可能已经超出了它的记忆范围。
2. **轻量级子 Agent 的局限：** 普通的子 Agent（Subagent）通常是“临时工”，完成单次调用后就被销毁，无法进行持续、并行的多轮异步协作。

---

## 架构设计：Agent Teams 是如何工作的？

以 [Learn Claude Code](https://learn.shareai.run/zh/s15/) 披露的团队设计为例，其核心不再依赖高成本的“共享内存”，而是采用了一套类似人类职场交互的**异步通信与管理机制**。

### 1. 核心纽带：MessageBus（文件收件箱）

团队中的每个 Agent（无论是 Lead 还是普通队友）在本地都有一个专属的收件箱文件（如 `~/.claude/teams/{team}/inboxes/{agent_name}.json`）。

* **发消息：** 向对方的收件箱文件中追加（Append）一行结构化 JSON 数据。
* **读消息：** 定期轮询并读取属于自己的文件内容，读完后即消费删除。
* **消息类型：** 包含文本、空闲通知（`idle_notification`）、审批请求（`permission_request`）、关机请求（`shutdown_request`）等 15 种类型。

### 2. 角色分工与队友启动 (`spawn_teammate`)

通常由一个 **Lead（主导者）** 负责全局规划，并通过特定的工具动态克隆、启动不同的**队友线程/进程**（如前端、后端、测试等）：

* 队友运行在各自独立的 Daemon 线程或 Tmux 窗格中，拥有隔离的上下文和**简化的工具集**（例如只允许其读写文件和使用 Bash）。
* 队友通常运行在一个“空闲循环（Idle Loop）”中，完成一轮工作后发通知挂起，等待新消息唤醒，直至收到关机指令。

### 3. Lead 消息注入与并发

* **并行工作：** Lead 可以同时派发任务给 Alice（建数据库）和 Bob（写 API 客户端），两者互不干扰，并行推进。
* **异步轮询：** Lead 的后台程序（如 `useInboxPoller`）会以秒级频率轮询自己的收件箱。一旦队友发回结果，该消息就会被自动包装成 XML 标签（如 `<teammate-message>`）注入到 Lead 下一轮的对话上下文中，由 Lead 统一汇总。

### 4. 权限冒泡机制 (Permission Bubbling)

这是团队多 Agent 设计中非常安全的一环。由于队友工具集可能涉及高危操作（如执行 `bash` 命令），但它们又无法直接与人类用户交互，因此衍生出冒泡机制：

1. **队友**遇到需要审批的操作时，向 **Lead** 发送 `permission_request`。
2. **Lead** 的轮询器捕获请求，在终端 UI 上为人类用户弹出带有对应队友颜色标记的审批对话框。
3. 用户审批后，**Lead** 将结果通过 `permission_response` 发回。
4. **队友**检测到允许信号，继续执行。

---

## 总结：子 Agent vs 团队队友

| 特性 | 子 Agent (Subagent) | 团队队友 (Teammate) |
| --- | --- | --- |
| **生命周期** | 一次性，用完即销毁 | 长期驻留，多轮交互（直至关机） |
| **通信机制** | 同步阻塞，只在结束时回传结论 | 异步收件箱，随时双向通信 |
| **上下文** | 完全隔离 | 通过消息线索有选择地共享信息 |
| **协作模式** | 树状垂直调用 | 网状平级或星状协同 |

通过这种“文件收件箱 + 独立线程轮询 + 权限冒泡”的轻量化架构，Agent Teams 成功把一个爆满的超长上下文拆分到了多个并行的专用 Agent 中，从而实现了复杂工程的自动化推进。

---

MessageBus: 文件收件箱
每个 Agent（包括 Lead 和队友）有一个 .jsonl 邮箱。发消息 = 往对方的文件里 append 一行 JSON。读消息 = 读文件 + 删除（消费式）：

```python
class MessageBus:
    def send(self, from_agent: str, to_agent: str,
             content: str, msg_type: str = "message"):
        msg = {"from": from_agent, "to": to_agent,
               "content": content, "type": msg_type,
               "ts": time.time()}
        inbox = MAILBOX_DIR / f"{to_agent}.jsonl"
        with open(inbox, "a") as f:
            f.write(json.dumps(msg) + "\n")

    def read_inbox(self, agent: str) -> list[dict]:
        inbox = MAILBOX_DIR / f"{agent}.jsonl"
        if not inbox.exists():
            return []
        msgs = [json.loads(line) for line in inbox.read_text().splitlines()]
        inbox.unlink()  # 消费式：读完删除
        return msgs
```

为什么用文件而不是内存队列？教学版选文件是因为直观、跨线程可观察。真实 CC 也用文件收件箱（~/.claude/teams/{team}/inboxes/），但加了 proper-lockfile 防并发写冲突。教学版的 read_inbox 有 read + unlink 竞态，多线程同时读可能丢消息，对教学场景可以接受。

spawn_teammate_thread: 启动队友
Lead 调用 spawn_teammate 工具启动一个队友。队友跑在自己的 daemon 线程里，有自己的 system prompt、自己的 messages、自己的简化工具集：

```python
def spawn_teammate_thread(name: str, role: str, prompt: str) -> str:
    system = f"You are '{name}', a {role}. Use tools to complete tasks."

    def run():
        messages = [{"role": "user", "content": prompt}]
        sub_tools = [bash, read_file, write_file, send_message]
        for _ in range(10):           # 最多 10 轮
            inbox = BUS.read_inbox(name)
            if inbox:
                messages.append({"role": "user",
                    "content": f"<inbox>{json.dumps(inbox)}</inbox>"})
            response = client.messages.create(
                model=MODEL, system=system, messages=messages[-20:],
                tools=sub_tools, max_tokens=8000)
            # ... 执行工具、处理结果
        # 完成后发 summary 给 Lead
        BUS.send(name, "lead", summary, "result")

    threading.Thread(target=run, daemon=True).start()
```

关键设计：

队友有简化工具集：bash、read、write、send_message。教学版省略了任务和 cron，聚焦通信机制。真实 CC 的队友也有 TaskCreate、TaskUpdate 等工具，任务系统是团队共享的
教学版限 10 轮：防止队友无限循环。真实 CC 用 idle loop：跑完一轮后发 idle_notification，等 inbox 消息，收到后继续，直到 shutdown_request 才退出
完成后自动汇报：BUS.send(name, "lead", summary) 把最终结果发到 Lead 的收件箱
Lead 的 inbox 注入
Lead 在每轮主循环结束后检查收件箱。队友发来的消息注入到 history 里，让 LLM 能看到并做出反应：

```python
# 主循环结束后
inbox = BUS.read_inbox("lead")
if inbox:
    inbox_text = "\n".join(
        f"From {m['from']}: {m['content'][:200]}" for m in inbox)
    history.append({"role": "user",
                    "content": f"[Inbox]\n{inbox_text}"})
```
教学版在用户输入循环外注入。CC 更精细，Lead 的 useInboxPoller 每 1 秒检查一次，有消息就提交为新的 turn，不需要等用户输入。