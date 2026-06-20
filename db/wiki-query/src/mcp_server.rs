//! MCP server:把 wiki-query 核心检索能力暴露为 MCP 工具,供 AI agent 调用。
//!
//! v5 重构:纯索引服务。工具集 4 个:
//!   wiki_search(FTS5 检索)/ wiki_list(结构化浏览)/
//!   wiki_sync(索引同步 + 查重报告)/ wiki_get_project_rules(规则注入)。
//! agent 用自己的文件工具读写 vault Markdown,wiki-query 只管 SQLite 索引。

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
use crate::models::ProjectRulesRequest;

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
pub struct SyncRequest {
    #[schemars(description = "查重范围:'incremental'(默认,只查本次新入库文档)或 'full'(全库两两查重)。可选。")]
    pub scope: Option<String>,
}

#[tool_router]
impl WikiServer {
    pub fn new(vault_root: String) -> Self {
        Self {
            vault_root,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "搜索 Obsidian 知识库文档(FTS5 全文检索,支持中文)。返回前 10 条匹配文档的元数据列表(路径/标题/描述/类型/标签,不含正文,需另用文件工具读全文)。用 1-2 个关键词比长句更好。当需要查找某个主题、概念、技术的相关知识时使用;也用于写入前查重。")]
    fn wiki_search(
        &self,
        Parameters(req): Parameters<SearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        match core::search(&req.query) {
            Ok(mut results) => {
                // score(bm25 负值)对 agent 是噪声,清掉。只保留已排序的顺序。
                for r in &mut results {
                    r.score = None;
                }
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".into()),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
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
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "同步 vault 的 Markdown 到 SQLite 索引,并返回同步报告 + 查重报告。agent 用文件工具写完 vault 文档后调用此工具,让索引跟上 + 检测是否有相似文档(防重复)。scope 参数:'incremental'(默认,只查本次新入库文档)或 'full'(全库查重,用户说'去重一下'时用)。查重报告只提示不自动合并,合并归 agent。")]
    fn wiki_sync(
        &self,
        Parameters(req): Parameters<SyncRequest>,
    ) -> Result<CallToolResult, McpError> {
        let scope = match req.scope.as_deref() {
            Some("full") => "full",
            _ => "incremental",
        };
        let report = crate::sync::run_with_scope(scope);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into()),
        )]))
    }

    #[tool(description = "获取可注入到当前项目根指令文件(CLAUDE.md / AGENTS.md)的 wiki-query 使用规则文本。agent 首次接入 wiki-query 时调用:拿到文本后,检查当前项目根的指令文件是否已含 '<!-- wiki-query-rules -->' 标记,未含则把规则文本(含标记)追加到文件末尾,已含则跳过(避免重复注入)。agentType 参数:'claude'(注入到 CLAUDE.md)或其他(注入到 AGENTS.md)。规则文本来自 vault 内 templates/project-rules.md。")]
    fn wiki_get_project_rules(
        &self,
        Parameters(req): Parameters<ProjectRulesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let template_path = std::path::Path::new(&self.vault_root)
            .join("templates")
            .join("project-rules.md");
        let rules = std::fs::read_to_string(&template_path).unwrap_or_else(|_| {
            format!(
                "[警告:未找到 templates/project-rules.md,vault_root={}]",
                self.vault_root
            )
        });
        let target_file = match req.agent_type.as_deref() {
            Some("claude") => "CLAUDE.md",
            _ => "AGENTS.md",
        };
        let inject_guide = format!(
            "\n\n---\n## 注入指引(给 agent)\n\
             目标文件:{}\n\
             查重标记:`<!-- wiki-query-rules -->`\n\
             流程:读当前项目根的 {},若不含上述标记,则把下方规则文本(含标记)追加到文件末尾。\
             若已含标记,跳过(避免重复注入)。\n",
            target_file, target_file
        );
        let output = format!(
            "<!-- wiki-query-rules -->\n{}\n<!-- /wiki-query-rules -->{}",
            rules, inject_guide
        );
        Ok(CallToolResult::success(vec![Content::text(output)]))
    }
}

/// 手写 ServerHandler 以提供 instructions(让 agent 在 initialize 阶段
/// 就理解仓库模型)。list_tools/call_tool 用 trait 默认实现委托 tool_router。
impl ServerHandler for WikiServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
            .with_instructions(
                "Obsidian 知识库纯索引 MCP。Markdown 是权威源,SQLite 是派生索引(可随时 rm + sync 重建)。\
                 三层结构:sources/(原始资料)/ wiki/(结构化知识)/ templates/(只读模板)。\
                 agent 用自己的文件工具读写 vault Markdown,wiki-query 只管 SQLite 索引。\
                 工具:wiki_search(FTS5 检索,返回 top10 元数据,不含正文)/ \
                 wiki_list(按 tag/type/dir 浏览)/ \
                 wiki_sync(同步索引 + 查重报告,scope=incremental/full)/ \
                 wiki_get_project_rules(获取注入到项目根的使用规则文本)。\
                 提炼工作流:读 agent 记忆 → wiki_search 查重 → 文件工具写 vault → wiki_sync → 删源。"
                    .to_string(),
            )
    }
}

/// 启动 MCP stdio server。由 main.rs 的 `serve` 子命令调用。
///
/// vault_root 优先 WIKI_VAULT_ROOT 环境变量,否则用 current_dir。
/// 启动时校验 cwd 是否为合法 vault(含 wiki/ 或 sources/ 目录),
/// 防止 MCP 客户端在错误目录拉起导致索引建在错误位置。
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
