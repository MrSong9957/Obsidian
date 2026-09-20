//! core::list 的 dir 目录前缀过滤测试:Windows 反斜杠路径 × 两种斜杠写法。
//!
//! 背景:Windows 上 sync 入库的 path 含 `\`(如 `wiki\_index.md`),而
//! dir 参数按约定用 `/`(如 `wiki/`、`sources/dev/`),直接 LIKE 前缀匹配
//! 会静默返回空。修复目标:比较前统一两侧分隔符,兼容已有数据库,
//! 不依赖删库或重新索引。
//!
//! 注意:测试修改进程全局状态(WIKI_DB_PATH 环境变量),
//! 必须用 GLOBAL_TEST_MUTEX 串行化,否则并发测试会互相干扰
//! (与 sync_edge_cases_test.rs 同一防护模式)。

use std::sync::Mutex;

/// 保护 WIKI_DB_PATH 环境变量的互斥锁。
/// 所有修改 WIKI_DB_PATH 的测试必须持有此锁。
static GLOBAL_TEST_MUTEX: Mutex<()> = Mutex::new(());

/// 建临时 DB(经 db::init 建表)并插入指定 path 的文档行,返回临时目录守卫。
/// path 原样入库,由调用方控制分隔符形式(模拟 Windows/跨平台/混合数据)。
fn setup_db(rows: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("db").join("wiki.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    std::env::set_var("WIKI_DB_PATH", db_path.to_str().unwrap());

    let conn = wiki_query::db::init().unwrap();
    for p in rows {
        conn.execute(
            "INSERT INTO documents (path, name, description, type, updated, tags)
             VALUES (?1, ?1, '', 'concept', '2026-09-20', '')",
            rusqlite::params![p],
        )
        .unwrap();
    }
    dir
}

/// 用给定 dir 过滤执行 list,返回排序后的 path 列表。
fn list_paths(dir: Option<&str>) -> Vec<String> {
    let filter = wiki_query::core::ListFilter {
        dir: dir.map(String::from),
        ..Default::default()
    };
    let mut paths: Vec<String> = wiki_query::core::list(&filter)
        .unwrap()
        .into_iter()
        .map(|r| r.path)
        .collect();
    paths.sort();
    paths
}

// ============================================================
// 反斜杠 DB 路径(Windows 现有索引形态)
// ============================================================

mod backslash_db_tests {
    use super::*;

    #[test]
    fn dir_forward_slash_matches_backslash_db_paths() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _dir = setup_db(&[
            r"wiki\_index.md",
            r"wiki\a.md",
            r"sources\dev\rust.md",
            r"agents\security-testing-gate.md",
        ]);

        let paths = list_paths(Some("wiki/"));
        assert_eq!(
            paths,
            vec![r"wiki\_index.md", r"wiki\a.md"],
            "dir='wiki/' 应匹配反斜杠 DB 路径: {:?}",
            paths
        );
    }

    #[test]
    fn dir_backslash_and_forward_slash_return_same_results() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _dir = setup_db(&[
            r"wiki\_index.md",
            r"wiki\a.md",
            r"sources\dev\rust.md",
        ]);

        let via_forward = list_paths(Some("wiki/"));
        let via_backslash = list_paths(Some(r"wiki\"));
        assert_eq!(
            via_forward, via_backslash,
            "两种斜杠写法应返回相同结果"
        );
        assert_eq!(via_forward.len(), 2, "应命中 wiki 下两条: {:?}", via_forward);
    }

    #[test]
    fn dir_forward_slash_matches_backslash_nested_paths() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _dir = setup_db(&[
            r"sources\dev\rust.md",
            r"sources\dev\deep\advanced.md",
            r"sources\learning\memory.md",
        ]);

        let paths = list_paths(Some("sources/dev/"));
        assert_eq!(
            paths,
            vec![r"sources\dev\deep\advanced.md", r"sources\dev\rust.md"],
            "深层子目录文档也应命中(含其下更深目录): {:?}",
            paths
        );
    }
}

// ============================================================
// 正斜杠 DB 路径(跨平台/未来归一化形态)—— 回归保护
// ============================================================

mod forward_slash_db_tests {
    use super::*;

    #[test]
    fn dir_forward_slash_still_matches_forward_slash_db_paths() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _dir = setup_db(&[
            "wiki/_index.md",
            "sources/dev/rust.md",
            "agents/security-testing-gate.md",
        ]);

        let paths = list_paths(Some("wiki/"));
        assert_eq!(paths, vec!["wiki/_index.md"], "正斜杠 DB 路径行为不变: {:?}", paths);
    }
}

// ============================================================
// 混合分隔符 DB 路径
// ============================================================

mod mixed_db_tests {
    use super::*;

    #[test]
    fn dir_forward_slash_matches_mixed_separator_db_paths() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _dir = setup_db(&[
            "wiki/a.md",
            r"wiki\b.md",
            "sources/dev/rust.md",
        ]);

        let paths = list_paths(Some("wiki/"));
        assert_eq!(
            paths,
            vec!["wiki/a.md", r"wiki\b.md"],
            "混合分隔符的 wiki 路径应全部命中: {:?}",
            paths
        );
    }
}

// ============================================================
// 前缀精确性:不混入相邻目录
// ============================================================

mod prefix_exactness_tests {
    use super::*;

    #[test]
    fn dir_does_not_bleed_into_sibling_directory() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _dir = setup_db(&[
            "sources/dev/rust.md",
            "sources/dev/deep/advanced.md",
            "sources/development/other.md",
            "sources/other.md",
        ]);

        let paths = list_paths(Some("sources/dev/"));
        assert_eq!(
            paths,
            vec!["sources/dev/deep/advanced.md", "sources/dev/rust.md"],
            "sources/dev/ 不应混入 sources/development/ 或 sources/ 根下文档: {:?}",
            paths
        );
    }

    #[test]
    fn dir_backslash_writing_same_exactness() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _dir = setup_db(&[
            r"sources\dev\rust.md",
            r"sources\development\other.md",
        ]);

        let paths = list_paths(Some(r"sources\dev\"));
        assert_eq!(
            paths,
            vec![r"sources\dev\rust.md"],
            "反斜杠写法同样不混入相邻目录: {:?}",
            paths
        );
    }
}

// ============================================================
// 不带 dir 时行为不变
// ============================================================

mod no_dir_tests {
    use super::*;

    #[test]
    fn no_dir_returns_all_rows_regardless_of_separator() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _dir = setup_db(&[
            "wiki/a.md",
            r"wiki\b.md",
            "sources/dev/rust.md",
        ]);

        let paths = list_paths(None);
        assert_eq!(
            paths.len(),
            3,
            "不带 dir 应返回全部行,与分隔符无关: {:?}",
            paths
        );
    }
}
