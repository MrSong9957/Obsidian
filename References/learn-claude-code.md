---
name: learn-claude-code
description: shareAI-lab/learn-claude-code 仓库学习笔记 — 学习 Claude Code 用法时参考
---

# Learn Claude Code 学习笔记

> 学习仓库：https://github.com/shareAI-lab/learn-claude-code

---

## safe_path() — 路径穿越防御

```python
def safe_path(p: str) -> Path:
    path = (WORKDIR / p).resolve()
    if not path.is_relative_to(WORKDIR):
        raise ValueError(f"Path escapes workspace: {p}")
    return path
```

### 三步机制

1. **拼接 + resolve**：`(WORKDIR / p).resolve()` 把用户输入拼到沙箱目录下，`.resolve()` 展开所有 `..`、`.`、符号链接，得到真正的绝对路径
2. **校验边界**：`is_relative_to(WORKDIR)` 检查最终路径是否仍在沙箱内
3. **通过返回 / 否则拒绝**：只有安全路径才返回

### 关键点

- **必须 resolve**：只检查字符串中的 `..` 不够，符号链接也能逃逸。`resolve()` 连符号链接一起展开
- **防御 Path Traversal**（OWASP Top 10）：典型攻击 `p = "../../etc/passwd"`，resolve 后变成 `/etc/passwd`，被 `is_relative_to` 拦截
- **比喻**：X 光安检——不管地址套多少层，透视到最终目的地再判断是否越界

---

## run_read() — 安全读取文件内容

```python
def run_read(path: str, limit: int = None) -> str:
    text = safe_path(path).read_text()
    lines = text.splitlines()
    if limit and limit < len(lines):
        lines = lines[:limit]
    return "\n".join(lines)[:50000]
```

### 流程

1. **安检 + 读取**：`safe_path(path).read_text()` — 先过路径安检，安全后才读文件
2. **拆行**：`splitlines()` 按换行符拆成列表
3. **行数限制**：`if limit and limit < len(lines)` — 用户指定了限制且文件超长时，截取前 N 行
4. **字符硬顶**：`[:50000]` — 最多返回 5 万字符，兜底保护

### 关键点

- **两层限流**：`limit` 是用户可控的软限制，`50000` 是系统硬天花板
- **安检复用**：直接调用 `safe_path()`，路径安全逻辑只写一次

---

## TOOL_HANDLERS — 工具路由表

```python
TOOL_HANDLERS = {
    "bash":       lambda **kw: run_bash(kw["command"]),
    "read_file":  lambda **kw: run_read(kw["path"], kw.get("limit")),
    "write_file": lambda **kw: run_write(kw["path"], kw["content"]),
    "edit_file":  lambda **kw: run_edit(kw["path"], kw["old_text"],
                                         kw["new_text"]),
}
```

### 是什么

字典，把工具名映射到对应的处理函数。AI 请求到达后，查表找到函数并执行。

### 为什么用 lambda

每个工具参数不同，但 AI 传来的格式统一是字典。lambda 是适配器——接收统一的 `**kw` 字典，从中提取该工具需要的参数，传给实际函数。

### 好处

- 调用代码只有一行：`TOOL_HANDLERS[tool](**kw)`，不需要 if/elif
- 加新工具只需在字典里加一行，不用改其他代码

### 完整链路

1. 系统把工具说明书（名称、参数、说明）发给 AI
2. AI 根据用户需求决定调哪个工具，返回工具名和参数
3. 系统查 TOOL_HANDLERS，找到对应函数执行
4. 结果返回给 AI

---

## stop_reason — 模型为什么停下来了

```python
if response.stop_reason != "tool_use":
    return
```

### stop_reason 的四种值

| 值 | 含义 |
|---|---|
| `"end_turn"` | 模型说完了，正常结束（纯文本回复） |
| `"tool_use"` | 模型决定调用工具，等你执行后把结果喂回来 |
| `"max_tokens"` | 达到 token 上限，被截断了 |
| `"stop_sequence"` | 命中了自定义停止序列 |

### 关键理解

- `stop_reason` 是模型**已经完成一轮输出后**告诉你它为什么停的，不是"请求"而是"结果"
- 模型每一轮只做一件事：**输出文本**或**输出工具调用**，这是二选一决策
- 这段代码是 agent loop 的退出条件：模型不再调工具 → 认为任务完成 → 结束循环
- **比喻**：接力赛里的"接力棒还在不在手里"——tool_use 是还在跑，其他就是跑完了

---

## response.content 遍历 — 内容块分发

```python
for block in response.content:
    if block.type == "tool_use":
        handler = TOOL_HANDLERS.get(block.name)
```

### 为什么需要遍历

