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

---

# 新版 工作区隔离

根据 [Learn Claude Code](https://learn.shareai.run/zh/s18/) 课程第 **s18 Worktree Isolation（工作区隔离）** 章节的正文，其核心概念、工作原理与教学版代码实现提炼如下：

---

## 1. 核心痛点与解决思路

* **痛点：** 在此前的 [Autonomous Agents](https://learn.shareai.run/zh/s17/) 阶段，虽然多 Agent 已经实现了自组织的任务认领，但它们都在**同一个物理工作目录**（`WORKDIR`）下并行操作。如果 Alice 正在重构认证模块并修改了 `config.py`，而 Bob 恰好也在重构 UI 登录页并修改了同一个 `config.py`，两者的写操作会互相覆盖，代码冲突严重，甚至无法干净地执行 `git rollback`。
* **解决思路：** **“各干各的目录，互不干扰”**。任务系统管“目标”，而 `worktree` 管“目录”。利用 **Git worktree** 机制，在同一个 Git 仓库中为每个任务动态创建完全独立的物理工作目录和独立的分支。Alice 和 Bob 在各自被隔离的路径下工作，彻底消除并发冲突。

---

## 2. 核心工作原理

1. **绑定机制：** Lead 提前创建任务和对应的 `worktree`，并进行 ID 绑定。绑定操作只写入任务的 `worktree` 属性，**不改变任务的 `pending` 状态**。
2. **动态切路径 (cwd)：** 队友在 Idle 状态下自动抢单（Claim）。一旦认领了绑定有 `worktree` 的任务，队友内部的执行上下文环境（`wt_ctx`）会自动切换，后续所有的 `bash`、`read_file`、`write_file` 均在该隔离目录下安全执行。
3. **安全收尾：** 任务完成后，提供 `keep_worktree`（保留分支供人类 Review 代码后合并）或 `remove_worktree`（清理物理目录和分支）。为防止 Agent 误删未提交的代码，删除时如果没有显式指定 `discard_changes=true`，系统会强制拦截并拒绝删除。

---

## 3. 核心功能代码实现

### ① 任务与 Worktree 的原子创建与绑定

Lead 用于初始化独立工作区的核心函数：

```python
import subprocess
from pathlib import Path

WORKTREES_DIR = Path(".worktrees")
WORKDIR = Path(".")

def validate_worktree_name(name: str):
    # 严格的安全校验：只允许常规字符，拒绝路径穿越（..）和非法注入
    import re
    if not re.match(r"^[A-Za-z0-9._-]{1,64}$", name):
        raise ValueError("Invalid worktree name")

def run_git(args: list[str]) -> tuple[bool, str]:
    # 底层统一执行 Git 指令，保证状态一致性
    r = subprocess.run(["git"] + args, cwd=WORKDIR, capture_output=True, text=True)
    return r.returncode == 0, r.stdout or r.stderr

def create_worktree(name: str, task_id: str = "") -> str:
    validate_worktree_name(name)
    path = WORKTREES_DIR / name
    
    # 执行 git worktree add 创建目录并检出独立新分支
    ok, result = run_git(["worktree", "add", str(path), "-b", f"wt/{name}", "HEAD"])
    if not ok:
        return f"Git error: {result}"
        
    if task_id:
        bind_task_to_worktree(task_id, name)
        
    log_event("create", name, task_id)
    return f"Worktree '{name}' created at {path}"

def bind_task_to_worktree(task_id: str, worktree_name: str):
    task = load_task(task_id)
    task.worktree = worktree_name  # 仅写入绑定字段，状态依然保持 pending
    save_task(task)

```

### ② 队友认领任务时的 CWD 动态切换环境

教学版中，通过为队友线程维护一个当前工作区上下文 `wt_ctx`，动态劫持写操作工具的执行路径：

```python
# 队友线程内部持有的独立工作上下文
wt_ctx = {"path": None}

def _run_claim_task(task_id: str) -> str:
    # 队友抢单
    result = claim_task(task_id, owner=name)
    if "Claimed" in result:
        task = load_task(task_id)
        # 如果抢到的任务绑定了工作区，瞬间激活上下文切换
        if task.worktree:
            wt_ctx["path"] = str(WORKTREES_DIR / task.worktree)
    return result

def _run_bash(command: str) -> str:
    # 劫持原有的基础 bash 工具，将其锁定在指定的 worktree 路径下执行
    return run_bash(command, cwd=wt_ctx["path"])

def _run_write_file(path: str, content: str) -> str:
    # 同理，写文件、读文件工具全部重定向至隔离目录
    target_path = Path(wt_ctx["path"]) / path if wt_ctx["path"] else Path(path)
    target_path.write_text(content)
    return f"Wrote to {target_path}"

```

### ③ 物理工作区安全清理状态机

防止意外丢失代码的安全销毁机制：

```python
def remove_worktree(name: str, discard_changes: bool = False) -> str:
    path = WORKTREES_DIR / name
    
    # 核心防御：如果有未提交的改动或 commits，默认拒绝删除
    if not discard_changes:
        files, commits = _count_worktree_changes(path) # 统计差异的伪函数
        if files > 0 or commits > 0:
            return "拒绝删除：该工作区有未提交的改动。请使用 discard_changes=True 强制丢弃，或使用 keep_worktree 保留。"
            
    # 强制清理
    ok, _ = run_git(["worktree", "remove", str(path), "--force"])
    if not ok: 
        return "Git worktree remove 失败"
        
    run_git(["branch", "-D", f"wt/{name}"]) # 删除对应分支
    log_event("remove", name)
    return f"Worktree '{name}' removed successfully."

def keep_worktree(name: str) -> str:
    # 选择保留，方便人类上屏进行最后的 code review 与 merge
    log_event("keep", name)
    return f"Worktree '{name}' kept for review (branch: wt/{name})"

```

### ④ 审计日志追加 (`log_event`)

系统将所有生命周期状态写入本地日志流，以便回溯与故障排查：

```python
import time

def log_event(event_type: str, worktree_name: str, task_id: str = ""):
    event = {
        "type": event_type,     # "create" | "remove" | "keep"
        "worktree": worktree_name,
        "task_id": task_id,
        "ts": time.time()
    }
    # 追加写入持久化审计文件
    with open(WORKTREES_DIR / "events.jsonl", "a") as f:
        f.write(json.dumps(event) + "\n")

```

---

## 4. 教学版 vs. 生产级 Claude Code 源码的差异

在真实的 [Claude Code 生产级源码](https://learn.shareai.run/zh/s18/) 中，针对工作区隔离设计了更强悍的底层逻辑：

* **进程级目录硬变更 (`process.chdir`)**：教学版非常克制，只是在工具调用时传递 `cwd=wt_ctx["path"]` 参数。而真实的 Claude Code（在 `EnterWorktreeTool.ts` 中）在开启当前会话的工作区时，直接调用了 Node.js 的 `process.chdir(worktreePath)`，从操作系统**进程级别直接硬切工作目录**，并同步更新内部的 `setCwd()` 和 `saveWorktreeState()`。
* **子 Agent 临时隔离机制 (`AgentTool.tsx`)**：CC 的 Worktree 系统分为两条完全解耦的路径：一条是上述人类会话的主动切入；另一条是当 `isolation: "worktree"` 时，后台自动调用 `createAgentWorktree()`。它不会影响人类当前终端的全局会话，仅仅专门给并行衍生的 Subagent 跑完任务使用，且在子 Agent 运行完无改动时**自动默默销毁**，有改动时才向人类返回其路径和分支。
* **独立的状态存储系统**：教学版为了方便理解，简单地把 `worktree` 作为字符串字段挂载到了任务数据结构（Task）中。而在 Claude Code 源码里，两者是完全独立的模块，系统通过专门的会话转录（Session Transcript）保存 `PersistedWorktreeSession` 对象，里面硬编码追踪了 `originalCwd`、`worktreePath`、`originalBranch`、`originalHeadCommit` 等 7 个极为硬核的底层 Git 指针状态，不依赖任何 Task 看板也能由 Agent 依靠强大的上下文自发关联。