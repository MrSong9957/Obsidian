---
name: mcp-builder
description: Build and test MCP (Model Context Protocol) servers that give Claude new capabilities. Use when user wants to create an MCP server, add tools to Claude, integrate external services, or test/debug an existing MCP server.
---

# MCP Server Building Skill

You now have expertise in building MCP (Model Context Protocol) servers. MCP enables Claude to interact with external services through a standardized protocol.

## What is MCP?

MCP servers expose:
- **Tools**: Functions Claude can call (like API endpoints)
- **Resources**: Data Claude can read (like files or database records)
- **Prompts**: Pre-built prompt templates

## Quick Start: Python MCP Server

### 1. Project Setup

```bash
# Create project
mkdir my-mcp-server && cd my-mcp-server
python3 -m venv venv && source venv/bin/activate

# Install MCP SDK
pip install mcp
```

### 2. Basic Server Template

```python
#!/usr/bin/env python3
"""my_server.py - A simple MCP server"""

from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent

# Create server instance
server = Server("my-server")

# Define a tool
@server.tool()
async def hello(name: str) -> str:
    """Say hello to someone.

    Args:
        name: The name to greet
    """
    return f"Hello, {name}!"

@server.tool()
async def add_numbers(a: int, b: int) -> str:
    """Add two numbers together.

    Args:
        a: First number
        b: Second number
    """
    return str(a + b)

# Run server
async def main():
    async with stdio_server() as (read, write):
        await server.run(read, write)

if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
```

### 3. Register with Claude

Add to `~/.claude/mcp.json`:
```json
{
  "mcpServers": {
    "my-server": {
      "command": "python3",
      "args": ["/path/to/my_server.py"]
    }
  }
}
```

## TypeScript MCP Server

### 1. Setup

```bash
mkdir my-mcp-server && cd my-mcp-server
npm init -y
npm install @modelcontextprotocol/sdk
```

### 2. Template

```typescript
// src/index.ts
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";

const server = new Server({
  name: "my-server",
  version: "1.0.0",
});

// Define tools
server.setRequestHandler("tools/list", async () => ({
  tools: [
    {
      name: "hello",
      description: "Say hello to someone",
      inputSchema: {
        type: "object",
        properties: {
          name: { type: "string", description: "Name to greet" },
        },
        required: ["name"],
      },
    },
  ],
}));

server.setRequestHandler("tools/call", async (request) => {
  if (request.params.name === "hello") {
    const name = request.params.arguments.name;
    return { content: [{ type: "text", text: `Hello, ${name}!` }] };
  }
  throw new Error("Unknown tool");
});

// Start server
const transport = new StdioServerTransport();
server.connect(transport);
```

## Advanced Patterns

### External API Integration

```python
import httpx
from mcp.server import Server

server = Server("weather-server")

@server.tool()
async def get_weather(city: str) -> str:
    """Get current weather for a city."""
    async with httpx.AsyncClient() as client:
        resp = await client.get(
            f"https://api.weatherapi.com/v1/current.json",
            params={"key": "YOUR_API_KEY", "q": city}
        )
        data = resp.json()
        return f"{city}: {data['current']['temp_c']}C, {data['current']['condition']['text']}"
```

### Database Access

```python
import sqlite3
from mcp.server import Server

server = Server("db-server")

@server.tool()
async def query_db(sql: str) -> str:
    """Execute a read-only SQL query."""
    if not sql.strip().upper().startswith("SELECT"):
        return "Error: Only SELECT queries allowed"

    conn = sqlite3.connect("data.db")
    cursor = conn.execute(sql)
    rows = cursor.fetchall()
    conn.close()
    return str(rows)
```

### Resources (Read-only Data)

```python
@server.resource("config://settings")
async def get_settings() -> str:
    """Application settings."""
    return open("settings.json").read()

@server.resource("file://{path}")
async def read_file(path: str) -> str:
    """Read a file from the workspace."""
    return open(path).read()
```

## Testing

### Quick Test with MCP Inspector

```bash
npx @anthropics/mcp-inspector python3 my_server.py
```

### Manual JSON-RPC Testing

MCP 基于 JSON-RPC 2.0 over stdio。测试时需要按顺序发送初始化消息：

```bash
# 1. Initialize handshake
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"notifications/initialized","params":{}}' | python3 my_server.py

# 2. List available tools
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"notifications/initialized","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/list","params":{}}' | python3 my_server.py

# 3. Call a tool
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"notifications/initialized","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"hello","arguments":{"name":"World"}}}' | python3 my_server.py
```

### PowerShell 测试（Windows）

```powershell
# Initialize + list tools
$init = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
$notif = '{"jsonrpc":"2.0","id":2,"method":"notifications/initialized","params":{}}'
$list = '{"jsonrpc":"2.0","id":3,"method":"tools/list","params":{}}'
"$init`n$notif`n$list" | py my_server.py
```

### 自动化测试脚本

```python
#!/usr/bin/env python3
"""test_mcp.py - Automated MCP server testing"""
import subprocess, json, sys

def test_server(script, commands):
    """Send JSON-RPC commands to MCP server and collect responses."""
    messages = [
        {"jsonrpc":"2.0","id":1,"method":"initialize",
         "params":{"protocolVersion":"2024-11-05","capabilities":{},
                   "clientInfo":{"name":"test","version":"1.0"}}},
        {"jsonrpc":"2.0","id":2,"method":"notifications/initialized","params":{}},
    ] + commands

    input_data = "\n".join(json.dumps(m) for m in messages)
    result = subprocess.run(
        ["python3", script],
        input=input_data, capture_output=True, text=True, timeout=10
    )

    for line in result.stdout.strip().split("\n"):
        if line:
            resp = json.loads(line)
            print(f"  id={resp.get('id')}: {json.dumps(resp.get('result', resp.get('error')), indent=2)[:200]}")

    if result.stderr:
        print(f"  stderr: {result.stderr[:200]}")

# Example usage:
# test_server("my_server.py", [
#     {"jsonrpc":"2.0","id":3,"method":"tools/list","params":{}},
#     {"jsonrpc":"2.0","id":4,"method":"tools/call",
#      "params":{"name":"hello","arguments":{"name":"Test"}}},
# ])
```

## Best Practices

1. **Clear tool descriptions**: Claude uses these to decide when to call tools
2. **Input validation**: Always validate and sanitize inputs
3. **Error handling**: Return meaningful error messages
4. **Async by default**: Use async/await for I/O operations
5. **Security**: Never expose sensitive operations without auth
6. **Idempotency**: Tools should be safe to retry
