use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub description: String,
    pub doc_type: String,
    pub updated: String,
    pub tags: String,
    /// bm25 分数(FTS5:越小越相关,负值)。供 find_similar 判断相似度。
    /// wiki_search 工具输出前会清空(对 agent 是噪声),内部排序用。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

/// 一对相似文档(查重报告的单条)。find_similar 返回 Vec<SimilarPair>。
#[derive(Debug, Clone, Serialize)]
pub struct SimilarPair {
    /// 候选文档路径(被查重的那篇)
    pub candidate: String,
    /// 与之相似的已有文档路径
    pub similar_to: String,
    /// bm25 相关度分数(越小越相关)
    pub score: f64,
}

/// wiki_sync 的返回报告。含同步统计 + 查重报告。
#[derive(Debug, Serialize)]
pub struct SyncReport {
    /// 本次扫描到的文档总数
    pub synced: i32,
    /// 本次新增/更新的文档数
    pub updated: i32,
    /// 本次清理的孤儿记录数
    pub pruned: i32,
    /// 本次同步失败的文件数(如非 UTF-8/DB 错误)。0 = 全部成功;
    /// 失败文件的索引保留旧数据,必须可从报告发现,不得静默。
    pub failed: i32,
    /// 查重范围:"incremental" 或 "full"
    pub scope: String,
    /// 查重报告:相似文档对(incremental 模式可能为空,full 模式含全库相似对)
    pub dedupe_report: Vec<SimilarPair>,
}

/// wiki_get_project_rules 的请求参数。
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProjectRulesRequest {
    /// 当前 agent 类型,决定推荐的目标文件名。
    /// "claude" → CLAUDE.md;"codex"/"cursor"/"opencode"/"mimo"/其他 → AGENTS.md。
    /// 可选,默认按 AGENTS.md。
    #[schemars(description = "当前 agent 类型:'claude'(注入到 CLAUDE.md)或其他(注入到 AGENTS.md)。可选,默认 AGENTS.md。")]
    pub agent_type: Option<String>,
}

