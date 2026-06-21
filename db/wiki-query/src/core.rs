//! core:集成层,串联 db(检索)。提供 search/list。
//!
//! v5 重构后只保留检索能力(写入/编辑归 agent 文件工具)。

use crate::db;
use crate::error::WikiError;
use crate::models::SearchResult;
use rusqlite::params;

/// 从原始查询字符串构造 FTS5 MATCH 表达式。
///
/// 流程:jieba 分词 → 过滤无效 token → 检测显式 OR 关键词 → 每个 token 加双引号 → 连接。
/// - 输入含显式 "OR" 关键词时,用 OR 连接(并移除 OR 本身)
/// - 否则默认用 AND 连接(多词必须全部命中,更精确)
/// - 返回空字符串表示无有效 token(调用方应返回空结果)
pub fn build_fts_query(input: &str) -> String {
    use crate::fts::tokenize_for_fts;
    let tokenized = tokenize_for_fts(input);
    let tokens: Vec<&str> = tokenized
        .split_whitespace()
        .filter(|t| !t.contains('"') && t.chars().any(|c| c.is_alphanumeric()))
        .collect();
    if tokens.is_empty() {
        return String::new();
    }
    // 检测是否包含显式 OR 关键词
    let has_or = tokens.iter().any(|t| *t == "OR");
    let connector = if has_or { " OR " } else { " AND " };
    // 过滤掉 OR 关键词本身(它是操作符不是搜索词)
    let search_tokens: Vec<&str> = tokens
        .iter()
        .filter(|t| **t != "OR")
        .copied()
        .collect();
    if search_tokens.is_empty() {
        return String::new();
    }
    search_tokens
        .iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(connector)
}

/// FTS5 检索的核心实现,返回带 bm25 score 的结果。
/// core::search 复用此函数。
///
/// score 为 bm25 原始值(越小越相关,负值),写入 SearchResult.score。
pub(crate) fn search_with_scores(query: &str) -> Result<Vec<SearchResult>, WikiError> {
    let fts_query = build_fts_query(query);
    if fts_query.is_empty() {
        return Ok(vec![]);
    }

    let conn = db::init()?;
    let mut stmt = conn.prepare(
        "SELECT d.path, d.name, d.description, d.type, d.updated, d.tags,
                bm25(documents_fts) AS score
         FROM documents_fts f
         JOIN documents d ON d.path = f.path
         WHERE documents_fts MATCH ?1
         ORDER BY score
         LIMIT 10",
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

/// 对单个候选文档,在 all_docs 中找相似文档(bm25 score ≤ threshold)。
/// 排除候选自身(path 相同)。返回相似对列表。
///
/// threshold 是 bm25 阈值:score 越小(越负)越相关。
/// 如 threshold=-2.0 表示 score ≤ -2.0 的才算相似。
/// sync 的 incremental/full 查重共用此函数。
pub fn find_similar(
    candidate: &SearchResult,
    all_docs: &[SearchResult],
    threshold: f64,
) -> Vec<crate::models::SimilarPair> {
    all_docs
        .iter()
        .filter(|d| d.path != candidate.path) // 排除自身
        .filter_map(|d| {
            let score = d.score.unwrap_or(0.0);
            if score <= threshold {
                Some(crate::models::SimilarPair {
                    candidate: candidate.path.clone(),
                    similar_to: d.path.clone(),
                    score,
                })
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sr(path: &str, score: f64) -> SearchResult {
        SearchResult {
            path: path.into(),
            name: String::new(),
            description: String::new(),
            doc_type: String::new(),
            updated: String::new(),
            tags: String::new(),
            score: Some(score),
        }
    }

    #[test]
    fn find_similar_returns_pair_when_score_below_threshold() {
        // score=-3.5 比阈值 -2.0 更相关(更负),应入报告
        let candidate = sr("wiki/new.md", 0.0);
        let all = vec![sr("wiki/new.md", 0.0), sr("wiki/exist.md", -3.5)];
        let pairs = find_similar(&candidate, &all, -2.0);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].candidate, "wiki/new.md");
        assert_eq!(pairs[0].similar_to, "wiki/exist.md");
    }

    #[test]
    fn find_similar_excludes_self() {
        let candidate = sr("wiki/a.md", 0.0);
        let all = vec![sr("wiki/a.md", -5.0)];
        let pairs = find_similar(&candidate, &all, -2.0);
        assert!(pairs.is_empty(), "不应把自己当相似文档");
    }

    #[test]
    fn find_similar_excludes_low_relevance() {
        // score=-1.0 不如阈值 -2.0 相关,不入报告
        let candidate = sr("wiki/new.md", 0.0);
        let all = vec![sr("wiki/new.md", 0.0), sr("wiki/other.md", -1.0)];
        let pairs = find_similar(&candidate, &all, -2.0);
        assert!(pairs.is_empty(), "相关度不够不应入报告");
    }

    #[test]
    fn find_similar_collects_multiple_pairs() {
        let candidate = sr("wiki/new.md", 0.0);
        let all = vec![
            sr("wiki/new.md", 0.0),
            sr("wiki/a.md", -3.5),
            sr("wiki/b.md", -3.0),
            sr("wiki/c.md", -0.5),
        ];
        let pairs = find_similar(&candidate, &all, -2.0);
        assert_eq!(pairs.len(), 2, "应含 a 和 b,c 不够相关");
    }
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
                doc_type: row.get(3)?,
                updated: row.get(4)?,
                tags: row.get(5)?,
                description: row.get(2)?,
                score: None,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(results)
}
