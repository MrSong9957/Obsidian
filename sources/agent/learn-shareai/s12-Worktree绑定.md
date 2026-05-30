---
name: s12 Worktree 绑定
description: 各干各的目录, 互不干扰——任务管目标, worktree 管目录（旧版）
type: concept
updated: 2026-05-07
tags: agent, learn-shareai, s12
source: https://learn.shareai.run/zh/s12/
---

> _"各干各的目录, 互不干扰"_ -- 任务管目标, worktree 管目录, 按 ID 绑定。

## 问题

到 s11, 智能体已经能自主认领和完成任务。但所有任务共享一个目录。两个智能体同时重构不同模块 -- A 改 `config.py`, B 也改 `config.py`, 未提交的改动互相污染, 谁也没法干净回滚。

任务板管 "做什么" 但不管 "在哪做"。解法: 给每个任务一个独立的 git worktree 目录, 用任务 ID 把两边关联起来。

## 解决方案

```
Control plane (.tasks/)              Execution plane (.worktrees/)
+------------------+                 +------------------------+
| task_1.json      |                 | auth-refactor/         |
| status: in_progress <------> branch: wt/auth-refactor  |
| worktree: "auth-refactor" |        | task_id: 1             |
+------------------+                 +------------------------+
| task_2.json      |                 | ui-login/              |
| status: pending  <------> branch: wt/ui-login        |
| worktree: "ui-login" |             | task_id: 2             |
+------------------+                 +------------------------+
```

## 工作原理

1. **创建任务。** 先把目标持久化。

```python
TASKS.create("Implement auth refactor")
# -> .tasks/task_1.json status=pending worktree=""
```

2. **创建 worktree 并绑定任务。**

```python
WORKTREES.create("auth-refactor", task_id=1)
# -> git worktree add -b wt/auth-refactor .worktrees/auth-refactor HEAD
# -> index.json gets new entry, task_1.json gets worktree="auth-refactor"
```

3. **在 worktree 中执行命令。** `cwd` 指向隔离目录。

4. **收尾。** 两种选择: `worktree_keep(name)` 或 `worktree_remove(name, complete_task=True)`。

5. **事件流。** 每个生命周期步骤写入 `.worktrees/events.jsonl`。

崩溃后从 `.tasks/` + `.worktrees/index.json` 重建现场。

## 相对 s11 的变更

| 组件 | 之前 (s11) | 之后 (s12) |
| --- | --- | --- |
| 协调 | 任务板 (owner/status) | 任务板 + worktree 显式绑定 |
| 执行范围 | 共享目录 | 每个任务独立目录 |
| 可恢复性 | 仅任务状态 | 任务状态 + worktree 索引 |
| 收尾 | 任务完成 | 任务完成 + 显式 keep/remove |
| 生命周期可见性 | 隐式日志 | `.worktrees/events.jsonl` 显式事件流 |

> 注：此为旧版 12 章内容，s18 提供了更详细的扩展版本。
