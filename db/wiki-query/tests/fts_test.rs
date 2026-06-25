//! Task 1.2 集成测试:FTS5 表 + 索引同步。
//!
//! 用真 lib API(wiki_query::frontmatter::parse, wiki_query::fts::tokenize_for_fts)
//! 替换之前的复制品 —— 解决 Task 1.2 review 提出的"测试测复制品而非被测代码"问题。
//!
//! 注:DB 建表仍是测试本地 SQL,因为 db::init 硬编码路径(待 Task 3.3 WIKI_DB_PATH 解决)。
//! 但分词/解析逻辑现在直接用 lib 真代码,不再漂移。

use rusqlite::Connection;
use std::fs;
use wiki_query::frontmatter;
use wiki_query::fts;

/// 在临时目录建迷你 vault,模拟 sync 把 docs 写入临时 DB。
/// 用真 lib 的 frontmatter::parse + fts::tokenize_for_fts(不再重复实现)。
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

    let db_path = vault.join("test.db");
    let db_path_str = db_path.to_str().unwrap().to_string();

    {
        let conn = Connection::open(&db_path_str).unwrap();
        conn.execute_batch(
            // schema 手动维护,须与 src/db.rs 保持一致(v5:删 content/word_count)。
            // 这里只建 sync_file 会 INSERT 的列,FTS5 content 列同步时单独写。
            "CREATE TABLE documents (
                id INTEGER PRIMARY KEY, path TEXT UNIQUE NOT NULL,
                name TEXT, description TEXT, type TEXT, updated TEXT,
                tags TEXT, modified_at TEXT
            );
            CREATE VIRTUAL TABLE documents_fts USING fts5(
                path, name, description, type, tags, content, tokenize='unicode61'
            );",
        )
        .unwrap();

        for (rel, content) in docs {
            // 真代码:用 lib 的 frontmatter::parse
            let (fm, body) = frontmatter::parse(content);
            let path_str = rel.replace('\\', "/");
            conn.execute(
                "INSERT INTO documents (path,name,description,type,updated,tags,modified_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7)
                 ON CONFLICT(path) DO UPDATE SET name=excluded.name,description=excluded.description,
                   type=excluded.type,updated=excluded.updated,tags=excluded.tags,modified_at=excluded.modified_at",
                rusqlite::params![
                    path_str, fm.name, fm.description, fm.doc_type, fm.updated, fm.tags,
                    "2026-06-20 00:00:00"
                ],
            ).unwrap();
            // FTS5 同步:真代码用 lib 的 fts::tokenize_for_fts 预分词
            conn.execute("DELETE FROM documents_fts WHERE path=?1", rusqlite::params![path_str]).unwrap();
            conn.execute(
                "INSERT INTO documents_fts (path,name,description,type,tags,content)
                 VALUES (?1,?2,?3,?4,?5,?6)",
                rusqlite::params![
                    path_str,
                    fts::tokenize_for_fts(&fm.name),
                    fts::tokenize_for_fts(&fm.description),
                    fm.doc_type,
                    fts::tokenize_for_fts(&fm.tags),
                    fts::tokenize_for_fts(&body),
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
    // "记忆系统" 入库后,搜 "记忆"(子串)应命中 —— cut_for_search 细分策略
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
    let docs = [
        ("wiki/a.md", "---\nname: a\ndescription: d\ntype: concept\nupdated: 2026-06-20\ntags: t\n---\n\nbody"),
    ];
    let (_dir, db_path) = setup_vault_with_docs(&docs);
    let conn = Connection::open(&db_path).unwrap();
    let d: i64 = conn.query_row("SELECT count(*) FROM documents", [], |r| r.get(0)).unwrap();
    let f: i64 = conn.query_row("SELECT count(*) FROM documents_fts", [], |r| r.get(0)).unwrap();
    assert_eq!(d, f, "documents 与 documents_fts 行数应一致");
}

/// frontmatter::parse 真代码的契约回归(确认测试用的是 lib 真代码)。
#[test]
fn uses_real_frontmatter_parse() {
    let (fm, body) = frontmatter::parse(
        "---\nname: 测试\ndescription: d\ntype: concept\nupdated: 2026-06-20\ntags: t\n---\n\n正文"
    );
    assert_eq!(fm.name, "测试");
    assert!(body.contains("正文"));
}
