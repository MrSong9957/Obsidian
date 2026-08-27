# db/

本目录存放数据库相关文件，由 `wiki-query` 工具管理。

## 目录结构

```
db/
├── README.md           # 本文件
├── wiki.db             # SQLite 数据库（已 gitignore，运行 sync 可重建）
└── wiki-query/         # Rust 查询工具源码及编译产物
    ├── Cargo.toml
    ├── Cargo.lock
    ├── build.sh        # 一键编译脚本（编译 + 拷贝二进制到本目录）
    ├── src/
    │   ├── main.rs     # CLI 入口，命令路由（sync/serve）
    │   ├── lib.rs      # 库入口
    │   ├── mcp_server.rs # MCP server 与 5 个工具定义
    │   ├── core.rs     # FTS 查询构建、search/list/查重检索
    │   ├── db.rs       # 数据库路径与初始化
    │   ├── models.rs   # SearchResult/SimilarPair/SyncReport 等结构体
    │   ├── sync.rs     # 同步与查重逻辑
    │   ├── fts.rs      # jieba 中文分词
    │   ├── frontmatter.rs # frontmatter 解析/序列化
    │   └── error.rs    # 错误类型
    ├── tests/          # 集成测试
    ├── target/         # Rust 编译缓存（已 gitignore）
    └── wiki-query.exe  # 编译后的二进制（build.sh 拷贝到此处）
```

## 使用方式

```bash
# 同步 MD 文件到数据库（首次全量，后续增量）
./db/wiki-query/wiki-query.exe sync

# 启动 MCP server（agent 通过 stdio 交互）
./db/wiki-query/wiki-query.exe serve
```

查询/读写走 MCP（agent 调用 `wiki_search` 等工具），非命令行。

## 性能对比

以 369 篇 MD 文档为基准，搜索 "learning"：

| 对比项 | wiki-query | grep |
|--------|-----------|------|
| 搜索耗时 | ~100ms | ~250ms |
| 返回结果数 | 6（含元数据） | 19（仅路径） |
| 模型判断相关性 | 元数据直接判断 | 需逐个读取文件 |
| 实际需读取正文 | 1-2 篇 | 5-10 篇 |

**核心优势**：wiki-query 的 `search` 返回结构化元数据（name、description、tags），模型无需读取文件即可判断相关性，大幅减少 token 消耗。数据量越大，优势越明显。

## 自动同步

可配置 Git `post-commit` hook，让每次 commit 后自动运行 `sync` 更新数据库（配置方法见 `db/wiki-query/README.md`）。

## 构建

```bash
cd db/wiki-query
cargo build --release
```

编译产物位于 `target/release/wiki-query.exe`，已复制到 `db/wiki-query/wiki-query.exe`。

## 依赖

- Rust (GNU 工具链)
- MinGW (GCC) — 用于编译 SQLite
- rusqlite (bundled SQLite)
- serde / serde_json (JSON 序列化)
- chrono (时间处理)

## 集成到 CLAUDE.md

在项目根目录的 `CLAUDE.md` 中添加以下内容，让 Claude Code 自动使用 wiki-query：

```markdown
# 本地 Wiki 检索工具说明
仓库内置了一个高性能的 Rust 查询工具,产物路径为 `./db/wiki-query/wiki-query.exe`。

**v5 纯索引服务**:agent 用自己的文件工具读写 vault Markdown,wiki-query 只管 SQLite 索引。成品只保留两个 CLI 子命令:
- `wiki-query sync` — 重建/更新 SQLite 索引(运维,git post-commit hook 用)
- `wiki-query serve` — 启动 MCP server(agent 通过 stdio 交互)

**5 个 MCP 工具**(agent 自动调用):
- `wiki_search "<关键词>"` — FTS5 全文检索,返回 top 10 元数据(用于找知识 / 写入前查重)
- `wiki_list` — 按 tag/type/dir 结构化浏览
- `wiki_sync` — 同步索引 + 查重报告(scope=incremental/full);agent 写完文件后调
- `wiki_get_project_rules` — 返回可注入到项目根指令文件的规则文本(首次接入时用)
- `wiki_check_scope` — 判断文件是否在 wiki 范围内(sources/wiki/templates);AI 提炼记忆前调用

**读写 vault 正文** — agent 用自己的文件工具干,wiki-query 不碰正文。

**搜索文件内容/代码/具体文本** — 使用 grep/glob:
- 查找函数名、变量名、代码片段
- 按文件名模式匹配
- 搜索非 .md 文件
```
