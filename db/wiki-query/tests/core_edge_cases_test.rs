//! core 模块边界测试:build_fts_query 边界 + list 过滤逻辑 + error 类型。
//!
//! 覆盖现有测试未触及的边界条件。

use wiki_query::core;
use wiki_query::frontmatter;
use wiki_query::fts;

// ============================================================
// build_fts_query 边界测试
// ============================================================

#[test]
fn build_fts_query_single_token() {
    let q = core::build_fts_query("rust");
    assert_eq!(q, r#""rust""#, "单 token 应直接引用");
}

#[test]
fn build_fts_query_empty_string() {
    let q = core::build_fts_query("");
    assert_eq!(q, "", "空查询应返回空字符串");
}

#[test]
fn build_fts_query_whitespace_only() {
    let q = core::build_fts_query("   ");
    assert_eq!(q, "", "纯空格查询应返回空字符串");
}

#[test]
fn build_fts_query_chinese_single() {
    let q = core::build_fts_query("记忆");
    // jieba 分词后应至少有一个 token
    assert!(!q.is_empty(), "中文单 token 不应为空");
    assert!(q.contains("记忆"), "应含原始词: {}", q);
}

#[test]
fn build_fts_query_multiple_or() {
    // 多个 OR 连接
    let q = core::build_fts_query("rust OR go OR python");
    assert!(q.contains(" OR "), "多 OR 应保持 OR 连接: {}", q);
    assert!(!q.contains(" AND "), "不应含 AND: {}", q);
}

#[test]
fn build_fts_query_mixed_chinese_english_and() {
    let q = core::build_fts_query("rust 系统");
    assert!(q.contains("AND"), "中英混合多词应默认 AND: {}", q);
}

#[test]
fn build_fts_query_or_with_chinese() {
    let q = core::build_fts_query("记忆 OR 系统");
    assert!(q.contains(" OR "), "中文 OR 应保持: {}", q);
}

#[test]
fn build_fts_query_lowercase_or_is_not_operator() {
    // 小写 or 是普通词,不当操作符 → 仍默认 AND
    let q = core::build_fts_query("this or that");
    assert!(q.contains("AND"), "小写 or 不应触发 OR 语义,实际: {}", q);
    assert!(!q.contains(" OR "), "小写 or 不应使用 OR 连接,实际: {}", q);
}

// ============================================================
// find_similar 边界测试
// ============================================================

fn sr(path: &str, score: f64) -> wiki_query::models::SearchResult {
    wiki_query::models::SearchResult {
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
fn find_similar_empty_all_docs() {
    let candidate = sr("wiki/new.md", 0.0);
    let all = vec![];
    let pairs = core::find_similar(&candidate, &all, -2.0);
    assert!(pairs.is_empty(), "空文档集应返回空");
}

#[test]
fn find_similar_only_self() {
    let candidate = sr("wiki/a.md", -5.0);
    let all = vec![sr("wiki/a.md", -5.0)];
    let pairs = core::find_similar(&candidate, &all, -2.0);
    assert!(pairs.is_empty(), "只有自身不应返回");
}

#[test]
fn find_similar_exact_threshold() {
    // score 恰好等于阈值,应入报告(≤)
    let candidate = sr("wiki/new.md", 0.0);
    let all = vec![sr("wiki/exist.md", -2.0)];
    let pairs = core::find_similar(&candidate, &all, -2.0);
    assert_eq!(pairs.len(), 1, "score == threshold 应入报告");
}

#[test]
fn find_similar_just_above_threshold() {
    // score 比阈值稍高(不够相关),不应入报告
    let candidate = sr("wiki/new.md", 0.0);
    let all = vec![sr("wiki/exist.md", -1.99)];
    let pairs = core::find_similar(&candidate, &all, -2.0);
    assert!(pairs.is_empty(), "score > threshold 不应入报告");
}

#[test]
fn find_similar_score_none_treated_as_zero() {
    // score = None 应视为 0.0
    let candidate = sr("wiki/new.md", 0.0);
    let mut doc = sr("wiki/other.md", 0.0);
    doc.score = None;
    let all = vec![doc];
    let pairs = core::find_similar(&candidate, &all, -2.0);
    // 0.0 > -2.0,不够相关
    assert!(pairs.is_empty(), "score=None(视为 0.0)不应入报告");
}

#[test]
fn find_similar_preserves_candidate_and_similar_to() {
    let candidate = sr("wiki/candidate.md", 0.0);
    let all = vec![sr("wiki/similar.md", -5.0)];
    let pairs = core::find_similar(&candidate, &all, -2.0);
    assert_eq!(pairs[0].candidate, "wiki/candidate.md");
    assert_eq!(pairs[0].similar_to, "wiki/similar.md");
    assert_eq!(pairs[0].score, -5.0);
}

// ============================================================
// 错误类型测试
// ============================================================

#[test]
fn wiki_error_display_invalid_frontmatter() {
    let e = wiki_query::error::WikiError::InvalidFrontmatter("missing name".into());
    assert!(e.to_string().contains("frontmatter"), "{}", e);
    assert!(e.to_string().contains("missing name"), "{}", e);
}

#[test]
fn wiki_error_display_path_traversal() {
    let e = wiki_query::error::WikiError::PathTraversal("../etc/passwd".into());
    assert!(e.to_string().contains("路径穿越"), "{}", e);
}

#[test]
fn wiki_error_display_policy_violation() {
    let e = wiki_query::error::WikiError::PolicyViolation("sources 禁删".into());
    assert!(e.to_string().contains("权限策略"), "{}", e);
}

#[test]
fn wiki_error_display_not_found() {
    let e = wiki_query::error::WikiError::NotFound("wiki/missing.md".into());
    assert!(e.to_string().contains("文档不存在"), "{}", e);
}

#[test]
fn wiki_error_display_io() {
    let e = wiki_query::error::WikiError::Io("permission denied".into());
    assert!(e.to_string().contains("IO 错误"), "{}", e);
}

#[test]
fn wiki_error_display_db() {
    let e = wiki_query::error::WikiError::Db("disk full".into());
    assert!(e.to_string().contains("数据库错误"), "{}", e);
}

#[test]
fn wiki_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let e: wiki_query::error::WikiError = io_err.into();
    assert!(matches!(e, wiki_query::error::WikiError::Io(_)));
}

#[test]
fn wiki_error_from_rusqlite_error() {
    let db_err = rusqlite::Error::QueryReturnedNoRows;
    let e: wiki_query::error::WikiError = db_err.into();
    assert!(matches!(e, wiki_query::error::WikiError::Db(_)));
}

#[test]
fn wiki_error_is_std_error() {
    let e = wiki_query::error::WikiError::NotFound("test".into());
    let _: &dyn std::error::Error = &e;
}

// ============================================================
// frontmatter 边界测试
// ============================================================

#[test]
fn frontmatter_parse_empty_content() {
    let (fm, body) = frontmatter::parse("");
    assert_eq!(fm.name, "");
    assert_eq!(body, "");
}

#[test]
fn frontmatter_parse_only_frontmatter_no_body() {
    let content = "---\nname: 测试\ndescription: d\ntype: concept\nupdated: 2026-06-20\ntags: t\n---";
    let (fm, body) = frontmatter::parse(content);
    assert_eq!(fm.name, "测试");
    assert_eq!(body, "", "无正文应返回空");
}

#[test]
fn frontmatter_parse_frontmatter_with_empty_values() {
    let content = "---\nname: \ndescription: \ntype: \nupdated: \ntags: \n---\n\n正文";
    let (fm, body) = frontmatter::parse(content);
    assert_eq!(fm.name, "");
    assert_eq!(fm.description, "");
    assert!(body.contains("正文"));
}

#[test]
fn frontmatter_parse_colon_in_value() {
    // value 中含冒号(如 URL)
    let content = "---\nname: 测试\ndescription: https://example.com\ntype: concept\nupdated: 2026-06-20\ntags: t\n---\n\n正文";
    let (fm, _) = frontmatter::parse(content);
    assert_eq!(fm.description, "https://example.com", "value 中的冒号应保留");
}

#[test]
fn frontmatter_parse_extra_fields_preserved() {
    let content = "---\nname: 测试\ndescription: d\ntype: concept\nupdated: 2026-06-20\ntags: t\nsource: [[ref.md]]\nconfidence: high\nstatus: draft\n---\n\n正文";
    let (fm, _) = frontmatter::parse(content);
    assert_eq!(fm.extras.get("source").unwrap(), "[[ref.md]]");
    assert_eq!(fm.extras.get("confidence").unwrap(), "high");
    assert_eq!(fm.extras.get("status").unwrap(), "draft");
}

#[test]
fn frontmatter_serialize_extras_sorted_by_key() {
    let mut extras = std::collections::BTreeMap::new();
    extras.insert("zebra".into(), "z".into());
    extras.insert("alpha".into(), "a".into());
    let fm = frontmatter::Frontmatter {
        name: "test".into(),
        description: "d".into(),
        doc_type: "concept".into(),
        updated: "2026-06-20".into(),
        tags: "t".into(),
        extras,
    };
    let md = frontmatter::serialize(&fm, "body");
    let alpha_pos = md.find("alpha").unwrap();
    let zebra_pos = md.find("zebra").unwrap();
    assert!(alpha_pos < zebra_pos, "extras 应按 key 排序");
}

#[test]
fn frontmatter_roundtrip_preserves_all_fields() {
    let mut extras = std::collections::BTreeMap::new();
    extras.insert("source".into(), "[[original.md]]".into());
    extras.insert("status".into(), "active".into());
    let fm = frontmatter::Frontmatter {
        name: "完整测试".into(),
        description: "描述含特殊字符: <>&\"'".into(),
        doc_type: "reference".into(),
        updated: "2026-06-22".into(),
        tags: "[tag1, tag2]".into(),
        extras,
    };
    let md = frontmatter::serialize(&fm, "# 标题\n\n正文内容");
    let (fm2, body) = frontmatter::parse(&md);

    assert_eq!(fm2.name, fm.name);
    assert_eq!(fm2.description, fm.description);
    assert_eq!(fm2.doc_type, fm.doc_type);
    assert_eq!(fm2.updated, fm.updated);
    assert_eq!(fm2.tags, fm.tags);
    assert_eq!(fm2.extras.get("source").unwrap(), "[[original.md]]");
    assert_eq!(fm2.extras.get("status").unwrap(), "active");
    assert!(body.contains("# 标题"));
    assert!(body.contains("正文内容"));
}

// ============================================================
// fts 分词边界测试
// ============================================================

#[test]
fn tokenize_for_fts_preserves_numbers() {
    let result = fts::tokenize_for_fts("version 2.0");
    assert!(
        result.split_whitespace().any(|w| w == "2.0" || w == "2"),
        "数字应保留: {}",
        result
    );
}

#[test]
fn tokenize_for_fts_punctuation_preserves_words() {
    // jieba cut_for_search 会把标点作为独立 token,但单词应保留
    let result = fts::tokenize_for_fts("hello, world!");
    assert!(
        result.contains("hello"),
        "hello 应保留: {}",
        result
    );
    assert!(
        result.contains("world"),
        "world 应保留: {}",
        result
    );
}

#[test]
fn tokenize_for_fts_very_long_text() {
    // 长文本不应 panic
    let long_text = "这是一个很长的文档。".repeat(1000);
    let result = fts::tokenize_for_fts(&long_text);
    assert!(!result.is_empty(), "长文本应正常分词");
}

#[test]
fn tokenize_for_fts_special_unicode() {
    // emoji 和特殊 Unicode 字符
    let result = fts::tokenize_for_fts("测试 🚀 emoji 支持");
    assert!(
        result.contains("测试"),
        "中文应正常分词: {}",
        result
    );
}

// ============================================================
// 模型序列化测试
// ============================================================

#[test]
fn search_result_score_none_not_serialized() {
    let sr = wiki_query::models::SearchResult {
        path: "test.md".into(),
        name: "test".into(),
        description: "d".into(),
        doc_type: "concept".into(),
        updated: "2026-06-22".into(),
        tags: "t".into(),
        score: None,
    };
    let json = serde_json::to_string(&sr).unwrap();
    assert!(!json.contains("score"), "score=None 不应序列化: {}", json);
}

#[test]
fn search_result_score_some_serialized() {
    let sr = wiki_query::models::SearchResult {
        path: "test.md".into(),
        name: "test".into(),
        description: "d".into(),
        doc_type: "concept".into(),
        updated: "2026-06-22".into(),
        tags: "t".into(),
        score: Some(-3.5),
    };
    let json = serde_json::to_string(&sr).unwrap();
    assert!(json.contains("score"), "score=Some 应序列化: {}", json);
    assert!(json.contains("-3.5"), "score 值应正确: {}", json);
}

#[test]
fn sync_report_serializes_correctly() {
    let report = wiki_query::models::SyncReport {
        synced: 10,
        updated: 3,
        pruned: 1,
        scope: "incremental".into(),
        dedupe_report: vec![],
    };
    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains(r#""synced":10"#));
    assert!(json.contains(r#""updated":3"#));
    assert!(json.contains(r#""pruned":1"#));
    assert!(json.contains(r#""scope":"incremental""#));
}

#[test]
fn similar_pair_serializes_correctly() {
    let pair = wiki_query::models::SimilarPair {
        candidate: "wiki/a.md".into(),
        similar_to: "wiki/b.md".into(),
        score: -3.5,
    };
    let json = serde_json::to_string(&pair).unwrap();
    assert!(json.contains("wiki/a.md"));
    assert!(json.contains("wiki/b.md"));
    assert!(json.contains("-3.5"));
}

// ============================================================
// CheckScopeRequest 反序列化测试
// ============================================================

#[test]
fn check_scope_request_deserializes() {
    let json = r#"{"path": "sources/dev/rust.md"}"#;
    let req: wiki_query::mcp_server::CheckScopeRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.path, "sources/dev/rust.md");
    assert!(req.vault_root.is_none(), "vault_root 应为 None");
}

#[test]
fn check_scope_request_with_vault_root() {
    let json = r#"{"path": ".claude/projects/xxx/memory/file.md", "vault_root": "E:\\Files\\Obsidian"}"#;
    let req: wiki_query::mcp_server::CheckScopeRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.path, ".claude/projects/xxx/memory/file.md");
    assert_eq!(req.vault_root.as_deref(), Some("E:\\Files\\Obsidian"));
}
