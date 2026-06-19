//! write 四态决策(学 Mem0 ADD/UPDATE/NOOP/CONFLICT)。
//!
//! 这是纯函数:输入"目标路径 + 新 frontmatter + 已有文档匹配 + 路径是否存在",
//! 输出决策状态。不碰文件系统、不碰数据库,易测。
//! 实际落盘在 core::write(根据决策执行 ADD/UPDATE 或返回 NOOP/CONFLICT)。

use crate::frontmatter::Frontmatter;
use crate::models::{ConflictCandidate, WriteStatus};

/// 决策输入。由 core::write 在调用前构造(查 FTS5 得 existing_matches)。
pub struct DecisionInput<'a> {
    pub target_path: &'a str,
    pub new_frontmatter: &'a Frontmatter,
    /// 用 new_frontmatter.name 做 FTS5 search 得到的已有文档(按相关度排序)
    pub existing_matches: Vec<ExistingDoc>,
    /// target_path 对应文件是否已存在于磁盘
    pub path_exists: bool,
}

/// 已有文档的精简表示(从 SearchResult 转换)。
#[derive(Debug, Clone)]
pub struct ExistingDoc {
    pub path: String,
    pub name: String,
    /// bm25 分数(越小越相关;MVP 暂未填,留字段供排序)
    pub score: f64,
}

/// 决策:返回 status + 候选列表(仅 CONFLICT 有候选)。
///
/// 决策规则(MVP,无 LLM,纯启发式):
/// 1. path 已存在 → UPDATE(覆盖语义,调用方明确给了同 path)
/// 2. 无任何已有匹配 → ADD
/// 3. 有匹配且存在 name 完全相同(大小写不敏感) → NOOP(等价已存在)
/// 4. 有匹配但 name 不同(相似主题) → CONFLICT(返回候选,不自动落盘)
pub fn decide(input: &DecisionInput) -> (WriteStatus, Option<Vec<ConflictCandidate>>) {
    // 规则 1:path 已存在 → UPDATE
    if input.path_exists {
        return (WriteStatus::Update, None);
    }

    // 规则 2:无任何已有匹配 → ADD
    if input.existing_matches.is_empty() {
        return (WriteStatus::Add, None);
    }

    // 规则 3:存在 name 完全相同(大小写不敏感) → NOOP
    let exact_match = input.existing_matches.iter().any(|d| {
        d.name.trim().to_lowercase() == input.new_frontmatter.name.trim().to_lowercase()
    });
    if exact_match {
        return (WriteStatus::Noop, None);
    }

    // 规则 4:有匹配但 name 不同 → CONFLICT
    let candidates: Vec<ConflictCandidate> = input
        .existing_matches
        .iter()
        .take(5)
        .map(|d| ConflictCandidate {
            path: d.path.clone(),
            name: d.name.clone(),
            score: d.score,
        })
        .collect();
    (WriteStatus::Conflict, Some(candidates))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fm(name: &str) -> Frontmatter {
        Frontmatter {
            name: name.into(),
            description: "d".into(),
            doc_type: "concept".into(),
            updated: "2026-06-20".into(),
            tags: "t".into(),
            ..Default::default()
        }
    }

    fn existing(path: &str, name: &str, score: f64) -> ExistingDoc {
        ExistingDoc {
            path: path.into(),
            name: name.into(),
            score,
        }
    }

    #[test]
    fn add_when_no_match_and_path_new() {
        let input = DecisionInput {
            target_path: "wiki/new.md",
            new_frontmatter: &fm("全新概念"),
            existing_matches: vec![],
            path_exists: false,
        };
        let (status, cands) = decide(&input);
        assert_eq!(status, WriteStatus::Add);
        assert!(cands.is_none());
    }

    #[test]
    fn update_when_path_exists_regardless_of_matches() {
        // path 已存在优先判 UPDATE(调用方明确指定了同 path)
        let input = DecisionInput {
            target_path: "wiki/exists.md",
            new_frontmatter: &fm("任意名"),
            existing_matches: vec![existing("wiki/other.md", "同名", -1.0)],
            path_exists: true,
        };
        let (status, _) = decide(&input);
        assert_eq!(status, WriteStatus::Update);
    }

    #[test]
    fn noop_when_exact_name_match_case_insensitive() {
        let input = DecisionInput {
            target_path: "wiki/different-path.md",
            new_frontmatter: &fm("Agent"),
            existing_matches: vec![existing("wiki/agent.md", "agent", -1.5)],
            path_exists: false,
        };
        let (status, cands) = decide(&input);
        assert_eq!(status, WriteStatus::Noop);
        assert!(cands.is_none());
    }

    #[test]
    fn conflict_when_similar_but_different_name() {
        let input = DecisionInput {
            target_path: "wiki/agent-pattern.md",
            new_frontmatter: &fm("Agent Pattern"),
            existing_matches: vec![existing("wiki/agent.md", "Agent", -2.0)],
            path_exists: false,
        };
        let (status, cands) = decide(&input);
        assert_eq!(status, WriteStatus::Conflict);
        let cands = cands.expect("conflict 应有候选");
        assert_eq!(cands.len(), 1);
        assert_eq!(cands[0].path, "wiki/agent.md");
    }

    #[test]
    fn conflict_candidates_capped_at_5() {
        // 多于 5 个匹配只返回前 5 个
        let matches: Vec<ExistingDoc> = (0..10)
            .map(|i| existing(&format!("wiki/m{}.md", i), &format!("M{}", i), -1.0))
            .collect();
        let input = DecisionInput {
            target_path: "wiki/x.md",
            new_frontmatter: &fm("Different"),
            existing_matches: matches,
            path_exists: false,
        };
        let (status, cands) = decide(&input);
        assert_eq!(status, WriteStatus::Conflict);
        assert_eq!(cands.unwrap().len(), 5);
    }

    #[test]
    fn noop_checks_trimmed_name() {
        // name 带空格也应匹配(trim 后比较)
        let input = DecisionInput {
            target_path: "wiki/x.md",
            new_frontmatter: &fm("  Agent  "),
            existing_matches: vec![existing("wiki/agent.md", "Agent", -1.0)],
            path_exists: false,
        };
        let (status, _) = decide(&input);
        assert_eq!(status, WriteStatus::Noop);
    }
}
