//! sync 仓库边界确认定向回归(2026-09-21 风险收口,基线 64c27b6):
//! 错误 cwd(恰含 wiki//sources/ 同名目录)+ WIKI_DB_PATH 指向目标库时,
//! 同步必须在任何索引访问(含 init/迁移)之前整体拒绝——不污染、不误清、不建库。
//! 另含 R3 证明性测试:迁移/中断留下的 FTS 缺口在同一轮 sync 内自愈,
//! 文件内容与 mtime 不变时检索恢复命中。
//!
//! 验收基于 DB 真实状态(documents + documents_fts)与 core::search 检索结果,
//! 不以"调用成功/无报错"为标准。
//!
//! 注意:测试修改进程全局状态(current_dir + WIKI_DB_PATH),必须
//! GLOBAL_TEST_MUTEX 串行化(与 functional_sync_test.rs 同一防护模式)。

use std::fs;
use std::path::Path;
use std::sync::Mutex;

/// 保护 current_dir + WIKI_DB_PATH 的互斥锁。
static GLOBAL_TEST_MUTEX: Mutex<()> = Mutex::new(());

/// 合法 frontmatter 样例,正文含检索唯一词"聚宝盆"。
const DOC: &str = "---\nname: 文档A\ndescription: 描述A\ntype: concept\nupdated: 2026-09-21\ntags: alpha\n---\n\n正文独有词聚宝盆";

/// 恢复全局状态(cwd + env)。测试体内在断言失败提前返回时由调用点前手动调用,
/// 故各测试统一在最后一步前完成恢复;失败泄漏由互斥锁限定在单测试内。
fn teardown(original_dir: &Path) {
    std::env::set_current_dir(original_dir).unwrap();
    std::env::remove_var("WIKI_DB_PATH");
}

/// 建仓库形状目录(wiki/ + sources/)并写入文件列表。
fn write_files(repo: &Path, files: &[&str]) {
    for f in files {
        let p = repo.join(f);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, DOC).unwrap();
    }
}

/// 直接打开 DB 查 path 列表(不经 db::init,避免副作用)。
fn table_paths(db: &Path, table: &str) -> Vec<String> {
    let conn = rusqlite::Connection::open(db).unwrap();
    let mut stmt = conn.prepare(&format!("SELECT path FROM {} ORDER BY path", table)).unwrap();
    stmt.query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
}

fn doc_paths(db: &Path) -> Vec<String> {
    table_paths(db, "documents")
}

fn fts_paths(db: &Path) -> Vec<String> {
    table_paths(db, "documents_fts")
}

fn modified_at_of(db: &Path, path: &str) -> String {
    let conn = rusqlite::Connection::open(db).unwrap();
    conn.query_row(
        "SELECT modified_at FROM documents WHERE path = ?1",
        rusqlite::params![path],
        |r| r.get(0),
    )
    .unwrap()
}

// ============================================================
// R1 边界确认:无法可靠确认仓库时,任何索引访问前整体拒绝
// ============================================================

/// 核心缺陷回归:错误 cwd 恰为 vault 形状(含 wiki//sources/ 与自己的文件),
/// WIKI_DB_PATH 指向已建好索引的真库——修复前会把错误 cwd 的文件写进真库,
/// 并按错误扫描集 prune 真库。
#[test]
fn wrong_repo_shaped_cwd_with_external_db_refused_before_any_write() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();

    // 真库:合法 vault,先正常 sync 出 2 行索引
    let real = tempfile::tempdir().unwrap();
    let real_db = real.path().join("db").join("wiki.db");
    fs::create_dir_all(real_db.parent().unwrap()).unwrap();
    write_files(real.path(), &["wiki/a.md", "wiki/b.md"]);
    std::env::set_var("WIKI_DB_PATH", real_db.to_str().unwrap());
    std::env::set_current_dir(real.path()).unwrap();
    wiki_query::sync::run_with_scope("incremental");
    assert_eq!(doc_paths(&real_db).len(), 2, "前置:真库应已建 2 行索引");

    // 错误 cwd:同样是 vault 形状,含一个真库没有的文件
    let wrong = tempfile::tempdir().unwrap();
    fs::create_dir_all(wrong.path().join("wiki")).unwrap();
    fs::create_dir_all(wrong.path().join("sources")).unwrap();
    write_files(wrong.path(), &["wiki/evil.md"]);
    std::env::set_current_dir(wrong.path()).unwrap();

    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(report.failed, -1, "无法确认仓库边界必须整体拒绝");
    assert_eq!(report.synced, 0, "拒绝后不得扫描任何文件");
    assert_eq!(report.pruned, 0, "拒绝后不得清理任何记录");
    let paths = doc_paths(&real_db);
    assert_eq!(
        paths,
        vec!["wiki/a.md".to_string(), "wiki/b.md".to_string()],
        "真库不得被污染(错误 cwd 文件入库)或误清(按错误扫描集 prune)"
    );
    assert!(
        !wrong.path().join("db").exists(),
        "拒绝路径不得在错误 cwd 下产生任何 DB 文件"
    );
    teardown(&original_dir);
}

