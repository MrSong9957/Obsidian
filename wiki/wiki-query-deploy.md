---
name: wiki-query MCP 部署
description: wiki-query MCP server 安装/配置/跨平台分发速查与踩坑 — 接入 wiki-query MCP、排查连接问题、发布新版本时参考
type: concept
updated: 2026-06-25
tags: wiki-query, mcp, deploy, github-actions, rust, cross-platform, release
source: "[[sources/projects/wiki-query-deploy-references]]"
---

# wiki-query MCP 部署

自研 Rust MCP server，为本 Obsidian 知识库提供 FTS5 检索（5 个工具：`wiki_search` / `wiki_list` / `wiki_sync` / `wiki_get_project_rules` / `wiki_check_scope`）。源码 `db/wiki-query/`。

## 安装（给使用者）

详见 `db/wiki-query/README.md`。两种方式：

- **推荐**：从 [Releases](https://github.com/MrSong9957/Obsidian/releases) 下载对应平台二进制，放到 `db/wiki-query/wiki-query[.exe]`
- **自己编译**：`bash db/wiki-query/build.sh`（需 Rust）

## MCP 配置位置（不同工具读不同文件）

| 工具 | 配置文件 | 字段 | 路径示例 |
|------|----------|------|----------|
| Claude Code | 项目根 `.mcp.json` | `mcpServers` | `E:\Files\Obsidian\.mcp.json` |
| ZCode | `~/.zcode/cli/config.json`（用户级） | `mcp.servers` | `C:\Users\<user>\.zcode\cli\config.json` |

两者 command 都指向 `db/wiki-query/wiki-query.exe`，args 为 `["serve"]`，env 设 `WIKI_VAULT_ROOT`。

## 踩坑速查

### 清理编译缓存前先查配置依赖

- 禁止：直接 `cargo clean` 而不检查 MCP 配置指向哪
  - 原因：`.mcp.json` 的 command 若指向 `target/release/wiki-query.exe`，clean 会删掉它，MCP server 启动失败
  - 替代：先把 exe 挪出 `target/` 到 `db/wiki-query/wiki-query.exe`（已解耦），或清理前 `grep command .mcp.json` 确认

← [[sources/projects/wiki-query-deploy-references.md]] L17-32

### ZCode 不读 .mcp.json

- 禁止：假设 `.mcp.json`（Claude 格式）对 ZCode 也生效
  - 原因：ZCode 读 `~/.zcode/cli/config.json` 的 `mcp.servers` 字段，根本不碰 `.mcp.json`。项目级 `.zcode/config.json` 日志显示 `configSourceProject: false`，也没真正加载
  - 替代：查应用日志的实际加载路径（`configSourceUser: true` 的位置最可靠），往那里写配置

← [[sources/projects/wiki-query-deploy-references.md]] L48-75

### target 编译缓存可压缩到只剩 exe

- 420M 的 `target/release/` 里，只有 16M 的 exe 是 MCP 运行时真正需要的，其余 ~400M 是 `deps/`、`build/` 等中间产物
- 验证方法：把 exe 单独拷到空目录跑 JSON-RPC 握手，能独立 serve 即证明无其他依赖
- 禁止：笼统判断"编译产物都是运行依赖"
  - 替代：隔离到干净环境实测运行时依赖

← [[sources/projects/wiki-query-deploy-references.md]] L34-47

### sync CLI 扫描根是 cwd，不是 WIKI_VAULT_ROOT

- `src/sync.rs:79` 写死 `Path::new(".")`，CLI sync 模式用当前工作目录扫描，与 serve 模式的 `WIKI_VAULT_ROOT` 不一致
- 在 `db/wiki-query/` 目录下跑 sync 只扫到 1 个 README，FTS5 索引近乎空，搜什么都是空
- 替代：CLI sync 必须从 vault 根目录执行（`cd <vault根> && db/wiki-query/wiki-query.exe sync`）

← [[sources/projects/wiki-query-deploy-references.md]] L77-92

## 发布新版本

推 `v*` tag 触发 GitHub Actions 三平台编译：

```bash
git tag -a v0.X.0 -m "说明" && git push origin v0.X.0
```

4 平台 matrix（windows / macos-intel / macos-arm / linux）各自编译，产物自动挂到 Release。

关键前提（写在 workflow 里）：
- 4 个 target 都是 runner **原生架构**（非交叉编译），避免 cross 工具链复杂性
- `rusqlite` 用 `bundled` feature → SQLite 编译进二进制，目标机无需系统 SQLite（跨平台可行的前提）
- `jieba-rs` 纯 Rust → 无 C 依赖

← [[sources/projects/wiki-query-deploy-references.md]] L94-130

> 详细调试过程见 [[sources/projects/wiki-query-deploy-references.md]]
