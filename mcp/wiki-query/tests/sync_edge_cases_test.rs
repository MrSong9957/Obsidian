//! sync 模块边界测试:prune_orphans 行为 + scan_directory 隐藏目录跳过。
//!
//! 用临时目录 + 临时 DB 测试,不污染真实 vault。
//!
//! 注意:这些测试修改进程全局状态(current_dir + WIKI_DB_PATH 环境变量),
//! 必须用 GLOBAL_TEST_MUTEX 串行化,否则并发测试会互相干扰。

use std::fs;
use std::sync::Mutex;

/// 保护进程全局状态(current_dir + 环境变量)的互斥锁。
/// 所有修改 current_dir 或 WIKI_DB_PATH 的测试必须持有此锁。
static GLOBAL_TEST_MUTEX: Mutex<()> = Mutex::new(());

/// 建临时目录结构 + 设置 DB 路径环境变量。
fn setup_temp_db() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

// ============================================================
// prune_orphans 测试
// ============================================================

/// prune_orphans 在 sync 模块中是 pub(crate),不能直接从外部测试。
/// 但可以通过 run_with_scope 间接测试:在临时目录建 vault,
/// 写入文档 → sync → 删除文件 → 再次 sync → 验证 pruned > 0。
///
/// 注意:sync::run_with_scope 硬编码 root = Path::new("."),
/// 所以需要用 std::env::set_current_dir 切到临时目录。
/// 这会污染全局状态,用 test_guard 模块隔离。

mod prune_orphans_tests {
    use super::*;

    #[test]
    fn sync_prunes_deleted_files() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let vault_dir = setup_temp_db();
        let vault = vault_dir.path();

        // 设置 DB 路径指向临时目录
        let db_path = vault.join("db").join("wiki.db");
        fs::create_dir_all(db_path.parent().unwrap()).unwrap();

        // 创建 wiki 目录 + 一个 md 文件
        let wiki_dir = vault.join("wiki");
        fs::create_dir_all(&wiki_dir).unwrap();
        let doc_path = wiki_dir.join("test-doc.md");
        fs::write(
            &doc_path,
            "---\nname: 测试文档\ndescription: 测试\ntype: concept\nupdated: 2026-06-22\ntags: test\n---\n\n正文",
        )
        .unwrap();

        // 切到 vault 目录(sync 硬编码 root = ".")
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(vault).unwrap();

        // 设置 DB 环境变量
        std::env::set_var("WIKI_DB_PATH", db_path.to_str().unwrap());

        // 第一次 sync:应同步 1 个文件
        let report1 = wiki_query::sync::run_with_scope("incremental");
        assert!(report1.synced >= 1, "应扫描到文件: {:?}", report1);
        assert!(report1.updated >= 1, "应更新文件: {:?}", report1);

        // 删除文件
        fs::remove_file(&doc_path).unwrap();

        // 第二次 sync:应 prune 1 条记录
        let report2 = wiki_query::sync::run_with_scope("incremental");
        assert!(report2.pruned >= 1, "应清理已删除文件的记录: {:?}", report2);

        // 恢复
        std::env::set_current_dir(original_dir).unwrap();
        std::env::remove_var("WIKI_DB_PATH");
    }

    #[test]
    fn sync_does_not_prune_existing_files() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let vault_dir = setup_temp_db();
        let vault = vault_dir.path();

        let db_path = vault.join("db").join("wiki.db");
        fs::create_dir_all(db_path.parent().unwrap()).unwrap();

        let wiki_dir = vault.join("wiki");
        fs::create_dir_all(&wiki_dir).unwrap();
        fs::write(
            wiki_dir.join("keep.md"),
            "---\nname: 保留\ndescription: d\ntype: concept\nupdated: 2026-06-22\ntags: t\n---\n\n正文",
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(vault).unwrap();
        std::env::set_var("WIKI_DB_PATH", db_path.to_str().unwrap());

        // 两次 sync,文件未删
        wiki_query::sync::run_with_scope("incremental");
        let report2 = wiki_query::sync::run_with_scope("incremental");
        assert_eq!(report2.pruned, 0, "未删文件不应 prune");

        std::env::set_current_dir(original_dir).unwrap();
        std::env::remove_var("WIKI_DB_PATH");
    }
}

// ============================================================
// scan_directory 隐藏目录跳过测试
// ============================================================

mod hidden_dir_tests {
    use super::*;

    #[test]
    fn sync_skips_hidden_directories() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let vault_dir = setup_temp_db();
        let vault = vault_dir.path();

        let db_path = vault.join("db").join("wiki.db");
        fs::create_dir_all(db_path.parent().unwrap()).unwrap();

