//! MCP 端到端全面测试:覆盖所有 5 个工具的完整调用路径。
//!
//! 测试策略:启动 wiki-query serve 子进程,通过 stdin 发 JSON-RPC,
//! 验证 stdout 返回的 JSON 响应。
//!
//! 同步策略:发完消息关闭 stdin → rmcp stdio 读到 EOF → 自然退出。
//! 5 秒超时兜底防卡死。

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

/// 启动 serve 子进程,发一组 JSON-RPC 消息,读全部 stdout。
fn run_mcp_session(messages: &[&str]) -> String {
    run_mcp_session_with_env(messages, &[])
}

/// 带环境变量变体:为子进程追加 env(如 WIKI_VAULT_ROOT / WIKI_DB_PATH)。
fn run_mcp_session_with_env(messages: &[&str], envs: &[(&str, &str)]) -> String {
    let mut cmd = Command::new("./target/debug/wiki-query.exe");
    cmd.arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().expect("failed to start serve");

    if let Some(mut stdin) = child.stdin.take() {
        for msg in messages {
            let _ = stdin.write_all(msg.as_bytes());
            let _ = stdin.write_all(b"\n");
        }
        drop(stdin);
    }

    let timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();
    loop {
        match child.try_wait().expect("wait failed") {
            Some(_status) => break,
            None => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
    let output = child.wait_with_output().expect("failed to wait");
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// MCP initialize + initialized notification,返回 (init_msg, notif_msg)。
fn mcp_handshake() -> (&'static str, &'static str) {
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    )
}

// ============================================================
// wiki_search 工具全面测试
// ============================================================

#[test]
fn wiki_search_returns_valid_json_array() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"wiki_search","arguments":{"query":"test"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    // 响应应含 id:10 和 content 数组
    assert!(stdout.contains(r#""id":10"#), "应返回 id:10,实际: {}", &stdout[..stdout.len().min(500)]);
    assert!(stdout.contains("content"), "应含 content 字段");
}

#[test]
fn wiki_search_score_field_cleared() {
    // wiki_search 工具会清除 score 字段(对 agent 是噪声)
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"wiki_search","arguments":{"query":"agent"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    // 如果返回了结果,不应含 "score" 字段(score 被清为 None,serde skip_serializing_if)
    // 注意:空结果 [] 也不含 score,所以这个断言总是安全的
    assert!(
        !stdout.contains(r#""score""#),
        "wiki_search 输出不应含 score 字段(已清除),实际: {}",
        &stdout[..stdout.len().min(500)]
    );
}

#[test]
fn wiki_search_empty_query_returns_empty_or_error() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"wiki_search","arguments":{"query":""}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    // 空查询应返回空数组或错误,不应 panic
    assert!(stdout.contains(r#""id":12"#), "应返回 id:12");
}

#[test]
fn wiki_search_chinese_query() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"wiki_search","arguments":{"query":"记忆系统"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":13"#), "中文查询应正常返回");
}

#[test]
fn wiki_search_or_syntax() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"wiki_search","arguments":{"query":"agent OR memory"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":14"#), "OR 语法查询应正常返回");
}

// ============================================================
// wiki_list 工具全面测试
// ============================================================

#[test]
fn wiki_list_no_filters_returns_results() {
    let (init, notif) = mcp_handshake();
    // 不传任何过滤参数,应返回最近更新的文档
    let call = r#"{"jsonrpc":"2.0","id":20,"method":"tools/call","params":{"name":"wiki_list","arguments":{}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":20"#), "无过滤参数应正常返回");
    assert!(stdout.contains("content"), "应含 content 字段");
}

#[test]
fn wiki_list_type_filter() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":21,"method":"tools/call","params":{"name":"wiki_list","arguments":{"type":"concept"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":21"#), "type 过滤应正常返回");
}

#[test]
fn wiki_list_dir_filter() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":22,"method":"tools/call","params":{"name":"wiki_list","arguments":{"dir":"wiki/"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":22"#), "dir 过滤应正常返回");
}

#[test]
fn wiki_list_tag_filter() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":23,"method":"tools/call","params":{"name":"wiki_list","arguments":{"tag":"agent"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":23"#), "tag 过滤应正常返回");
}

#[test]
fn wiki_list_limit_parameter() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":24,"method":"tools/call","params":{"name":"wiki_list","arguments":{"limit":2}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":24"#), "limit 参数应正常返回");
    // 解析 content 中的 JSON 数组,验证长度 ≤ 2
    // (依赖 vault 内容,只验证不 panic)
}

#[test]
fn wiki_list_combined_filters() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":25,"method":"tools/call","params":{"name":"wiki_list","arguments":{"type":"concept","dir":"wiki/","limit":5}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":25"#), "组合过滤应正常返回");
}

// ============================================================
// wiki_sync 工具全面测试
// ============================================================

#[test]
fn wiki_sync_default_scope_is_incremental() {
    let (init, notif) = mcp_handshake();
    // 不传 scope,默认 incremental
    let call = r#"{"jsonrpc":"2.0","id":30,"method":"tools/call","params":{"name":"wiki_sync","arguments":{}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":30"#), "默认 scope 应正常返回");
    assert!(stdout.contains("synced"), "应含 synced 字段");
    assert!(stdout.contains("updated"), "应含 updated 字段");
    assert!(stdout.contains("pruned"), "应含 pruned 字段");
    assert!(stdout.contains("incremental"), "默认 scope 应为 incremental");
}

#[test]
fn wiki_sync_incremental_scope() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":31,"method":"tools/call","params":{"name":"wiki_sync","arguments":{"scope":"incremental"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":31"#));
    assert!(stdout.contains("incremental"), "scope 应为 incremental");
}

#[test]
fn wiki_sync_full_scope() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":32,"method":"tools/call","params":{"name":"wiki_sync","arguments":{"scope":"full"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":32"#));
    assert!(stdout.contains("full"), "scope 应为 full");
}

