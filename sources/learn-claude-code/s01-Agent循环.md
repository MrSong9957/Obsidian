# s01 Agent 循环

## 核心问题

大语言模型（LLM）只能处理文本——给它一段话，它返回一段话。但真正的 Agent 需要能**操作真实世界**：执行命令、读写文件、搜索代码。

怎么让只会"说话"的模型动起手来？

## 解决方案：一个 while 循环

整个 AI Agent 的核心秘密就一个模式：

```
while stop_reason == "tool_use":
    把工具执行结果喂回模型
```

模型说"我要执行命令" → 你执行 → 把结果喂回去 → 模型看了结果再决定下一步 → 直到模型觉得任务完成，不再调用工具。

> 比喻：模型是一个坐在办公室里的项目经理，它自己不能动手，但可以发工单（tool_use）。你是一个跑腿的执行者，拿到工单就干活，干完把结果送回来。项目结束了，经理说"搞定了"（stop_reason != "tool_use"），你就可以下班了。

---

## 完整数据流

```
┌──────────┐     ┌──────────────────────┐     ┌──────────────┐
│   User   │     │                      │     │              │
│  prompt  │ ──> │     LLM (Claude)     │ ──> │  run_bash()  │
│          │     │                      │     │  执行命令     │
└──────────┘     │  stop_reason 决定:   │     └──────┬───────┘
                 │  "tool_use" → 继续   │            │
                 │  "end_turn" → 结束   │            │ stdout/stderr
                 └──────────┬───────────┘            │
                            ^                        ▼
                            │              ┌──────────────────┐
                            │              │  tool_result     │
                            └──────────────│  喂回模型        │
                                           └──────────────────┘
                            （循环继续，直到 stop_reason != "tool_use"）
```

---

## 源代码逐段解析

### 1. 初始化：连接 API

```python
from anthropic import Anthropic
from dotenv import load_dotenv

load_dotenv(override=True)      # 从 .env 文件加载环境变量

if os.getenv("ANTHROPIC_BASE_URL"):
    os.environ.pop("ANTHROPIC_AUTH_TOKEN", None)  # 兼容不同认证方式

client = Anthropic(base_url=os.getenv("ANTHROPIC_BASE_URL"))
MODEL = os.environ["MODEL_ID"]
```

这段做的事情很直白：从 `.env` 文件读配置，创建 API 客户端。`override=True` 表示 `.env` 里的值覆盖系统环境变量。

```python
SYSTEM = f"You are a coding agent at {os.getcwd()}. Use bash to solve tasks. Act, don't explain."
```

系统提示词告诉模型三件事：
- 你在哪（当前工作目录）
- 你的能力（用 bash 执行命令）
- 你的风格（做了再说，别啰嗦）

```python
TOOLS = [{
    "name": "bash",
    "description": "Run a shell command.",
    "input_schema": {
        "type": "object",
        "properties": {"command": {"type": "string"}},
        "required": ["command"],
    },
}]
```

这是告诉模型"你有一个工具可以用"的 JSON Schema。`input_schema` 定义了工具接受的参数：一个必填的 `command` 字符串。模型看到这个声明后，就知道它可以输出一个结构化的工具调用请求。

### 2. run_bash：模型与真实世界的桥梁

```python
def run_bash(command: str) -> str:
    # 第一道防线：危险命令黑名单
    dangerous = ["rm -rf /", "sudo", "shutdown", "reboot", "> /dev/"]
    if any(d in command for d in dangerous):
        return "Error: Dangerous command blocked"

    try:
        r = subprocess.run(
            command, shell=True,        # shell=True 让它可以执行管道、重定向等
            cwd=os.getcwd(),            # 在当前目录执行
            capture_output=True,        # 捕获 stdout 和 stderr
            text=True,                  # 输出为字符串（不是 bytes）
            timeout=120                 # 120 秒超时
        )
        out = (r.stdout + r.stderr).strip()  # 合并标准输出和错误输出
        return out[:50000] if out else "(no output)"  # 截断到 50000 字符
    except subprocess.TimeoutExpired:
        return "Error: Timeout (120s)"
```

这个函数做了四件事：

| 步骤 | 做了什么 | 为什么 |
|------|---------|--------|
| 黑名单检查 | 匹配 `rm -rf /`、`sudo` 等 | 防止模型执行破坏性命令 |
| `subprocess.run` | 在子进程里执行 shell 命令 | 让模型能操作真实系统 |
| 合并输出 | `r.stdout + r.stderr` | 模型需要看到错误信息才能调试 |
| 截断 | `out[:50000]` | API 有 token 上限，不能无限长 |

> `subprocess.run` 就是从 Python 里启动一个子进程来跑命令。`shell=True` 意味着命令会通过 `/bin/sh` 执行，所以可以用管道 `|`、重定向 `>` 等 shell 语法。

### 3. agent_loop：核心循环模式

