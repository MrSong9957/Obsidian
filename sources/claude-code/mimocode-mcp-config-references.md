---
name: MiMoCode MCP 配置详细参考
description: 完整的 MiMoCode MCP 配置调试过程、配置文件位置对比、验证步骤、问题排查
type: reference
updated: 2026-06-27
tags: mimocode, mcp, wiki-query, configuration, troubleshooting, debugging
---

# MiMoCode MCP 配置详细参考

## 调试过程时间线

### 2026-06-27 会话

1. **初始问题**：用户请求接入 wiki MCP，但工具未在会话中激活
2. **配置检查**：发现 `.mcp.json` 配置正确，但 MiMoCode 不读取
3. **根因定位**：MiMoCode 读取 `~/.claude.json` → `mcpServers`（全局配置）
4. **解决方案**：添加 wiki 服务器配置到全局配置文件
5. **验证成功**：重启后 wiki_search、wiki_list、wiki_sync 工具可用

## 配置文件对比

### MiMoCode 配置

```json
// ~/.claude.json (全局配置)
{
  "mcpServers": {
    "zai-mcp-server": { ... },
    "web-reader": { ... },
    "web-search-prime": { ... },
    "zread": { ... },
    "wiki": {
      "command": "E:\\Files\\Obsidian\\db\\wiki-query\\wiki-query.exe",
      "args": ["serve"],
      "env": {
        "WIKI_VAULT_ROOT": "E:\\Files\\Obsidian"
      },
      "type": "stdio"
    }
  }
}
```

### Claude Code 配置

```json
// 项目根目录/.mcp.json (项目级配置)
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

## 验证步骤

### 步骤 1：检查配置文件

```bash
# 检查 MiMoCode 全局配置
cat ~/.claude.json | grep -A 10 "wiki"

# 检查项目级配置
cat .mcp.json
```

### 步骤 2：测试 MCP 服务器

```bash
# 测试 CLI 命令（只支持 sync 和 serve）
./db/wiki-query/wiki-query.exe sync
./db/wiki-query/wiki-query.exe serve
```

### 步骤 3：验证工具可用性

在 MiMoCode 会话中测试：
- `wiki_search`：搜索关键词
- `wiki_list`：列出文档
- `wiki_sync`：同步索引

## 问题排查清单

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| MCP 工具未出现 | 配置文件位置错误 | 检查 `~/.claude.json` |
| 工具未生效 | 未重启会话 | 重启 MiMoCode |
| wiki-query 命令不可用 | CLI 限制 | 通过 MCP 协议调用 |
| 索引为空 | 未建立索引 | 运行 `wiki-query sync` |
| 规则未注入 | 未添加规则 | 在 CLAUDE.md 添加规则 |

## 关键命令参考

```bash
# 建立索引
./db/wiki-query/wiki-query.exe sync

# 启动 MCP 服务器（通常由客户端自动启动）
./db/wiki-query/wiki-query.exe serve

# 检查索引状态
ls -la db/wiki.db
```

## 配置模板

### MiMoCode 全局配置

```json
{
  "mcpServers": {
    "wiki": {
      "command": "E:\\Files\\Obsidian\\db\\wiki-query\\wiki-query.exe",
      "args": ["serve"],
      "env": {
        "WIKI_VAULT_ROOT": "E:\\Files\\Obsidian"
      },
      "type": "stdio"
    }
  }
}
```

### CLAUDE.md 规则模板

```markdown
<!-- wiki-query-rules -->
# Wiki-Query 规则

## 工具列表
| 工具 | 用途 |
|------|------|
| wiki_search | FTS5 全文检索 |
| wiki_list | 结构化浏览 |
| wiki_sync | 同步索引 |
| wiki_get_project_rules | 获取规则 |
| wiki_check_scope | 检查文件范围 |

## 写入目标判断
- 精简全面 → wiki/
- 详细过程 → sources/

## 常见工作流
1. 搜索并阅读
2. 写入并索引
3. 提炼记忆到 Wiki
4. 去重
5. 结构化浏览
6. 首次接入 Wiki-Query
```