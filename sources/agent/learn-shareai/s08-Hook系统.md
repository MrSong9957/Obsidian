---
name: "s08 Hook 系统"
description: "不改主循环代码，也能在关键时机插入额外行为"
updated: 2026-05-07
tags: [agent, learn-shareai, s08]
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
