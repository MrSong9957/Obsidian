---
name: s18 Worktree 隔离
description: 任务板解决做什么，worktree 解决在哪做而不互相踩到
type: concept
updated: 2026-05-07
tags: agent, learn-shareai, s18
source: https://learn.shareai.run/zh/s18/
---

> _任务板解决"做什么"，worktree 解决"在哪做而不互相踩到"。_

## 这一章要解决什么问题

到 `s17` 为止，系统已经可以拆任务、认领任务、让多个 agent 并行推进不同工作。但如果所有人都在同一个工作目录里改文件，很快就会出现互相污染的问题。

任务系统已经回答了"谁做什么"，却还没有回答：

**每个任务应该在哪个独立工作空间里执行。**

## 最小心智模型

```
任务板
  负责回答：做什么、谁在做、状态如何

worktree 注册表
  负责回答：在哪做、目录在哪、对应哪个任务

两者通过 task_id 连起来
```

**任务记录工作目标，worktree 记录执行车道。**

## 关键数据结构

### 1. TaskRecord（扩展）

```python
task = {
    "id": 12,
    "subject": "Refactor auth flow",
    "status": "in_progress",
    "owner": "alice",
    "worktree": "auth-refactor",
    "worktree_state": "active",
    "last_worktree": "auth-refactor",
    "closeout": None,
}
```

### 2. WorktreeRecord

```python
worktree = {
    "name": "auth-refactor",
    "path": ".worktrees/auth-refactor",
    "branch": "wt/auth-refactor",
    "task_id": 12,
    "status": "active",
    "last_entered_at": 1710000000.0,
    "last_command_at": 1710000012.0,
    "last_command_preview": "pytest tests/auth -q",
    "closeout": None,
}
```

worktree 记录回答的不只是"目录在哪"，还回答最近什么时候进入过、最近跑过什么命令、最后是怎么收尾的。

### 3. CloseoutRecord

```python
closeout = {
    "action": "keep",
    "reason": "Need follow-up review",
    "at": 1710000100.0,
}
```

### 4. EventRecord

```python
event = {
    "event": "worktree.closeout.keep",
    "task_id": 12,
    "worktree": "auth-refactor",
    "reason": "Need follow-up review",
    "ts": 1710000100.0,
}
```

## 最小实现

### 第一步：先有任务，再有 worktree

```python
task = tasks.create("Refactor auth flow")
worktrees.create("auth-refactor", task_id=task["id"])
```

### 第二步：创建 worktree 并写入注册表

```python
def create(self, name: str, task_id: int):
    path = self.root / ".worktrees" / name
    branch = f"wt/{name}"
    run_git(["worktree", "add", "-b", branch, str(path), "HEAD"])
    record = {
        "name": name,
        "path": str(path),
        "branch": branch,
        "task_id": task_id,
        "status": "active",
    }
    self.index["worktrees"].append(record)
    self._save_index()
```

### 第三步：同时更新任务记录

```python
def bind_worktree(task_id: int, name: str):
    task = tasks.load(task_id)
    task["worktree"] = name
    task["last_worktree"] = name
    task["worktree_state"] = "active"
    if task["status"] == "pending":
        task["status"] = "in_progress"
    tasks.save(task)
```

### 第四步：显式进入车道，再在对应目录里执行命令

```python
worktree_enter("auth-refactor")
worktree_run("auth-refactor", "pytest tests/auth -q")
```

隔离的核心：**同一个命令，在不同 `cwd` 里执行，影响范围就不一样。**

### 第五步：收尾时显式走 worktree_closeout

```python
worktree_closeout(
    name="auth-refactor",
    action="keep",  # or "remove"
    reason="Need follow-up review",
    complete_task=False,
)
```

## 为什么 worktree_state 和 status 要分开

任务 `status` 回答：这件工作现在是 `pending`、`in_progress` 还是 `completed`
`worktree_state` 回答：这条执行车道现在是 `active`、`kept`、`removed` 还是 `unbound`

典型例子：任务已经 completed，但 worktree 仍然 kept（保留目录给 reviewer 看）。

## 一句话记住

**任务系统管"做什么"，worktree 系统管"在哪做且互不干扰"。**
