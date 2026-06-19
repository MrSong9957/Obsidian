use std::env;

mod db;
mod fts;
mod models;
mod sync;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: wiki-query <command> [args]");
        eprintln!("Commands:");
        eprintln!("  sync              - Sync MD files to database");
        eprintln!("  search <query>    - Search documents by metadata");
        eprintln!("  show <path>       - Show full document");
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
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}
