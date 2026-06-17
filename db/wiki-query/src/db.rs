use rusqlite::{Connection, Result};
use std::path::Path;
use crate::models::{Document, SearchResult};

const DB_PATH: &str = "db/wiki.db";

pub fn init() -> Result<Connection> {
    let db_dir = Path::new(DB_PATH).parent().unwrap();
    std::fs::create_dir_all(db_dir).ok();

    let conn = Connection::open(DB_PATH)?;

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
        CREATE INDEX IF NOT EXISTS idx_type ON documents(type);"
    )?;

    Ok(conn)
}

pub fn search(query: &str) {
    let conn = init().expect("Failed to init database");

    let pattern = format!("%{}%", query);
    let mut stmt = conn
        .prepare(
            "SELECT path, name, description, type, updated, tags
             FROM documents
             WHERE name LIKE ?1
                OR description LIKE ?1
                OR tags LIKE ?1
                OR path LIKE ?1
             ORDER BY updated DESC
             LIMIT 20",
        )
        .expect("Failed to prepare statement");

    let results: Vec<SearchResult> = stmt
        .query_map(rusqlite::params![pattern], |row| {
            Ok(SearchResult {
                path: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                doc_type: row.get(3)?,
                updated: row.get(4)?,
                tags: row.get(5)?,
            })
        })
        .expect("Query failed")
        .filter_map(|r| r.ok())
        .collect();

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