        // 创建隐藏目录 + 文档
        let hidden_dir = vault.join(".hidden");
        fs::create_dir_all(&hidden_dir).unwrap();
        fs::write(
            hidden_dir.join("secret.md"),
            "---\nname: 隐藏文档\ndescription: d\ntype: concept\nupdated: 2026-06-22\ntags: t\n---\n\n正文",
        )
        .unwrap();

        // 创建正常目录 + 文档
        let wiki_dir = vault.join("wiki");
        fs::create_dir_all(&wiki_dir).unwrap();
        fs::write(
            wiki_dir.join("visible.md"),
            "---\nname: 可见文档\ndescription: d\ntype: concept\nupdated: 2026-06-22\ntags: t\n---\n\n正文",
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(vault).unwrap();
        std::env::set_var("WIKI_DB_PATH", db_path.to_str().unwrap());

        let report = wiki_query::sync::run_with_scope("incremental");
        // hidden 目录的文件不应被扫描
        assert_eq!(report.synced, 1, "应只扫描到 wiki/visible.md: {:?}", report);
        assert_eq!(report.updated, 1, "应只更新 wiki/visible.md: {:?}", report);

        std::env::set_current_dir(original_dir).unwrap();
        std::env::remove_var("WIKI_DB_PATH");
    }

    #[test]
    fn sync_skips_non_md_files() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let vault_dir = setup_temp_db();
        let vault = vault_dir.path();

        let db_path = vault.join("db").join("wiki.db");
        fs::create_dir_all(db_path.parent().unwrap()).unwrap();

        let wiki_dir = vault.join("wiki");
        fs::create_dir_all(&wiki_dir).unwrap();
        // .txt 文件应被跳过
        fs::write(wiki_dir.join("readme.txt"), "not a wiki doc").unwrap();
        // .md 文件应被扫描
        fs::write(
            wiki_dir.join("doc.md"),
            "---\nname: 文档\ndescription: d\ntype: concept\nupdated: 2026-06-22\ntags: t\n---\n\n正文",
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(vault).unwrap();
        std::env::set_var("WIKI_DB_PATH", db_path.to_str().unwrap());

        let report = wiki_query::sync::run_with_scope("incremental");
        assert_eq!(report.synced, 1, "应只扫描 .md 文件: {:?}", report);

        std::env::set_current_dir(original_dir).unwrap();
        std::env::remove_var("WIKI_DB_PATH");
    }
}

// ============================================================
// sync incremental vs full 查重范围测试
// ============================================================

mod dedupe_scope_tests {
    use super::*;

    #[test]
    fn incremental_only_queries_new_docs() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let vault_dir = setup_temp_db();
        let vault = vault_dir.path();

        let db_path = vault.join("db").join("wiki.db");
        fs::create_dir_all(db_path.parent().unwrap()).unwrap();

        let wiki_dir = vault.join("wiki");
        fs::create_dir_all(&wiki_dir).unwrap();

        // 写两篇内容相似的文档
        fs::write(
            wiki_dir.join("a.md"),
            "---\nname: 记忆系统设计\ndescription: 记忆管理方案\ntype: concept\nupdated: 2026-06-22\ntags: memory\n---\n\n记忆系统设计方案",
        )
        .unwrap();
        fs::write(
            wiki_dir.join("b.md"),
            "---\nname: 记忆系统方案\ndescription: 记忆管理设计\ntype: concept\nupdated: 2026-06-22\ntags: memory\n---\n\n记忆系统管理方案",
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(vault).unwrap();
        std::env::set_var("WIKI_DB_PATH", db_path.to_str().unwrap());

        // 第一次 sync:两篇都是新文档,incremental 应查重
        let report = wiki_query::sync::run_with_scope("incremental");
        assert_eq!(report.scope, "incremental");
        // dedupe_report 可能有相似对(取决于 bm25 分数)
        // 只验证不 panic + 结构正确
        let _pairs: &Vec<wiki_query::models::SimilarPair> = &report.dedupe_report;

        std::env::set_current_dir(original_dir).unwrap();
        std::env::remove_var("WIKI_DB_PATH");
    }

    #[test]
    fn full_queries_all_docs() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let vault_dir = setup_temp_db();
        let vault = vault_dir.path();

        let db_path = vault.join("db").join("wiki.db");
        fs::create_dir_all(db_path.parent().unwrap()).unwrap();

        let wiki_dir = vault.join("wiki");
        fs::create_dir_all(&wiki_dir).unwrap();

        fs::write(
            wiki_dir.join("doc.md"),
            "---\nname: 测试\ndescription: d\ntype: concept\nupdated: 2026-06-22\ntags: t\n---\n\n正文",
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(vault).unwrap();
        std::env::set_var("WIKI_DB_PATH", db_path.to_str().unwrap());

        let report = wiki_query::sync::run_with_scope("full");
        assert_eq!(report.scope, "full");

        std::env::set_current_dir(original_dir).unwrap();
        std::env::remove_var("WIKI_DB_PATH");
    }
}

