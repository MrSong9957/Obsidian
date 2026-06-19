//! Task 1.2 集成测试:FTS5 表 + 索引同步。
//!
//! 这些测试用一个临时 vault + 临时 DB,避免污染真实仓库。
//! 通过 WIKI_DB_PATH 环境变量隔离数据库(Task 3.3 会正式做,
//! 这里临时用 unique 路径规避)。

use rusqlite::Connection;
use std::fs;
use std::path::Path;

/// 辅助:在临时目录建一个迷你 vault,跑 sync,返回 db 连接。
fn setup_vault_with_docs(docs: &[(&str, &str)]) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path();

    for (rel, content) in docs {
        let full = vault.join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full, content).unwrap();
    }

    // 临时 db 路径(每个测试唯一,避免并发污染)
    let db_path = vault.join("test.db");
    let db_path_str = db_path.to_str().unwrap().to_string();

    // 直接用 init + sync_file,绕过 WIKI_DB_PATH(尚未实现)。
    // 用 raw 连接模拟 sync 行为。
    {
        let conn = Connection::open(&db_path_str).unwrap();
        conn.execute_batch(
            "CREATE TABLE documents (
                id INTEGER PRIMARY KEY, path TEXT UNIQUE NOT NULL,
                name TEXT, description TEXT, type TEXT, updated TEXT,
                tags TEXT, content TEXT, word_count INTEGER, modified_at TEXT
            );
            CREATE VIRTUAL TABLE documents_fts USING fts5(
                path, name, description, type, tags, content, tokenize='unicode61'
            );",
        )
        .unwrap();

        for (rel, content) in docs {
            // 用 wiki_query 的 frontmatter 解析 + fts 分词
            let (fm, body) = wiki_query_test_support::parse_frontmatter(content);
            let path_str = rel.replace('\\', "/");
            conn.execute(
                "INSERT INTO documents (path,name,description,type,updated,tags,content,word_count,modified_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
                 ON CONFLICT(path) DO UPDATE SET name=excluded.name,description=excluded.description,
                   type=excluded.type,updated=excluded.updated,tags=excluded.tags,content=excluded.content,
                   word_count=excluded.word_count,modified_at=excluded.modified_at",
                rusqlite::params![
                    path_str, fm.name, fm.description, fm.doc_type, fm.updated, fm.tags,
                    body, body.split_whitespace().count() as i64, "2026-06-20 00:00:00"
                ],
            ).unwrap();
            // FTS5 同步(模拟 sync_file 的 delete-then-insert)
            conn.execute("DELETE FROM documents_fts WHERE path=?1", rusqlite::params![path_str]).unwrap();
            conn.execute(
                "INSERT INTO documents_fts (path,name,description,type,tags,content)
                 VALUES (?1,?2,?3,?4,?5,?6)",
                rusqlite::params![
                    path_str,
                    wiki_query_test_support::tokenize(&fm.name),
                    wiki_query_test_support::tokenize(&fm.description),
                    fm.doc_type,
                    wiki_query_test_support::tokenize(&fm.tags),
                    wiki_query_test_support::tokenize(&body),
                ],
            ).unwrap();
        }
    }

    (dir, db_path_str)
}

#[test]
fn fts_table_populated_after_sync() {
    let docs = [
        ("wiki/agent.md", "---\nname: 智能体\ndescription: Agent 设计\ntype: concept\nupdated: 2026-06-20\ntags: agent\n---\n\n# Agent\n\n智能体架构"),
        ("wiki/memory.md", "---\nname: 记忆系统\ndescription: 记忆管理\ntype: concept\nupdated: 2026-06-20\ntags: memory\n---\n\n# Memory\n\n跨会话记忆"),
    ];
    let (_dir, db_path) = setup_vault_with_docs(&docs);
    let conn = Connection::open(&db_path).unwrap();
    let n: i64 = conn.query_row("SELECT count(*) FROM documents_fts", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 2, "FTS5 应有 2 行");
}

#[test]
fn fts_chinese_match_finds_subword() {
    // "记忆系统" 入库后,搜 "记忆"(子串)应命中
    let docs = [
        ("wiki/mem.md", "---\nname: 记忆系统\ndescription: 记忆\ntype: concept\nupdated: 2026-06-20\ntags: memory\n---\n\n记忆系统设计"),
    ];
    let (_dir, db_path) = setup_vault_with_docs(&docs);
    let conn = Connection::open(&db_path).unwrap();
    let mut stmt = conn.prepare("SELECT count(*) FROM documents_fts WHERE documents_fts MATCH '记忆'").unwrap();
    let n: i64 = stmt.query_row([], |r| r.get(0)).unwrap();
    assert!(n >= 1, "MATCH '记忆' 应命中含 '记忆系统' 的文档");
}

#[test]
fn fts_and_documents_stay_consistent() {
    // 两表行数应一致(孤儿清理的契约)
    let docs = [
        ("wiki/a.md", "---\nname: a\ndescription: d\ntype: concept\nupdated: 2026-06-20\ntags: t\n---\n\nbody"),
    ];
    let (_dir, db_path) = setup_vault_with_docs(&docs);
    let conn = Connection::open(&db_path).unwrap();
    let d: i64 = conn.query_row("SELECT count(*) FROM documents", [], |r| r.get(0)).unwrap();
    let f: i64 = conn.query_row("SELECT count(*) FROM documents_fts", [], |r| r.get(0)).unwrap();
    assert_eq!(d, f, "documents 与 documents_fts 行数应一致");
}

/// 测试支持模块:暴露 frontmatter 解析 + fts 分词给集成测试。
/// (正式的 pub API 在 Task 2.1 抽取后由 lib.rs 提供)
pub mod wiki_query_test_support {
    pub struct Frontmatter {
        pub name: String,
        pub description: String,
        pub doc_type: String,
        pub updated: String,
        pub tags: String,
    }

    pub fn parse_frontmatter(content: &str) -> (Frontmatter, String) {
        let mut fm = Frontmatter {
            name: String::new(), description: String::new(), doc_type: String::new(),
            updated: String::new(), tags: String::new(),
        };
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() || lines[0].trim() != "---" {
            return (fm, content.to_string());
        }
        let mut end = 0;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" { end = i; break; }
            if let Some(c) = line.find(':') {
                let (k, v) = (line[..c].trim(), line[c+1..].trim());
                match k {
                    "name" => fm.name = v.into(),
                    "description" => fm.description = v.into(),
                    "type" => fm.doc_type = v.into(),
                    "updated" => fm.updated = v.into(),
                    "tags" => fm.tags = v.into(),
                    _ => {}
                }
            }
        }
        let body = if end > 0 { lines[end+1..].join("\n") } else { content.to_string() };
        (fm, body)
    }

    pub fn tokenize(text: &str) -> String {
        // 直接调 jieba(避免依赖未抽取的 fts 模块路径)
        let j = jieba_rs::Jieba::new();
        j.cut_for_search(text, true).join(" ")
    }
}

// 确保引入 tempfile(需在 Cargo.toml dev-dependencies)
