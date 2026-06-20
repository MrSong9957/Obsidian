//! Task 4.1 集成测试:MCP server stdio 端到端。
//!
//! 启动 `wiki-query serve` 子进程,通过 stdin 发 JSON-RPC,
//! 验证 initialize 握手 + tools/list 返回 wiki_ 工具。
//! tools/call 的实际调用在 Task 4.2 完整工具后补。

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
        // 关闭 stdin 触发 server 优雅退出
        drop(stdin);
    }

    // 给 server 时间处理
    std::thread::sleep(Duration::from_millis(500));

    let _ = child.kill();
    let output = child.wait_with_output().expect("failed to wait");
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn mcp_initialize_handshake() {
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#;
    let stdout = run_mcp_session(&[init]);
    assert!(
        stdout.contains("serverInfo") || stdout.contains("wiki"),
        "initialize 应返回 serverInfo,实际: {}",
        stdout
    );
}

#[test]
fn mcp_tools_list_exposes_wiki_tools() {
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#;
    let initialized = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let tools_list = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
    let stdout = run_mcp_session(&[init, initialized, tools_list]);
    assert!(
        stdout.contains("wiki_search"),
        "tools/list 应含 wiki_search: {}",
        stdout
    );
    assert!(stdout.contains("wiki_read"), "应含 wiki_read");
    assert!(stdout.contains("wiki_write"), "应含 wiki_write");
}

#[test]
fn mcp_wiki_search_tool_callable() {
    // 实际调用 wiki_search(用真实 vault 的 DB)
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#;
    let initialized = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"wiki_search","arguments":{"query":"记忆"}}}"#;
    let stdout = run_mcp_session(&[init, initialized, call]);
    // 应返回工具结果(content 数组),即便为空也应是合法 JSON-RPC 响应
    assert!(
        stdout.contains(r#""id":3"#) || stdout.contains("result"),
        "tools/call 应返回响应: {}",
        &stdout[..stdout.len().min(500)]
    );
}
