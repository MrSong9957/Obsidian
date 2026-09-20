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
/// 返回 (规范化路径, 是否越出根)。越出根:`..` 在根边界上继续上跳,
/// 该路径实际指向根之外,作用域判定必须为 false。
fn normalize_path_checked(path: &str) -> (String, bool) {
    let normalized = path.replace('\\', "/");

    // 去除前导 / 和 ./
    let stripped = normalized
        .trim_start_matches('/')
        .trim_start_matches("./");

    // 解析 .. 和 . 路径段
    let segments: Vec<&str> = stripped.split('/').collect();
    let mut resolved: Vec<&str> = Vec::new();
    let mut escaped_root = false;
    for seg in &segments {
        match *seg {
            "." => {}
            ".." => {
                if resolved.pop().is_none() {
                    escaped_root = true;
                }
            }
            s if !s.is_empty() => resolved.push(s),
            _ => {}
        }
    }
    (resolved.join("/"), escaped_root)
}

fn normalize_path(path: &str) -> String {
    normalize_path_checked(path).0
}

/// 从绝对路径中去除 vault root 前缀,返回相对路径。
/// 例如 `strip_vault_root("/home/user/vault/wiki/page.md", "/home/user/vault")`
/// 返回 `"wiki/page.md"`。
pub fn strip_vault_root(path: &str, vault_root: &str) -> String {
    let normalized_root = normalize_path(vault_root);
    let normalized_path = normalize_path(path);
    // 先用尾部斜杠匹配(安全,不会误匹配 vault → vault2)
    if let Some(rest) = normalized_path.strip_prefix(&format!("{}/", normalized_root)) {
        return rest.to_string();
    }
    // 路径恰好等于 root 时,返回空字符串(表示 root 本身就是目标)
    if normalized_path == normalized_root {
        return String::new();
    }
    // 前缀不匹配,原样返回
    path.to_string()
}

/// 判断路径是否属于 wiki 知识库的作用域。
/// 作用域: `sources/`、`wiki/`、`templates/` 三个目录下的 `.md` 文件。
/// 传入的路径应为相对路径(如 `wiki/page.md`、`sources/dev/rust.md`)。
/// 如果是绝对路径,需先调用 `strip_vault_root` 去除前缀。
pub fn is_in_wiki_scope(path: &str) -> bool {
    let (normalized, escaped_root) = normalize_path_checked(path);
    // `..` 越出根的路径实际指向 vault 之外,不在作用域内
    if escaped_root {
        return false;
    }
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
///
/// 库不可用(如 schema 不兼容)时不 panic、不半坏:报告 failed=-1 表示
/// 整体失败,stderr 给出可观察错误与重建指引。
pub fn run_with_scope(scope: &str) -> SyncReport {
    let conn = match init() {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "[wiki-sync] 错误: {}。未执行任何同步,索引保持原状。{}",
                e,
                "若为 schema 不兼容,删除 db 目录后重跑 sync 即可重建。"
            );
            return SyncReport {
                synced: 0,
                updated: 0,
                pruned: 0,
                failed: -1,
                scope: scope.into(),
                dedupe_report: vec![],
            };
        }
    };

    // 分隔符口径统一:迁移存量反斜杠路径,保证入库/prune/exists 同一口径
    normalize_legacy_separators(&conn);

    let root = Path::new(".");

    let mut count = 0;
    let mut updated = 0;
    let mut failed = 0;
    let mut updated_paths: Vec<String> = Vec::new(); // 本次更新的路径(incremental 查重用)
    let mut on_disk: std::collections::HashSet<String> = std::collections::HashSet::new();
    scan_directory(root, &conn, &mut count, &mut updated, &mut failed, &mut on_disk, &mut updated_paths);

    // cwd 守卫:当前目录不含 wiki/ 或 sources/ 时视为不在 vault 内,
    // 跳过 prune——防止错误 cwd 下 on_disk 不含真库路径导致整库误清。
    // 与 serve 启动校验同一判据;合法 vault 内删空文件不受影响(目录仍在)。
    let cwd_is_vault =
        Path::new(".").join("wiki").is_dir() || Path::new(".").join("sources").is_dir();
    let pruned = if cwd_is_vault {
        prune_orphans(&conn, &on_disk)
    } else {
        eprintln!(
            "[wiki-sync] 警告: 当前目录不含 wiki/ 或 sources/,跳过 prune 以防误清非本库索引(cwd={:?})",
            std::env::current_dir()
        );
        0
    };

    // 查重:scope 决定候选集
    let dedupe_report = build_dedupe_report(scope, &updated_paths, &conn);

    SyncReport {
        synced: count,
        updated,
        pruned: pruned as i32,
        failed,
        scope: scope.into(),
        dedupe_report,
    }
}

