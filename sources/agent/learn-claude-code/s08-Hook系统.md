---
name: s08 Hook 系统
description: 不改主循环代码，也能在关键时机插入额外行为
type: concept
updated: 2026-05-07
tags: agent, learn-shareai, s08
source: https://learn.shareai.run/zh/s08/
---

> _不改主循环代码，也能在关键时机插入额外行为。_

## 这章要解决什么问题

到了 `s07`，我们已经能在工具执行前做权限判断。

但很多真实需求并不属于"允许 / 拒绝"这条线，而属于：

* 在某个固定时机顺手做一点事
* 不改主循环主体，也能接入额外规则
* 让用户或插件在系统边缘扩展能力

如果每增加一个需求，你都去修改主循环，主循环就会越来越重，最后谁都不敢动。

所以这一章要引入的机制是：

**主循环只负责暴露"时机"，真正的附加行为交给 hook。**

## 什么是 hook

`hook` 可以理解成一个"预留插口"：

1. 主系统运行到某个固定时机
2. 把当前上下文交给 hook
3. hook 返回结果
4. 主系统再决定下一步怎么继续

**hook 让系统可扩展，但不要求主循环理解每个扩展需求。**

## 最小心智模型

教学版先只讲 3 个事件：

* `SessionStart`
* `PreToolUse`
* `PostToolUse`

## 教学版统一返回约定

| 退出码 | 含义 |
| --- | --- |
| `0` | 正常继续 |
| `1` | 阻止当前动作 |
| `2` | 注入一条补充消息，再继续 |

hook 最核心的 3 种作用：观察、拦截、补充。

## 关键数据结构

### 1. HookEvent

```python
event = {
    "name": "PreToolUse",
    "payload": {
        "tool_name": "bash",
        "input": {"command": "pytest"},
    },
}
```

### 2. HookResult

```python
result = {
    "exit_code": 0,
    "message": "",
}
```

### 3. HookRunner

```python
class HookRunner:
    def run(self, event_name: str, payload: dict) -> dict:
        ...
```

主循环知道事件名，hook runner 知道怎么调扩展逻辑。

## 最小实现

### 第一步：准备事件到处理器的映射

```python
HOOKS = {
    "SessionStart": [on_session_start],
    "PreToolUse": [pre_tool_guard],
    "PostToolUse": [post_tool_log],
}
```

### 第二步：统一运行 hook

```python
def run_hooks(event_name: str, payload: dict) -> dict:
    for handler in HOOKS.get(event_name, []):
        result = handler(payload)
        if result["exit_code"] in (1, 2):
            return result
    return {"exit_code": 0, "message": ""}
```

### 第三步：接进主循环

```python
pre = run_hooks("PreToolUse", {
    "tool_name": block.name,
    "input": block.input,
})
if pre["exit_code"] == 1:
    results.append(blocked_tool_result(pre["message"]))
    continue
if pre["exit_code"] == 2:
    messages.append({"role": "user", "content": pre["message"]})

output = run_tool(...)

post = run_hooks("PostToolUse", {
    "tool_name": block.name,
    "input": block.input,
    "output": output,
})
```

**hook 不是主循环的替代品，hook 是主循环在固定时机对外发出的调用。**

## 学完这一章，你应该真正掌握什么

1. hook 的作用，是在固定时机扩展系统，而不是改写主循环
2. hook 至少需要"事件名 + payload + 返回结果"这三样东西
3. 教学版可以先用统一的 `0 / 1 / 2` 返回约定
4. `PreToolUse` 和 `PostToolUse` 已经足够支撑最核心的扩展能力

## 一句话记住

**Hook 的本质，是在固定时机扩展系统，而不是改写主循环。**

---

# 新版 Hooks

根据课程正文，**Hooks（生命周期钩子与扩展）** 的核心概念、工作原理与教学版代码实现提炼如下：

---

## 1. 核心痛点与解决思路

* **痛点：** 随着智能体功能的不断扩展，我们经常需要加入各种横切关注点（Cross-cutting concerns），例如“记录每次 Bash 调用”、“操作后自动 `git add`”、“工具输出过大时进行提醒”等。如果把这些逻辑全部硬编码塞进主循环（`agent_loop`）体内，循环代码会迅速膨胀且变得支离破碎。你原本只想扩展 Agent 的外部行为，结果却把作为核心的循环本身改毁了。
* **解决思路：** **“挂在循环上，不写进循环里（Hang on the Loop, Don't Write into It）”**。让主循环保持纯粹和高内聚，它只负责在固定的生命周期节点调用 `trigger_hooks(event)` 发出通知。所有的扩展逻辑（如权限检查、审计日志、自动收尾等）全部作为回调函数注册在外部的钩子注册表（Hook Registry）中，实现核心与扩展的彻底解耦。

