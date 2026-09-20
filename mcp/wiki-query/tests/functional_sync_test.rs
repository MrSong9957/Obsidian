//! P0 功能测试:F1 索引同步数据正确性 + F2 prune/错误 cwd 防护。
//!
//! 验收基于 DB 真实状态(documents + documents_fts 两表),
//! 不以"调用成功/无报错"为标准。
//!
//! 注意:测试修改进程全局状态(current_dir + WIKI_DB_PATH 环境变量),
//! 必须用 GLOBAL_TEST_MUTEX 串行化(与 sync_edge_cases_test.rs 同一防护模式)。

use std::fs;
use std::path::Path;
use std::sync::Mutex;

/// 保护 current_dir + WIKI_DB_PATH 的互斥锁。
static GLOBAL_TEST_MUTEX: Mutex<()> = Mutex::new(());

/// 合法 frontmatter 的样例文档。
const DOC_A: &str = "---\nname: 文档A\ndescription: 描述A\ntype: concept\nupdated: 2026-09-20\ntags: alpha\n---\n\n正文内容阿尔法";
/// 更新后的样例(新描述 + 新正文独有词)。
const DOC_A_V2: &str = "---\nname: 文档A\ndescription: 描述A第二版\ntype: concept\nupdated: 2026-09-21\ntags: alpha\n---\n\n正文内容贝塔新版";

/// 建临时 vault 目录 + 独立 DB 路径,切 cwd,返回临时目录。
/// 测试结束由调用方负责恢复 cwd 并 remove_var(见各测试尾部)。
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

/// 恢复全局状态(cwd + env)。
fn teardown(original_dir: &Path) {
    std::env::set_current_dir(original_dir).unwrap();
    std::env::remove_var("WIKI_DB_PATH");
}

/// 直接打开 DB 查询单值(不经过 db::init,避免副作用)。
fn query_scalar(db_path: &Path, sql: &str) -> String {
    let conn = rusqlite::Connection::open(db_path).unwrap();
    conn.query_row(sql, [], |row| row.get(0)).unwrap_or_default()
}

/// documents 总行数。
fn count_all_documents(db_path: &Path) -> i64 {
    let conn = rusqlite::Connection::open(db_path).unwrap();
    conn.query_row("SELECT COUNT(*) FROM documents", [], |r| r.get(0))
        .unwrap_or(0)
}