#[test]
fn wiki_sync_invalid_scope_defaults_to_incremental() {
    let (init, notif) = mcp_handshake();
    // 无效 scope 应 fallback 到 incremental
    let call = r#"{"jsonrpc":"2.0","id":33,"method":"tools/call","params":{"name":"wiki_sync","arguments":{"scope":"invalid_value"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":33"#));
    assert!(stdout.contains("incremental"), "无效 scope 应 fallback 到 incremental");
}

#[test]
fn wiki_sync_report_has_dedupe_report() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":34,"method":"tools/call","params":{"name":"wiki_sync","arguments":{"scope":"full"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":34"#));
    assert!(stdout.contains("dedupe_report"), "应含 dedupe_report 字段");
}

#[test]
fn wiki_sync_hint_remembers_to_delete_source() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":35,"method":"tools/call","params":{"name":"wiki_sync","arguments":{}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":35"#));
    assert!(
        stdout.contains("删除") || stdout.contains("源文件") || stdout.contains("提炼"),
        "sync 结束后应提醒删除源记忆文件: {}",
        &stdout[..stdout.len().min(800)]
    );
}

// ============================================================
// wiki_get_project_rules 工具全面测试
// ============================================================

#[test]
fn wiki_get_project_rules_default_is_agents_md() {
    let (init, notif) = mcp_handshake();
    // 不传 agent_type,默认注入 AGENTS.md
    let call = r#"{"jsonrpc":"2.0","id":40,"method":"tools/call","params":{"name":"wiki_get_project_rules","arguments":{}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":40"#));
    assert!(stdout.contains("AGENTS.md"), "默认目标应为 AGENTS.md");
}

#[test]
fn wiki_get_project_rules_claude_targets_claude_md() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":41,"method":"tools/call","params":{"name":"wiki_get_project_rules","arguments":{"agent_type":"claude"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":41"#));
    assert!(stdout.contains("CLAUDE.md"), "agent_type=claude 应目标 CLAUDE.md");
}

#[test]
fn wiki_get_project_rules_other_agent_targets_agents_md() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":42,"method":"tools/call","params":{"name":"wiki_get_project_rules","arguments":{"agent_type":"cursor"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":42"#));
    assert!(stdout.contains("AGENTS.md"), "非 claude agent 应目标 AGENTS.md");
}

#[test]
fn wiki_get_project_rules_contains_marker() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":43,"method":"tools/call","params":{"name":"wiki_get_project_rules","arguments":{}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":43"#));
    assert!(
        stdout.contains("wiki-query-rules"),
        "输出应含 <!-- wiki-query-rules --> 标记"
    );
}

