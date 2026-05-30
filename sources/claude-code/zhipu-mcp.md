---
name: 网络与 API（详细）
description: MCP安装完整指南、配置参考、故障排除 — 安装或排查 MCP 问题时参考
type: concept
updated: 2026-04-17
tags: network, api, mcp, installation, troubleshooting
source: https://example.com
---
# 智谱 AI MCP 服务器安装指南（完整版）

本文档介绍如何在 Claude Code 中安装智谱 AI 提供的 4 个 MCP 服务器，为 Claude 增加视觉理解、联网搜索、网页读取和开源仓库查询能力。

## 前置条件

- **Claude Code** 已安装并可用
- **Node.js** >= v18（视觉理解 MCP 需要）
- **智谱 API Key**：在 [open.bigmodel.cn](https://open.bigmodel.cn/) 注册后获取

## MCP 服务器一览

| 名称 | 类型 | 功能 | 传输方式 |
|---|---|---|---|
| **zai-mcp-server** | 视觉理解 | 截图转代码、OCR、图表分析、视频理解 | stdio（本地） |
| **web-search-prime** | 联网搜索 | 实时网络搜索，返回标题、摘要、来源 | HTTP（云端） |
| **web-reader** | 网页读取 | 抓取网页内容，提取正文、元数据、链接 | HTTP（云端） |
| **zread** | 开源仓库 | 搜索 GitHub 仓库文档、查看目录、读文件 | HTTP（云端） |

## 各服务器提供的工具

**zai-mcp-server（视觉理解）**

| 工具 | 说明 |
|---|---|
| `ui_to_artifact` | UI 截图转代码/设计规范 |
| `extract_text_from_screenshot` | OCR 文本提取 |
| `diagnose_error_screenshot` | 错误截图分析 |
| `understand_technical_diagram` | 技术图表解读 |
| `analyze_data_visualization` | 图表/仪表板分析 |
| `ui_diff_check` | UI 截图对比 |
| `image_analysis` | 通用图像理解 |
| `video_analysis` | 视频分析（MP4/MOV/M4V，最大 8MB） |

**web-search-prime（联网搜索）**

| 工具 | 说明 |
|---|---|
| `webSearchPrime` | 网络搜索，返回标题、URL、摘要、站点名称 |

**web-reader（网页读取）**

| 工具 | 说明 |
|---|---|
| `webReader` | 从 URL 抓取网页内容，返回标题、正文、元数据 |

**zread（开源仓库）**

| 工具 | 说明 |
|---|---|
| `search_doc` | 搜索 GitHub 仓库的文档、新闻、Issue、PR |
| `get_repo_structure` | 获取 GitHub 仓库目录结构 |
| `read_file` | 读取 GitHub 仓库中指定文件内容 |

## 方法一：Claude Code 一键安装（推荐）

将 `<YOUR_API_KEY>` 替换为你的智谱 API Key，逐条执行：

### 1. 视觉理解 MCP（stdio 类型）

```bash
claude mcp add -s user zai-mcp-server \
  --env Z_AI_API_KEY=<YOUR_API_KEY> \
  -- npx -y "@z_ai/mcp-server"
```

### 2. 联网搜索 MCP（HTTP 类型）

```bash
claude mcp add -s user -t http web-search-prime \
  https://open.bigmodel.cn/api/mcp/web_search_prime/mcp \
  --header "Authorization: Bearer <YOUR_API_KEY>"
```

### 3. 网页读取 MCP（HTTP 类型）

```bash
claude mcp add -s user -t http web-reader \
  https://open.bigmodel.cn/api/mcp/web_reader/mcp \
  --header "Authorization: Bearer <YOUR_API_KEY>"
```

### 4. 开源仓库 MCP（HTTP 类型）

```bash
claude mcp add -s user -t http zread \
  https://open.bigmodel.cn/api/mcp/zread/mcp \
  --header "Authorization: Bearer <YOUR_API_KEY>"
```

### 一键安装助手（替代方案）

智谱提供了交互式安装工具，可以一次性配置多个 MCP：

```bash
npx @z_ai/coding-helper
```

按提示选择要安装的服务，工具会自动完成配置。

## 方法二：手动配置

编辑 `~/.claude.json`（用户级配置），在 `mcpServers` 字段中添加：

```json
{
  "mcpServers": {
    "zai-mcp-server": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@z_ai/mcp-server"],
      "env": {
        "Z_AI_API_KEY": "<YOUR_API_KEY>"
      }
    },
    "web-search-prime": {
      "type": "http",
      "url": "https://open.bigmodel.cn/api/mcp/web_search_prime/mcp",
      "headers": {
        "Authorization": "Bearer <YOUR_API_KEY>"
      }
    },
    "web-reader": {
      "type": "http",
      "url": "https://open.bigmodel.cn/api/mcp/web_reader/mcp",
      "headers": {
        "Authorization": "Bearer <YOUR_API_KEY>"
      }
    },
    "zread": {
      "type": "http",
      "url": "https://open.bigmodel.cn/api/mcp/zread/mcp",
      "headers": {
        "Authorization": "Bearer <YOUR_API_KEY>"
      }
    }
  }
}
```

### SSE 回退配置

部分客户端不支持 StreamableHTTP，可改用 SSE 传输：

```json
{
  "web-search-prime": {
    "type": "sse",
    "url": "https://open.bigmodel.cn/api/mcp/web_search_prime/sse?Authorization=<YOUR_API_KEY>"
  },
  "web-reader": {
    "type": "sse",
    "url": "https://open.bigmodel.cn/api/mcp/web_reader/sse?Authorization=<YOUR_API_KEY>"
  },
  "zread": {
    "type": "sse",
    "url": "https://open.bigmodel.cn/api/mcp/zread/sse?Authorization=<YOUR_API_KEY>"
  }
}
```

## 其他客户端配置

### Cline（VS Code 插件）

在 VS Code 设置中找到 `cline.mcpServers`，添加与上方「手动配置」相同的 JSON。

### OpenCode

编辑 `~/.config/opencode/opencode.json`，在 `mcp` 字段中配置。格式与 Claude Code 不同：本地 MCP 使用 `"type": "local"` + `"command"` 数组，远程 MCP 使用 `"type": "remote"` + `"url"`，详见 [[sources/opencode.md|OpenCode 配置指南]]。

### Cursor

Settings -> MCP Servers -> 添加新的 MCP Server，填入对应的 URL 和 Header。

## 验证安装

安装完成后，在 Claude Code 中执行：

```bash
claude mcp list
```

确认 4 个 MCP 服务器均显示 **Connected** 状态。

也可以在对话中直接测试，例如：

```
帮我搜索一下 Claude Code 最新版本的信息
```

如果联网搜索 MCP 正常工作，Claude 会调用 `webSearchPrime` 工具返回实时结果。

## 从 .env 文件读取 API Key 安装

如果 API Key 已存在 `.env` 文件中，注意以下坑：

**Key 变量名可能不叫 `ZHIPU_API_KEY`**。通过 Anthropic 兼容接口使用智谱时，Key 存为 `ANTHROPIC_API_KEY`，base URL 指向 `open.bigmodel.cn`。查找时用这个变量名。

**`.env` 文件可能有 Windows 换行符（`\r\n`）**。直接 `source .env` 会报 `$'\r': command not found`，导致所有安装命令失败。用以下方式提取 Key：

```bash
# 安全提取，去除 \r
KEY=$(grep ANTHROPIC_API_KEY .env | tr -d '\r' | cut -d= -f2)

# 然后用 $KEY 代替 <YOUR_API_KEY>
claude mcp add -s user zai-mcp-server --env Z_AI_API_KEY="$KEY" -- npx -y "@z_ai/mcp-server"
claude mcp add -s user -t http web-search-prime https://open.bigmodel.cn/api/mcp/web_search_prime/mcp --header "Authorization: Bearer $KEY"
claude mcp add -s user -t http web-reader https://open.bigmodel.cn/api/mcp/web_reader/mcp --header "Authorization: Bearer $KEY"
claude mcp add -s user -t http zread https://open.bigmodel.cn/api/mcp/zread/mcp --header "Authorization: Bearer $KEY"
```

> **实测记录（2026-04-08）**：4 个 MCP 均安装成功并通过测试。视觉理解 MCP 对部分 Wikipedia 图片 URL 报 400 错误（图片格式解析失败），但对 CDN 直链正常，建议优先使用稳定的 CDN 图片地址。

## 故障排除

### MCP 显示 "Failed to connect"

1. **检查 API Key**：确认 Key 有效且未过期
2. **检查网络**：确认能访问 `open.bigmodel.cn`
3. **重启 Claude Code**：退出后重新启动

### 视觉理解 MCP 启动失败

1. **检查 Node.js 版本**：需要 >= v18
   ```bash
   node --version
   ```
2. **手动测试 npx**：
   ```bash
   Z_AI_API_KEY=your_key npx -y "@z_ai/mcp-server"
   ```

### 视觉理解 MCP 图片分析报 400

- 部分图片 URL（如 Wikipedia 缩略图）会触发"图片输入格式/解析错误"
- 改用 CDN 直链或本地文件路径，成功率更高

### HTTP 类型 MCP 返回 401

- 确认 `Authorization` Header 格式正确：`Bearer <key>`（注意 Bearer 和 key 之间有空格）
- 检查 Key 是否有对应服务的权限

### SSE 模式连接失败

- 部分客户端需要使用 SSE URL 中的 `?Authorization=` 参数而非 Header
- 尝试切换 SSE 和 HTTP 类型

## 参考链接

- [智谱 AI 开放平台](https://open.bigmodel.cn/)
- [视觉理解 MCP 文档](https://docs.bigmodel.cn/cn/coding-plan/mcp/vision-mcp-server)
- [联网搜索 MCP 文档](https://docs.bigmodel.cn/cn/coding-plan/mcp/search-mcp-server)
- [网页读取 MCP 文档](https://docs.bigmodel.cn/cn/coding-plan/mcp/reader-mcp-server)
- [开源仓库 MCP 文档](https://docs.bigmodel.cn/cn/coding-plan/mcp/zread-mcp-server)
- [@z_ai/mcp-server npm](https://www.npmjs.com/package/@z_ai/mcp-server)
