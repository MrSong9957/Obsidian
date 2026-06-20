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

/// FTS5 检索的核心实现,返回带 bm25 score 的结果。
/// core::search、core::write(决策用)、db::search(CLI)共用此函数,
/// 避免 SQL/tokenize 逻辑重复(review Important-3)。
///
/// score 为 bm25 原始值(越小越相关,负值),写入 SearchResult.score。
pub(crate) fn search_with_scores(query: &str) -> Result<Vec<SearchResult>, WikiError> {
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
        "SELECT d.path, d.name, d.description, d.type, d.updated, d.tags,
                bm25(documents_fts) AS score
         FROM documents_fts f
         JOIN documents d ON d.path = f.path
         WHERE documents_fts MATCH ?1
         ORDER BY score
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
                score: Some(row.get(6)?),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(results)
}

/// 对外检索:返回元数据列表(score 字段填充)。
pub fn search(query: &str) -> Result<Vec<SearchResult>, WikiError> {
    search_with_scores(query)
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

    // 3. 查 FTS5 看有无相似文档(用 name 做查询)。
    //    错误必须传播:若 FTS5 损坏/锁失败,静默判 ADD 会写重复内容(review Important-2)。
    let matches = search_with_scores(&new_fm.name)?;
    let existing_docs: Vec<ExistingDoc> = matches
        .iter()
        .map(|r| ExistingDoc {
            path: r.path.clone(),
            name: r.name.clone(),
            score: r.score.unwrap_or(0.0), // 透传真 bm25,decide 内部按此排序
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
    //
    // UPDATE 路径保留旧文件的 extras(source/aliases 等扩展字段),
    // 避免 agent 更新正文时丢失溯源信息(review Important-1)。
    // 策略:从旧文件 parse 出 extras,与新 fm 的 extras 合并(新值优先,
    // MCP 路径新 fm.extras 为空,等于完整保留旧 extras)。
    let effective_fm = if status == WriteStatus::Update {
        let mut merged = new_fm.clone();
        if let Ok(old_content) = std::fs::read_to_string(&abs) {
            let (old_fm, _) = frontmatter::parse(&old_content);
            for (k, v) in old_fm.extras {
                merged.extras.entry(k).or_insert(v);
            }
        }
        merged
    } else {
        new_fm.clone()
    };
    let content = frontmatter::serialize(&effective_fm, body);
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

/// list 过滤条件。各字段可选,组合 AND 语义。
#[derive(Debug, Default, Clone)]
pub struct ListFilter {
    pub tag: Option<String>,
    pub doc_type: Option<String>,
    /// 目录前缀过滤,如 "wiki/"、"sources/dev/"
    pub dir: Option<String>,
    pub limit: Option<i64>,
}

/// 按结构化字段过滤列出文档(查 documents 表,非 FTS5)。
///
/// 与 search 区别:search 是全文语义检索(返回相关度排序),
/// list 是结构化过滤(按 tag/type/目录精确匹配,返回全部元数据含正文以外字段)。
pub fn list(filter: &ListFilter) -> Result<Vec<SearchResult>, WikiError> {
    let conn = db::init()?;
    let mut conditions: Vec<String> = vec![];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    if let Some(t) = &filter.tag {
        conditions.push("tags LIKE ?".to_string());
        params_vec.push(Box::new(format!("%{}%", t)));
    }
    if let Some(t) = &filter.doc_type {
        conditions.push("type = ?".to_string());
        params_vec.push(Box::new(t.clone()));
    }
    if let Some(d) = &filter.dir {
        conditions.push("path LIKE ?".to_string());
        params_vec.push(Box::new(format!("{}%", d)));
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    let limit = filter.limit.unwrap_or(100);
    let sql = format!(
        "SELECT path, name, description, type, updated, tags
         FROM documents {} ORDER BY updated DESC LIMIT ?",
        where_clause
    );
    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec
        .iter()
        .map(|p| p.as_ref())
        .chain(std::iter::once(&limit as &dyn rusqlite::ToSql))
        .collect();
    let results = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(SearchResult {
                path: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                doc_type: row.get(3)?,
                updated: row.get(4)?,
                tags: row.get(5)?,
                score: None,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(results)
}
