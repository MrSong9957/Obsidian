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

/// 分隔符归一化:仅在 Windows 分隔符语义(`MAIN_SEPARATOR == '\\'`)下把 `\`
/// 替换为 `/`。Unix 上 `\` 是合法文件名字符而非分隔符,替换会破坏索引路径与
/// 磁盘的一致性;Unix 的 Path 文本形式本就以 `/` 分隔,无需替换。
/// `_with` 变体供测试注入分隔符语义(本机无法运行另一种平台的文件系统)。
fn normalize_separators_with(main_sep: char, s: &str) -> String {
    if main_sep == '\\' {
        s.replace('\\', "/")
    } else {
        s.to_string()
    }
}

fn normalize_separators(s: &str) -> String {
    normalize_separators_with(std::path::MAIN_SEPARATOR, s)
}

/// sync 边界确认,必须在任何索引访问(含 init/迁移)之前执行:
/// 1. 当前目录须为仓库形状(含 wiki/ 或 sources/ 目录;serve 启动校验为同类
///    存在性检查,但其可受 WIKI_VAULT_ROOT 指向他处且用 exists——sync 侧
///    仅看 cwd 且要求 is_dir,更严格);
/// 2. 索引库须位于当前目录之内——相对路径经 `..` 解析后须仍在 cwd 内
///    (不得逃逸),绝对路径做分隔符/`..` 归一化 + Windows 忽略大小写的
///    前缀比较。
/// 任一不成立即整体拒绝——防止错误 cwd(恰为 vault 形状)+ WIKI_DB_PATH 指向
/// 真库时污染或误清目标索引。错误文案不含绝对路径(安全口径同 FINDING-1/4),
/// 保留排查指引。
fn confirm_sync_boundary() -> Result<(), String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("无法获取当前目录({}),已拒绝同步", e))?;
    let vault_shaped = cwd.join("wiki").is_dir() || cwd.join("sources").is_dir();
    if !vault_shaped {
        return Err(
            "当前目录不含 wiki/ 或 sources/,不是 wiki 仓库根,已拒绝同步(未打开索引库)。\
             请从仓库根目录运行,或修正启动 cwd。"
                .into(),
        );
    }
    let db_file = crate::db::db_path();
    let db_abs = Path::new(&db_file);
    let db_abs = if db_abs.is_absolute() {
        db_abs.to_path_buf()
    } else {
        cwd.join(db_abs)
    };
    if !is_path_under(&cwd, &db_abs) {
        return Err(
            "索引库不在当前目录内,无法确认其属于本仓库,已拒绝同步(未打开索引库)。\
             请从仓库根运行(索引库默认 db/wiki.db),或检查 WIKI_DB_PATH 是否误指其他仓库的索引。"
                .into(),
        );
    }
    Ok(())
}

/// 判断 path 是否位于 base 目录之下。两侧先经 `normalize_path` 解析 `..`/`.`
/// 段并统一分隔符,再做前缀比较;Windows 文件系统大小写不敏感,比较忽略大小写。
fn is_path_under(base: &Path, path: &Path) -> bool {
    let base_n = normalize_path(&base.to_string_lossy());
    let path_n = normalize_path(&path.to_string_lossy());
    let (b, p) = if cfg!(windows) {
        (base_n.to_lowercase(), path_n.to_lowercase())
    } else {
        (base_n, path_n)
    };
    p.starts_with(&format!("{}/", b))
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
        "Scan complete: {} files found, {} updated, {} pruned, {} failed, {} similar pairs (scope={})",
        report.synced, report.updated, report.pruned, report.failed, report.dedupe_report.len(), report.scope
    );
}

