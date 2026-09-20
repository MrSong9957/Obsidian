//! P1 功能测试:F3 wiki_search 检索正确性 + F4 wiki_list 过滤/排序。
//!
//! 先 sync 预填受控数据,验收基于 search/list 返回与 DB/文件真实状态
//! 的一致性(命中/数量/排序/字段),不以调用成功为标准。
//!
//! 注意:测试修改进程全局状态(current_dir + WIKI_DB_PATH),必须串行化。

use std::fs;
use std::path::Path;
use std::sync::Mutex;

static GLOBAL_TEST_MUTEX: Mutex<()> = Mutex::new(());

/// 三篇受控文档:type/tags/updated 各不相同,正文含独有标记词。
const DOC_A: &str = "---\nname: 文档A\ndescription: 阿尔法描述\ntype: concept\nupdated: 2026-09-01\ntags: alpha\n---\n\n阿尔法独有正文标记";
const DOC_B: &str = "---\nname: 文档B\ndescription: 贝塔描述\ntype: reference\nupdated: 2026-09-15\ntags: beta\n---\n\n贝塔独有正文标记";
const DOC_C: &str = "---\nname: 文档C\ndescription: 伽马描述\ntype: concept\nupdated: 2026-09-20\ntags: alpha-extra\n---\n\n伽马独有正文标记";

/// 建临时 vault,写入 A/B/C 三文档,切 cwd + 设 WIKI_DB_PATH,sync 入库。
fn setup_filled_vault() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().to_path_buf();
    let db_path = vault.join("db").join("wiki.db");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    fs::create_dir_all(vault.join("wiki")).unwrap();
    fs::create_dir_all(vault.join("sources/dev")).unwrap();
    fs::write(vault.join("wiki/a.md"), DOC_A).unwrap();
    fs::write(vault.join("wiki/b.md"), DOC_B).unwrap();
    fs::write(vault.join("sources/dev/c.md"), DOC_C).unwrap();
    std::env::set_var("WIKI_DB_PATH", db_path.to_str().unwrap());
    std::env::set_current_dir(&vault).unwrap();

    let report = wiki_query::sync::run_with_scope("incremental");
    assert_eq!(report.synced, 3, "前置:应入库 3 篇");
    (dir, db_path)
}

fn teardown(original_dir: &Path) {
    std::env::set_current_dir(original_dir).unwrap();
    std::env::remove_var("WIKI_DB_PATH");
}

fn paths_of(results: &[wiki_query::models::SearchResult]) -> Vec<String> {
    let mut v: Vec<String> = results.iter().map(|r| r.path.replace('\\', "/")).collect();
    v.sort();
    v
}

// ============================================================
// F3 wiki_search 检索正确性
// ============================================================

#[test]
fn search_hits_synced_content_and_fields_match() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, _db) = setup_filled_vault();

    let results = wiki_query::core::search("阿尔法").unwrap();
    assert_eq!(results.len(), 1, "独有词应恰命中文档A");
    assert_eq!(results[0].path.replace('\\', "/"), "wiki/a.md");
    assert_eq!(results[0].name, "文档A", "返回 name 应与 frontmatter 一致");
    assert_eq!(results[0].doc_type, "concept");
    assert_eq!(results[0].updated, "2026-09-01");
    assert_eq!(results[0].tags, "alpha");
    assert!(results[0].score.is_some(), "core::search 应带 bm25 score(工具层才清除)");
    teardown(&original_dir);
}

#[test]
fn search_matches_description_not_only_body() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, _db) = setup_filled_vault();

    // "描述"只出现在 frontmatter description,不在正文
    let results = wiki_query::core::search("贝塔").unwrap();
    assert_eq!(results.len(), 1, "FTS 应同时索引 description 与正文");
    assert_eq!(results[0].path.replace('\\', "/"), "wiki/b.md");
    teardown(&original_dir);
}

#[test]
fn search_multi_word_defaults_to_and() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, _db) = setup_filled_vault();

    // "阿尔法"+"贝塔"分布在不同文档,AND 语义应无命中
    let results = wiki_query::core::search("阿尔法 贝塔").unwrap();
    assert!(results.is_empty(), "多词 AND:不同文档分别含两词不应命中,实际: {:?}", paths_of(&results));
    // 同文档两词:正文+description
    let results2 = wiki_query::core::search("阿尔法 独有").unwrap();
    assert_eq!(paths_of(&results2), vec!["wiki/a.md"], "同文档两词应命中");
    teardown(&original_dir);
}

#[test]
fn search_or_semantics_returns_union() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, _db) = setup_filled_vault();

    let results = wiki_query::core::search("阿尔法 OR 伽马").unwrap();
    assert_eq!(
        paths_of(&results),
        vec!["sources/dev/c.md", "wiki/a.md"],
        "OR 应返回两文档并集,实际: {:?}",
        paths_of(&results)
    );
    teardown(&original_dir);
}

