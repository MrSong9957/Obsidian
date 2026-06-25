use std::env;
use wiki_query::sync;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: wiki-query <command> [args]");
        eprintln!("Commands:");
        eprintln!("  sync              - Sync MD files to database (运维,git hook 用)");
        eprintln!("  serve             - Start MCP server (stdio, 本地无云)");
        eprintln!();
        eprintln!("查询/读写走 MCP(agent 调用),非命令行。");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "sync" => sync::run(),
        "serve" => {
            // tokio runtime 运行 async MCP server(阻塞直到连接关闭)
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            if let Err(e) = rt.block_on(wiki_query::mcp_server::run()) {
                eprintln!("[wiki-mcp] server 异常退出: {:?}", e);
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}