/// 把库中反斜杠路径统一为正斜杠口径(与扫描入库同一口径)。
/// 若正斜杠版本已存在(同一逻辑文件两条记录)则删除旧行去重,
/// documents 与 documents_fts 两表同步。幂等:迁移后无反斜杠行。
fn normalize_legacy_separators(conn: &Connection) {
    let legacy: Vec<String> = {
        let mut stmt = match conn.prepare("SELECT path FROM documents WHERE path LIKE '%\\%'") {
            Ok(s) => s,
            Err(_) => return,
        };
        stmt.query_map([], |r| r.get::<_, String>(0))
            .map(|iter| iter.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    };
    for old in legacy {
        let new = old.replace('\\', "/");
        if new == old {
            continue;
        }
        let dup: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM documents WHERE path = ?1",
                params![new],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let result: Result<(), rusqlite::Error> = (|| {
            conn.execute("BEGIN", [])?;
            let step: rusqlite::Result<usize> = if dup > 0 {
                // 同一逻辑文件两条记录:删旧行(正斜杠行是本次/后续 sync 的权威形态)
                conn.execute("DELETE FROM documents WHERE path = ?1", params![old])
                    .and_then(|_| conn.execute("DELETE FROM documents_fts WHERE path = ?1", params![old]))
            } else {
                conn.execute("UPDATE documents SET path = ?2 WHERE path = ?1", params![old, new])
                    .and_then(|_| conn.execute("UPDATE documents_fts SET path = ?2 WHERE path = ?1", params![old, new]))
            };
            step?;
            if let Err(e) = conn.execute("COMMIT", []) {
                let _ = conn.execute("ROLLBACK", []);
                return Err(e);
            }
            Ok(())
        })();
        match result {
            Ok(()) => eprintln!("[wiki-sync] 路径分隔符迁移: {} -> {}", old, new),
            Err(e) => eprintln!("[wiki-sync] 警告: 分隔符迁移失败 {}: {}", old, e),
        }
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
    let threshold = -2.0;
    let all_docs = load_all_docs(conn);

    let candidates: Vec<&crate::models::SearchResult> = if scope == "full" {
        all_docs.iter().collect()
    } else {
        updated_paths
            .iter()
            .filter_map(|p| all_docs.iter().find(|d| &d.path == p))
            .collect()
    };

    let mut pairs = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for candidate in &candidates {
        let similar = search_similar_by_name(conn, &candidate.path, &candidate.name);
        for (similar_path, score) in similar {
            if score > threshold {
                continue; // 不够相关
            }
            let key = if candidate.path < similar_path {
                format!("{}|{}", candidate.path, similar_path)
            } else {
                format!("{}|{}", similar_path, candidate.path)
            };
            if seen.insert(key) {
                pairs.push(SimilarPair {
                    candidate: candidate.path.clone(),
                    similar_to: similar_path,
                    score,
                });
            }
        }
    }
    pairs
}

/// 从 documents 表加载所有文档的元数据（不含 score，score 由查重时实时计算）。
fn load_all_docs(conn: &Connection) -> Vec<crate::models::SearchResult> {
    let mut stmt = match conn.prepare(
        "SELECT path, name, description, type, updated, tags FROM documents",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map([], |row| {
        Ok(crate::models::SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            doc_type: row.get(3)?,
            updated: row.get(4)?,
            tags: row.get(5)?,
            score: None,
        })
    })
    .ok()
    .map(|iter| iter.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

/// 用 candidate_name 做 FTS5 查询，在全库中找 BM25 分数最低（最相关）的文档。
/// 返回 Vec<(path, score)>，排除 candidate 自身的 path。
/// 这是真正的跨文档相似度计算。
fn search_similar_by_name(
    conn: &Connection,
    candidate_path: &str,
    candidate_name: &str,
) -> Vec<(String, f64)> {
    let fts_query = crate::core::build_fts_query(candidate_name);
    if fts_query.is_empty() {
        return vec![];
    }
    let mut stmt = match conn.prepare(
        "SELECT d.path, bm25(documents_fts) AS score
         FROM documents_fts f
         JOIN documents d ON d.path = f.path
         WHERE documents_fts MATCH ?1
         ORDER BY score
         LIMIT 20",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map(rusqlite::params![fts_query], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f64>(1)?,
        ))
    })
    .ok()
    .map(|iter| {
        iter.filter_map(|r| r.ok())
            .filter(|(path, _)| path != candidate_path)
            .collect()
    })
    .unwrap_or_default()
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
        // 归一化比对:库内历史反斜杠行与本次扫描的正斜杠行是同一逻辑文件
        let normalized = path.replace('\\', "/");
        if on_disk.contains(&normalized) {
            continue;
        }
        // 二次确认:文件确实不存在才删。
        // 防止 read_dir 因文件锁定跳过(scan_directory 里 flatten→显式 Err skip)
        // 导致文件不进 on_disk → 误删有效文档。exists() 为真说明文件还在,保留。
        let abs = Path::new(".").join(path);
        if abs.exists() {
            continue;
        }
        // 父目录失联保护:父目录不存在时无法区分"用户删除整个子树"与
        // "子树临时失联(挂载/权限/锁定)",保守保留索引行,防止批量误清。
        // Markdown 真本恢复可见后照常更新;父目录存在的单文件删除仍正常 prune。
        if abs.parent().map_or(false, |p| !p.exists()) {
            eprintln!(
                "[wiki-sync] 警告: 父目录缺失,疑似子树失联,保留索引行: {}",
                path
            );
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
    failed: &mut i32,
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
            scan_directory(&path, conn, count, updated, failed, on_disk, updated_paths);
            continue;
        }

        // 只处理 .md 文件
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        *count += 1;

        let relative_path = path.strip_prefix("./").unwrap_or(&path);
        // 统一 '/' 口径:DB 存储与 prune/exists 比对共用,避免 Windows(\)
        // 与 Unix(/)表示差异造成重复索引或漏比对
        let path_str = relative_path.to_string_lossy().replace('\\', "/");
        on_disk.insert(path_str.clone());

        // 检查是否需要更新
        if needs_update(conn, &path_str, &path) {
            if sync_file(conn, &path_str, &path).is_ok() {
                *updated += 1;
                updated_paths.push(path_str.clone());
            } else {
                // 失败必须可观察:索引保留旧数据,报告区分"全部成功"与"部分失败"
                *failed += 1;
                eprintln!(
                    "[wiki-sync] 警告: 文件同步失败(索引保留旧数据): {:?}",
                    path
                );
            }
        }
    }
}

fn needs_update(conn: &Connection, path_str: &str, file_path: &Path) -> bool {
    let metadata = match std::fs::metadata(file_path) {
        Ok(m) => m,
        Err(_) => return true,
    };

    let modified = match metadata.modified() {
        Ok(m) => m,
        Err(_) => return true, // 无法获取修改时间（FAT32/网络挂载），强制更新
    };
    let modified_dt: chrono::DateTime<Local> = modified.into();
    // 毫秒粒度:秒级字符串会让同一秒内的连续修改被判"未变"而漏更新
    let modified_str = modified_dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string();

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
    let modified = metadata.modified()
        .map_err(|e| format!("无法获取修改时间（可能是 FAT32/网络挂载）: {}", e))?;
    let modified_dt: chrono::DateTime<Local> = modified.into();
    // 与 needs_update 同粒度(毫秒),保证同秒连续修改能被再次更新
    let modified_str = modified_dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string();

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
