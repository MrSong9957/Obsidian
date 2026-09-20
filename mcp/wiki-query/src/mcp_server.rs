//! MCP server:把 wiki-query 核心检索能力暴露为 MCP 工具,供 AI agent 调用。
//!
//! v5 重构:纯索引服务。工具集 5 个:
//!   wiki_search(FTS5 检索)/ wiki_list(结构化浏览)/
//!   wiki_sync(索引同步 + 查重报告)/ wiki_get_project_rules(规则注入)/
//!   wiki_check_scope(路径范围判断)。
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
    #[schemars(description = "搜索查询(支持中文,FTS5 全文检索)。多词默认 AND(全部命中),需任一命中用大写 OR 分隔(如 'mimo OR provider')。")]
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CheckScopeRequest {
    #[schemars(description = "要检查的文件路径(如 'sources/dev/rust.md' 或 '.claude/projects/xxx/memory/file.md')。")]
    pub path: String,
    #[schemars(description = "可选:vault 根路径(如 'E:\\Files\\Obsidian')。传入后会自动去除路径中的 vault root 前缀再判断。")]
    pub vault_root: Option<String>,
}

#[tool_router]
impl WikiServer {
    pub fn new(vault_root: String) -> Self {
        Self {
            vault_root,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "搜索 Obsidian 知识库文档(FTS5 全文检索,支持中文)。返回前 10 条匹配文档的元数据列表(路径/标题/描述/类型/标签,不含正文,需另用文件工具读全文)。多词默认 AND(全部命中),需任一命中用大写 OR 分隔。当需要查找某个主题、概念、技术的相关知识时使用;也用于写入前查重。")]
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
        let report_json = serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into());
        // 提炼提醒:sync 成功仅代表索引更新,不代表提炼已完成;
        // 必须先确认目标知识写入成功,才允许删除源记忆文件。
        let hint = "\n\n[提醒] 如果本次 sync 用于记忆提炼:请先确认目标知识文件已成功写入(sources/ 或 wiki/ 下可检索到),再删除源记忆文件以避免重复提炼;sync 成功本身不代表提炼已完成。";
        Ok(CallToolResult::success(vec![
            Content::text(report_json),
            Content::text(hint.to_string()),
        ]))
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
            // 不回显 vault_root 绝对路径(安全审计 FINDING-1)
            "[警告:未找到 templates/project-rules.md,请检查 WIKI_VAULT_ROOT 配置]".to_string()
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

    #[tool(description = "判断文件路径是否在 wiki 管辖范围内(sources/ wiki/ templates/)。AI 提炼记忆前调用:在范围内 → 跳过(已是 wiki 的一部分);不在范围内 → 需要提炼。支持相对路径和绝对路径,自动处理 Windows/Unix 路径分隔符差异。")]
    fn wiki_check_scope(
        &self,
        Parameters(req): Parameters<CheckScopeRequest>,
    ) -> Result<CallToolResult, McpError> {
        // vault_root 参数缺省时回退 WIKI_VAULT_ROOT(serve.sh 恒设置):
        // 绝对路径若无根可剥离会假阴性,工具描述承诺支持绝对路径。
        let root = req
            .vault_root
            .clone()
            .or_else(|| std::env::var("WIKI_VAULT_ROOT").ok());
        let path_to_check = match &root {
            Some(r) => crate::sync::strip_vault_root(&req.path, r),
            None => req.path.clone(),
        };
        let in_scope = crate::sync::is_in_wiki_scope(&path_to_check);
        let (scope_dir, suggestion) = if in_scope {
            let dir = if path_to_check.starts_with("sources/") || path_to_check == "sources" {
                "sources"
            } else if path_to_check.starts_with("wiki/") || path_to_check == "wiki" {
                "wiki"
            } else {
                "templates"
            };
            (Some(dir.to_string()), "文件已在 wiki 范围内,不需要提炼。如果内容需要更新,直接编辑即可。".to_string())
        } else {
            (None, "文件不在 wiki 范围内,需要提炼到 sources/ 或 wiki/ 目录。".to_string())
        };
        let result = serde_json::json!({
            "in_scope": in_scope,
            "path": path_to_check,
            "scope_dir": scope_dir,
            "suggestion": suggestion,
        });
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".into()),
        )]))
    }
}

/// 手写 ServerHandler 以提供 instructions(让 agent 在 initialize 阶段
/// 就理解仓库模型)。list_tools/call_tool/get_tool 显式委托 tool_router——
/// rmcp 1.7 的 #[tool_router] 宏只生成 tool_router 字段和 Self::tool_router()
/// 构造方法,不自动生成 ServerHandler 的 list_tools 委托。若只 impl get_info,
/// list_tools 会用 trait 默认实现(返回空列表),tools/list 就是 []。
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
                 wiki_get_project_rules(获取注入到项目根的使用规则文本)/ \
                 wiki_check_scope(判断文件是否在 wiki 范围内,sources/wiki/templates)。\
                 提炼工作流:读 agent 记忆 → wiki_search 查重 → 文件工具写 vault → wiki_sync → 删源。"
                    .to_string(),
            )
    }

    /// 委托给 tool_router:返回 5 个工具的定义。
    /// 不写这个的话,trait 默认实现返回空列表,tools/list 就是 []。
    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<
        Output = Result<rmcp::model::ListToolsResult, McpError>,
    > + rmcp::service::MaybeSendFuture + '_ {
        let tools = self.tool_router.list_all();
        std::future::ready(Ok(rmcp::model::ListToolsResult {
            tools,
            ..Default::default()
        }))
    }

    /// 委托给 tool_router:按 name 调用对应工具。
    fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::CallToolResult, McpError>>
        + rmcp::service::MaybeSendFuture + '_ {
        let tool_call_context =
            rmcp::handler::server::tool::ToolCallContext::new(self, request, context);
        async move { self.tool_router.call(tool_call_context).await }
    }

    /// 委托给 tool_router:按 name 查工具定义(用于参数校验)。
    fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        self.tool_router.get(name).cloned()
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
        // 不打印 vault_root 绝对路径(安全审计 FINDING-4)
        eprintln!(
            "[wiki-mcp] 警告: 当前目录未找到 wiki/ 或 sources/ 目录。\
             可用 WIKI_VAULT_ROOT 环境变量指定正确的 vault 路径。"
        );
    }

    let service = WikiServer::new(vault_root)
        .serve(stdio())
        .await
        .inspect_err(|e| eprintln!("[wiki-mcp] serve 错误: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