/// vault 形状 cwd 但 WIKI_DB_PATH 指向 cwd 之外的库:库不归属本目录,拒绝。
#[test]
fn vault_shaped_cwd_with_db_outside_cwd_refused() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();

    let external = tempfile::tempdir().unwrap();
    let ext_db = external.path().join("db").join("wiki.db");
    fs::create_dir_all(ext_db.parent().unwrap()).unwrap();
    write_files(external.path(), &["wiki/a.md", "wiki/b.md"]);
    std::env::set_var("WIKI_DB_PATH", ext_db.to_str().unwrap());
    std::env::set_current_dir(external.path()).unwrap();
    wiki_query::sync::run_with_scope("incremental");
    assert_eq!(doc_paths(&ext_db).len(), 2, "前置:外部库 2 行");

    // cwd 换成另一个 vault 形状目录,DB 仍指外部库
    let vault = tempfile::tempdir().unwrap();
    write_files(vault.path(), &["wiki/local.md"]);
    std::env::set_current_dir(vault.path()).unwrap();

    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(report.failed, -1, "库在 cwd 之外无法确认归属,必须拒绝");
    assert_eq!(report.synced, 0);
    assert_eq!(
        doc_paths(&ext_db),
        vec!["wiki/a.md".to_string(), "wiki/b.md".to_string()],
        "外部库不得被本目录的扫描污染或 prune"
    );
    assert!(
        !vault.path().join("db").exists(),
        "拒绝路径不得在 cwd 下产生 DB 文件"
    );
    teardown(&original_dir);
}

/// 库在 cwd 内但目录无 wiki//sources/(非仓库形状):不确认,拒绝且不 init。
#[test]
fn db_inside_cwd_without_vault_shape_refused() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("db").join("wiki.db");
    fs::create_dir_all(db.parent().unwrap()).unwrap();
    // 游离 md:确认"拒绝后不得扫描"
    fs::write(dir.path().join("loose.md"), DOC).unwrap();
    std::env::set_var("WIKI_DB_PATH", db.to_str().unwrap());
    std::env::set_current_dir(dir.path()).unwrap();

    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(report.failed, -1, "非仓库形状目录必须拒绝");
    assert_eq!(report.synced, 0, "不得把游离 md 索引进库");
    assert!(
        !db.exists(),
        "拒绝路径不得执行 init(连 DB 文件都不创建)"
    );
    teardown(&original_dir);
}

/// 生产主形态:不设 WIKI_DB_PATH,默认相对路径 db/wiki.db 必然落在 cwd 内,
/// 边界确认放行,同步正常工作(锁定默认分支,防止确认逻辑误伤常规路径)。
#[test]
fn default_relative_db_path_passes_boundary_and_syncs() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();
    std::env::remove_var("WIKI_DB_PATH");

    let repo = tempfile::tempdir().unwrap();
    write_files(repo.path(), &["wiki/rel.md", "sources/dev/s.md"]);
    std::env::set_current_dir(repo.path()).unwrap();

    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(report.failed, 0, "默认相对库路径必须放行");
    assert_eq!(report.synced, 2);
    let db = repo.path().join("db").join("wiki.db");
    assert_eq!(doc_paths(&db).len(), 2, "默认路径建库并入库");
    let results = wiki_query::core::search("聚宝盆").unwrap();
    assert_eq!(results.len(), 2, "默认路径下可检索");
    teardown(&original_dir);
}

// ============================================================
// WIKI_DB_PATH 路径形态:相对在内可用;`..` 逃逸 cwd 拒绝
// ============================================================

