---
name: wiki-query 跨平台部署详细记录
description: wiki-query MCP server 从单机配置到 GitHub Actions 三平台分发的完整过程、踩坑与调试故事
type: reference
updated: 2026-06-25
tags: wiki-query, mcp, deploy, github-actions, rust, cross-platform
---

# wiki-query 跨平台部署详细记录

本文记录 2026-06-25 一次会话中，把自研 wiki-query MCP server 从"只在作者机器上能跑"做成"任何人 clone 后下载即用"的完整过程，包含多个踩坑和修正。

## 背景

wiki-query 是一个 Rust 写的 MCP server（基于 `rmcp` + tokio，stdio 传输），为本 Obsidian 知识库提供 FTS5 全文检索（`wiki_search` / `wiki_list` / `wiki_sync` / `wiki_get_project_rules` / `wiki_check_scope` 五个工具）。源码在 `db/wiki-query/`，索引库是 `db/wiki.db`。

仓库里有一份 `db/wiki-query/README.md` 是面向使用者的安装说明（本文是开发者视角的踩坑记录）。

## 踩坑 1：cargo clean 误删 MCP 依赖的 exe

**起因**：发现 `db/` 目录 3.2G，判断是编译缓存，执行 `cargo clean --manifest-path db/wiki-query/Cargo.toml`。

**问题**：clean 后重新 `cargo build --release`，首次失败（Windows "拒绝访问 os error 5"），第二次重试才成功。

**根因**：`.mcp.json` 配置的 command 指向 `db/wiki-query/target/release/wiki-query.exe`，而 `cargo clean` 把整个 `target/` 删了，导致 MCP server 找不到 exe。

**教训**：
- **清理编译缓存前，必须先 grep 配置文件，确认缓存里没有运行时依赖的产物。** 不能只看路径名（`target` 看起来是纯缓存）就判断可删。
- 正确做法是先把 exe 挪出 `target/`，与编译缓存解耦。

## 踩坑 2：误判"target 没有可压缩空间"

**起因**：清理后 `target/release/` 又有 420M，我说"没有可压缩空间了"。

**问题**：这个判断是错的。420M 里只有 16M 是 MCP 真正需要的 exe，其余 ~400M 全是 `deps/`、`build/` 等编译中间产物。

**验证方法**：把 `wiki-query.exe`（16M）单独拷到一个空目录 `/tmp/exe_test/`，从那里运行 JSON-RPC 握手测试——完全不依赖任何其他文件就能正常 serve。**铁证：exe 能脱离 target 独立运行。**

**教训**：不能把"cargo 编译产物"笼统当成"MCP 运行依赖"。要验证运行时实际依赖，最可靠的方法是**把它隔离到一个干净环境跑一次**。

## 踩坑 3：ZCode 不读 .mcp.json

**起因**：要给 ZCode 配上 wiki-query MCP。

**问题**：项目根的 `.mcp.json`（Claude Code 格式，`mcpServers` 字段）对 ZCode **从未生效过**。UI 里显示了 server 信息（路径、`serve`、`stdio`），但 "0 个工具"。

**排查过程**：
1. 从 ZCode 官方文档确认：ZCode 读的是 `config.json` 的 `mcp.servers` 字段，不是 `.mcp.json`。
2. 先写工作区级 `E:\Files\Obsidian\.zcode\config.json`——重启后 UI 显示两个 wiki 条目，但日志显示 `configSourceProject: false`、`runtimeHasMcpConfig: false`，MCP engine 没真正加载。
3. 改写用户级 `~/.zcode/cli/config.json`（日志里 `configSourceUser: true` 的位置）——重启后工具加载成功（5 个 `wiki_*` 工具可用）。
4. 删掉重复的工作区级配置，只留用户级。

**教训**：
- **不同工具读不同配置文件，不能假设通用。** Claude Code 读 `.mcp.json`（`mcpServers`），ZCode 读 `.zcode/config.json`（`mcp.servers`）。
- 配置不生效时，**看应用日志的实际加载路径**比查文档更可靠。日志里 `configSourceXxx: true/false` 直接告诉你哪个位置被读了。

