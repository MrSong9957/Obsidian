//! MCP 入口同步边界业务断言(2026-09-21 第二轮收口):
//! 通过真实 wiki_sync 工具入口同步临时文件,以检索命中证明内容入库;
//! 并保留"正常同步成功"与"被守卫拒绝"的对照——不能只检查 JSON 字段。
//!
//! 受支持启动配置证据:
//! - 仓库根 .mcp.json: {"command":"bash","args":["mcp/wiki-query/serve.sh"],"cwd":"."}
//!   → 进程 cwd = vault 根,sync 正常(本文件 T-A 同形态);
//! - serve.sh 头注释同款用法;
//! - 用户级 config.json 无 cwd、仅 WIKI_VAULT_ROOT:sync 历史上不消费该变量,
//!   非 vault cwd 下拒绝正是 R1 修复目标(错误 cwd 整库 prune 事故形态),
//!   check_scope 等只读工具不受影响。
//!
//! 验证仅用临时仓库和临时数据库,不触碰真实索引。

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

/// frontmatter description 唯一词"玄铁令",正文唯一词"屠龙刀"。
/// 两者分别证明 FTS 的 description 列与 content 列真实入库。
const DOC: &str = "---\nname: MCP验收文档\ndescription: 描述含玄铁令\ntype: concept\nupdated: 2026-09-21\ntags: e2e\n---\n\n正文独有词屠龙刀";

/// 启动 serve 子进程(指定 cwd + env),发 JSON-RPC,读全部 stdout。
fn run_serve_session(cwd: &Path, envs: &[(&str, &str)], messages: &[&str]) -> String {
    let exe = std::env::current_dir()
        .unwrap()
        .join("target/debug/wiki-query.exe");
    let mut cmd = Command::new(&exe);
    cmd.arg("serve")
        .current_dir(cwd)
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
    let timeout = Duration::from_secs(10);
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

fn handshake() -> (&'static str, &'static str) {
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    )
}

/// 直接打开 DB 查 documents 的 path 列表。
fn doc_paths(db: &Path) -> Vec<String> {
    let conn = rusqlite::Connection::open(db).unwrap();
    let mut stmt = conn
        .prepare("SELECT path FROM documents ORDER BY path")
        .unwrap();
    stmt.query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
}

/// 建仓库形状目录(wiki/ + sources/)并写入文件列表。
fn write_vault_files(repo: &Path, files: &[&str]) {
    for f in files {
        let p = repo.join(f);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, DOC).unwrap();
    }
}