/// 带查重的 sync。scope="incremental" 查本次新文档,scope="full" 查全库。
/// 返回 SyncReport(同步统计 + 查重报告)。
///
/// 仓库边界确认先行(`confirm_sync_boundary`):无法确认"当前目录 + 索引库"
/// 属于同一仓库时整体拒绝(failed=-1),不 init、不迁移、不扫描、不清理。
/// 库不可用(如 schema 不兼容)时同样不 panic、不半坏:failed=-1 表示
/// 整体失败,stderr 给出可观察错误与重建指引。
pub fn run_with_scope(scope: &str) -> SyncReport {
    // 边界确认必须先于 init:错误 cwd + 外部真库的场景下,连打开 DB
    // (产生 WAL 文件)都不允许,更不允许迁移/扫描/清理。
    if let Err(msg) = confirm_sync_boundary() {
        eprintln!("[wiki-sync] 错误: {}", msg);
        return SyncReport {
            synced: 0,
            updated: 0,
            pruned: 0,
            failed: -1,
            scope: scope.into(),
            dedupe_report: vec![],
        };
    }

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

    // 边界已在函数入口确认,扫描/迁移/清理三阶段作用于同一已确认仓库。
    let pruned = prune_orphans(&conn, &on_disk);

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

/// 把库中反斜杠路径统一为正斜杠口径(与扫描入库同一口径,仅 Windows 语义下
/// 有实际替换;Unix 见 `normalize_separators`)。若正斜杠版本已存在(同一逻辑
/// 文件两条记录)则删除旧行去重,documents 与 documents_fts 两表同步。
/// 迁移后若 FTS 侧与新口径暂不对齐(如历史中断残留),由扫描阶段
/// `needs_update` 的 FTS 缺失检查在同一轮 sync 内自愈。幂等:迁移后无反斜杠行。
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
        let new = normalize_separators(&old);
        if new == old {
            // Unix 上文件名可含字面 '\',不是分隔符,原样保留
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
/// 路径口径:DB 存储与 on_disk 集合均经 `normalize_separators` 统一——
/// Windows 上为正斜杠(存量反斜杠行由 `normalize_legacy_separators` 迁移),
/// Unix 上原样保留(`\` 可为合法文件名字符)。跨平台共享同一个 DB 仍不支持:
/// 边界确认会拒绝"库不在当前目录内"的同步,平台间文件系统语义差异由
/// 各自平台的口径独立保证。
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
        // (仅 Windows 语义替换;Unix 上原样,`\` 为文件名字符)
        let normalized = normalize_separators(path);
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
        // 入库口径与 prune/迁移一致(见 normalize_separators):Windows 统一 '/',
        // Unix 原样保留('\' 为合法文件名字符)
        let path_str = normalize_separators(&relative_path.to_string_lossy());
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

#[cfg(test)]
mod boundary_unit_tests {
    use super::*;

    /// R4:分隔符归一化按平台语义——Windows 替换,Unix 原样保留
    /// ('\' 是合法文件名字符)。
    #[test]
    fn normalize_separators_follows_platform_semantics() {
        assert_eq!(
            normalize_separators_with('\\', "wiki\\a\\b.md"),
            "wiki/a/b.md",
            "Windows 语义:反斜杠是分隔符,须归一"
        );
        assert_eq!(
            normalize_separators_with('/', "wiki\\a\\b.md"),
            "wiki\\a\\b.md",
            "Unix 语义:反斜杠是文件名字符,必须原样保留"
        );
        assert_eq!(normalize_separators_with('/', "wiki/a.md"), "wiki/a.md");
    }

    /// 本平台默认入口与 MAIN_SEPARATOR 语义一致。
    #[test]
    fn normalize_separators_default_matches_platform() {
        let expected = if std::path::MAIN_SEPARATOR == '\\' {
            "x/y"
        } else {
            "x\\y"
        };
        assert_eq!(normalize_separators("x\\y"), expected);
    }

    #[test]
    fn is_path_under_basic() {
        let base = Path::new("/repo");
        assert!(is_path_under(base, Path::new("/repo/db/wiki.db")));
        assert!(is_path_under(base, Path::new("/repo/db/sub/x.db")));
        assert!(
            !is_path_under(base, Path::new("/repository/db/wiki.db")),
            "同级同名前缀不得误判为子目录"
        );
        assert!(!is_path_under(base, Path::new("/other/db/wiki.db")));
    }

    #[test]
    fn is_path_under_resolves_dot_segments() {
        let base = Path::new("/repo");
        assert!(
            !is_path_under(base, Path::new("/repo/../elsewhere/db/wiki.db")),
            "`..` 解析后逃出 base 不得算 under"
        );
        assert!(is_path_under(base, Path::new("/repo/sub/../db/wiki.db")));
    }

    #[cfg(windows)]
    #[test]
    fn is_path_under_case_insensitive_on_windows() {
        assert!(is_path_under(
            Path::new("E:\\Repo"),
            Path::new("e:\\repo\\db\\wiki.db")
        ));
    }
}
