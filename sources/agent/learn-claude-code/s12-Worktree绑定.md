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

---

# 新版 Task System

根据 [Learn Claude Code](https://learn.shareai.run/zh/s12/) 课程第 **[s12 Task System（多智能体任务看板）](https://learn.shareai.run/zh/s12/)** 章节的正文，其核心概念、工作原理与教学版代码实现提炼如下：

---

## 1. 核心痛点与解决思路

* **痛点：** 当面对如“重构整个 Auth 模块并部署”这种复杂的宏观目标时，如果直接让单个智能体大步向前，它很容易陷入“眉毛胡子一把抓”的混乱状态。单线程大模型无法独自在脑海中优雅地梳理出“建表 $\rightarrow$ 写路由 $\rightarrow$ 写逻辑 $\rightarrow$ 跑测试 $\rightarrow$ 部署”的拓扑依赖链条，往往走着走着就忘了某一步，或陷入死循环。
* **解决思路：** **“像人类团队一样，用看板管理任务”**。引入 **Task Graph（任务拓扑图）** 与 **看板状态机（Kanban Board）**。框架不再让大模型自由发挥，而是逼迫它必须先将宏观大目标拆解为一张具备显式依赖关系的结构化任务网。通过代码在底层控制状态流转，只有当置信度前置依赖全部完结（Done）时，后续任务才可被执行，实现了长程复杂任务的“确定性履约”。

---

## 2. 任务状态机工作原理

一个任务（Task）在生命周期中严格在四个状态桶中流转，由底层框架代码控制原子转移：

| 状态 (Status) | 说明 | 状态流转触发条件 |
| --- | --- | --- |
| **`waiting`** | 等待中 | 刚创建，或其下游前置依赖（`dependencies`）中仍有未完成的卡片。 |
| **`ready`** | 就绪 | 所有前置阻碍全部清除，可以被激活或分发给 Agent 认领。 |
| **`active`** | 执行中 | 智能体（或子 Agent）正在物理执行该任务。 |
| **`done`** | 已完成 | 执行成功，**自动触发广播，解锁后续所有依赖它的 `waiting` 任务**。 |

---

## 3. 核心功能代码实现（297 LOC 教学版内核）

### ① 数据结构定义

使用面向对象方式定义单个 `Task` 及核心的 `TaskBoard`（任务看板）：

```python
from dataclasses import dataclass, field
from typing import List, Dict

@dataclass
class Task:
    id: str                  # 唯一标识，如 "T1", "T2"
    title: str               # 任务名称
    dependencies: List[str]  # 依赖的其它任务 ID 列表
    status: str = "waiting"  # 状态：waiting, ready, active, done
    result: str = ""         # 执行成功后的产物概要

class TaskBoard:
    def __init__(self):
        self.tasks: Dict[str, Task] = {}

    def add_task(self, id: str, title: str, dependencies: List[str]):
        """向看板添加任务，并初始化其就绪状态"""
        status = "ready" if not dependencies else "waiting"
        self.tasks[id] = Task(id, title, dependencies, status)

    def update_status(self, id: str, new_status: str, result: str = ""):
        """核心状态机原子转换逻辑"""
        if id not in self.tasks: return
        task = self.tasks[id]
        task.status = new_status
        if result: task.result = result

        # 核心级联：当某个任务达成 done，立刻刷新整个看板，激活所有被释放的 ready 任务
        if new_status == "done":
            self._refresh_board()

    def _refresh_board(self):
        """检查所有 waiting 任务，若其前置依赖均已 done，则自动晋升为 ready"""
        for task in self.tasks.values():
            if task.status == "waiting":
                # 检查是否所有依赖都已变为 done
                if all(self.tasks[dep].status == "done" for dep in task.dependencies if dep in self.tasks):
                    task.status = "ready"

    def get_next_ready_task(self) -> Task | None:
        """获取当前看板中第一个可以立刻开工的任务"""
        for task in self.tasks.values():
            if task.status == "ready":
                return task
        return None

    def render_board(self) -> str:
        """将看板状态可视化输出，用于注入 System Prompt"""
        lines = ["[=== TASK BOARD ===]"]
        for t in self.tasks.values():
            dep_str = f" (waits for {', '.join(t.dependencies)})" if t.dependencies else ""
            lines.append(f"- [{t.status.upper()}] {t.id}: {t.title}{dep_str}")
        return "\n".join(lines)

```

### ② 任务看板操纵工具集 (Tools)

将看板操作封装为标准的原子级工具，供大模型在第一阶段拆解任务时调用：

```python
BOARD = TaskBoard()

TOOLS = [
    {
        "name": "create_task_matrix",
        "description": "Initialize or append multiple tasks with explicit dependencies to the project board.",
        "input_schema": {
            "type": "object",
            "properties": {
                "tasks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "title": {"type": "string"},
                            "dependencies": {"type": "array", "items": {"type": "string"}}
                        },
                        "required": ["id", "title", "dependencies"]
                    }
                }
            },
            "required": ["tasks"]
        }
    },
    {
        "name": "mark_task_done",
        "description": "Mark a running active task as successfully accomplished.",
        "input_schema": {
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "result_summary": {"type": "string"}
            },
            "required": ["id", "result_summary"]
        }
    }
]

def run_create_task_matrix(tasks: list) -> str:
    for t in tasks:
        BOARD.add_task(t["id"], t["title"], t["dependencies"])
    return "Task matrix approved. Current view:\n" + BOARD.render_board()

def run_mark_task_done(id: str, result_summary: str) -> str:
    BOARD.update_status(id, "done", result_summary)
    return f"Task {id} set to DONE. Board updated successfully."

TOOL_HANDLERS = {
    "create_task_matrix": run_create_task_matrix,
    "mark_task_done": run_mark_task_done
}

```

### ③ 驱动多任务调度的外层 Agent 环

在外层通过循环消费看板，将大任务拆解为小步快跑的确定性原子执行流：

```python
def orchestrate_project(user_goal: str):
    print(f"[Orchestrator] Planning for goal: {user_goal}")
    history = [{"role": "user", "content": f"Please decompose this big goal into tasks using create_task_matrix: {user_goal}"}]
    
    # 阶段 1：强迫大模型先动笔，把整个大任务拆解填充进看板中
    # (调用规划 LLM 执行工具 create_task_matrix，代码省略...)
    
    # 阶段 2：看板驱动的原子步进状态闭环
    while True:
        task = BOARD.get_next_ready_task()
        if not task:
            # 既没有 ready 也没有 waiting 的任务，说明全盘合拢完结
            if all(t.status == "done" for t in BOARD.tasks.values()):
                print("\n🎉 [SUCCESS] All tasks accomplished perfectly!")
            else:
                print("\n⛔ [DEADLOCK] Deadlock detected or no ready tasks left.")
            break
            
        print(f"\n🚀 [EXECUTE] Promoting {task.id} ('{task.title}') to ACTIVE...")
        BOARD.update_status(task.id, "active")
        
        # 将局部单个原子卡片打包发给具体的执行 Agent（或 Subagent）去心无旁骛地攻坚
        task_prompt = f"Perform this specific single task: {task.title}."
        if task.dependencies:
            # 自动把所有前置卡片的产物结项总结作为附加上下文喂入，消灭信息孤岛
            context = "\n".join(f"Pre-requisite {dep} result: {BOARD.tasks[dep].result}" for dep in task.dependencies)
            task_prompt += f"\nContext from previous tasks:\n{context}"
            
        # 跑核心 Agent Loop 攻坚此任务...
        # execution_result = run_single_agent_loop(task_prompt)
        execution_result = "Database is set up on port 5432." 
        
        # 攻坚结束，物理调用工具将卡片归档，引发下一轮看板刷新
        run_mark_task_done(task.id, execution_result)

```

---

## 4. 教学版 vs. 生产级 Claude Code 源码的差异

在真实的 [Claude Code 生产级源码](https://learn.shareai.run/zh/s12/)（主要分布在 `src/services/tasks/` 下的 `TaskGraph.ts`、`KanbanCoordinator.ts`）中，这套系统具有极高的工业防错设计：

* **严格的拓扑死锁检测与环路验证 (`hasCycle`)：** 教学版随大流信任模型拆出来的 `dependencies`。而真实源码在模型调用 `create_task_matrix` 的原子瞬间，底层框架会立刻将其转化为有向图并跑一遍 **Kahn 拓扑排序算法**。一旦检测到模型拆出了环路依赖（例如 T1 依赖 T2，T2 又反向依赖 T1），框架会一票否决（Reject）此工具调用，并在 `tool_result` 中报错纠错，命令大模型重新进行无环图（DAG）拆解，彻底掐灭了永不就绪的死锁风险。
* **文件阴影持久层 (`.tasks.json` 弹性恢复断点)：** CC 源码的 `TaskBoard` 具备完全的**状态刷盘与可重入能力**。每一次任务流转（如 `mark_task_done`），都会原子性地把整张图的最新状态同步写进项目根目录的 `.tasks.json` 隐藏文件中。如果网络断线、API 崩溃或者人类强行通过 `Ctrl+C` 掐断了 CLI 终端，当下次再次运行 `claude` 恢复工作时，框架会静默读取该 JSON，瞬间**恢复前一次长达数小时任务中断处的断点看板**，拒绝重复劳动。
* **广度优先的物理多进程并发 (`Parallel Execution`)：** 教学版采用了一个简单的 `while` 循环单线程履约卡片。生产级源码的 Coordinator 在执行时，如果发现当前看板内同时涌现出了 3 个完全没有互相依赖关系的 `ready` 卡片（例如 T2:"编写前端组件"、T3:"配置 CI 脚本"），系统会瞬间触发 [Background Tasks 并发机制](https://learn.shareai.run/zh/s12/)，动态派生（spawn）出 3 个完全平行的子进程（Subagents）并发攻坚这三张卡片，使其在物理上多轨并进，极大地压榨并提升了软件工程的交付吞吐率。