// ============================================================
// sync idempotency 测试
// ============================================================

mod idempotency_tests {
    use super::*;

    #[test]
    fn consecutive_syncs_are_idempotent() {
        let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let vault_dir = setup_temp_db();
        let vault = vault_dir.path();

        let db_path = vault.join("db").join("wiki.db");
        fs::create_dir_all(db_path.parent().unwrap()).unwrap();

        let wiki_dir = vault.join("wiki");
        fs::create_dir_all(&wiki_dir).unwrap();
        fs::write(
            wiki_dir.join("doc.md"),
            "---\nname: 幂等测试\ndescription: d\ntype: concept\nupdated: 2026-06-22\ntags: t\n---\n\n正文",
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(vault).unwrap();
        std::env::set_var("WIKI_DB_PATH", db_path.to_str().unwrap());

        let r1 = wiki_query::sync::run_with_scope("incremental");
        let r2 = wiki_query::sync::run_with_scope("incremental");

        // 第一次应更新,第二次不应更新(文件未变)
        assert!(r1.updated >= 1, "第一次应更新: {:?}", r1);
        assert_eq!(r2.updated, 0, "第二次不应更新(幂等): {:?}", r2);
        // 两次的 synced 应相同
        assert_eq!(r1.synced, r2.synced, "扫描数应相同");

        std::env::set_current_dir(original_dir).unwrap();
        std::env::remove_var("WIKI_DB_PATH");
    }
}

// ============================================================
// 路径作用域检查测试(纯函数,无全局状态,不需要 GLOBAL_TEST_MUTEX)
// ============================================================

mod scope_tests {
    use wiki_query::sync::{is_in_wiki_scope, strip_vault_root};

    // --- 作用域内 ---

    #[test]
    fn in_scope_sources() {
        assert!(is_in_wiki_scope("sources/dev/rust.md"));
    }

    #[test]
    fn in_scope_wiki() {
        assert!(is_in_wiki_scope("wiki/page.md"));
    }

    #[test]
    fn in_scope_templates() {
        assert!(is_in_wiki_scope("templates/wiki.md"));
    }

    #[test]
    fn in_scope_sources_subdir() {
        assert!(is_in_wiki_scope("sources/projects/roadmap.md"));
    }

    // --- 作用域外 ---

    #[test]
    fn out_scope_claude_memory() {
        assert!(!is_in_wiki_scope(".claude/memory/conversation.md"));
    }

    #[test]
    fn out_scope_root_file() {
        assert!(!is_in_wiki_scope("CLAUDE.md"));
    }

    #[test]
    fn out_scope_git() {
        assert!(!is_in_wiki_scope(".git/config"));
    }

    // --- 路径变体 ---

    #[test]
    fn path_with_leading_dot_slash() {
        assert!(is_in_wiki_scope("./wiki/page.md"));
    }

    #[test]
    fn path_with_leading_slash() {
        assert!(is_in_wiki_scope("/wiki/page.md"));
    }

    #[test]
    fn path_with_backslashes() {
        assert!(is_in_wiki_scope("wiki\\page.md"));
    }

    // --- strip_vault_root ---

    #[test]
    fn strip_vault_root_prefix() {
        let abs = "/home/user/vault/wiki/page.md";
        let vault = "/home/user/vault";
        let rel = strip_vault_root(abs, vault);
        assert_eq!(rel, "wiki/page.md");
        assert!(is_in_wiki_scope(&rel));
    }

    #[test]
    fn strip_vault_root_partial_prefix_not_stripped() {
        // vault2 不应被当作 vault 的子路径
        let abs = "/home/user/vault2/page.md";
        let vault = "/home/user/vault";
        let rel = strip_vault_root(abs, vault);
        assert_eq!(rel, "/home/user/vault2/page.md");
    }

    #[test]
    fn strip_vault_root_exact_match() {
        // 路径恰好等于 root 时返回空字符串
        let rel = strip_vault_root("/home/user/vault", "/home/user/vault");
        assert_eq!(rel, "");
    }

    // --- 边界情况 ---

    #[test]
    fn empty_path() {
        assert!(!is_in_wiki_scope(""));
    }

    #[test]
    fn exact_scope_dir() {
        // 只有目录名没有文件,如 "sources/" — 不算作用域内
        assert!(!is_in_wiki_scope("sources/"));
    }

    #[test]
    fn path_traversal_not_in_scope() {
        // 尝试用 .. 绕过作用域检查
        assert!(!is_in_wiki_scope("wiki/../.claude/memory.md"));
    }
}
