//! v5 Task 5 集成测试:sync 的查重报告(incremental + full)。
//!
//! 用真实 vault 跑 sync,验证返回的 SyncReport 结构 + scope 字段。
//! 具体相似对数依赖 vault 内容,这里只验证结构正确。

use wiki_query::sync;

#[test]
fn sync_incremental_returns_report_with_incremental_scope() {
    let report = sync::run_with_scope("incremental");
    assert_eq!(report.scope, "incremental");
    assert!(report.synced >= 0, "synced 应非负");
    assert!(report.updated >= 0, "updated 应非负");
    assert!(report.pruned >= 0, "pruned 应非负");
    // dedupe_report 是 Vec,结构正确即可
    let _pairs: &Vec<wiki_query::models::SimilarPair> = &report.dedupe_report;
}

#[test]
fn sync_full_returns_report_with_full_scope() {
    let report = sync::run_with_scope("full");
    assert_eq!(report.scope, "full");
    assert!(report.synced >= 0);
}

#[test]
fn sync_run_cli_entry_does_not_panic() {
    // run() 是 CLI 入口,内部调 run_with_scope 并 println。
    // 不 panic 即可(它会打印到 stdout)。
    sync::run();
}