## 踩坑 4：sync CLI 扫描根写死 cwd

**起因**：手测 `wiki_search` 一直返回空，但库里有 396 个文件。

**问题**：CLI `sync` 子命令的扫描根写死成 `Path::new(".")`（`src/sync.rs:79`），用的是当前工作目录，**没用 `WIKI_VAULT_ROOT` 环境变量**。而 MCP server 模式（`serve`）用的是 `WIKI_VAULT_ROOT`。两者不一致。

**后果**：在 `db/wiki-query/` 目录下跑 `sync`，只会扫到 1 个 README，FTS5 索引几乎是空的，所以搜什么都搜不到。

**教训**：同一程序的 CLI 模式和 server 模式如果对"工作根目录"的解析方式不一致，会埋下难以察觉的坑。调用 CLI sync 时必须从 vault 根目录执行。

## 解决方案：跨平台分发

### 失败的中间方案：build.sh

最初用 `build.sh`（`cargo build` + 拷贝 exe）作为分发方案。**这是退步**：把 Windows 用户也从"直接用 exe"搞成"要自己装 Rust 编译"，Mac/Linux 用户一直没照顾到。

### 正确方案：GitHub Actions 三平台预编译 + Release 分发

`.github/workflows/release.yml`：推 `v*` tag 触发，在 4 个平台 matrix 编译，产物挂到 Release。

```yaml
matrix:
  include:
    - runner: windows-latest
      target: x86_64-pc-windows-msvc
      artifact: wiki-query-windows-x86_64.exe
    - runner: macos-13          # Intel
      target: x86_64-apple-darwin
      artifact: wiki-query-macos-x86_64
    - runner: macos-latest      # Apple Silicon
      target: aarch64-apple-darwin
      artifact: wiki-query-macos-aarch64
    - runner: ubuntu-latest
      target: x86_64-unknown-linux-gnu
      artifact: wiki-query-linux-x86_64
```

关键设计点：
- **4 个 target 都是 runner 原生架构**（非交叉编译），降低风险——避免了 cross 工具链的复杂性。
- **rusqlite 用 `bundled` feature**——SQLite 编译进二进制，目标机器无需装系统 SQLite，这是跨平台可行的前提。
- **jieba-rs 是纯 Rust**——无 C 依赖，跨平台无坑。
- 用 `softprops/action-gh-release@v2` 上传资产，`generate_release_notes: true` 自动生成 changelog。

### 实测结果（v0.1.0，2026-06-25）

| 平台 | 状态 | 耗时 |
|------|------|------|
| Windows x86_64 | ✅ 成功 | 4m5s |
| macOS aarch64 (M 系列) | ✅ 成功 | 1m54s |
| Linux x86_64 | ✅ 成功 | 2m31s |
| macOS x86_64 (Intel) | ⏳ 排队 | `macos-13` runner 资源紧张，队列等待 |

**4 平台编译全部通过**，3 个资产已发布。macOS Intel 是 GitHub Actions 的 `macos-13` runner 资源瓶颈（非 workflow 错误），队列空了会自动补跑。

## 最终架构

```
db/
├── wiki.db                      # SQLite 索引库（运行 sync 生成）
└── wiki-query/
    ├── wiki-query.exe           # 可执行文件（build.sh 或下载 Release 生成）
    ├── src/                     # Rust 源码
    ├── Cargo.toml / Cargo.lock  # 依赖锁定（二进制应用必须提交 Cargo.lock）
    ├── build.sh                 # 本地编译脚本（给想自己编译的人）
    └── README.md                # 面向使用者的安装说明
```

exe 与 `target/` 编译缓存**完全解耦**：`cargo clean` 不再影响 MCP server。

## 相关文件

- `.github/workflows/release.yml` — CI 发布 workflow
- `.gitignore` — 忽略 `wiki-query[.exe]` 和 `target/`
- `.mcp.json` — Claude Code 的 MCP 配置（指向 `db/wiki-query/wiki-query.exe`）
- `~/.zcode/cli/config.json` — ZCode 的 MCP 配置（用户级，不在仓库里）
- `db/wiki-query/README.md` — 使用者安装说明