fn count_rows(db_path: &Path, table: &str, path: &str) -> i64 {
    let conn = rusqlite::Connection::open(db_path).unwrap();
    conn.query_row(
        &format!("SELECT COUNT(*) FROM {} WHERE path = ?1", table),
        rusqlite::params![path],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

// ============================================================
// F1 索引同步数据正确性
// ============================================================

#[test]
fn sync_new_file_writes_documents_and_fts_rows() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();
    fs::write("wiki/a.md", DOC_A).unwrap();

    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(report.synced, 1, "应扫描到 1 个文件");
    assert_eq!(report.updated, 1, "新文件应更新");
    assert_eq!(report.pruned, 0);
    // documents 行:5 个 frontmatter 字段逐一与文件一致
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let (name, description, doc_type, updated, tags): (String, String, String, String, String) = conn
        .query_row(
            "SELECT name, description, type, updated, tags FROM documents WHERE path = ?1",
            rusqlite::params!["wiki\\a.md"],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .unwrap_or_else(|_| {
            // 非 Windows 平台路径为正斜杠
            conn.query_row(
                "SELECT name, description, type, updated, tags FROM documents WHERE path = ?1",
                rusqlite::params!["wiki/a.md"],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap()
        });
    assert_eq!(name, "文档A", "name 应取自 frontmatter");
    assert_eq!(description, "描述A");
    assert_eq!(doc_type, "concept");
    assert_eq!(updated, "2026-09-20");
    assert_eq!(tags, "alpha");
    // FTS 行:同 path 恰 1 行,content 含正文
    assert!(count_rows(&db_path, "documents_fts", "wiki\\a.md") == 1
        || count_rows(&db_path, "documents_fts", "wiki/a.md") == 1,
        "FTS 表应有该文档一行");
    teardown(&original_dir);
}

#[test]
fn sync_update_refreshes_both_tables() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();
    fs::write("wiki/a.md", DOC_A).unwrap();
    wiki_query::sync::run_with_scope("incremental");

    // 跨秒修改,确保秒级 mtime 字符串变化
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fs::write("wiki/a.md", DOC_A_V2).unwrap();
    let report2 = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(report2.updated, 1, "内容变化后应更新");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let description: String = conn
        .query_row("SELECT description FROM documents WHERE path LIKE '%a.md'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(description, "描述A第二版", "documents 应为新值");
    let content: String = conn
        .query_row("SELECT content FROM documents_fts WHERE path LIKE '%a.md'", [], |r| r.get(0))
        .unwrap();
    assert!(content.contains("贝塔"), "FTS content 应为新正文,实际: {}", content);
    assert!(!content.contains("阿尔法"), "FTS 不应残留旧正文");
    teardown(&original_dir);
}

#[test]
fn sync_delete_prunes_both_tables() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();
    fs::write("wiki/a.md", DOC_A).unwrap();
    fs::write("wiki/b.md", DOC_A).unwrap();
    wiki_query::sync::run_with_scope("incremental");

    fs::remove_file("wiki/a.md").unwrap();
    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(report.pruned, 1, "删除的文件应被 prune");
    // 两表都不得残留(不是只删 documents)
    assert_eq!(count_rows(&db_path, "documents", "wiki\\a.md")
        + count_rows(&db_path, "documents", "wiki/a.md"), 0, "documents 应无残留");
    assert_eq!(count_rows(&db_path, "documents_fts", "wiki\\a.md")
        + count_rows(&db_path, "documents_fts", "wiki/a.md"), 0, "documents_fts 应无残留");
    // 幸存文件不受影响
    assert_eq!(count_rows(&db_path, "documents", "wiki\\b.md")
        + count_rows(&db_path, "documents", "wiki/b.md"), 1, "幸存文件应保留");
    teardown(&original_dir);
}

#[test]
fn sync_idempotent_db_state_identical() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();
    fs::write("wiki/a.md", DOC_A).unwrap();
    wiki_query::sync::run_with_scope("incremental");

    let dump = |p: &Path| {
        let conn = rusqlite::Connection::open(p).unwrap();
        let mut stmt = conn.prepare(
            "SELECT path, name, description, type, updated, tags, modified_at FROM documents ORDER BY path",
        ).unwrap();
        let docs: Vec<(String, String, String, String, String, String, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        let fts: Vec<(String, String)> = conn
            .prepare("SELECT path, content FROM documents_fts ORDER BY path")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        (docs, fts)
    };

    let before = dump(&db_path);
    let report2 = wiki_query::sync::run_with_scope("incremental");
    let after = dump(&db_path);

    assert_eq!(report2.updated, 0, "未变文件第二次 sync 不应更新");
    assert_eq!(report2.pruned, 0);
    assert_eq!(before, after, "幂等:两次 sync 之间两表状态应完全一致(含 modified_at)");
    teardown(&original_dir);
}

#[test]
fn sync_without_frontmatter_indexes_empty_meta_full_body() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();
    fs::write("wiki/plain.md", "纯正文没有frontmatter混合内容").unwrap();

    wiki_query::sync::run_with_scope("incremental");

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let (name, doc_type): (String, String) = conn
        .query_row("SELECT name, type FROM documents WHERE path LIKE '%plain.md'", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!(name, "", "无 frontmatter 时 name 应为空串(回退)");
    assert_eq!(doc_type, "", "无 frontmatter 时 type 应为空串(回退)");
    let content: String = conn
        .query_row("SELECT content FROM documents_fts WHERE path LIKE '%plain.md'", [], |r| r.get(0))
        .unwrap();
    assert!(content.contains("frontmatter"), "FTS content 应含全文分词(回退为 body),实际: {}", content);
    teardown(&original_dir);
}

#[test]
fn sync_unterminated_frontmatter_falls_back_to_body() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();
    fs::write("wiki/broken.md", "---\nname: 未闭合\n没有闭合分隔线").unwrap();

    wiki_query::sync::run_with_scope("incremental");

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let name: String = conn
        .query_row("SELECT name FROM documents WHERE path LIKE '%broken.md'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(name, "", "未闭合 frontmatter 应回退:name 不应取到值");
    teardown(&original_dir);
}

#[test]
fn sync_deep_nested_path_keeps_full_relative_path() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();
    fs::create_dir_all("wiki/dev/deep").unwrap();
    fs::write("wiki/dev/deep/nested.md", DOC_A).unwrap();

    wiki_query::sync::run_with_scope("incremental");

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let path: String = conn
        .query_row("SELECT path FROM documents WHERE name = '文档A'", [], |r| r.get(0))
        .unwrap();
    let normalized = path.replace('\\', "/");
    assert_eq!(normalized, "wiki/dev/deep/nested.md", "入库 path 应含全部层级");
    teardown(&original_dir);
}

/// 缺陷回归:同一秒内写 v1 → sync → 改 v2 → sync,
/// 秒级 mtime 字符串相同导致第二次被跳过,FTS 留旧正文。
/// 期望:内容确实变了就应更新(亚秒粒度)。
#[test]
fn sync_same_second_rewrite_still_updates() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();
    // 预热 jieba(首次加载词典数百 ms),否则首次 sync 会把第二次写入推出对齐秒
    wiki_query::fts::tokenize_for_fts("预热");
    // 对齐到秒边界附近开始,保证两次写入落在同一秒
    // (跨秒则 mtime 字符串不同,测试失去复现意义)
    loop {
        let ms = chrono::Local::now().timestamp_subsec_millis();
        if ms < 400 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    fs::write("wiki/t.md", "第一版内容甲").unwrap();
    wiki_query::sync::run_with_scope("incremental");

    // 短暂等待推进 mtime,但保持在同一秒内
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write("wiki/t.md", "第一版内容甲加上第二版内容乙").unwrap();
    // 若已接近秒边界,补一次对齐检查失败也无妨——两次写入时间戳即证据
    wiki_query::sync::run_with_scope("incremental");

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let content: String = conn
        .query_row("SELECT content FROM documents_fts WHERE path LIKE '%t.md'", [], |r| r.get(0))
        .unwrap();
    assert!(
        content.contains("乙"),
        "同秒内修改后 FTS 应为新正文,实际: {}",
        content
    );
    teardown(&original_dir);
}

/// 缺陷回归:sync_file 失败(非 UTF-8 文件)不得静默,
/// 报告必须能区分"全部成功"与"部分失败",且失败文件保留旧索引。
#[test]
fn sync_failed_file_reported_not_silent() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();
    fs::write("wiki/good.md", DOC_A).unwrap();
    wiki_query::sync::run_with_scope("incremental");
    let rows_before = count_all_documents(&db_path);

    // good.md 保持不变;新增一个非 UTF-8 文件(sync_file 读入必失败)
    fs::write("wiki/bad.md", [0xff, 0xfe, 0x00, 0x01]).unwrap();
    let report = wiki_query::sync::run_with_scope("incremental");

    let v = serde_json::to_value(&report).unwrap();
    assert!(
        v.get("failed").is_some(),
        "SyncReport 应含 failed 字段以区分全部成功与部分失败,实际: {}",
        v
    );
    let failed = v.get("failed").and_then(|f| f.as_i64()).unwrap_or(-1);
    assert_eq!(failed, 1, "应报告 1 个失败文件");
    assert_eq!(
        count_all_documents(&db_path),
        rows_before,
        "失败文件不应改变 documents 行数(坏文件不入库,好文件不重写)"
    );
    teardown(&original_dir);
}

