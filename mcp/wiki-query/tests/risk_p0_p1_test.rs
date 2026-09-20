//! 暂缓风险 P0/P1 回归测试:
//! P0-1 整子树失联时的批量 prune 安全;
//! P0-2 旧/不兼容 SQLite schema 的安全处理;
//! P1  跨平台路径分隔符(\ vs /)在入库、exists、prune 的一致性。
//!
//! 验收基于实际 DB 行数/path 集合/FTS 状态,不以 SyncReport 为准。
//! 契约基础:Markdown 是唯一真本,DB 可重建——失联时保守保留,
//! 不兼容时整体拒绝并提示重建,均优于批量误清或半坏索引。
//!
//! 注意:修改 current_dir + WIKI_DB_PATH,必须 GLOBAL_TEST_MUTEX 串行化。

use std::fs;
use std::path::Path;
use std::sync::Mutex;

static GLOBAL_TEST_MUTEX: Mutex<()> = Mutex::new(());

const DOC: &str = "---\nname: 测试文档\ndescription: 描述\ntype: concept\nupdated: 2026-09-20\ntags: t\n---\n\n正文内容";

/// 建空 vault(仅 wiki/ 目录)+ 独立 DB,切 cwd。
fn setup_vault() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().to_path_buf();
    let db_path = vault.join("db").join("wiki.db");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    fs::create_dir_all(vault.join("wiki")).unwrap();
    std::env::set_var("WIKI_DB_PATH", db_path.to_str().unwrap());
    std::env::set_current_dir(&vault).unwrap();
    (dir, db_path)
}

fn teardown(original_dir: &Path) {
    std::env::set_current_dir(original_dir).unwrap();
    std::env::remove_var("WIKI_DB_PATH");
}

fn count_all_documents(db_path: &Path) -> i64 {
    let conn = rusqlite::Connection::open(db_path).unwrap();
    conn.query_row("SELECT COUNT(*) FROM documents", [], |r| r.get(0))
        .unwrap_or(-1)
}

fn fts_count(db_path: &Path, path_norm: &str) -> i64 {
    let conn = rusqlite::Connection::open(db_path).unwrap();
    conn.query_row(
        "SELECT COUNT(*) FROM documents_fts WHERE REPLACE(path, '\\', '/') = ?1",
        rusqlite::params![path_norm],
        |r| r.get(0),
    )
    .unwrap_or(-1)
}

/// 全部 documents path(原始形态)。
fn all_paths(db_path: &Path) -> Vec<String> {
    let conn = rusqlite::Connection::open(db_path).unwrap();
    let mut stmt = conn.prepare("SELECT path FROM documents ORDER BY path").unwrap();
    stmt.query_map([], |r| r.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
}

// ============================================================
// P0-1 整子树失联:不得批量清空;单文件删除仍 prune;恢复后可更新
// ============================================================

/// 目录级失联(整个子目录暂时不可访问/不存在)时,该子树的索引行
/// 必须保守保留;未受影响目录不受影响;单文件正常删除仍会 prune
/// (由 sync_delete_prunes_both_tables 回归保护,此处不重复)。
#[test]
fn sync_subtree_unavailable_keeps_rows_and_others_intact() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();

    // 三个区域:keep(不受影响)、victim(将失联,含嵌套)
    fs::write("wiki/keep.md", DOC).unwrap();
    fs::create_dir_all("wiki/victim/deep").unwrap();
    fs::write("wiki/victim/a.md", DOC).unwrap();
    fs::write("wiki/victim/deep/b.md", DOC).unwrap();
    wiki_query::sync::run_with_scope("incremental");
    assert_eq!(count_all_documents(&db_path), 3, "前置:3 行入库");

    // 模拟子树失联:目录整体不可见(移出到扫描根之外)
    let detached = tempfile::tempdir_in(std::env::temp_dir()).unwrap();
    fs::rename("wiki/victim", detached.path().join("victim")).unwrap();

    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(
        report.pruned, 0,
        "目录级失联不得被当作正常删除批量 prune"
    );
    assert_eq!(
        count_all_documents(&db_path),
        3,
        "失联子树的 DB 行必须保留,实际: {:?}",
        all_paths(&db_path)
    );
    // FTS 状态同步保留
    assert_eq!(fts_count(&db_path, "wiki/victim/a.md"), 1, "失联行 FTS 应保留");
    assert_eq!(fts_count(&db_path, "wiki/victim/deep/b.md"), 1);
    // 未受影响目录仍正确
    assert_eq!(fts_count(&db_path, "wiki/keep.md"), 1);
    teardown(&original_dir);
}

