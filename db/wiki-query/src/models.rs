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
