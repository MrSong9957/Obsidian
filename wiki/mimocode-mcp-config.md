---
name: MiMoCode MCP 配置指南
description: MiMoCode 与 Claude Code 的 MCP 配置差异、wiki-query 接入流程、常见问题排查
type: concept
updated: 2026-06-27
tags: mimocode, mcp, wiki-query, configuration, troubleshooting
source: [[sources/claude-code/mimocode-mcp-config.md]]
---

# MiMoCode MCP 配置指南

## 核心差异

**MiMoCode 与 Claude Code 的 MCP 配置位置不同**：

| 配置文件 | 用途 | MiMoCode 支持 |
|---------|------|--------------|
| `~/.claude.json` → `mcpServers` | 全局 MCP 服务器配置 | ✅ 主配置 |
| 项目根目录 `.mcp.json` | 项目级 MCP 配置 | ❌ 不读取 |

## 快速接入步骤

### 1. 配置 MCP 服务器

在 `~/.claude.json` 的 `mcpServers` 部分添加：

```json
"wiki": {
  "command": "E:\\Files\\Obsidian\\db\\wiki-query\\wiki-query.exe",
  "args": ["serve"],
  "env": {
    "WIKI_VAULT_ROOT": "E:\\Files\\Obsidian"
  },
  "type": "stdio"
}
```

### 2. 注入使用规则

在 `CLAUDE.md` 末尾添加 wiki-query 规则。

### 3. 建立索引

```bash
cd E:\Files\Obsidian
./db/wiki-query/wiki-query.exe sync
```

### 4. 重启会话

重启 MiMoCode 使 MCP 服务器生效。

## 验证方法

| 工具 | 测试命令 | 预期结果 |
|------|---------|---------|
| `wiki_search` | 搜索 "claude code" | 返回相关文档列表 |
| `wiki_list` | 列出 wiki/ 目录 | 返回文档列表 |
| `wiki_sync` | 同步索引 | 显示同步统计 |

## 常见问题

### 问题 1：MCP 工具未出现

**原因**：配置文件位置错误或未重启会话

**解决**：
1. 确认 `~/.claude.json` 包含 wiki 服务器配置
2. 重启 MiMoCode 会话

### 问题 2：wiki-query 命令不可用

**原因**：wiki-query CLI 只支持 `sync` 和 `serve`

**解决**：查询工具必须通过 MCP 协议调用，不能直接命令行执行

### 问题 3：索引为空

**解决**：运行 `wiki-query sync` 建立索引

## 踩坑记录

1. **CLI 命令误解**：尝试 `wiki-query.exe --help`、`get-project-rules` 等命令均失败
   - 这些是 MCP 工具，不是 CLI 命令
   - 需要通过 MCP 协议调用

2. **配置文件混淆**：项目根目录 `.mcp.json` 对 MiMoCode 无效
   - MiMoCode 只读取 `~/.claude.json` 全局配置

3. **规则注入方式**：`wiki_get_project_rules` 工具不可直接调用
   - 需要手动从 skill 文档提取规则内容