use rusqlite::{Connection, Result};
use std::path::Path;

const DEFAULT_DB_PATH: &str = "db/wiki.db";

/// 返回当前 DB 路径。优先 WIKI_DB_PATH 环境变量,否则默认 db/wiki.db。
///
/// 注意:env 在集成测试(tests/ 独立 crate)和生产都生效。init() 会在
/// 使用非默认路径时打 stderr 日志,避免生产误设 WIKI_DB_PATH 静默指向错误库
/// (review Important-4)。
pub fn db_path() -> String {
    std::env::var("WIKI_DB_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_string())
}

pub fn init() -> Result<Connection> {
    let path = db_path();
    // 非默认路径时打日志,防止生产误设 WIKI_DB_PATH 静默指向错误库(review Important-4)
    if path != DEFAULT_DB_PATH {
        eprintln!("[wiki-query] 使用非默认 DB 路径: {}", path);
    }
    let db_dir = Path::new(&path).parent().unwrap();
    std::fs::create_dir_all(db_dir).ok();

    let conn = Connection::open(&path)?;

    // 并发安全:WAL 模式 + busy_timeout(review C1)。
    // WAL 允许读写并发(MCP serve 常驻 + git hook sync + 多 agent 同时访问),
    // busy_timeout 让短时锁竞争自动重试而非立即 SQLITE_BUSY 失败。
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY,
            path TEXT UNIQUE NOT NULL,
            name TEXT,
            description TEXT,
            type TEXT,
            updated TEXT,
            tags TEXT,
            modified_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_type ON documents(type);
        -- FTS5 自有内容表(不用 content='documents',避免外部内容表的更新陷阱)。
        -- 字段值在 sync 时由 fts::tokenize_for_fts 做 jieba 预分词,
        -- 使 unicode61 tokenizer 能按词命中中文。
        -- 注意:FTS5 的 content 是倒排索引的原始数据(必须存才能检索),不是「存档正文」。
        -- documents 表不存正文(符合 v5「SQLite 只存元数据」原则)。
        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            path, name, description, type, tags, content,
            tokenize='unicode61'
        );"
    )?;

    Ok(conn)
}
