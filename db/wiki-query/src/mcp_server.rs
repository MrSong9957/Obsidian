//! MCP server:把 wiki-query 核心能力暴露为 MCP 工具,供 AI agent 调用。
//!
//! 传输:本地 stdio(agent 拉起子进程,无云无服务器)。
//! 工具:wiki_search / wiki_read / wiki_write(Task 4.2 再加 list/patch/delete)。
//! 工具直接调 wiki_query::core 的函数,单一真相源。

use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::Parameters,
    },
    model::{CallToolResult, Content, Implementation, ServerCapabilities, ServerInfo},
    schemars,
    service::ServiceExt,
    tool, tool_router,
    transport::stdio,
};
use serde::Deserialize;
use crate::core;
use crate::error::WikiError;
use crate::frontmatter::Frontmatter;

/// 把 WikiError 转成 MCP 响应。
///
/// 业务可恢复错误(policy/frontmatter/路径穿越)→ is_error=true 的成功结构,
/// agent 能读 message 调整行为(如改路径重试)。
/// 系统错误(DB/IO)→ Err(McpError::internal_error),触发 JSON-RPC error。
fn err_to_result(e: WikiError) -> Result<CallToolResult, McpError> {
    match &e {
        // 业务可恢复错误:用 tool-level error(agent 能读 message 调整行为,如改路径重试)
        WikiError::PolicyViolation(_)
        | WikiError::InvalidFrontmatter(_)
        | WikiError::PathTraversal(_)
        | WikiError::NotFound(_) => Ok(CallToolResult::error(vec![Content::text(
            e.to_string(),
        )])),
        // 系统错误:返回 JSON-RPC error(DB/IO 损坏,非调用方能解决)
        WikiError::Io(_) | WikiError::Db(_) => Err(McpError::internal_error(e.to_string(), None)),
    }
}

/// MCP server 状态:持有 vault root 路径 + tool 路由,供工具访问。
#[derive(Clone)]
pub struct WikiServer {
    pub vault_root: String,
    // 由 #[tool_router] 宏生成的 list_tools/call_tool 委托方法使用,
    // rustc 的 dead_code 分析看穿不了宏,故显式 allow。
    #[allow(dead_code)]
    tool_router: ToolRouter<WikiServer>,
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListRequest {
    #[schemars(description = "按标签过滤(子串匹配),如 'agent'。可选")]
    pub tag: Option<String>,
    #[schemars(description = "按 type 过滤(精确匹配),如 'concept'。可选")]
    #[serde(rename = "type")]
    pub doc_type: Option<String>,
    #[schemars(description = "按目录前缀过滤,如 'wiki/' 或 'sources/dev/'。可选")]
    pub dir: Option<String>,
    #[schemars(description = "最大返回条数,默认 100")]
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PatchRequest {
    #[schemars(description = "目标文档相对路径,如 wiki/agent.md")]
    pub path: String,
    #[schemars(description = "要编辑的 section 标题(## 或 ### 级 Markdown heading 的文本,不含 # 号)")]
    pub heading: String,
    #[schemars(description = "该 section 的新内容(Markdown,替换原 section 内容,heading 行本身保留)")]
    pub new_content: String,
}

#[tool_router]
impl WikiServer {
    pub fn new(vault_root: String) -> Self {
        Self {
            vault_root,
            tool_router: Self::tool_router(),
        }
    }
    #[tool(description = "搜索 Obsidian 知识库文档(FTS5 全文检索,支持中文)。返回前 20 条匹配文档的元数据列表(路径/标题/描述/类型/标签/相关度分数,不含正文,需另调 wiki_read)。用 1-2 个关键词比长句更好。当需要查找某个主题、概念、技术的相关知识时使用。")]
    fn wiki_search(
        &self,
        Parameters(req): Parameters<SearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        match core::search(&req.query) {
            Ok(results) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".into()),
            )])),
            Err(e) => err_to_result(e),
        }
    }

    #[tool(description = "读取 Obsidian 知识库中指定文档的完整内容(Markdown 原文)。参数 path 是相对路径,如 wiki/agent.md 或 sources/dev/rust.md。需要看文档全文时使用。")]
    fn wiki_read(
        &self,
        Parameters(req): Parameters<ReadRequest>,
    ) -> Result<CallToolResult, McpError> {
        match core::read(&self.vault_root, &req.path) {
            Ok(content) => Ok(CallToolResult::success(vec![Content::text(content)])),
            Err(e) => err_to_result(e),
        }
    }

    #[tool(description = "向 Obsidian 知识库写入或更新文档。写入前自动决策:新建(ADD)/更新(UPDATE)/跳过(NOOP,已存在同名文档)/冲突(CONFLICT,有相似文档需人工裁决)。会校验 frontmatter 字段完整性 + 三层目录权限(sources 可写禁删/wiki 可写/templates 只读)。UPDATE 时自动保留旧文档的 source/aliases 等扩展字段(不丢失溯源信息)。需要记录新知识或更新现有文档时使用。")]
    fn wiki_write(
        &self,
        Parameters(req): Parameters<WriteRequest>,
    ) -> Result<CallToolResult, McpError> {
        let fm = Frontmatter {
            name: req.name,
            description: req.description,
            doc_type: req.doc_type,
            updated: String::new(), // core::serialize 会自动填今天
            tags: req.tags,
            ..Default::default()
        };
        match core::write(&self.vault_root, &req.path, &fm, &req.body) {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".into()),
            )])),
            Err(e) => err_to_result(e),
        }
    }

    #[tool(description = "按结构化条件列出 Obsidian 知识库文档(按标签/类型/目录过滤)。返回元数据列表(路径/标题/描述/类型/标签)。与 wiki_search 区别:search 是全文语义检索,list 是精确字段过滤(如列出某类型全部文档、某目录下所有文件)。参数均可选,不传则返回最近更新的文档。")]
    fn wiki_list(
        &self,
        Parameters(req): Parameters<ListRequest>,
    ) -> Result<CallToolResult, McpError> {
        let filter = core::ListFilter {
            tag: req.tag,
            doc_type: req.doc_type,
            dir: req.dir,
            limit: req.limit,
        };
        match core::list(&filter) {
            Ok(results) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".into()),
            )])),
            Err(e) => err_to_result(e),
        }
    }

    #[tool(description = "局部编辑文档的某个 section(按 Markdown heading 切分替换),避免整篇重写丢失未改部分。参数 path 文档路径,heading 要编辑的标题文本(不含 # 号,如 '配置'),new_content 该 section 的新内容。heading 或文件不存在返回错误。适合修改长文档的某一节,而非重写全文。")]
    fn wiki_patch(
        &self,
        Parameters(req): Parameters<PatchRequest>,
    ) -> Result<CallToolResult, McpError> {
        match core::patch(&self.vault_root, &req.path, &req.heading, &req.new_content) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "section 已更新".to_string(),
            )])),
            Err(e) => err_to_result(e),
        }
    }
}