---

## 2. 核心工作原理

教学版框架提炼出了 4 个最核心的事件，它们精准覆盖了一个完整的 Agent Turn 生命周期：

| 事件 (Event) | 触发时机 | 典型用途 |
| --- | --- | --- |
| **UserPromptSubmit** | 用户输入提交后、进入 LLM 前 | 输入验证、注入工作目录/上下文信息 |
| **PreToolUse** | 工具执行前 | 运行时权限检查、执行审计日志、入参拦截修改 |
| **PostToolUse** | 工具执行后 | 处理副作用（自动编译/提交）、输出大小检查与截断 |
| **Stop** | 循环即将退出时（模型返回非工具调用） | 统一收尾清理、打印统计信息、甚至强制续跑 |

---

## 3. 核心功能代码实现（232 LOC 教学版内核）

### ① 钩子注册表与触发机制内核

整个机制的底层是由一个简单的事件映射字典和两个注册/触发函数驱动的：

```python
# 核心事件注册表
HOOKS = {
    "UserPromptSubmit": [],
    "PreToolUse": [],
    "PostToolUse": [],
    "Stop": [],
}

def register_hook(event: str, callback):
    """外部扩展通过此函数向注册表挂载回调"""
    if event in HOOKS:
        HOOKS[event].append(callback)

def trigger_hooks(event: str, *args):
    """主循环在生命周期节点调用此函数触发通知"""
    for callback in HOOKS[event]:
        result = callback(*args)
        # 约定：如果钩子返回了非 None 值，代表需要干预主循环的行为（如拦截或强制续跑）
        if result is not None:
            return result
    return None

```

### ② 具体钩子回调的编写与注册（外部扩展）

将 s03 的权限硬编码逻辑以及各种日志、统计逻辑剥离出来，包装成独立的外部钩子：

```python
# 1. 注入上下文钩子 (UserPromptSubmit)
def context_inject_hook(query: str) -> str | None:
    print(f"\033[90m[HOOK] UserPromptSubmit: working in {WORKDIR} \033[0m")
    return None  # 返回 None 代表放行，不修改输入

# 2. 权限校验钩子 (PreToolUse)
def permission_hook(block):
    if block.name == "bash":
        for pattern in DENY_LIST:
            if pattern in block.input.get("command", ""):
                return "Permission denied by deny list"  # 返回字符串代表拦截执行
    if block.name in ("write_file", "edit_file"):
        path = block.input.get("path", "")
        if not (WORKDIR / path).resolve().is_relative_to(WORKDIR):
            choice = input("   Allow? [y/N] ").strip().lower()
            if choice not in ("y", "yes"):
                return "Permission denied by user"
    return None

# 3. 日志记录钩子 (PreToolUse)
def log_hook(block):
    print(f"[HOOK] Agent is proposing to use tool: {block.name}")

# 4. 大输出提醒钩子 (PostToolUse)
def large_output_hook(block, output):
    if len(str(output)) > 100000:
        print(f"[HOOK] ⚠ Large output detected from tool {block.name}")

# 5. 最终统计钩子 (Stop)
def summary_hook(messages: list) -> str | None:
    # 统计整个会话中实际成功的工具调用次数
    tool_count = sum(1 for m in messages for b in (m.get("content") if isinstance(m.get("content"), list) else []) 
                     if isinstance(b, dict) and b.get("type") == "tool_result")
    print(f"\033[90m[HOOK] Stop: session finished with {tool_count} tool calls\033[0m")
    return None  # 返回 None 允许退出；若返回字符串则会作为 user 消息喂给模型强制续跑

# 统一向注册表挂载
register_hook("UserPromptSubmit", context_inject_hook)
register_hook("PreToolUse", permission_hook)
register_hook("PreToolUse", log_hook)
register_hook("PostToolUse", large_output_hook)
register_hook("Stop", summary_hook)

```

### ③ 挂载了 Hooks 的稳定 Agent Loop 内核

主循环体内不再有任何具体的业务检查代码，只有纯粹的模型-工具闭环，通过 `trigger_hooks` 嵌入扩展点：

