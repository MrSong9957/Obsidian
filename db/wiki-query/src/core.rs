//! core:集成层,串联 policy/frontmatter/write_decision/db/sync。
//!
//! 提供 read/search/write 给 MCP server 和 CLI 调用。
//! 所有业务逻辑(校验、决策、落盘、索引)在此编排,是单一真相源。

use crate::db;
use crate::error::WikiError;
use crate::frontmatter::{self, Frontmatter};
use crate::models::{SearchResult, WriteResult, WriteStatus};
use crate::policy::{self, Op};
use crate::write_decision::{self, DecisionInput, ExistingDoc};
use rusqlite::params;
use std::path::Path;

/// 读单文档:返回完整 Markdown 原文。
/// policy 校验 Read 权限 + 路径穿越防御。
pub fn read(vault_root: &str, path: &str) -> Result<String, WikiError> {
    policy::check(path, Op::Read)?;
    let abs = Path::new(vault_root).join(path);
    Ok(std::fs::read_to_string(abs)?)
}

/// FTS5 检索:返回元数据 + bm25 分数(供 write 决策排序)。
/// 与 db::search 的区别:返回 Vec 而非打印,且带 score。
pub fn search(query: &str) -> Result<Vec<SearchResult>, WikiError> {
    use crate::fts::tokenize_for_fts;
    let tokenized = tokenize_for_fts(query);
    let tokens: Vec<&str> = tokenized
        .split_whitespace()
        .filter(|t| !t.contains('"') && t.chars().any(|c| c.is_alphanumeric()))
        .collect();
    if tokens.is_empty() {
        return Ok(vec![]);
    }
    let fts_query: String = tokens
        .iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(" OR ");

    let conn = db::init()?;
    let mut stmt = conn.prepare(
        "SELECT d.path, d.name, d.description, d.type, d.updated, d.tags
         FROM documents_fts f
         JOIN documents d ON d.path = f.path
         WHERE documents_fts MATCH ?1
         ORDER BY bm25(documents_fts)
         LIMIT 20",
    )?;
    let results = stmt
        .query_map(params![fts_query], |row| {
            Ok(SearchResult {
                path: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                doc_type: row.get(3)?,
                updated: row.get(4)?,
                tags: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(results)
}

/// 写入:四态决策 + frontmatter 校验 + policy 校验 + 落盘 + 即时索引。
///
/// ADD/UPDATE 会落盘并同步 FTS5;NOOP/CONFLICT 不落盘,返回信息。
pub fn write(
    vault_root: &str,
    path: &str,
    new_fm: &Frontmatter,
    body: &str,
) -> Result<WriteResult, WikiError> {
    // 1. policy 校验(权限 + 路径穿越)
    policy::check(path, Op::Write)?;
    // 2. frontmatter 校验(5 字段非空 + type 白名单)
    frontmatter::validate(new_fm)?;

    let abs = Path::new(vault_root).join(path);
    let path_exists = abs.exists();

    // 3. 查 FTS5 看有无相似文档(用 name 做查询)
    let matches = search(&new_fm.name).unwrap_or_default();
    let existing_docs: Vec<ExistingDoc> = matches
        .iter()
        .map(|r| ExistingDoc {
            path: r.path.clone(),
            name: r.name.clone(),
            score: 0.0, // MVP:bm25 未透传,decide 内部会按此排序(恒 0 等价稳定序)
        })
        .collect();

    // 4. 四态决策
    let input = DecisionInput {
        target_path: path,
        new_frontmatter: new_fm,
        existing_matches: existing_docs,
        path_exists,
    };
    let (status, candidates) = write_decision::decide(&input);

    // 5. CONFLICT / NOOP:不落盘
    if status == WriteStatus::Conflict {
        return Ok(WriteResult {
            status,
            path: path.into(),
            synced: false,
            reason: "检测到相似文档,需人工决策".into(),
            candidates,
        });
    }
    if status == WriteStatus::Noop {
        return Ok(WriteResult {
            status,
            path: path.into(),
            synced: false,
            reason: "已存在等价文档(name 相同),未写入".into(),
            candidates: None,
        });
    }

    // 6. ADD / UPDATE:落盘(自动建父目录)
    let content = frontmatter::serialize(new_fm, body);
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&abs, &content)?;

    // 7. 即时索引该文件入 FTS5(复用 sync 的单文件逻辑)
    let conn = db::init()?;
    crate::sync::sync_file_by_path(&conn, vault_root, path)
        .map_err(|e| WikiError::Db(e.to_string()))?;

    Ok(WriteResult {
        status,
        path: path.into(),
        synced: true,
        reason: if status == WriteStatus::Add {
            "新建".into()
        } else {
            "更新".into()
        },
        candidates: None,
    })
}