/// 手写 ServerHandler 以提供 instructions(让 agent 在 initialize 阶段
/// 就理解仓库模型)。list_tools/call_tool 用 trait 默认实现委托 tool_router。
impl ServerHandler for WikiServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
            .with_instructions(
                "Obsidian 知识库 MCP。三层结构:sources/(原始资料,只读为主)/ \
                 wiki/(结构化知识)/ templates/(只读模板)。写入走四态决策\
                 (ADD/UPDATE/NOOP/CONFLICT),自动查重避免重复。权限:\
                 sources 可写禁删,wiki 可写,templates 只读。工具:\
                 wiki_search(检索)/ wiki_read(读全文)/ wiki_write(写入)/ \
                 wiki_list(列举)/ wiki_patch(局部编辑)。"
                    .to_string(),
            )
    }
}

/// 启动 MCP stdio server。由 main.rs 的 `serve` 子命令调用。
///
/// vault_root 优先 WIKI_VAULT_ROOT 环境变量,否则用 current_dir。
/// 启动时校验 cwd 是否为合法 vault(含 wiki/ 或 sources/ 目录),
/// 防止 MCP 客户端在错误目录拉起导致文件写到 vault 外(review Important-2)。
pub async fn run() -> anyhow::Result<()> {
    let vault_root = std::env::var("WIKI_VAULT_ROOT")
        .or_else(|_| {
            std::env::current_dir()?
                .to_str()
                .map(String::from)
                .ok_or_else(|| anyhow::anyhow!("当前目录路径含非法字符"))
        })?;

    // 启动校验:vault 应含 wiki/ 或 sources/(典型 Obsidian 知识库结构)
    let has_wiki = std::path::Path::new(&vault_root).join("wiki").exists();
    let has_sources = std::path::Path::new(&vault_root).join("sources").exists();
    if !has_wiki && !has_sources {
        eprintln!(
            "[wiki-mcp] 警告: vault_root={} 下未找到 wiki/ 或 sources/ 目录。\
             若 MCP 客户端在错误目录启动,文件会写到 vault 外。\
             可用 WIKI_VAULT_ROOT 环境变量指定正确的 vault 路径。",
            vault_root
        );
    }

    let service = WikiServer::new(vault_root)
        .serve(stdio())
        .await
        .inspect_err(|e| eprintln!("[wiki-mcp] serve 错误: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
