---
name: s19 MCP 与插件
description: 工具不必都写死在主程序里——外部进程也可以把能力接进你的 agent
type: concept
updated: 2026-05-07
tags: agent, learn-shareai, s19
source: https://learn.shareai.run/zh/s19/
---

> _工具不必都写死在主程序里。外部进程也可以把能力接进你的 agent。_

## 这一章到底在讲什么

前面所有章节里，工具基本写在你自己的 Python 代码里。但真实系统走到一定阶段，会很自然地需要：

> "能不能让外部程序也把工具接进来，而不用每次都改主程序？"

这就是 MCP 要解决的问题。

## 先用最简单的话解释 MCP

**一套让 agent 和外部工具程序对话的统一协议。**

主线：

1. 启动一个外部工具服务进程
2. 问它"你有哪些工具"
3. 当模型要用它的工具时，把请求转发给它
4. 再把结果带回 agent 主循环

## 最小心智模型

```
LLM
|
| asks to call a tool
v
Agent tool router
|
+-- native tool -> 本地 Python handler
|
+-- MCP tool -> 外部 MCP server
|
v
return result
```

## 最小系统里最重要的三件事

### 1. MCP client

负责启动外部进程、发送请求、接收响应。

### 2. 工具名前缀规则

```
mcp__{server}__{tool}

例如：
mcp__postgres__query
mcp__browser__open_tab
```

### 3. 统一路由器

本地工具交给本地 handler，MCP 工具交给 MCP client。

## Plugin 又是什么

MCP 解决"外部工具怎么通信"，plugin 解决"这些外部工具配置怎么被发现"。

```json
{
    "name": "my-db-tools",
    "version": "1.0.0",
    "mcpServers": {
        "postgres": {
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-postgres"]
        }
    }
}
```

## 最小实现步骤

### 第一步：写一个 MCPClient

至少要有三个能力：`connect()`、`list_tools()`、`call_tool()`。

### 第二步：把外部工具标准化成 agent 能看懂的工具定义

### 第三步：加前缀

区分本地工具和外部工具。

### 第四步：写一个 router

```python
if tool_name.startswith("mcp__"):
    return mcp_router.call(tool_name, arguments)
else:
    return native_handler(arguments)
```

### 第五步：仍然走同一条权限管道

**MCP 工具虽然来自外部，但不能绕开 permission。**

## 如何接到整个系统里

```
启动时
-> PluginLoader 找到 manifest
-> 得到 server 配置
-> MCP client 连接 server
-> list_tools 并标准化名字
-> 和 native tools 一起合并进同一个工具池

运行时
-> LLM 产出 tool_use
-> 统一权限闸门
-> native route 或 mcp route
-> 结果标准化
-> tool_result 回到同一个主循环
```

**进入方式不同，但进入后必须回到同一条控制面和执行面。**

## Plugin、MCP Server、MCP Tool 不要混成一层

| 层级 | 它是什么 | 它负责什么 |
| --- | --- | --- |
| plugin manifest | 一份配置声明 | 发现和启动哪些 server |
| MCP server | 一个外部进程/连接对象 | 对外暴露一组能力 |
| MCP tool | server 暴露的一项具体调用能力 | 真正被模型点名调用 |

最短的话说：

* plugin 负责"发现"
* server 负责"连接"
* tool 负责"调用"

## 一句话记住

**MCP 的本质，不是协议名词堆砌，而是把外部工具安全、统一地接进你的 agent。**

---

# 新版 MCP Tools 