/// 普通相对路径(解析后仍在 cwd 内)必须可用,不得因边界确认误伤。
#[test]
fn relative_db_path_inside_cwd_allowed() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();

    let repo = tempfile::tempdir().unwrap();
    write_files(repo.path(), &["wiki/rel.md"]);
    std::env::set_var("WIKI_DB_PATH", "data/nested/wiki.db");
    std::env::set_current_dir(repo.path()).unwrap();

    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(report.failed, 0, "cwd 内相对路径必须放行");
    assert_eq!(report.synced, 1);
    let db = repo.path().join("data").join("nested").join("wiki.db");
    assert_eq!(doc_paths(&db), vec!["wiki/rel.md".to_string()]);
    teardown(&original_dir);
}

/// `..` 逃逸 cwd 的相对路径:解析后库在 cwd 之外,无法确认归属,必须拒绝;
/// 目标库逻辑内容保持不变。锁定实现行为(设计表述曾误写"相对路径必然满足")。
#[test]
fn relative_db_path_with_dotdot_escape_refused() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();

    // 父目录下两个兄弟目录:vault(cwd)与 sibling(库所在,经 ../ 可达)
    let parent = tempfile::tempdir().unwrap();
    let vault = parent.path().join("vault");
    let sibling = parent.path().join("sibling");
    fs::create_dir_all(vault.join("wiki")).unwrap();
    fs::create_dir_all(vault.join("sources")).unwrap();
    fs::create_dir_all(sibling.join("db")).unwrap();
    fs::create_dir_all(sibling.join("wiki")).unwrap();
    fs::write(vault.join("wiki").join("local.md"), DOC).unwrap();
    fs::write(sibling.join("wiki").join("s1.md"), DOC).unwrap();
    fs::write(sibling.join("wiki").join("s2.md"), DOC).unwrap();

    // 前置:从 sibling cwd 正常建库(相对默认路径)2 行
    let sib_db = sibling.join("db").join("wiki.db");
    std::env::set_var("WIKI_DB_PATH", "db/wiki.db");
    std::env::set_current_dir(&sibling).unwrap();
    wiki_query::sync::run_with_scope("incremental");
    assert_eq!(doc_paths(&sib_db).len(), 2, "前置:兄弟库 2 行");

    // cwd=vault,库经 ../ 逃逸指向 sibling
    std::env::set_var("WIKI_DB_PATH", "../sibling/db/wiki.db");
    std::env::set_current_dir(&vault).unwrap();

    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(
        report.failed, -1,
        "`..` 逃逸 cwd 的库无法确认归属,必须拒绝"
    );
    assert_eq!(report.synced, 0, "拒绝后不得扫描");
    assert_eq!(
        doc_paths(&sib_db),
        vec!["wiki/s1.md".to_string(), "wiki/s2.md".to_string()],
        "拒绝后目标库逻辑内容必须原样保留"
    );
    assert!(
        !vault.join("db").exists(),
        "拒绝路径不得在 cwd 下产生 DB 文件"
    );
    teardown(&original_dir);
}

// ============================================================
// R2 拒绝文案:保留排查指引,不回显绝对路径
// ============================================================

/// 子进程验证 CLI sync 的拒绝输出:stderr 有指引、无绝对路径;
/// stdout 汇总暴露 failed=-1(可观察)。
#[test]
fn refusal_output_guides_without_absolute_path() {
    let dir = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let ext_db = external.path().join("db").join("wiki.db");
    fs::create_dir_all(ext_db.parent().unwrap()).unwrap();

    // CARGO_BIN_EXE 由 cargo 注入绝对路径,不依赖进程 cwd——本二进制内
    // 其余测试会突变 cwd,且子进程测试不持互斥锁,并行下不能用相对定位
    let exe = std::path::PathBuf::from(env!("CARGO_BIN_EXE_wiki-query"));
    let out = std::process::Command::new(&exe)
        .arg("sync")
        .current_dir(dir.path())
        .env("WIKI_DB_PATH", &ext_db)
        .output()
        .expect("failed to spawn wiki-query sync");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(stderr.contains("拒绝同步"), "拒绝原因必须可观察,stderr: {}", stderr);
    assert!(
        stderr.contains("WIKI_DB_PATH") || stderr.contains("仓库根"),
        "错误说明必须保留排查指引,stderr: {}",
        stderr
    );
    assert!(
        !stderr.contains(dir.path().to_str().unwrap()),
        "stderr 不得回显 cwd 绝对路径: {}",
        stderr
    );
    assert!(
        !stderr.contains(external.path().to_str().unwrap()),
        "stderr 不得回显 DB 绝对路径: {}",
        stderr
    );
    assert!(
        stdout.contains("-1 failed"),
        "CLI 汇总必须暴露整体失败,stdout: {}",
        stdout
    );
}

