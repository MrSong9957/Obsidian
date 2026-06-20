use std::path::Path;
use chrono::Local;
use rusqlite::{Connection, params};
use crate::db::init;
use crate::frontmatter;
use crate::models::{SyncReport, SimilarPair};

/// CLI 入口:全量 sync + incremental 查重,打印报告。
pub fn run() {
    let report = run_with_scope("incremental");
    println!(
        "Scan complete: {} files found, {} updated, {} pruned, {} similar pairs (scope={})",
        report.synced, report.updated, report.pruned, report.dedupe_report.len(), report.scope
    );
}

/// 带查重的 sync。scope="incremental" 查本次新文档,scope="full" 查全库。
/// 返回 SyncReport(同步统计 + 查重报告)。
pub fn run_with_scope(scope: &str) -> SyncReport {
    let conn = init().expect("Failed to init database");
    let root = Path::new(".");

    let mut count = 0;
    let mut updated = 0;
    let mut updated_paths: Vec<String> = Vec::new(); // 本次更新的路径(incremental 查重用)
    let mut on_disk: std::collections::HashSet<String> = std::collections::HashSet::new();
    scan_directory(root, &conn, &mut count, &mut updated, &mut on_disk, &mut updated_paths);

    let pruned = prune_orphans(&conn, &on_disk);

    // 查重:scope 决定候选集
    let dedupe_report = build_dedupe_report(scope, &updated_paths, &conn);

    SyncReport {
        synced: count,
        updated,
        pruned: pruned as i32,
        scope: scope.into(),
        dedupe_report,
    }
}

/// 构建查重报告。
/// incremental:候选集 = 本次更新的文档(updated_paths)。
/// full:候选集 = 全库所有文档。
fn build_dedupe_report(
    scope: &str,
    updated_paths: &[String],
    conn: &Connection,
) -> Vec<SimilarPair> {
    let threshold = -2.0; // bm25 阈值:score ≤ -2.0 算相似。实测调。

    let all_docs = load_all_docs_with_scores(conn);
    // 候选集:full 用全库,incremental 用本次更新的文档
    let candidates: Vec<crate::models::SearchResult> = if scope == "full" {
        all_docs.clone()
    } else {
        updated_paths
            .iter()
            .filter_map(|p| all_docs.iter().find(|d| &d.path == p).cloned())
            .collect()
    };

    let mut pairs = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for candidate in &candidates {
        for pair in crate::core::find_similar(candidate, &all_docs, threshold) {
            // 双向去重:(a,b) 与 (b,a) 只报一次
            let key = if pair.candidate < pair.similar_to {
                format!("{}|{}", pair.candidate, pair.similar_to)
            } else {
                format!("{}|{}", pair.similar_to, pair.candidate)
            };
            if seen.insert(key) {
                pairs.push(pair);
            }
        }
    }
    pairs
}

/// 从 documents 表加载所有文档,对每个用 name 做 FTS5 search 拿 score。
/// 性能 O(n²)(每个文档都搜一次),MVP 够用,文档量大时优化。
fn load_all_docs_with_scores(conn: &Connection) -> Vec<crate::models::SearchResult> {
    let paths: Vec<(String, String)> = {
        let mut stmt = match conn.prepare("SELECT path, name FROM documents") {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .ok()
            .map(|iter| iter.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    };
    paths
        .iter()
        .filter_map(|(p, n)| {
            // 用 name 做 search,找到自己那条的 score
            crate::core::search_with_scores(n).ok().and_then(|mut results| {
                // 取出自身(按 path 匹配),它应排在最前(name 完全匹配)
                if let Some(idx) = results.iter().position(|r| r.path == *p) {
                    Some(results.remove(idx))
                } else {
                    results.into_iter().next()
                }
            })
        })
        .collect()
}

/// 删除数据库里存在但磁盘上不存在的记录,返回删除条数。
/// 同时清理 documents 和 documents_fts(保持两表一致)。
///
/// 注意:路径规范化约定 —— on_disk 与 DB 里存的 path 都用 `strip_prefix("./")`
/// 后的形式,Windows 上含 `\`。跨平台同步(如从 Linux 同步用 `/` 的 path)会
/// 因分隔符不一致触发误删,目前仅支持单平台 sync。
fn prune_orphans(conn: &Connection, on_disk: &std::collections::HashSet<String>) -> usize {
    let db_paths: Vec<String> = {
        let mut stmt = match conn.prepare("SELECT path FROM documents") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("prune_orphans: prepare 失败: {}", e);
                return 0;
            }
        };
        stmt.query_map([], |row| row.get::<_, String>(0))
            .ok()
            .map(|iter| iter.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    };

    let mut pruned = 0;
    for path in &db_paths {
        if on_disk.contains(path) {
            continue;
        }
        // documents 删成功才删 FTS5,保证两表一致(避免 documents 有、FTS5 无)。
        // 用事务包两条 DELETE,任一失败则整体回滚。
        let result: Result<(), rusqlite::Error> = (|| {
            conn.execute("BEGIN", [])?;
            let docs_deleted = conn.execute("DELETE FROM documents WHERE path = ?1", params![path])?;
            conn.execute("DELETE FROM documents_fts WHERE path = ?1", params![path])?;
            conn.execute("COMMIT", [])?;
            if docs_deleted > 0 {
                pruned += 1;
            }
            Ok(())
        })();
        if let Err(e) = result {
            let _ = conn.execute("ROLLBACK", []);
            eprintln!("prune_orphans: 删除 {} 失败(已回滚): {}", path, e);
        }
    }
    pruned
}

fn scan_directory(
    dir: &Path,
    conn: &Connection,
    count: &mut i32,
    updated: &mut i32,
    on_disk: &mut std::collections::HashSet<String>,
    updated_paths: &mut Vec<String>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // 跳过隐藏目录
        if path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        if path.is_dir() {
            scan_directory(&path, conn, count, updated, on_disk, updated_paths);
            continue;
        }

        // 只处理 .md 文件
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        *count += 1;

        let relative_path = path.strip_prefix("./").unwrap_or(&path);
        let path_str = relative_path.to_string_lossy().to_string();
        on_disk.insert(path_str.clone());

        // 检查是否需要更新
        if needs_update(conn, &path_str, &path) && sync_file(conn, &path_str, &path).is_ok() {
            *updated += 1;
            updated_paths.push(path_str.clone());
        }
    }
}

