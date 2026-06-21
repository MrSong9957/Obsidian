use std::path::Path;
use chrono::Local;
use rusqlite::{Connection, params};
use crate::db::init;
use crate::frontmatter;
use crate::models::{SyncReport, SimilarPair};

// ============================================================
// 路径作用域检查
// ============================================================

/// 将路径中的反斜杠替换为正斜杠,去除开头的 `./` 和前导 `/`,
/// 并解析 `..` / `.` 路径段。内部统一用正斜杠。
fn normalize_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");

    // 去除前导 / 和 ./
    let stripped = normalized
        .trim_start_matches('/')
        .trim_start_matches("./");

    // 解析 .. 和 . 路径段
    let segments: Vec<&str> = stripped.split('/').collect();
    let mut resolved: Vec<&str> = Vec::new();
    for seg in &segments {
        match *seg {
            "." => {}
            ".." => {
                resolved.pop();
            }
            s if !s.is_empty() => resolved.push(s),
            _ => {}
        }
    }
    resolved.join("/")
}

/// 从绝对路径中去除 vault root 前缀,返回相对路径。
/// 例如 `strip_vault_root("/home/user/vault/wiki/page.md", "/home/user/vault")`
/// 返回 `"wiki/page.md"`。
pub fn strip_vault_root(path: &str, vault_root: &str) -> String {
    let normalized_root = normalize_path(vault_root);
    let normalized_path = normalize_path(path);
    // 尝试去除 root 前缀(带或不带尾部斜杠)
    let root_variants = [
        format!("{}/", normalized_root),
        normalized_root.clone(),
    ];
    for root in &root_variants {
        if let Some(rest) = normalized_path.strip_prefix(root) {
            return rest.to_string();
        }
    }
    // 前缀不匹配,原样返回
    normalized_path
}