// ============================================================
// F2 prune / 错误 cwd 防护
// ============================================================

/// 缺陷回归:错误 cwd(无 wiki//sources/ 目录)+ 指向已有数据的 DB 时,
/// 不得因 on_disk 为空把已有索引整库 prune 掉。
#[test]
fn sync_wrong_cwd_does_not_prune_existing_db() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();

    // 先在"合法 vault"里建好索引(2 行)
    let (_dir, db_path) = setup_vault();
    fs::write("wiki/a.md", DOC_A).unwrap();
    fs::write("wiki/b.md", DOC_A).unwrap();
    wiki_query::sync::run_with_scope("incremental");
    let rows_before = count_all_documents(&db_path);
    assert_eq!(rows_before, 2, "前置:索引应有 2 行");

    // 切到无 wiki//sources/ 的目录(cwd 变了,DB 还是同一个)
    let other = tempfile::tempdir().unwrap();
    std::env::set_current_dir(other.path()).unwrap();

    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(
        report.pruned, 0,
        "错误 cwd 下不得 prune 已有索引(整库清空即此缺陷)"
    );
    assert_eq!(
        count_all_documents(&db_path),
        rows_before,
        "DB 行数必须原样保留"
    );
    teardown(&original_dir);
}

/// 正常 prune 的精确性:删 A 留 B,B 的字段与 modified_at 原样保留。
#[test]
fn sync_prune_keeps_surviving_rows_intact() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();
    fs::write("wiki/a.md", DOC_A).unwrap();
    fs::write("wiki/b.md", DOC_A).unwrap();
    wiki_query::sync::run_with_scope("incremental");
    let b_before = query_scalar(
        &db_path,
        "SELECT COALESCE(modified_at,'') FROM documents WHERE path LIKE '%b.md'",
    );

    fs::remove_file("wiki/a.md").unwrap();
    wiki_query::sync::run_with_scope("incremental");

    let b_after = query_scalar(
        &db_path,
        "SELECT COALESCE(modified_at,'') FROM documents WHERE path LIKE '%b.md'",
    );
    assert_eq!(b_before, b_after, "幸存行的 modified_at 不应被 prune 触碰");
    assert_eq!(b_after.len() > 0, true, "幸存行应存在");
    teardown(&original_dir);
}

/// 文件移动(旧路径消失+新路径出现)应表现为 prune 旧记录 + 索引新记录,
/// 而非新旧两条并存。
#[test]
fn sync_moved_file_prunes_old_and_indexes_new_path() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_vault();
    fs::write("wiki/a.md", DOC_A).unwrap();
    wiki_query::sync::run_with_scope("incremental");

    fs::create_dir_all("wiki/moved").unwrap();
    fs::rename("wiki/a.md", "wiki/moved/a.md").unwrap();
    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(report.pruned, 1, "旧路径记录应被 prune");
    assert_eq!(report.updated, 1, "新路径应作为文档入库");
    assert_eq!(
        count_rows(&db_path, "documents", "wiki\\a.md") + count_rows(&db_path, "documents", "wiki/a.md"),
        0,
        "旧路径不应残留"
    );
    let new_path: String = query_scalar(
        &db_path,
        "SELECT path FROM documents WHERE name = '文档A'",
    );
    assert_eq!(
        new_path.replace('\\', "/"),
        "wiki/moved/a.md",
        "新路径应含完整层级"
    );
    teardown(&original_dir);
}