fn needs_update(conn: &Connection, path_str: &str, file_path: &Path) -> bool {
    let metadata = match std::fs::metadata(file_path) {
        Ok(m) => m,
        Err(_) => return true,
    };

    let modified = metadata.modified().unwrap();
    let modified_dt: chrono::DateTime<Local> = modified.into();
    let modified_str = modified_dt.format("%Y-%m-%d %H:%M:%S").to_string();

    let stored: String = conn
        .query_row(
            "SELECT modified_at FROM documents WHERE path = ?1",
            params![path_str],
            |row| row.get(0),
        )
        .unwrap_or_default();

    if stored != modified_str {
        return true;
    }

    // documents 表虽未变,但 FTS5 可能缺失该行(如 FTS5 表新建后首次回填)。
    // 任一表缺失都判定需更新,保证两表最终一致。
    let in_fts: i64 = conn
        .query_row(
            "SELECT count(*) FROM documents_fts WHERE path = ?1",
            params![path_str],
            |row| row.get(0),
        )
        .unwrap_or(0);
    in_fts == 0
}

fn sync_file(conn: &Connection, path_str: &str, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let (meta, body) = frontmatter::parse(&content);

    let metadata = std::fs::metadata(file_path)?;
    let modified = metadata.modified().unwrap();
    let modified_dt: chrono::DateTime<Local> = modified.into();
    let modified_str = modified_dt.format("%Y-%m-%d %H:%M:%S").to_string();

    let word_count = body.split_whitespace().count() as i64;

    // 用事务包 documents upsert + FTS5 同步,保证两表原子一致。
    // 中途崩溃(如进程被杀)不会出现 documents 已更新但 FTS5 旧记录已删未插的状态;
    // 即便发生,needs_update 的 FTS5-缺失检查下次 sync 会自愈。
    conn.execute("BEGIN", [])?;
    let result: Result<(), rusqlite::Error> = (|| {
        conn.execute(
            "INSERT INTO documents (path, name, description, type, updated, tags, content, word_count, modified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                type = excluded.type,
                updated = excluded.updated,
                tags = excluded.tags,
                content = excluded.content,
                word_count = excluded.word_count,
                modified_at = excluded.modified_at",
            params![
                path_str,
                meta.name,
                meta.description,
                meta.doc_type,
                meta.updated,
                meta.tags,
                body,
                word_count,
                modified_str
            ],
        )?;

        // 同步 FTS5 索引:对文本字段做 jieba 预分词。
        // FTS5 无 ON CONFLICT UPDATE,用 delete-then-insert 模式更新。
        use crate::fts::tokenize_for_fts;
        conn.execute(
            "DELETE FROM documents_fts WHERE path = ?1",
            params![path_str],
        )?;
        conn.execute(
            "INSERT INTO documents_fts (path, name, description, type, tags, content)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                path_str,
                tokenize_for_fts(&meta.name),
                tokenize_for_fts(&meta.description),
                meta.doc_type,
                tokenize_for_fts(&meta.tags),
                tokenize_for_fts(&body),
            ],
        )?;
        Ok(())
    })();
    match result {
        Ok(()) => {
            conn.execute("COMMIT", [])?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(Box::from(e))
        }
    }
}
