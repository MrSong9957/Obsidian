# wiki-query MCP 接入指南

把 Obsidian 知识库通过 wiki-query MCP server 暴露给 AI agent。**本地 stdio 传输，无云无服务器**——agent 拉起 wiki-query.exe 子进程，通过管道通信。

## 前置准备

### 1. 编译 release 二进制（首次）

```bash
cd E:\Files\Obsidian
bash db/wiki-query/build.sh
# 产物:db/wiki-query/wiki-query.exe
```

> release 首次编译较慢（rmcp + tokio 优化）。日常开发可用 debug 版（`cargo build`，产物在 `target/debug/`）。

### 2. 同步索引（首次 + 每次 git 操作后）

```bash
cd E:\Files\Obsidian
db/wiki-query/wiki-query.exe sync
```

建立/更新 SQLite FTS5 索引（`db/wiki.db`）。建议配 git post-commit hook 自动跑（见下）。

## 5 个工具

| 工具 | 用途 |
|------|------|
| `wiki_search` | FTS5 全文检索（支持中文），返回元数据列表 |
| `wiki_list` | 按标签/类型/目录结构化过滤列出文档 |
| `wiki_sync` | 同步索引 + 查重报告（scope=incremental/full） |
| `wiki_get_project_rules` | 获取注入到项目根指令文件的使用规则文本 |
| `wiki_check_scope` | 判断文件路径是否在 wiki 管辖范围内 |

## 各工具接入配置

### Claude Code

项目根 `.mcp.json`（仓库内，团队共享）或 `~/.claude/mcp.json`（个人）：

```json
{
  "mcpServers": {
    "wiki": {
      "command": "E:\\Files\\Obsidian\\db\\wiki-query\\wiki-query.exe",
      "args": ["serve"],
      "cwd": "E:\\Files\\Obsidian"
    }
  }
}
```

> **关键**：`cwd` 必须是 Obsidian vault 根（含 wiki/ sources/ 目录）。wiki-query serve 以 cwd 作为 vault_root（也可用 `WIKI_VAULT_ROOT` 环境变量指定）。若 cwd 错误，启动时会有 stderr 警告。

配置后重启 Claude Code，`/mcp` 命令应显示 wiki server + 5 个工具。

### MiMo-Code / OpenCode

`~/.config/opencode/mimocode.jsonc`（或项目内 `.mimocode/mimocode.jsonc`）：

```jsonc
{
  "mcp": {
    "wiki": {
      "type": "local",
      "command": ["E:\\Files\\Obsidian\\db\\wiki-query\\wiki-query.exe", "serve"],
      "enabled": true,
      "environment": {
        "WIKI_VAULT_ROOT": "E:\\Files\\Obsidian"
      }
    }
  }
}
```

> 用 `environment.WIKI_VAULT_ROOT` 指定 vault（比依赖 cwd 更可靠，不同 MCP 客户端默认 cwd 不一致）。

### Cursor

`Settings → MCP → Add MCP Server`，或 `.cursor/mcp.json`：

```json
{
  "mcpServers": {
    "wiki": {
      "command": "E:\\Files\\Obsidian\\db\\wiki-query\\wiki-query.exe",
      "args": ["serve"],
      "env": {
        "WIKI_VAULT_ROOT": "E:\\Files\\Obsidian"
      }
    }
  }
}
```

### ZCode

ZCode 的 MCP 配置（参考其文档，格式类似）：

```json
{
  "mcpServers": {
    "wiki": {
      "command": "E:\\Files\\Obsidian\\db\\wiki-query\\wiki-query.exe",
      "args": ["serve"],
      "env": {
        "WIKI_VAULT_ROOT": "E:\\Files\\Obsidian"
      }
    }
  }
}
```

## git post-commit hook（自动 sync 索引）

`.git/hooks/post-commit`：

```sh
#!/bin/sh
cd "$(git rev-parse --show-toplevel)"
./db/wiki-query/wiki-query.exe sync > /dev/null 2>&1
```

```bash
chmod +x .git/hooks/post-commit  # Unix;Windows 用 git bash 或直接运行
```

这样每次 commit 后索引自动更新，agent 立即可查到新内容。

## 验证接入

### 方法 1：MCP Inspector（推荐）

```bash
npx @modelcontextprotocol/inspector E:\Files\Obsidian\db\wiki-query\wiki-query.exe serve
```

浏览器打开 Inspector，应看到 5 个 wiki_ 工具，可直接测试调用。

### 方法 2：命令行手动验证

```bash
# 发 initialize 握手
cd E:\Files\Obsidian
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}' | db/wiki-query/wiki-query.exe serve
```

应返回含 serverInfo 的 JSON-RPC 响应。

## 故障排除

| 症状 | 原因 | 解决 |
|------|------|------|
| search 永远返回空 | DB 未 sync 或 cwd 错误 | 跑 `wiki-query sync`;确认 `WIKI_VAULT_ROOT` 指向 vault 根 |
| 启动警告 "未找到 wiki/ 或 sources/" | cwd 不是 vault 根 | 设 `WIKI_VAULT_ROOT` 或确保 agent 在 vault 根启动 |
| 中文搜不到 | jieba 分词 + sync 未跑 | 确认 sync 过;FTS5 用 cut_for_search 细分，单字也应命中 |