```python
def agent_loop(messages: list):
    while True:
        # ① 调用 API
        response = client.messages.create(
            model=MODEL, system=SYSTEM, messages=messages,
            tools=TOOLS, max_tokens=8000,
        )

        # ② 把模型回复加到对话历史
        messages.append({"role": "assistant", "content": response.content})

        # ③ 检查：模型还要继续用工具吗？
        if response.stop_reason != "tool_use":
            return    # 不用了 → 循环结束，控制权交回调用者

        # ④ 执行每个工具调用，收集结果
        results = []
        for block in response.content:
            if block.type == "tool_use":
                print(f"\033[33m$ {block.input['command']}\033[0m")  # 打印执行的命令
                output = run_bash(block.input["command"])             # 执行
                print(output[:200])                                   # 打印结果前 200 字符
                results.append({                                      # 包装成 tool_result
                    "type": "tool_result",
                    "tool_use_id": block.id,    # 必须和模型的 tool_use 请求配对
                    "content": output,
                })

        # ⑤ 把工具结果作为 user 消息加回对话
        messages.append({"role": "user", "content": results})
        # 然后回到 while True 顶部，带着结果再次调用模型
```

数据流的五个步骤对应关系：

```
① 调用 API          → 把整个对话历史发给 Claude
② append assistant  → 把模型回复存下来（保持对话完整）
③ 检查 stop_reason  → 模型说了算：继续还是停止
④ 执行工具          → 模型要执行命令就执行，收集结果
⑤ append user       → 工具结果以 user 角色喂回（API 要求这样）
```

> 为什么工具结果的角色是 `"user"`？因为 Anthropic API 的消息格式要求 tool_result 必须跟在 assistant 的 tool_use 后面，并且以 user 消息的形式传入。这不是设计选择，是 API 规范。

### 4. `__main__`：交互式入口

```python
if __name__ == "__main__":
    history = []                           # 对话历史，跨轮次保持
    while True:
        try:
            query = input("\033[36ms01 >> \033[0m")   # 读用户输入（青色提示符）
        except (EOFError, KeyboardInterrupt):          # Ctrl+C / Ctrl+D 退出
            break
        if query.strip().lower() in ("q", "exit", ""): # 空输入也退出
            break

        history.append({"role": "user", "content": query})  # 用户消息加入历史
        agent_loop(history)                                   # 进入 agent 循环

        # 打印模型最终回复
        response_content = history[-1]["content"]
        if isinstance(response_content, list):
            for block in response_content:
                if hasattr(block, "text"):
                    print(block.text)
        print()
```

外层循环管用户交互（读输入、打印结果），内层 `agent_loop` 管模型-工具交互。两层循环各司其职：

```
外层 while True:  用户 ↔ Agent（多轮对话）
    内层 while True:  模型 ↔ 工具（单次用户请求内的多步执行）
```

---

## 实际运行示例

用户输入：`看看当前目录有什么文件`

```
=== 第 1 轮（agent_loop） ===

messages = [{"role": "user", "content": "看看当前目录有什么文件"}]

模型返回:
  stop_reason = "tool_use"
  content = [tool_use(name="bash", input={"command": "ls -la"}, id="toolu_xxx")]

执行:
  $ ls -la
  total 32
  drwxr-xr-x  5 user user 4096 Apr  2 10:00 .
  drwxr-xr-x  3 user user 4096 Apr  2 09:00 ..
  -rw-r--r--  1 user user 1200 Apr  2 10:00 s01_agent_loop.py
  ...

messages 追加:
  assistant: [tool_use(bash, "ls -la")]
  user: [tool_result("total 32\ndrwxr-xr-x ...")]

=== 第 2 轮（agent_loop） ===

模型看到 ls 的输出，整理后回复:
  stop_reason = "end_turn"
  content = [text("当前目录有以下文件：\n1. s01_agent_loop.py ...")]

stop_reason != "tool_use" → 循环结束，控制权回到外层
打印: "当前目录有以下文件：..."
```

---

## 关键 Python 知识点

### subprocess.run

```python
r = subprocess.run("ls -la", shell=True, capture_output=True, text=True, timeout=120)
r.stdout    # 标准输出，str 类型（因为 text=True）
r.stderr    # 错误输出
r.returncode  # 退出码，0 表示成功
```

`subprocess.run` 是 Python 执行外部命令的标准方式。`shell=True` 会启动一个 shell 来解释命令，代价是有一点安全风险（命令注入）。

### while True 循环模式

```python
while True:
    result = do_something()
    if should_stop(result):
        return    # 用 return 而不是 break，直接结束整个函数
```

`while True` + 条件 `return` 是"不确定迭代次数"场景的标准写法。这里迭代次数取决于模型的决定，事先无法预知。

### stop_reason 检查

```python
if response.stop_reason != "tool_use":
    return
```

`stop_reason` 是 API 返回的字段，告诉调用者模型为什么停下来：
- `"tool_use"`：模型想调用工具，还没说完
- `"end_turn"`：模型说完了，正常结束
- `"max_tokens"`：模型被 token 上限截断了

---

## 一句话总结

**Agent 的全部秘密：把工具执行结果喂回模型，直到模型自己决定停下来。**

后面所有脚本（s02-s12）都是在这个循环上叠加新能力——工具更多、状态更丰富、隔离更彻底，但核心循环从未改变。
