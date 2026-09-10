# wiki-query MCP 接入指南

把 Obsidian 知识库通过 wiki-query MCP server 暴露给 AI agent。**本地 stdio 传输，无云无服务器**——agent 经 `mcp/wiki-query/serve.sh` 启动器拉起 wiki-query 子进程，通过管道通信。

## 前置准备

### 1. 编译 release 二进制（首次）

```bash
cd "$(git rev-parse --show-toplevel)"
bash mcp/wiki-query/build.sh
# 产物:mcp/wiki-query/wiki-query[.exe](Windows 带 .exe;本地构建产物,Git 不携带)
```

> release 首次编译较慢（rmcp + tokio 优化）。日常开发可用 debug 版（`cargo build`，产物在 `target/debug/`）。

### 2. 同步索引（首次 + 每次 git 操作后）

```bash
cd "$(git rev-parse --show-toplevel)"
mcp/wiki-query/wiki-query.exe sync   # Unix/macOS/Linux 产物名无 .exe
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

项目根 `.mcp.json`（仓库内，团队共享，已按此配置）：

```json
{
  "mcpServers": {
    "wiki": {
      "command": "bash",
      "args": ["mcp/wiki-query/serve.sh"],
      "cwd": "."
    }
  }
}
```

> **关键**：vault root 由 `serve.sh` 启动器解析为仓库根（默认值；如显式设置 `WIKI_VAULT_ROOT` 环境变量则外部值优先），不依赖任何个人绝对路径。若解析结果错误，启动时会有 stderr 警告。前提：Windows 需 Git Bash（`bash` 在 PATH 上）。

配置后重启 Claude Code，`/mcp` 命令应显示 wiki server + 5 个工具。

### MiMo-Code / OpenCode

`~/.config/opencode/mimocode.jsonc`（或项目内 `.mimocode/mimocode.jsonc`）：

```jsonc
{
  "mcp": {
    "wiki": {
      "type": "local",
      "command": ["bash", "mcp/wiki-query/serve.sh"],
      "enabled": true
    }
  }
}
```

> vault root 由 `serve.sh` 解析为仓库根，无需配置 `environment`。若该客户端不把工作目录设为项目根（相对路径解析失败），改用其支持的变量/绝对写法指向 `mcp/wiki-query/serve.sh`；确需非仓库根 vault 时才设 `environment.WIKI_VAULT_ROOT`。

### Cursor

`Settings → MCP → Add MCP Server`，或 `.cursor/mcp.json`：

```json
{
  "mcpServers": {
    "wiki": {
      "command": "bash",
      "args": ["mcp/wiki-query/serve.sh"]
    }
  }
}
```

> vault root 由 `serve.sh` 解析为仓库根。若 Cursor 的工作目录不是项目根（启动失败提示找不到脚本），改用其支持的写法指向 `mcp/wiki-query/serve.sh`。

### ZCode

仓库根 `.mcp.json` 已按此配置（ZCode stdio 支持 `command`/`args`/`cwd`/`env` 字段）：

```json
{
  "mcpServers": {
    "wiki": {
      "command": "bash",
      "args": ["mcp/wiki-query/serve.sh"],
      "cwd": "."
    }
  }
}
```

## git post-commit hook（自动 sync 索引）

`.git/hooks/post-commit`：

```sh
#!/bin/sh
cd "$(git rev-parse --show-toplevel)"
./mcp/wiki-query/wiki-query.exe sync > /dev/null 2>&1   # Unix/macOS/Linux 产物名无 .exe
```

```bash
chmod +x .git/hooks/post-commit  # Unix;Windows 用 git bash 或直接运行
```

这样每次 commit 后索引自动更新，agent 立即可查到新内容。

## 验证接入

### 方法 1：MCP Inspector（推荐）

```bash
npx @modelcontextprotocol/inspector bash mcp/wiki-query/serve.sh
```

在仓库根执行。浏览器打开 Inspector，应看到 5 个 wiki_ 工具，可直接测试调用。

### 方法 2：命令行手动验证

```bash
# 发 initialize 握手（在仓库根执行）
cd "$(git rev-parse --show-toplevel)"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}' | bash mcp/wiki-query/serve.sh
```

应返回含 serverInfo 的 JSON-RPC 响应。

## 故障排除

| 症状 | 原因 | 解决 |
|------|------|------|
| search 永远返回空 | DB 未 sync 或 vault root 错误 | 跑 `wiki-query sync`;确认 `WIKI_VAULT_ROOT`（如设置过）指向 vault 根 |
| 启动警告 "未找到 wiki/ 或 sources/" | 解析出的 vault root 不是仓库根 | 移除外部 `WIKI_VAULT_ROOT`（默认用仓库根）或将其设为 vault 根 |
| 中文搜不到 | jieba 分词 + sync 未跑 | 确认 sync 过;FTS5 用 cut_for_search 细分，单字也应命中 |