// ============================================================
// R3 证明性测试:FTS 缺口在同一轮 sync 内自愈(锁行为,不修代码)
// ============================================================

/// documents 有行、FTS 无行(历史中断残留)→ 下一轮 sync 强制重写,
/// 文件内容与 mtime 不变,检索恢复命中。
#[test]
fn fts_gap_from_interrupted_write_self_heals_without_file_change() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();

    let repo = tempfile::tempdir().unwrap();
    let db = repo.path().join("db").join("wiki.db");
    fs::create_dir_all(db.parent().unwrap()).unwrap();
    write_files(repo.path(), &["wiki/gap.md"]);
    std::env::set_var("WIKI_DB_PATH", db.to_str().unwrap());
    std::env::set_current_dir(repo.path()).unwrap();
    wiki_query::sync::run_with_scope("incremental");

    // 构造中断残留:只删 FTS 行,documents 行与文件都保持原样
    {
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute(
            "DELETE FROM documents_fts WHERE path = 'wiki/gap.md'",
            [],
        )
        .unwrap();
    }
    let modified_before = modified_at_of(&db, "wiki/gap.md");

    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(report.updated, 1, "FTS 缺失必须触发强制重写");
    assert_eq!(fts_paths(&db), vec!["wiki/gap.md".to_string()], "FTS 行必须恢复");
    let results = wiki_query::core::search("聚宝盆").unwrap();
    assert_eq!(results.len(), 1, "自愈后必须可检索");
    assert_eq!(results[0].path, "wiki/gap.md");
    assert_eq!(
        modified_at_of(&db, "wiki/gap.md"),
        modified_before,
        "文件未修改,modified_at 不得漂移"
    );
    teardown(&original_dir);
}

/// 迁移 dup 分支的对抗性前置(documents 存 '\'+'/' 双行、FTS 仅存 '\' 行,
/// 正常流程不可达,等价于跨平台混用/外部损坏)→ 迁移删旧行后,
/// 扫描阶段 needs_update 的 FTS 缺失检查自愈,收敛为单行且可检索。
#[test]
fn migration_dup_with_legacy_only_fts_row_converges_and_retrievable() {
    let _guard = GLOBAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let original_dir = std::env::current_dir().unwrap();

    let repo = tempfile::tempdir().unwrap();
    let db = repo.path().join("db").join("wiki.db");
    fs::create_dir_all(db.parent().unwrap()).unwrap();
    write_files(repo.path(), &["wiki/d.md"]);
    std::env::set_var("WIKI_DB_PATH", db.to_str().unwrap());
    std::env::set_current_dir(repo.path()).unwrap();
    wiki_query::sync::run_with_scope("incremental");

    // 构造跨平台残迹:documents 双行 + FTS 仅存 '\' 行(拷贝字段,仅换 path)
    {
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute(
            "INSERT INTO documents (path, name, description, type, updated, tags, modified_at)
             SELECT 'wiki\\d.md', name, description, type, updated, tags, modified_at
             FROM documents WHERE path = 'wiki/d.md'",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO documents_fts (path, name, description, type, tags, content)
             SELECT 'wiki\\d.md', name, description, type, tags, content
             FROM documents_fts WHERE path = 'wiki/d.md'",
            [],
        )
        .unwrap();
        conn.execute(
            "DELETE FROM documents_fts WHERE path = 'wiki/d.md'",
            [],
        )
        .unwrap();
    }
    assert_eq!(
        doc_paths(&db),
        vec!["wiki/d.md".to_string(), "wiki\\d.md".to_string()],
        "前置:documents 应存双行"
    );
    assert_eq!(fts_paths(&db), vec!["wiki\\d.md".to_string()], "前置:FTS 应仅存旧行");

    let report = wiki_query::sync::run_with_scope("incremental");

    assert_eq!(
        doc_paths(&db),
        vec!["wiki/d.md".to_string()],
        "迁移去重后必须收敛为单行正斜杠记录"
    );
    assert_eq!(fts_paths(&db), vec!["wiki/d.md".to_string()], "FTS 必须与新口径对齐");
    assert_eq!(report.pruned, 0, "文件在盘,不得触发 prune");
    let results = wiki_query::core::search("聚宝盆").unwrap();
    assert_eq!(results.len(), 1, "收敛后必须可检索");
    assert_eq!(results[0].path, "wiki/d.md");
    teardown(&original_dir);
}
