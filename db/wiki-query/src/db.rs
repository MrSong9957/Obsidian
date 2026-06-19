use rusqlite::{Connection, Result};
use std::path::Path;
use crate::models::Document;

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

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY,
            path TEXT UNIQUE NOT NULL,
            name TEXT,
            description TEXT,
            type TEXT,
            updated TEXT,
            tags TEXT,
            content TEXT,
            word_count INTEGER,
            modified_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_name ON documents(name);
        CREATE INDEX IF NOT EXISTS idx_type ON documents(type);
        -- FTS5 自有内容表(不用 content='documents',避免外部内容表的更新陷阱)。
        -- 字段值在 sync 时由 fts::tokenize_for_fts 做 jieba 预分词,
        -- 使 unicode61 tokenizer 能按词命中中文。
        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            path, name, description, type, tags, content,
            tokenize='unicode61'
        );"
    )?;

    Ok(conn)
}

pub fn search(query: &str) {
    // 复用 core::search_with_scores,避免 SQL/tokenize 逻辑重复(review Important-3)。
    match crate::core::search_with_scores(query) {
        Ok(results) => {
            let json = serde_json::to_string_pretty(&results).unwrap();
            println!("{}", json);
        }
        Err(e) => {
            eprintln!("search 失败: {}", e);
            println!("[]");
        }
    }
}

pub fn show(path: &str) {
    let conn = init().expect("Failed to init database");

    let result = conn.query_row(
        "SELECT id, path, name, description, type, updated, tags, content, word_count, modified_at
         FROM documents
         WHERE path = ?1",
        rusqlite::params![path],
        |row| {
            Ok(Document {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                doc_type: row.get(4)?,
                updated: row.get(5)?,
                tags: row.get(6)?,
                content: row.get(7)?,
                word_count: row.get(8)?,
                modified_at: row.get(9)?,
            })
        },
    );

    match result {
        Ok(doc) => {
            let json = serde_json::to_string_pretty(&doc).unwrap();
            println!("{}", json);
        }
        Err(_) => {
            eprintln!("Document not found: {}", path);
            std::process::exit(1);
        }
    }
}
