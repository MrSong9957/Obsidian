//! Task 3.2 集成测试:core::write 端到端(落盘 + 即时索引 + 四态决策)。
//!
//! 用 WIKI_DB_PATH 指向 tempdir 隔离数据库,不污染真实 vault。
//! 注意:同一文件的测试默认串行执行,env 竞争可控。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use wiki_query::core;
use wiki_query::frontmatter::Frontmatter;
use wiki_query::models::WriteStatus;

static SEQ: AtomicU64 = AtomicU64::new(0);

/// 全局锁:WIKI_DB_PATH 是进程级环境变量,测试若并发会互相覆盖。
/// 用此锁强制 core_write_test 内的测试串行,保证每个测试的 env 隔离。
static DB_LOCK: Mutex<()> = Mutex::new(());

/// 每个测试用唯一 db 路径。调用方须持有 DB_LOCK。
fn setup_locked() -> (std::sync::MutexGuard<'static, ()>, tempfile::TempDir, String) {
    let guard = DB_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let vault_root = dir.path().to_str().unwrap().to_string();
    let n = SEQ.fetch_add(1, Ordering::SeqCst);
    let db_path = dir.path().join(format!("test_{}.db", n));
    std::env::set_var("WIKI_DB_PATH", db_path.to_str().unwrap());
    wiki_query::db::init().unwrap();
    (guard, dir, vault_root)
}

fn fm(name: &str) -> Frontmatter {
    Frontmatter {
        name: name.into(),
        description: "测试描述".into(),
        doc_type: "concept".into(),
        updated: "2026-06-20".into(),
        tags: "test".into(),
        ..Default::default()
    }
}

#[test]
fn write_add_creates_file_and_indexes() {
    let (_guard, _dir, vault) = setup_locked();
    let path = "wiki/test-add.md";
    let result = core::write(&vault, path, &fm("全新测试文档"), "# 正文内容").unwrap();
    assert_eq!(result.status, WriteStatus::Add);
    assert!(result.synced);
    // 文件落盘
    let content = std::fs::read_to_string(format!("{}/{}", vault, path)).unwrap();
    assert!(content.contains("name: 全新测试文档"));
    // 即时索引:write 后立即可搜到
    let hits = core::search("全新测试文档").unwrap();
    assert!(hits.iter().any(|h| h.path.contains("test-add")), "应能搜到新文档");
}

#[test]
fn write_update_when_path_exists() {
    let (_guard, _dir, vault) = setup_locked();
    let path = "wiki/upd.md";
    // 第一次写
    core::write(&vault, path, &fm("原文档"), "原正文").unwrap();
    // 同 path 再写(不同 name)→ UPDATE
    let result = core::write(&vault, path, &fm("改后文档"), "新正文").unwrap();
    assert_eq!(result.status, WriteStatus::Update);
    assert!(result.synced);
}

#[test]
fn write_noop_when_same_name_different_path() {
    let (_guard, _dir, vault) = setup_locked();
    // 先建 agent.md
    core::write(&vault, "wiki/agent.md", &fm("Agent"), "正文").unwrap();
    // 跨 path 写同名 → NOOP
    let result = core::write(&vault, "wiki/other.md", &fm("Agent"), "正文").unwrap();
    assert_eq!(result.status, WriteStatus::Noop);
    assert!(!result.synced);
    // other.md 不应落盘
    assert!(!std::path::Path::new(&format!("{}/wiki/other.md", vault)).exists());
}

#[test]
fn write_conflict_returns_candidates_no_write() {
    let (_guard, _dir, vault) = setup_locked();
    // 先建 agent.md
    core::write(&vault, "wiki/agent.md", &fm("Agent"), "正文").unwrap();
    // 相似但不同 name → CONFLICT
    let result = core::write(&vault, "wiki/agent-pattern.md", &fm("Agent Pattern"), "正文").unwrap();
    assert_eq!(result.status, WriteStatus::Conflict);
    assert!(!result.synced);
    let cands = result.candidates.expect("CONFLICT 应有候选");
    assert!(!cands.is_empty());
    // agent-pattern.md 不应落盘
    assert!(!std::path::Path::new(&format!("{}/wiki/agent-pattern.md", vault)).exists());
}

#[test]
fn write_rejects_templates_path() {
    let (_guard, _dir, vault) = setup_locked();
    let result = core::write(&vault, "templates/x.md", &fm("T"), "正文");
    assert!(result.is_err(), "templates 只读,应被 policy 拒绝");
}

#[test]
fn write_rejects_path_traversal() {
    let (_guard, _dir, vault) = setup_locked();
    let result = core::write(&vault, "wiki/../../etc.md", &fm("T"), "正文");
    assert!(result.is_err(), "路径穿越应被拒绝");
}

#[test]
fn write_rejects_invalid_frontmatter() {
    let (_guard, _dir, vault) = setup_locked();
    let bad_fm = Frontmatter {
        name: "".into(), // 空 name
        description: "d".into(),
        doc_type: "concept".into(),
        ..Default::default()
    };
    let result = core::write(&vault, "wiki/x.md", &bad_fm, "正文");
    assert!(result.is_err(), "空 name 应被 frontmatter 校验拒绝");
}

#[test]
fn write_creates_parent_dirs() {
    let (_guard, _dir, vault) = setup_locked();
    // 深层路径,父目录不存在
    let path = "wiki/sub/deep/nested.md";
    let result = core::write(&vault, path, &fm("深层文档"), "正文").unwrap();
    assert_eq!(result.status, WriteStatus::Add);
    assert!(std::path::Path::new(&format!("{}/{}", vault, path)).exists());
}