```python
def agent_loop(messages):
    while True:
        # 调用 LLM 决策
        response = client.messages.create(model=MODEL, messages=messages, tools=TOOLS)
        messages.append({"role": "assistant", "content": response.content})
        
        # --- 【节点 4: Stop 拦截】 ---
        if response.stop_reason != "tool_use":
            force_continue_prompt = trigger_hooks("Stop", messages)
            if force_continue_prompt:
                # 如果 Stop 钩子强行返回了文本，向模型注入该文本并阻断退出，强行续跑
                messages.append({"role": "user", "content": force_continue_prompt})
                continue
            break  # 允许体面退出
            
        results = []
        for block in response.content:
            if block.type == "tool_use":
                
                # --- 【节点 2: PreToolUse 拦截】 ---
                blocked = trigger_hooks("PreToolUse", block)
                if blocked:
                    # 如果任一 PreToolUse 钩子返回了拒绝原因，直接终止工具 handler 执行
                    results.append({
                        "type": "tool_result", 
                        "tool_use_id": block.id, 
                        "content": str(blocked)
                    })
                    continue
                
                # 正常分发并执行工具
                handler = TOOL_HANDLERS.get(block.name)
                output = handler(**block.input) if handler else f"Unknown: {block.name}"
                
                # --- 【节点 3: PostToolUse 通知】 ---
                trigger_hooks("PostToolUse", block, output)
                
                results.append({"type": "tool_result", "tool_use_id": block.id, "content": output})
                
        messages.append({"role": "user", "content": results})

# --- 【节点 1: UserPromptSubmit 通知】 ---
query = input("s04 >> ")
trigger_hooks("UserPromptSubmit", query)
history = [{"role": "user", "content": query}]
agent_loop(history)

```

---

## 4. 教学版 vs. 生产级 Claude Code 源码的差异

在真实的 [Claude Code 生产级源码](https://learn.shareai.run/zh/s04/)（主要分布在 `toolHooks.ts`、`hooks.ts`、`stopHooks.ts`）中，这套切面架构的精密度和覆盖面达到了工业级水准：

* **27 个细分事件矩阵：** 教学版为了降低心智负担只保留了最核心的 4 个事件。CC 实际定义了高达 27 个生命周期事件，延伸到了系统的方方面面，包括会话级的 `SessionStart` / `SessionEnd`、多 Agent 团队协作的 `TeammateIdle` / `TaskCompleted`、上下文压缩前后的 `PreCompact` / `PostCompact`，以及环境层面的 `CwdChanged` / `FileChanged` 等。
* **结构化 HookResult 的权限防御不变式：** CC 的钩子返回值不是简单的纯文本，而是包含 14 个可选字段的强类型 `HookResult`（支持修改模型工具输入 `updatedInput`、注入阻塞纠错错误 `blockingError` 等）。源码中有一条铁律：**Hook 的 `allow` 决策绝对不能绕过全局规则（Hook 'allow' cannot bypass deny/ask rules）**。也就是说，即使外部编写的 Hook 脚本由于漏洞或恶意设定放行（返回 `allow`），系统底层的 `settings.json` 安全策略依然会对其执行强行扣押，杜绝了通过动态挂载钩子来突破安全沙箱的可能。
* **防死循环重入状态机 (`stopHookActive`)：** 在处理 `Stop` 钩子时，如果钩子发现模型任务没完成并产生 `blockingError` 迫使模型重新自纠续跑，系统会自动在全局 State 中将 `stopHookActive` 标志位置为 `true`。当模型下一轮再次试图退出、重新触发 Stop 钩子时，钩子看到该标志就会保持沉默不再报错。这精准防御了“模型要退出 $\rightarrow$ 钩子报错纠正 $\rightarrow$ 模型修改后再退出 $\rightarrow$ 钩子再次报错”的导致系统永不停机的死循环 Bug。
* **优雅停机续航机制 (`hook_stopped_continuation`)：** 当 `PostToolUse` 钩子评估当前执行结果后决定紧急避险时（例如发现文件被改坏了），它可以返回 `preventContinuation: true`。此时系统并不会引发程序崩溃，而是会抛出一个标准的 `hook_stopped_continuation` 信号。主循环接收到该信号后，会安全地保存当前的对话转录快照，并向终端优雅提示完成退出，实现了 Agent 的“体面停机”。