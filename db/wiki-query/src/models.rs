use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Document {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub description: String,
    pub doc_type: String,
    pub updated: String,
    pub tags: String,
    pub content: String,
    pub word_count: i64,
    pub modified_at: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub description: String,
    pub doc_type: String,
    pub updated: String,
    pub tags: String,
}

/// write 四态决策的结果状态(学 Mem0 ADD/UPDATE/NOOP/CONFLICT)。
/// serde rename_all="lowercase" 保证 JSON 输出小写,对 MCP 调用方友好。
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WriteStatus {
    Add,
    Update,
    Noop,
    Conflict,
}

/// write 检测到冲突时返回的候选文档(供人工/调用方决策)。
#[derive(Debug, Clone, Serialize)]
pub struct ConflictCandidate {
    pub path: String,
    pub name: String,
    pub score: f64,
}

/// write 的返回结果。core::write 和 MCP wiki_write 都返回此类型。
#[derive(Debug, Clone, Serialize)]
pub struct WriteResult {
    pub status: WriteStatus,
    pub path: String,
    /// 是否已同步进 FTS5 索引(ADD/UPDATE 为 true,NOOP/CONFLICT 为 false)
    pub synced: bool,
    pub reason: String,
    /// 仅 CONFLICT 时有值:相似文档候选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates: Option<Vec<ConflictCandidate>>,
}
