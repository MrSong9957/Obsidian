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