模型的回复不是一整个字符串，而是一个**内容块列表**。一条回复可以同时包含文字和多个工具调用：

```json
[
  { "type": "text", "text": "让我帮你查一下..." },
  { "type": "tool_use", "name": "get_weather", "input": {"city": "北京"} },
  { "type": "tool_use", "name": "get_weather", "input": {"city": "上海"} }
]
```

### 关键点

- 模型可以**一边说话，一边调多个工具**，所以必须遍历每个 block，只处理 `tool_use` 类型
- `.get()` 而不是 `[key]`：模型调了没注册的工具时返回 `None`，不会崩
- 这两行是整个工具执行的核心分发逻辑——从"模型指令"到"代码执行"的桥梁

---

## 工具执行三件套 — 执行、兜底、回传

```python
try:
    output = handler(**block.input) if handler else f"Unknown tool: {block.name}"
except Exception as e:
    output = f"Error: {e}"
print(f"> {block.name}: {str(output)[:200]}")
results.append({"type": "tool_result", "tool_use_id": block.id, "content": str(output)})
```

### 三步各解决什么问题

1. **执行 + 兜底**：handler 存在就调用，不存在返回提示；外层 except 兜住内部错误——无论发生什么，都给模型一个回复
2. **可见性**：截断到 200 字符打印，让用户知道调了什么、返回了什么，不刷屏
3. **回传**：包装成 `tool_result`，关键是 `tool_use_id` 配对——模型一轮可能调多个工具，靠 ID 知道哪个结果对应哪个请求

### 关键点

- `str(output)` 确保无论工具返回什么类型都能转成字符串喂给 API
- **核心原则**：无论发生什么，都不能静默失败
- 这些防御没有一个是"聪明的设计"，全是踩坑后的补丁

---

## `**block.input` 解包 — 字典变参数

```python
output = handler(**block.input)
```

### 做了什么

`block.input` 是字典 `{"city": "北京", "unit": "celsius"}`，`**` 把它展开成关键字参数：

```python
handler(**block.input)
# 等价于
handler(city="北京", unit="celsius")

# 而不是
handler({"city": "北京", "unit": "celsius"})  # 整个字典作为一个参数，函数用不了
```

### 关键点

- 这是**调用形式的转换**，不是类型转换——字符串还是字符串，数字还是数字
- 模型定义工具时的 `input_schema` 参数名，必须和 handler 函数的参数名**完全一致**
- **比喻**：拆快递——把打包好的字典，拆成一个个独立参数递给函数

---

## todo 心跳检测 — 追踪模型是否跑偏

```python
used_todo = False
# ... 循环中 ...
if block.name == "todo":
    used_todo = True
```

### 为什么 todo 需要特殊处理

其他工具（读文件、查天气）是"干活的"，调完就完。todo 工具是**记录用户意图**的，它影响的是 agent loop 结束后的行为。`used_todo` 不是追踪工具复杂度，而是充当**心跳信号**——模型还在对照清单执行就是"活着"。

### 偏离检测与提醒注入

```python
rounds_since_todo = 0 if used_todo else rounds_since_todo + 1

if rounds_since_todo >= 3:
    results.insert(0, {"type": "text", "text": "<reminder>Update your todos.</reminder>"})
```

- 三元表达式：本轮调了 todo 就重置计数，没调就 +1
- 连续 3 轮没调 todo → 检测到偏离，在结果列表**最前面**插入提醒
- `insert(0, ...)` 而不是 `append()`：让提醒排在最前面，模型优先看到

### 关键点

- 这是在**消息流里注入提示词**——不改系统提示，不改工具定义，只在模型跑偏时"拍一下肩膀"
- 和 `stop_reason` 的退出条件不同，这是一个**超时兜底**：逻辑上没结束，但行为已经偏了

---

## messages.append — 对话历史的增长

```python
messages.append({"role": "user", "content": results})
```

### 作用

把本轮收集的所有 `tool_result` 和可能的提醒文本，作为用户侧消息追加到对话历史末尾。下一轮循环把整个 `messages` 发给 API，模型看到完整链路后继续决策。

### 对话历史的结构

```
messages = [
  {"role": "user",      "content": "帮我规划学习路线"},       ← 用户原始输入
  {"role": "assistant",  "content": [tool_use, tool_use]},    ← 模型调工具
  {"role": "user",      "content": [tool_result, tool_result]}, ← 工具结果（这一步）
  {"role": "assistant",  "content": [...]},                    ← 模型继续
  ...
]
```

### 关键点

- `tool_result` 在 API 协议里属于**用户侧消息**（role: "user"），因为是在"回答"模型的工具调用请求
- 每轮循环 messages 增长两条（assistant 的输出 + user 侧的工具结果），对话就这样一步步积累

---