/// 失联子树恢复可见后,重新 sync 应恢复正常更新(不退化成永久失效)。
#[test]
fn sync_subtree_recovery_resumes_updates() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();

    fs::create_dir_all("wiki/victim").unwrap();
    fs::write("wiki/victim/a.md", DOC).unwrap();
    wiki_query::sync::run_with_scope("incremental");

    let detached = tempfile::tempdir_in(std::env::temp_dir()).unwrap();
    fs::rename("wiki/victim", detached.path().join("victim")).unwrap();
    wiki_query::sync::run_with_scope("incremental");
    assert_eq!(count_all_documents(&db_path), 1, "失联期保留 1 行");

    // 恢复可见 + 内容变更(跨秒保证 mtime 变化)
    fs::create_dir_all("wiki/victim").unwrap();
    fs::write("wiki/victim/a.md", DOC).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fs::write(
        "wiki/victim/a.md",
        "---\nname: 测试文档\ndescription: 描述\ntype: concept\nupdated: 2026-09-21\ntags: t\n---\n\n恢复后的新正文词汇",
    )
    .unwrap();
    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(report.updated, 1, "恢复后应正常更新");
    assert_eq!(count_all_documents(&db_path), 1, "恢复后仍恰一行(无重复)");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let content: String = conn
        .query_row(
            "SELECT content FROM documents_fts WHERE REPLACE(path, '\\', '/') = 'wiki/victim/a.md'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(content.contains("词汇"), "恢复后 FTS 应为新正文,实际: {}", content);
    teardown(&original_dir);
}

// ============================================================
// P0-2 旧/不兼容 schema:整体拒绝 + 可观察,不得半坏
// ============================================================

/// 缺列旧库(documents 无 modified_at,无 FTS 表)必须被整体拒绝:
/// failed=-1(库不可用信号)且不产生任何索引行,而非"部分成功"。
#[test]
fn sync_rejects_incompatible_schema_observably() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();

    // 手工建旧 schema:6 列(缺 modified_at),无 documents_fts
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute(
        "CREATE TABLE documents (
            id INTEGER PRIMARY KEY,
            path TEXT UNIQUE NOT NULL,
            name TEXT, description TEXT, type TEXT, updated TEXT, tags TEXT
        )",
        [],
    )
    .unwrap();
    drop(conn);

    fs::write("wiki/ok.md", DOC).unwrap();
    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(
        report.failed, -1,
        "不兼容 schema 应以 failed=-1 标记整体失败(而非逐文件 failed 计数的半坏状态)"
    );
    assert_eq!(report.synced, 0, "库不可用时不得报告扫描成功");
    assert_eq!(
        count_all_documents(&db_path),
        0,
        "不得向不兼容库写入任何索引行"
    );
    teardown(&original_dir);
}

/// 不兼容库上 search 必须返回错误(可观察),而非静默空结果冒充"无匹配"。
#[test]
fn search_on_incompatible_db_returns_error() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute(
        "CREATE TABLE documents (id INTEGER PRIMARY KEY, path TEXT UNIQUE NOT NULL, name TEXT, description TEXT, type TEXT, updated TEXT, tags TEXT)",
        [],
    )
    .unwrap();
    drop(conn);

    let result = wiki_query::core::search("任意词");
    assert!(
        result.is_err(),
        "不兼容库上 search 应返回错误而非静默空结果"
    );
    teardown(&original_dir);
}

// ============================================================
// P1 跨平台路径分隔符:统一 '/' 口径,不误删/不重复
// ============================================================

