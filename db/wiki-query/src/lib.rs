//! wiki-query 核心库。供 CLI(main.rs)和 MCP server(mcp_server.rs)共用,
//! 也是集成测试(tests/)访问内部模块的入口。
//!
//! 模块职责:
//! - `db` — SQLite 连接 + FTS5 检索
//! - `sync` — Markdown → SQLite 索引同步
//! - `models` — 序列化数据结构(Document/SearchResult 等)
//! - `frontmatter` — frontmatter 解析/校验/序列化
//! - `fts` — jieba CJK 分词桥接
//! - `error` — 统一错误类型

pub mod db;
pub mod error;
pub mod fts;
pub mod frontmatter;
pub mod models;
pub mod policy;
pub mod sync;
