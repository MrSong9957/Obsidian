//! MCP server:把 wiki-query 核心能力暴露为 MCP 工具,供 AI agent 调用。
//!
//! 传输:本地 stdio(agent 拉起子进程,无云无服务器)。
//! 工具:wiki_search / wiki_read / wiki_write(Task 4.2 再加 list/patch/delete)。
//! 工具直接调 wiki_query::core 的函数,单一真相源。

use rmcp::{
    handler::server::wrapper::Parameters,
    schemars,
    service::ServiceExt,
    tool, tool_router,
    transport::stdio,
};
use serde::Deserialize;
use crate::core;
use crate::frontmatter::Frontmatter;

/// MCP server 状态:持有 vault root 路径,供工具访问。
#[derive(Clone)]
pub struct WikiServer {
    pub vault_root: String,
}

// ---- 请求结构体(schemars JsonSchema 让 MCP 暴露参数 schema 给 agent)----

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchRequest {
    #[schemars(description = "搜索查询(支持中文,FTS5 全文检索)。用 1-2 个关键词比长句更好。")]
    pub query: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReadRequest {
    #[schemars(description = "文档相对路径,如 wiki/agent.md 或 sources/dev/rust.md")]
    pub path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WriteRequest {
    #[schemars(description = "目标文档相对路径,如 wiki/new-concept.md")]
    pub path: String,
    #[schemars(description = "frontmatter name 字段(标题)")]
    pub name: String,
    #[schemars(description = "frontmatter description 字段(一句话说明)")]
    pub description: String,
    #[schemars(description = "frontmatter type 字段,须为 concept/reference/summary/contradiction 等合法类型")]
    #[serde(rename = "type")]
    pub doc_type: String,
    #[schemars(description = "frontmatter tags 字段(逗号分隔)")]
    pub tags: String,
    #[schemars(description = "正文(Markdown,frontmatter 之后的完整内容)")]
    pub body: String,
}

#[tool_router(server_handler)]
impl WikiServer {
    #[tool(description = "搜索 Obsidian 知识库文档(FTS5 全文检索,支持中文)。返回匹配文档的元数据列表(路径/标题/描述/类型/标签/相关度分数)。当需要查找某个主题、概念、技术的相关知识时使用。")]
    fn wiki_search(&self, Parameters(req): Parameters<SearchRequest>) -> String {
        match core::search(&req.query) {
            Ok(results) => serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".into()),
            Err(e) => format!("搜索失败: {}", e),
        }
    }

    #[tool(description = "读取 Obsidian 知识库中指定文档的完整内容(Markdown 原文)。参数 path 是相对路径,如 wiki/agent.md。需要看文档全文时使用。")]
    fn wiki_read(&self, Parameters(req): Parameters<ReadRequest>) -> String {
        match core::read(&self.vault_root, &req.path) {
            Ok(content) => content,
            Err(e) => format!("读取失败: {}", e),
        }
    }

    #[tool(description = "向 Obsidian 知识库写入或更新文档。写入前自动决策:新建(ADD)/更新(UPDATE)/跳过(NOOP,已存在同名文档)/冲突(CONFLICT,有相似文档需人工裁决)。会校验 frontmatter 字段完整性 + 三层目录权限(sources 可写禁删/wiki 可写/templates 只读)。需要记录新知识或更新现有文档时使用。")]
    fn wiki_write(&self, Parameters(req): Parameters<WriteRequest>) -> String {
        let fm = Frontmatter {
            name: req.name,
            description: req.description,
            doc_type: req.doc_type,
            updated: String::new(), // core::serialize 会自动填今天
            tags: req.tags,
            ..Default::default()
        };
        match core::write(&self.vault_root, &req.path, &fm, &req.body) {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".into()),
            Err(e) => format!("写入失败: {}", e),
        }
    }
}

/// 启动 MCP stdio server。由 main.rs 的 `serve` 子命令调用。
pub async fn run() -> anyhow::Result<()> {
    let vault_root = std::env::current_dir()?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("当前目录路径含非法字符"))?
        .to_string();

    let service = WikiServer { vault_root }
        .serve(stdio())
        .await
        .inspect_err(|e| eprintln!("[wiki-mcp] serve 错误: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