/// 旧库含反斜杠路径(windows 形态),磁盘同名文件在扫描时会产生正斜杠
/// 形态的 path:两者是同一逻辑文件,sync 后必须恰一行(统一 '/' 口径),
/// 不得既保留旧行又插入新行造成重复;嵌套路径同样成立;
/// 不含分隔符差异的普通行不受迁移影响。
#[test]
fn sync_mixed_separator_db_migrates_without_duplicates() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();

    // 预填旧式反斜杠行(windows 形态),含嵌套;外加一行纯小写无分隔符问题的普通行
    let conn = wiki_query::db::init().unwrap();
    for (p, name) in [
        (r"wiki\a.md", "文档A"),
        (r"sources\deep\c.md", "文档C"),
        ("wiki/plain.md", "普通行"),
    ] {
        conn.execute(
            "INSERT INTO documents (path, name, description, type, updated, tags, modified_at)
             VALUES (?1, ?2, 'd', 'concept', '2026-09-01', 't', '2026-09-01 00:00:00.000')",
            rusqlite::params![p, name],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO documents_fts (path, name, description, type, tags, content)
             VALUES (?1, ?2, 'd', 'concept', 't', 'body')",
            rusqlite::params![p, name],
        )
        .unwrap();
    }
    drop(conn);

    // 磁盘真实文件(逻辑上与反斜杠行是同一批文件)
    fs::create_dir_all("sources/deep").unwrap();
    fs::write("wiki/a.md", DOC).unwrap();
    fs::write("sources/deep/c.md", DOC).unwrap();
    fs::write("wiki/plain.md", DOC).unwrap();

    wiki_query::sync::run_with_scope("incremental");

    let paths = all_paths(&db_path);
    assert!(
        paths.iter().all(|p| !p.contains('\\')),
        "统一口径后全库不应再有反斜杠路径,实际: {:?}",
        paths
    );
    // 无重复:归一后 path 集合恰 3 个
    let mut normalized: Vec<String> = paths.iter().map(|p| p.replace('\\', "/")).collect();
    normalized.sort();
    let before_dedup = normalized.len();
    normalized.dedup();
    assert_eq!(before_dedup, normalized.len(), "同一逻辑文件不得有两条索引: {:?}", normalized);
    assert_eq!(
        normalized,
        vec!["sources/deep/c.md", "wiki/a.md", "wiki/plain.md"],
        "嵌套与普通路径均应保持正确语义"
    );
    // FTS 无重复
    assert_eq!(fts_count(&db_path, "wiki/a.md"), 1, "FTS 应恰一行");
    // search/list 不得返回重复 path
    let results = wiki_query::core::search("正文内容").unwrap();
    let mut sp: Vec<String> = results.iter().map(|r| r.path.replace('\\', "/")).collect();
    sp.sort();
    let n = sp.len();
    sp.dedup();
    assert_eq!(n, sp.len(), "search 返回不得含重复 path: {:?}", sp);
    teardown(&original_dir);
}

/// DB 已是正斜杠形态(Unix/WSL 建的库)时,Windows 侧扫描/exists/prune
/// 不得因分隔符表示差异误删已有行。
#[test]
fn sync_forward_slash_db_rows_not_pruned() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();

    let conn = wiki_query::db::init().unwrap();
    for p in ["wiki/a.md", "sources/dev/c.md"] {
        conn.execute(
            "INSERT INTO documents (path, name, description, type, updated, tags, modified_at)
             VALUES (?1, 'n', 'd', 'concept', '2026-09-01', 't', '2026-09-01 00:00:00.000')",
            rusqlite::params![p],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO documents_fts (path, name, description, type, tags, content)
             VALUES (?1, 'n', 'd', 'concept', 't', 'body')",
            rusqlite::params![p],
        )
        .unwrap();
    }
    drop(conn);

    // 磁盘真实文件与 DB 行是同一批(仅表示形式可能不同)
    fs::create_dir_all("sources/dev").unwrap();
    fs::write("wiki/a.md", DOC).unwrap();
    fs::write("sources/dev/c.md", DOC).unwrap();

    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(report.pruned, 0, "正斜杠 DB 行不得被误 prune");
    assert_eq!(count_all_documents(&db_path), 2, "两行均应保留");
    teardown(&original_dir);
}