根据 [Learn Claude Code](https://learn.shareai.run/zh/s19/) 课程第 **s19 MCP Tools（外接工具与标准协议）** 章节的正文，其核心概念、工作原理与教学版代码实现提炼如下：

---

## 1. 核心痛点与解决思路

* **痛点：** 在此前的架构中（s01 到 s18），Agent 的所有工具（如 `bash`、`read_file`、`task`、`worktree`）都是硬编码手写的。如果企业想接入外部的 Jira API、部署系统或 Notion 知识库，就必须为每个服务手动编写一套 Schema 定义、输入验证与错误处理代码，无法低成本扩展。
* **解决思路：** **引入 Model Context Protocol（MCP，模型上下文协议）**。建立一套标准的“发现与调用”契约。外部服务只需实现 `tools/list`（列出工具有什么）与 `tools/call`（执行工具），Agent 就能通过通用的 **MCP Tool Bridge（工具桥接器）** 动态发现并装配它们，不管外部服务是用什么语言编写的。

---

## 2. 核心工作原理与架构

[Model Context Protocol](https://learn.shareai.run/zh/s19/) 通过动态组装打破了内置工具的局限：

1. **动态发现：** Lead 智能体调用 `connect_mcp` 工具连接外部服务，服务实时返回它所支持的工具集。
2. **命名空间隔离：** 为防止不同外部服务的工具名冲突，组装时统一加前缀规范化为 `mcp__<server>__<tool>`。
3. **无缓存刷新：** 每次连接新 MCP 服务后，工具池和系统提示词（System Prompt）会动态发生变化，系统必须**立即卸载并失效原有的 Prompt 缓存**，重新组装上下文，否则模型无法识别到新工具。

---

## 3. 核心功能代码实现

### ① MCP 客户端模拟 (`MCPClient`)

教学版通过内存 Handler 模拟外部 Server 的 `tools/list` 和 `tools/call`（生产级则是通过 stdio 管道或网络发送标准 JSON-RPC 请求）：

```python
class MCPClient:
    def __init__(self, name: str):
        self.name = name
        self.tools: list[dict] = []        # 存放动态发现的工具 Schema 列表
        self._handlers: dict[str, callable] = {} # 存放对应的执行函数指针

    def register(self, tool_defs: list[dict], handlers: dict[str, callable]):
        """模拟 MCP 协议中的 tools/list 发现机制"""
        self.tools = tool_defs
        self._handlers = handlers

    def call_tool(self, tool_name: str, args: dict) -> str:
        """模拟 MCP 协议中的 tools/call 调用响应机制"""
        handler = self._handlers.get(tool_name)
        if not handler:
            return f"MCP error: unknown tool '{tool_name}'"
        try:
            return handler(**args)
        except Exception as e:
            return f"MCP execution error: {str(e)}"

```

### ② 动态工具池组装算法 (`assemble_tool_pool`)

将静态内置工具与所有已连接的外部 MCP 工具融合成统一的工具映射表，并执行安全名称规范化：

```python
import re

mcp_clients: dict[str, MCPClient] = {}
BUILTIN_TOOLS = [...] # 内置工具 Schema 数组
BUILTIN_HANDLERS = {...} # 内置工具执行映射表

def normalize_mcp_name(name: str) -> str:
    # 严格防御：所有非 [a-zA-Z0-9_-] 的字符统一替换为下划线，防注入与冲突
    return re.sub(r'[^a-zA-Z0-9_-]', '_', name)

def assemble_tool_pool() -> tuple[list[dict], dict]:
    tools = list(BUILTIN_TOOLS)
    handlers = dict(BUILTIN_HANDLERS)
    
    # 遍历所有通过 mcp 连进来的外部客户端
    for server_name, mcp_client in mcp_clients.items():
        safe_server = normalize_mcp_name(server_name)
        
        for tool_def in mcp_client.tools:
            safe_tool = normalize_mcp_name(tool_def["name"])
            # 核心：使用双下划线拼接，形成全链路隔离的命名空间
            prefixed_name = f"mcp__{safe_server}__{safe_tool}"
            
            # 克隆 Schema 并重写名称
            mcp_tool_def = dict(tool_def)
            mcp_tool_def["name"] = prefixed_name
            
            # 教学版在描述中附加权限标注 (readOnly) / (destructive) 供大模型识别
            tools.append(mcp_tool_def)
            
            # 使用闭包完成运行时动态分发绑定
            handlers[prefixed_name] = (
                lambda *, c=mcp_client, t=tool_def["name"], **kw: c.call_tool(t, kw)
            )
            
    return tools, handlers

```

### ③ 无缓存的 Agent Loop 演进

由于 MCP 工具接入会改变工具池，不能再像旧章节（s10-s18）一样对基础 Prompt 进行长期持久化缓存，必须实时构建：

```python
def agent_loop(messages, context):
    while True:
        # 核心逻辑：每次循环开始前，实时重新动态构建工具池与 System Prompt
        tools, handlers = assemble_tool_pool()
        system_prompt = assemble_system_prompt(context, tools)
        
        response = client.messages.create(
            model=MODEL, system=system_prompt, messages=messages, tools=tools
        )
        messages.append({"role": "assistant", "content": response.content})
        
        if response.stop_reason != "tool_use":
            break
            
        results = []
        has_connected_mcp = False
        
        for block in response.content:
            if block.type == "tool_use":
                if block.name == "connect_mcp":
                    has_connected_mcp = True
                
                # 在融合后的动态 handlers 中查表分发执行
                handler = handlers.get(block.name)
                output = handler(**block.input) if handler else "Tool not found"
                results.append({"type": "tool_result", "tool_use_id": block.id, "content": output})
                
        messages.append({"role": "user", "content": results})
        
        # 核心防御：如果本轮调用了连接工具，必须在下一轮调用 LLM 前强制立刻重刷
        if has_connected_mcp:
            tools, handlers = assemble_tool_pool()

```

---

## 4. 教学版 vs. 生产级 Claude Code 源码的差异

在真实的 [Claude Code 工业级生产级源码](https://learn.shareai.run/zh/s19/) 中，MCP 体系的实现非常庞大且硬核：

* **6 种 Transport 传输模式支持：** 教学版只演示了进程内的 mock 映射。而 Claude Code 源码（`services/mcp/client.ts`）完整支持 **6 种通信传输层**：`stdio`（通过子进程标准输入输出跨平台调用）、`sse`（Server-Sent Events）、`http`、`ws`（WebSocket）、`sse-ide` 和 `sdk`。在连接远程服务器时，系统甚至支持对多达 20 个远程 Server 进行批量并发拉取。
* **精密的全局缓存断点控制：** 生产级在合并工具集时，不会无序混合。`assembleToolPool()` 会将内置工具与 MCP 工具分开各自排序，并**确保内置工具始终置于数组最前方**。因为系统的 `claude_code_system_cache_policy`（全局 Prompt 缓存切片策略）会将断点精准卡在最后一个内置工具的尾部，混排会直接破坏大模型的缓存命中率，导致 Token 费用暴涨。
* **反向 Channel 通知机制 (`channelNotification.ts`)：** 教学版仅支持 Agent 单向调用 MCP 服务的工具（Agent $\rightarrow$ Server）。真实源码中支持**双向反向通知**：当外部 Server 声明了 `claude/channel` 特性后，它可以反向向 Agent 推送特定的 XML 标签内容，甚至发起 `permission_request` 权限申请，让用户在 CLI 终端输入 5 位短 ID 完成远程确认交互。
* **完整的 OAuth 2.0 + PKCE 认证体系：** 真实源码（`auth.ts`）内嵌了符合 RFC 标准的客户端认证流。不仅能调起本地浏览器回调获取授权码，还能将 Token 自动持久化至系统的安全硬件（macOS Keychain / Linux 加密文件 / Windows 凭据管理器）中，并在过期前 5 分钟在后台静默刷新，支持跨应用访问（XAA）防反复弹窗。