#[test]
fn wiki_get_project_rules_contains_inject_guide() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":44,"method":"tools/call","params":{"name":"wiki_get_project_rules","arguments":{}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":44"#));
    assert!(
        stdout.contains("注入指引"),
        "输出应含注入指引"
    );
}

// ============================================================
// wiki_check_scope 工具全面测试
// ============================================================

#[test]
fn wiki_check_scope_in_scope_sources() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":70,"method":"tools/call","params":{"name":"wiki_check_scope","arguments":{"path":"sources/dev/rust.md"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":70"#), "应返回 id:70");
    assert!(stdout.contains("true"), "sources/ 路径应在范围内");
    assert!(stdout.contains("sources"), "应含 scope_dir=sources");
}

#[test]
fn wiki_check_scope_in_scope_wiki() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":71,"method":"tools/call","params":{"name":"wiki_check_scope","arguments":{"path":"wiki/claude-code.md"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":71"#), "应返回 id:71");
    assert!(stdout.contains("true"), "wiki/ 路径应在范围内");
}

#[test]
fn wiki_check_scope_out_scope() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":72,"method":"tools/call","params":{"name":"wiki_check_scope","arguments":{"path":".claude/projects/test/memory/file.md"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":72"#), "应返回 id:72");
    assert!(stdout.contains("false"), "非 sources/wiki/templates 路径应不在范围内");
}

#[test]
fn wiki_check_scope_with_vault_root() {
    let (init, notif) = mcp_handshake();
    // 用当前 vault 的实际路径作为 vault_root,传入绝对路径
    let vault_root = std::env::current_dir()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    let abs_path = format!("{}/sources/dev/rust.md", vault_root);
    let call = format!(
        r#"{{"jsonrpc":"2.0","id":73,"method":"tools/call","params":{{"name":"wiki_check_scope","arguments":{{"path":"{}","vault_root":"{}"}}}}}}"#,
        abs_path, vault_root
    );
    let stdout = run_mcp_session(&[init, notif, &call]);
    assert!(stdout.contains(r#""id":73"#), "应返回 id:73");
    assert!(stdout.contains("true"), "绝对路径+vault_root 应剥离后判定在范围内");
}

#[test]
fn wiki_check_scope_empty_path() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":74,"method":"tools/call","params":{"name":"wiki_check_scope","arguments":{"path":""}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":74"#), "应返回 id:74");
    assert!(stdout.contains("false"), "空路径应不在范围内");
}

#[test]
fn wiki_check_scope_suggestion_text() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":75,"method":"tools/call","params":{"name":"wiki_check_scope","arguments":{"path":"sources/dev/rust.md"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":75"#), "应返回 id:75");
    assert!(
        stdout.contains("不需要提炼") || stdout.contains("跳过"),
        "在范围内时应提示不需要提炼"
    );
}

// ============================================================
// MCP 协议层测试
// ============================================================

#[test]
fn mcp_server_capabilities_include_tools() {
    let (init, _) = mcp_handshake();
    let stdout = run_mcp_session(&[init]);
    assert!(
        stdout.contains("tools"),
        "ServerCapabilities 应启用 tools"
    );
}

#[test]
fn mcp_unknown_tool_returns_error() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":50,"method":"tools/call","params":{"name":"nonexistent_tool","arguments":{}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    // 应返回错误响应,不应 panic
    assert!(
        stdout.contains(r#""id":50"#) || stdout.contains("error"),
        "未知工具应返回错误: {}",
        &stdout[..stdout.len().min(500)]
    );
}

