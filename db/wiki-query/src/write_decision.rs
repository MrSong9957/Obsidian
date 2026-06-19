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
///
/// 前置条件:new_frontmatter.name 应已通过 frontmatter::validate 校验非空。
/// 此处用 debug_assert 兜底(绕过 validate 直接构造 DecisionInput 时报警)。
pub fn decide(input: &DecisionInput) -> (WriteStatus, Option<Vec<ConflictCandidate>>) {
    debug_assert!(
        !input.new_frontmatter.name.trim().is_empty(),
        "decide: name must be validated non-empty upstream"
    );

    // 兜底排序:即使调用方未保证顺序,按 score 升序(bm25 越小越相关)后取前 5。
    // 这样 CONFLICT 候选的相关度顺序由纯函数自身保证,不依赖外部契约。
    let mut sorted = input.existing_matches.clone();
    sorted.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));

    // 规则 1:path 已存在 → UPDATE
    if input.path_exists {
        return (WriteStatus::Update, None);
    }

    // 规则 2:无任何已有匹配 → ADD
    if sorted.is_empty() {
        return (WriteStatus::Add, None);
    }

    // 规则 3:存在 name 完全相同(大小写不敏感) → NOOP
    let target_lower = input.new_frontmatter.name.trim().to_lowercase();
    let exact_match = sorted.iter().any(|d| d.name.trim().to_lowercase() == target_lower);
    if exact_match {
        return (WriteStatus::Noop, None);
    }

    // 规则 4:有匹配但 name 不同 → CONFLICT,取排序后前 5 个
    let candidates: Vec<ConflictCandidate> = sorted
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

    #[test]
    fn update_short_circuits_even_with_name_match() {
        // path_exists=true 且有同名匹配 → 仍判 UPDATE(path 优先于 name)
        // pin 住 short-circuit 语义防回归
        let input = DecisionInput {
            target_path: "wiki/agent.md",
            new_frontmatter: &fm("Agent"),
            existing_matches: vec![existing("wiki/agent.md", "Agent", -1.0)],
            path_exists: true,
        };
        let (status, cands) = decide(&input);
        assert_eq!(status, WriteStatus::Update);
        assert!(cands.is_none(), "UPDATE 不应返回候选");
    }

    #[test]
    fn decide_sorts_candidates_by_score() {
        // 验证兜底排序:输入乱序,候选应按 score 升序(最相关在前)
        let matches = vec![
            existing("wiki/least.md", "B", -0.5),   // 最不相关(score 大)
            existing("wiki/most.md", "A", -3.0),    // 最相关(score 小)
            existing("wiki/mid.md", "C", -1.5),
        ];
        let input = DecisionInput {
            target_path: "wiki/new.md",
            new_frontmatter: &fm("Different"),
            existing_matches: matches,
            path_exists: false,
        };
        let (status, cands) = decide(&input);
        assert_eq!(status, WriteStatus::Conflict);
        let cands = cands.unwrap();
        // 最相关的(score=-3.0)应排第一
        assert_eq!(cands[0].path, "wiki/most.md");
        assert_eq!(cands[1].path, "wiki/mid.md");
        assert_eq!(cands[2].path, "wiki/least.md");
    }
}