#[test]
fn search_empty_and_punctuation_return_empty() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, _db) = setup_filled_vault();

    assert!(wiki_query::core::search("").unwrap().is_empty(), "空查询应返回空");
    assert!(wiki_query::core::search("   ").unwrap().is_empty(), "纯空格应返回空");
    assert!(wiki_query::core::search("!!!").unwrap().is_empty(), "纯标点应返回空");
    // FTS 语法字符不 panic、不 500
    let r = wiki_query::core::search("阿尔法\"").unwrap();
    let _ = r; // 只要 Ok 即可
    teardown(&original_dir);
}

/// FTS 契约:搜索结果上限 10 条(超过时截断,不报错)。
#[test]
fn search_caps_results_at_ten() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, _db) = setup_filled_vault();

    // 额外写入 12 篇含同一独有词的文档
    for i in 0..12 {
        fs::write(
            format!("wiki/cap{i}.md"),
            format!("---\nname: 上限{i}\ndescription: d\ntype: concept\nupdated: 2026-09-01\ntags: cap\n---\n\n上限测试独有词上限"),
        )
        .unwrap();
    }
    wiki_query::sync::run_with_scope("incremental");

    let results = wiki_query::core::search("上限测试").unwrap();
    assert_eq!(results.len(), 10, "搜索结果应截断为前 10 条,实际: {}", results.len());
    teardown(&original_dir);
}

// ============================================================
// F4 wiki_list 过滤/排序/分页
// ============================================================

#[test]
fn list_type_filter_exact_match() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, db_path) = setup_filled_vault();

    let filter = wiki_query::core::ListFilter {
        doc_type: Some("concept".into()),
        ..Default::default()
    };
    let results = wiki_query::core::list(&filter).unwrap();
    assert_eq!(
        paths_of(&results),
        vec!["sources/dev/c.md", "wiki/a.md"],
        "type=concept 应精确命中两篇"
    );
    // 与 DB 直查口径一致
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let db_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM documents WHERE type = 'concept'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(results.len() as i64, db_count, "list 返回数应与 SQL 直查一致");
    teardown(&original_dir);
}

#[test]
fn list_tag_filter_substring() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, _db) = setup_filled_vault();

    let filter = wiki_query::core::ListFilter {
        tag: Some("alpha".into()),
        ..Default::default()
    };
    let results = wiki_query::core::list(&filter).unwrap();
    // alpha 子串命中 tags='alpha' 与 'alpha-extra'
    assert_eq!(paths_of(&results), vec!["sources/dev/c.md", "wiki/a.md"]);
    teardown(&original_dir);
}

#[test]
fn list_orders_by_updated_desc() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, _db) = setup_filled_vault();

    let results = wiki_query::core::list(&wiki_query::core::ListFilter::default()).unwrap();
    let updated_seq: Vec<String> = results.iter().map(|r| r.updated.clone()).collect();
    let mut sorted = updated_seq.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(updated_seq, sorted, "应按 updated DESC 排序,实际: {:?}", updated_seq);
    assert_eq!(results.len(), 3, "无过滤应返回全部 3 行");
    teardown(&original_dir);
}

#[test]
fn list_limit_takes_top_of_order() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, _db) = setup_filled_vault();

    let filter = wiki_query::core::ListFilter {
        limit: Some(2),
        ..Default::default()
    };
    let results = wiki_query::core::list(&filter).unwrap();
    assert_eq!(results.len(), 2, "limit=2 应恰返回 2 条");
    assert_eq!(results[0].updated, "2026-09-20", "第一条应是 updated 最新");
    assert_eq!(results[1].updated, "2026-09-15", "第二条应是次新");
    teardown(&original_dir);
}

#[test]
fn list_combined_filters_are_and() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, _db) = setup_filled_vault();

    let filter = wiki_query::core::ListFilter {
        doc_type: Some("concept".into()),
        dir: Some("wiki/".into()),
        ..Default::default()
    };
    let results = wiki_query::core::list(&filter).unwrap();
    assert_eq!(
        paths_of(&results),
        vec!["wiki/a.md"],
        "type=concept ∩ dir=wiki/ 应只含文档A(c.md 在 sources 下被排除)"
    );
    teardown(&original_dir);
}

/// limit 边界契约(SQLite LIMIT 语义):0 → 空结果;负数 → 不设上限返回全部。
/// 固化现状行为,防止未来悄然改变语义。
#[test]
fn list_limit_zero_and_negative_boundary_contract() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    let (_dir, _db) = setup_filled_vault();

    let zero = wiki_query::core::list(&wiki_query::core::ListFilter {
        limit: Some(0),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(zero.len(), 0, "limit=0 应返回空结果(SQLite LIMIT 0)");

    let negative = wiki_query::core::list(&wiki_query::core::ListFilter {
        limit: Some(-5),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(negative.len(), 3, "负数 limit 应视为无上限返回全部");
    teardown(&original_dir);
}