/// 判断路径是否属于 wiki 知识库的作用域。
/// 作用域: `sources/`、`wiki/`、`templates/` 三个目录下的 `.md` 文件。
/// 传入的路径应为相对路径(如 `wiki/page.md`、`sources/dev/rust.md`)。
/// 如果是绝对路径,需先调用 `strip_vault_root` 去除前缀。
pub fn is_in_wiki_scope(path: &str) -> bool {
    let normalized = normalize_path(path);
    let in_scope_dirs = ["sources/", "wiki/", "templates/"];
    in_scope_dirs.iter().any(|dir| normalized.starts_with(dir))
}

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
    // 一次查询拿全库 path+name+description+type+updated+tags(元数据)
    let rows: Vec<(String, String, String, String, String, String)> = {
        let mut stmt = match conn.prepare(
            "SELECT path, name, description, type, updated, tags FROM documents",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .ok()
        .map(|iter| iter.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    };

    // 对每个文档,用传入的 conn 内联 FTS5 查询拿 bm25 score(不开新连接)。
    // 之前每个文档调 core::search_with_scores 各开一次 db::init 连接,O(n) 连接,
    // 是 sync(full) 极慢的主因。现在复用 conn,O(n) 次同连接查询。
    rows.into_iter()
        .map(|(path, name, description, doc_type, updated, tags)| {
            let score = fts_score_for_name(conn, &path, &name);
            crate::models::SearchResult {
                path,
                name,
                description,
                doc_type,
                updated,
                tags,
                score,
            }
        })
        .collect()
}

/// 用传入的 conn 对单个文档(按 path)的 name 做 FTS5 查询,返回 bm25 score。
/// 复用现有连接,不开新连接。查不到(如 name 为空/无 token)返回 None。
fn fts_score_for_name(conn: &Connection, path: &str, name: &str) -> Option<f64> {
    use crate::fts::tokenize_for_fts;
    let tokenized = tokenize_for_fts(name);
    let fts_query: String = tokenized
        .split_whitespace()
        .filter(|t| !t.contains('"') && t.chars().any(|c| c.is_alphanumeric()))
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(" OR ");
    if fts_query.is_empty() {
        return None;
    }
    conn.query_row(
        "SELECT bm25(documents_fts) FROM documents_fts
         WHERE documents_fts MATCH ?1 AND path = ?2",
        params![fts_query, path],
        |row| row.get(0),
    )
    .ok()
}

/// 删除数据库里存在但磁盘上不存在的记录,返回删除条数。
/// 同时清理 documents 和 documents_fts(保持两表一致)。
///
/// 注意:路径规范化约定 —— on_disk 与 DB 里存的 path 都用 `strip_prefix("./")`
/// 后的形式,Windows 上含 `\`。跨平台同步(如从 Linux 同步用 `/` 的 path)会
/// 因分隔符不一致触发误删,目前仅支持单平台 sync。
///
/// 删除前对每个候选 path 做 exists() 二次确认:文件实际还在(如被杀毒锁导致
/// read_dir 跳过)则保留,不误删。
pub(crate) fn prune_orphans(conn: &Connection, on_disk: &std::collections::HashSet<String>) -> usize {
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
        // 二次确认:文件确实不存在才删。
        // 防止 read_dir 因文件锁定跳过(scan_directory 里 flatten→显式 Err skip)
        // 导致文件不进 on_disk → 误删有效文档。exists() 为真说明文件还在,保留。
        let abs = Path::new(".").join(path);
        if abs.exists() {
            continue;
        }
        // documents 删成功才删 FTS5,保证两表一致(避免 documents 有、FTS5 无)。
        // 用事务包两条 DELETE,任一失败则整体回滚。
        let result: Result<(), rusqlite::Error> = (|| {
            conn.execute("BEGIN", [])?;
            let docs_deleted = conn.execute("DELETE FROM documents WHERE path = ?1", params![path])?;
            conn.execute("DELETE FROM documents_fts WHERE path = ?1", params![path])?;
            // COMMIT 失败也要 ROLLBACK,避免连接卡在打开事务(与 sync_file 同理)
            if let Err(e) = conn.execute("COMMIT", []) {
                let _ = conn.execute("ROLLBACK", []);
                return Err(e);
            }
            if docs_deleted > 0 {
                pruned += 1;
            }
            Ok(())
        })();
        if let Err(e) = result {
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

    for entry in entries {
        // 显式处理 DirEntry 错误(不用 flatten):文件被杀毒/索引器锁定时
        // read_dir 返回 Err。这里记录警告并 skip,不进 on_disk。
        // 真正的防误删由 prune_orphans 的二次确认(exists 检查)兜底。
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!(
                    "[wiki-sync] 警告:读取目录项失败 {:?}: {},跳过",
                    dir, e
                );
                continue;
            }
        };
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

    // 用事务包 documents upsert + FTS5 同步,保证两表原子一致。
    // 中途崩溃(如进程被杀)不会出现 documents 已更新但 FTS5 旧记录已删未插的状态;
    // 即便发生,needs_update 的 FTS5-缺失检查下次 sync 会自愈。
    //
    // documents 表只存元数据(不含正文,符合 v5 原则);
    // documents_fts 存正文(FTS5 倒排索引的原始数据,检索根基)。
    conn.execute("BEGIN", [])?;
    let result: Result<(), rusqlite::Error> = (|| {
        conn.execute(
            "INSERT INTO documents (path, name, description, type, updated, tags, modified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                type = excluded.type,
                updated = excluded.updated,
                tags = excluded.tags,
                modified_at = excluded.modified_at",
            params![
                path_str,
                meta.name,
                meta.description,
                meta.doc_type,
                meta.updated,
                meta.tags,
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
            // COMMIT 失败也必须 ROLLBACK,否则连接卡在打开事务,
            // 后续所有 sync_file 的 BEGIN 会报 "cannot start a transaction
            // within a transaction",整个 sync 静默失败。
            if let Err(e) = conn.execute("COMMIT", []) {
                let _ = conn.execute("ROLLBACK", []);
                return Err(Box::from(e));
            }
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(Box::from(e))
        }
    }
}