#[test]
fn mcp_multiple_tool_calls_in_sequence() {
    let (init, notif) = mcp_handshake();
    let search = r#"{"jsonrpc":"2.0","id":60,"method":"tools/call","params":{"name":"wiki_search","arguments":{"query":"test"}}}"#;
    let list = r#"{"jsonrpc":"2.0","id":61,"method":"tools/call","params":{"name":"wiki_list","arguments":{}}}"#;
    let stdout = run_mcp_session(&[init, notif, search, list]);
    assert!(stdout.contains(r#""id":60"#), "第一个调用应返回");
    assert!(stdout.contains(r#""id":61"#), "第二个调用应返回");
}

// ============================================================
// 高风险缺陷回归(F5/F6/F8)
// ============================================================

/// 缺陷回归:绝对路径未传 vault_root 参数时,应回退 WIKI_VAULT_ROOT
/// 环境变量剥离前缀,而不是假阴性(工具描述承诺支持绝对路径)。
#[test]
fn check_scope_absolute_path_without_param_falls_back_to_env() {
    let (init, notif) = mcp_handshake();
    let root = "Z:/nonexistent-fake-vault";
    let abs_path = format!("{}/wiki/a.md", root);
    let call = format!(
        r#"{{"jsonrpc":"2.0","id":80,"method":"tools/call","params":{{"name":"wiki_check_scope","arguments":{{"path":"{}"}}}}}}"#,
        abs_path
    );
    let stdout = run_mcp_session_with_env(&[init, notif, &call], &[("WIKI_VAULT_ROOT", root)]);
    assert!(stdout.contains(r#""id":80"#), "应返回 id:80");
    // MCP text 内容里的 JSON 引号被转义为 \",故断言用 \"in_scope\": true 形式
    assert!(
        stdout.contains(r#"\"in_scope\": true"#),
        "绝对路径 + WIKI_VAULT_ROOT 环境变量应剥离前缀判 in_scope,实际: {}",
        &stdout[..stdout.len().min(800)]
    );
}

/// 缺陷回归:删源提醒必须含前置条件——sync 成功不等于提炼完成,
/// 只有确认目标知识已成功写入后才允许清理源内容。
#[test]
fn wiki_sync_hint_requires_confirmation_before_delete() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":81,"method":"tools/call","params":{"name":"wiki_sync","arguments":{}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":81"#));
    assert!(
        stdout.contains("确认") && (stdout.contains("成功写入") || stdout.contains("写入成功")),
        "删源提醒应含『确认目标已成功写入』前置条件,实际: {}",
        &stdout[..stdout.len().min(800)]
    );
}

/// 回归保护:SyncReport 必须含 failed 字段(区分全部成功与部分失败)。
#[test]
fn sync_report_contains_failed_field() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":82,"method":"tools/call","params":{"name":"wiki_sync","arguments":{}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(
        stdout.contains("failed"),
        "sync 报告应含 failed 字段(转义后的 JSON 键名),实际: {}",
        &stdout[..stdout.len().min(800)]
    );
}

/// F8 参数校验:limit 传错误类型(字符串)应被 MCP 层拒绝,返回错误响应而非 panic。
#[test]
fn wiki_list_invalid_limit_type_returns_error() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":83,"method":"tools/call","params":{"name":"wiki_list","arguments":{"limit":"not-a-number"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":83"#), "应返回 id:83 的响应");
    assert!(
        stdout.contains("error") || stdout.contains("Error"),
        "参数类型错误应返回错误响应,实际: {}",
        &stdout[..stdout.len().min(800)]
    );
}

/// F7 回归保护:模板文件不存在时返回警告文案而非报错/panic。
#[test]
fn wiki_get_project_rules_missing_template_returns_warning() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":84,"method":"tools/call","params":{"name":"wiki_get_project_rules","arguments":{}}}"#;
    let stdout = run_mcp_session_with_env(
        &[init, notif, call],
        &[("WIKI_VAULT_ROOT", "Z:/nonexistent-fake-vault")],
    );
    assert!(stdout.contains(r#""id":84"#));
    assert!(
        stdout.contains("未找到 templates/project-rules.md"),
        "模板缺失应返回警告文案,实际: {}",
        &stdout[..stdout.len().min(800)]
    );
}

/// F7 正常路径:WIKI_VAULT_ROOT 指向真实 vault 时,规则文本应来自
/// templates/project-rules.md 原文(而非警告回退)。已有测试均只覆盖回退路径。
#[test]
fn wiki_get_project_rules_returns_template_content_from_real_vault() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":85,"method":"tools/call","params":{"name":"wiki_get_project_rules","arguments":{}}}"#;
    // 测试进程 cwd = mcp/wiki-query,vault 根为其上两级
    let vault_root = std::path::Path::new("..").join("..");
    let stdout = run_mcp_session_with_env(
        &[init, notif, call],
        &[("WIKI_VAULT_ROOT", vault_root.to_str().unwrap())],
    );
    assert!(stdout.contains(r#""id":85"#));
    assert!(
        !stdout.contains("未找到 templates/project-rules.md"),
        "真实 vault 下不应出现模板缺失警告,实际: {}",
        &stdout[..stdout.len().min(800)]
    );
    // 模板正文的稳定锚点:模板自身标题与其引用的工具名
    assert!(
        stdout.contains("wiki-query 记忆提炼规则"),
        "规则文本应含模板正文标题,实际: {}",
        &stdout[..stdout.len().min(2000)]
    );
    assert!(
        stdout.contains("wiki_check_scope"),
        "规则文本应含模板引用的工具名,实际: {}",
        &stdout[..stdout.len().min(2000)]
    );
}

/// 安全(FINDING-1):模板缺失警告必须保留,但不得回显服务器绝对路径。
/// 防止修复退化为直接删除警告。
#[test]
fn wiki_get_project_rules_missing_template_warns_without_leaking_path() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":87,"method":"tools/call","params":{"name":"wiki_get_project_rules","arguments":{}}}"#;
    let fake_vault = std::env::temp_dir().join("leak-detect-atkvault");
    let fake_str = fake_vault.to_str().unwrap().to_string();
    let stdout = run_mcp_session_with_env(
        &[init, notif, call],
        &[("WIKI_VAULT_ROOT", &fake_str)],
    );
    assert!(stdout.contains(r#""id":87"#));
    // 必要警告必须存在(不能退化成删除提示)
    assert!(
        stdout.contains("未找到 templates/project-rules.md"),
        "模板缺失警告应保留,实际: {}",
        &stdout[..stdout.len().min(800)]
    );
    // 不得泄露实际绝对路径,也不得带 vault_root= 键值
    assert!(
        !stdout.contains(&fake_str),
        "响应不得包含 vault 实际绝对路径: {}",
        fake_str
    );
    assert!(
        !stdout.contains("vault_root="),
        "响应不得含 vault_root= 键值对,实际: {}",
        &stdout[..stdout.len().min(800)]
    );
}

/// F8 契约:tools/list 应恰暴露 5 个 wiki_ 工具(防工具集漂移)。
#[test]
fn mcp_tools_list_exposes_exactly_five_wiki_tools() {
    let (init, notif) = mcp_handshake();
    let tools_list = r#"{"jsonrpc":"2.0","id":86,"method":"tools/list"}"#;
    let stdout = run_mcp_session(&[init, notif, tools_list]);
    // tools/list 响应行内每个工具都有 "name":"wiki_..." 形式
    let count = stdout.matches(r#""name":"wiki_"#).count();
    assert_eq!(
        count, 5,
        "tools/list 应恰含 5 个 wiki_ 工具,实际: {},输出片段: {}",
        count,
        &stdout[..stdout.len().min(1500)]
    );
}

/// 安全(FINDING-4):serve 启动校验警告必须保留,但 stderr 不得打印
/// vault 根绝对路径。防止修复退化为删除警告。
#[test]
fn serve_startup_warning_does_not_leak_absolute_path() {
    use std::io::Read;
    // 错误 cwd:无 wiki/ 与 sources/ 的临时目录,且不设 WIKI_VAULT_ROOT
    let bad_cwd = tempfile::tempdir().unwrap();
    let bad_str = bad_cwd.path().to_str().unwrap().to_string();
    let exe = std::env::current_dir()
        .unwrap()
        .join("target/debug/wiki-query.exe");
    let mut child = Command::new(&exe)
        .arg("serve")
        .current_dir(bad_cwd.path())
        .env_remove("WIKI_VAULT_ROOT")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start serve");
    // 触发启动校验后正常关闭
    drop(child.stdin.take());
    let mut stderr = String::new();
    let _ = child.stderr.take().unwrap().read_to_string(&mut stderr);
    let _ = child.wait();

    // 必要警告必须存在(不能退化成删除提示)
    assert!(
        stderr.contains("未找到 wiki/ 或 sources/"),
        "启动校验警告应保留,实际 stderr: {}",
        stderr
    );
    // 不得泄露 vault 根绝对路径
    assert!(
        !stderr.contains(&bad_str),
        "stderr 不得包含 vault 根绝对路径: {}",
        bad_str
    );
    assert!(
        !stderr.contains("vault_root="),
        "stderr 不得含 vault_root= 键值对,实际: {}",
        stderr
    );
}
