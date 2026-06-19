use rusqlite::{Connection, Result};
use std::path::Path;
use crate::models::{Document, SearchResult};

const DEFAULT_DB_PATH: &str = "db/wiki.db";

/// 返回当前 DB 路径。优先 WIKI_DB_PATH 环境变量,否则默认 db/wiki.db。
/// 测试通过设置 WIKI_DB_PATH 指向 tempdir 隔离数据库。
pub fn db_path() -> String {
    std::env::var("WIKI_DB_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_string())
}

pub fn init() -> Result<Connection> {
    let path = db_path();
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
    let conn = init().expect("Failed to init database");

    // FTS5 检索:查询词先经 jieba 预分词,每个 token 加引号做 phrase,
    // OR 连接(任一 token 命中即召回,bm25 排序保证多词命中靠前)。
    use crate::fts::tokenize_for_fts;
    let tokenized = tokenize_for_fts(query);
    // FTS5 phrase 语法是 "..."，双引号在 phrase 内无法转义(FTS5 限制)。
    // 过滤掉含 " 的 token,避免构造出未闭合的 phrase 导致语法错误。
    // 同时过滤纯标点 token(jieba 偶尔切出独立标点)。
    let tokens: Vec<&str> = tokenized
        .split_whitespace()
        .filter(|t| !t.contains('"') && t.chars().any(|c| c.is_alphanumeric()))
        .collect();
    if tokens.is_empty() {
        println!("[]");
        return;
    }
    let fts_query: String = tokens
        .iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(" OR ");

    // FTS5 表无 updated 列,JOIN documents 取 updated。
    // bm25 分数越小越相关(FTS5 约定),ASC 排序即最相关在前。
    let stmt_result = conn.prepare(
        "SELECT d.path, d.name, d.description, d.type, d.updated, d.tags
         FROM documents_fts f
         JOIN documents d ON d.path = f.path
         WHERE documents_fts MATCH ?1
         ORDER BY bm25(documents_fts)
         LIMIT 20",
    );

    let mut stmt = match stmt_result {
        Ok(s) => s,
        Err(e) => {
            eprintln!("search: prepare 失败: {}", e);
            println!("[]");
            return;
        }
    };

    let results: Vec<SearchResult> = stmt
        .query_map(rusqlite::params![fts_query], |row| {
            Ok(SearchResult {
                path: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                doc_type: row.get(3)?,
                updated: row.get(4)?,
                tags: row.get(5)?,
            })
        })
        .map(|iter| iter.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    let json = serde_json::to_string_pretty(&results).unwrap();
    println!("{}", json);
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
