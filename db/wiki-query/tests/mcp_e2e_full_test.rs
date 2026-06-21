//! MCP 端到端全面测试:覆盖所有 4 个工具的完整调用路径。
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
    let mut child = Command::new("./target/debug/wiki-query.exe")
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start serve");

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