/// 正常业务路径:受支持形态(cwd=vault 根,库在 cwd 内)下,经真实 wiki_sync
/// 入口同步临时文件,再经真实 wiki_search 入口以正文唯一词检索命中——
/// 证明内容真实入库,而非仅返回字段齐全。
#[test]
fn mcp_sync_indexes_temp_files_and_search_hits() {
    let vault = tempfile::tempdir().unwrap();
    let db = vault.path().join("db").join("wiki.db");
    fs::create_dir_all(db.parent().unwrap()).unwrap();
    write_vault_files(vault.path(), &["wiki/mcp.md", "sources/dev/s.md"]);

    let envs = [
        ("WIKI_DB_PATH", db.to_str().unwrap()),
        ("WIKI_VAULT_ROOT", vault.path().to_str().unwrap()),
    ];
    let (init, notif) = handshake();
    let sync_call = r#"{"jsonrpc":"2.0","id":30,"method":"tools/call","params":{"name":"wiki_sync","arguments":{}}}"#;
    // 注意:rmcp 并发处理 tool calls(响应可能乱序),同一会话内串行发送的
    // sync 与 search 之间没有顺序保证。有依赖的调用必须拆会话——这也与
    // 真实使用一致(sync 与 search 是不同时刻的工具调用)。
    let sync_out = run_serve_session(vault.path(), &envs, &[init, notif, sync_call]);

    // 同步必须是真实成功,不是被守卫拒绝
    assert!(
        sync_out.contains(r#"\"failed\": 0"#),
        "wiki_sync 必须成功完成(failed=0),输出: {}",
        &sync_out[..sync_out.len().min(1200)]
    );
    assert!(
        sync_out.contains(r#"\"synced\": 2"#),
        "两个临时文件必须被扫描入库,输出: {}",
        &sync_out[..sync_out.len().min(1200)]
    );

    // 独立会话检索:正文唯一词与描述唯一词都必须命中入库文档
    let (init2, notif2) = handshake();
    let search_body = r#"{"jsonrpc":"2.0","id":31,"method":"tools/call","params":{"name":"wiki_search","arguments":{"query":"屠龙刀"}}}"#;
    let search_desc = r#"{"jsonrpc":"2.0","id":32,"method":"tools/call","params":{"name":"wiki_search","arguments":{"query":"玄铁令"}}}"#;
    let search_out = run_serve_session(vault.path(), &envs, &[init2, notif2, search_body, search_desc]);

    // 正文词命中 → FTS content 列真实入库;描述词命中 → 元数据列入库
    assert_eq!(
        search_out.matches(r#"\"path\": \"wiki/mcp.md\""#).count(),
        2,
        "正文词与描述词两次检索都应命中同一文档,输出: {}",
        &search_out[..search_out.len().min(2500)]
    );
    assert!(
        search_out.contains("玄铁令"),
        "描述词检索结果应带回元数据,输出: {}",
        &search_out[..search_out.len().min(2500)]
    );
    // 两表落库复核(不经 MCP,直接查库)
    assert_eq!(
        doc_paths(&db),
        vec!["sources/dev/s.md".to_string(), "wiki/mcp.md".to_string()],
        "documents 必须含两个临时文件"
    );
}

/// 错误路径对照:非 vault cwd + 库指向目标仓库 → wiki_sync 明确拒绝
/// (failed=-1),目标库逻辑内容保持不变——与上一测试形成成功/拒绝对照。
#[test]
fn mcp_sync_from_non_vault_cwd_refuses_and_leaves_db_intact() {
    let vault = tempfile::tempdir().unwrap();
    let db = vault.path().join("db").join("wiki.db");
    fs::create_dir_all(db.parent().unwrap()).unwrap();
    write_vault_files(vault.path(), &["wiki/a.md", "wiki/b.md"]);
    // 先正常建库
    let envs = [("WIKI_DB_PATH", db.to_str().unwrap())];
    let (init, notif) = handshake();
    let setup_sync = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"wiki_sync","arguments":{}}}"#;
    let setup_out = run_serve_session(vault.path(), &envs, &[init, notif, setup_sync]);
    assert!(
        setup_out.contains(r#"\"failed\": 0"#),
        "前置:正常形态必须同步成功,输出: {}",
        &setup_out[..setup_out.len().min(800)]
    );
    assert_eq!(doc_paths(&db).len(), 2, "前置:库应 2 行");

    // 非 vault cwd(无 wiki//sources/),DB 指向上面建好的库
    let elsewhere = tempfile::tempdir().unwrap();
    let (init2, notif2) = handshake();
    let sync_call = r#"{"jsonrpc":"2.0","id":40,"method":"tools/call","params":{"name":"wiki_sync","arguments":{}}}"#;
    let stdout = run_serve_session(elsewhere.path(), &envs, &[init2, notif2, sync_call]);

    assert!(
        stdout.contains(r#"\"failed\": -1"#),
        "非 vault cwd 必须被守卫拒绝,输出: {}",
        &stdout[..stdout.len().min(1200)]
    );
    assert_eq!(
        doc_paths(&db),
        vec!["wiki/a.md".to_string(), "wiki/b.md".to_string()],
        "拒绝后目标库逻辑内容必须原样保留"
    );
}
