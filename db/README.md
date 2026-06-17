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
    ├── src/
    │   ├── main.rs     # 入口，命令路由
    │   ├── db.rs       # 数据库初始化、search/show 查询
    │   ├── models.rs   # Document、SearchResult 结构体
    │   └── sync.rs     # YAML 解析、增量同步逻辑
    ├── target/         # Rust 编译缓存（已 gitignore）
    └── wiki-query.exe  # 编译后的二进制
```

## 使用方式

```bash
# 同步 MD 文件到数据库（首次全量，后续增量）
./db/wiki-query/wiki-query.exe sync

# 搜索文档（返回 JSON 元数据列表）
./db/wiki-query/wiki-query.exe search "关键词"

# 查看完整文档（返回 JSON，含正文）
./db/wiki-query/wiki-query.exe show "path/to/file.md"
```

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

仓库配置了 Git `post-commit` hook，每次 commit 后自动运行 `sync` 更新数据库。

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
仓库内置了一个高性能的 Rust 查询工具，路径为 `./db/wiki-query/wiki-query.exe`。
当用户让你查找、分析、总结、关联知识库内的文档时，请按场景选择工具：

**查找文档（按主题/关键词）** — 使用 wiki-query：
1. 先使用 `wiki-query search "<关键词>"` 检索相关文档的元数据。
2. 锁定特定文档后，使用 `wiki-query show "<文档相对路径>"` 读取正文。

**搜索文件内容/代码/具体文本** — 使用 grep/glob：
- 查找函数名、变量名、代码片段
- 按文件名模式匹配
- 搜索非 .md 文件
```
