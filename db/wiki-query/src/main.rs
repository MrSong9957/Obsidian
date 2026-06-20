use std::env;
use wiki_query::{db, sync};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: wiki-query <command> [args]");
        eprintln!("Commands:");
        eprintln!("  sync              - Sync MD files to database");
        eprintln!("  search <query>    - Search documents by metadata");
        eprintln!("  show <path>       - Show full document");
        eprintln!("  serve             - Start MCP server (stdio, 本地无云)");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "sync" => sync::run(),
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: wiki-query search <query>");
                std::process::exit(1);
            }
            db::search(&args[2])
        }
        "show" => {
            if args.len() < 3 {
                eprintln!("Usage: wiki-query show <path>");
                std::process::exit(1);
            }
            db::show(&args[2])
        }
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
