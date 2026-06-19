use std::path::Path;
use chrono::Local;
use rusqlite::{Connection, params};
use crate::db::init;

#[derive(Debug)]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub doc_type: String,
    pub updated: String,
    pub tags: String,
}

pub fn parse_frontmatter(content: &str) -> (Metadata, String) {
    let mut meta = Metadata {
        name: String::new(),
        description: String::new(),
        doc_type: String::new(),
        updated: String::new(),
        tags: String::new(),
    };

    let lines: Vec<&str> = content.lines().collect();

    // 检查是否以 --- 开头
    if lines.is_empty() || lines[0].trim() != "---" {
        return (meta, content.to_string());
    }

    let mut end_idx = 0;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_idx = i;
            break;
        }
        // 解析 key: value
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            let value = line[colon_pos + 1..].trim();

            match key {
                "name" => meta.name = value.to_string(),
                "description" => meta.description = value.to_string(),
                "type" => meta.doc_type = value.to_string(),
                "updated" => meta.updated = value.to_string(),
                "tags" => meta.tags = value.to_string(),
                _ => {}
            }
        }
    }

    let body = if end_idx > 0 {
        lines[(end_idx + 1)..].join("\n")
    } else {
        content.to_string()
    };

    (meta, body)
}

pub fn run() {
    let conn = init().expect("Failed to init database");
    let root = Path::new(".");

    println!("Scanning MD files...");
    let mut count = 0;
    let mut updated = 0;

    // 先收集磁盘上实际存在的路径,扫描完用于孤儿清理
    let mut on_disk: std::collections::HashSet<String> = std::collections::HashSet::new();
    scan_directory(root, &conn, &mut count, &mut updated, &mut on_disk);

    // 孤儿清理:删除 documents 和 documents_fts 中磁盘上已不存在的记录。
    // 修复 Explore review 发现的 bug —— 文件删除/重命名后旧记录会永久残留。
    let pruned = prune_orphans(&conn, &on_disk);

    println!(
        "Scan complete: {} files found, {} updated, {} pruned",
        count, updated, pruned
    );
}

/// 删除数据库里存在但磁盘上不存在的记录,返回删除条数。
/// 同时清理 documents 和 documents_fts(保持两表一致)。
fn prune_orphans(conn: &Connection, on_disk: &std::collections::HashSet<String>) -> usize {
    let db_paths: Vec<String> = {
        let mut stmt = match conn.prepare("SELECT path FROM documents") {
            Ok(s) => s,
            Err(_) => return 0,
        };
        let rows: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .ok()
            .map(|iter| iter.filter_map(|r| r.ok()).collect())
            .unwrap_or_default();
        rows
    };

    let mut pruned = 0;
    for path in &db_paths {
        if !on_disk.contains(path) {
            let _ = conn.execute("DELETE FROM documents WHERE path = ?1", params![path]);
            let _ = conn.execute("DELETE FROM documents_fts WHERE path = ?1", params![path]);
            pruned += 1;
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
            scan_directory(&path, conn, count, updated, on_disk);
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
        if needs_update(conn, &path_str, &path) {
            if sync_file(conn, &path_str, &path).is_ok() {
                *updated += 1;
            }
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
    let (meta, body) = parse_frontmatter(&content);

    let metadata = std::fs::metadata(file_path)?;
    let modified = metadata.modified().unwrap();
    let modified_dt: chrono::DateTime<Local> = modified.into();
    let modified_str = modified_dt.format("%Y-%m-%d %H:%M:%S").to_string();

    let word_count = body.split_whitespace().count() as i64;

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
}
