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
