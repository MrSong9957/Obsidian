# Wiki-Query 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建 Rust 实现的 SQLite 查询工具，让 Claude Code 快速查找 Obsidian 仓库中的 MD 文档

**Architecture:** 索引与内容分离——SQLite 存储文档元数据+正文，两步查询模式（search 返回元数据列表 → show 返回完整内容），基于文件修改时间的增量同步，Git Hook 自动触发

**Tech Stack:** Rust, rusqlite (bundled SQLite), serde/serde_json (JSON 输出), chrono (时间处理)

## Global Constraints

- 启动时间 < 10ms
- 数据库路径：`db/wiki.db`（相对于项目根目录）
- 项目路径：`db/wiki-query/`
- 二进制输出：`db/wiki-query/wiki-query.exe`
- 手动解析 YAML frontmatter（无 serde_yaml 依赖）
- 使用 `LIKE '%query%'` 模糊查询，FTS5 作为未来扩展备用

---

### Task 1: 项目初始化与 Cargo 配置

**Files:**
- Create: `db/wiki-query/Cargo.toml`
- Create: `db/wiki-query/src/main.rs`

**Interfaces:**
- Produces: 可编译的 Rust 项目骨架

- [ ] **Step 1: 创建项目目录结构**

```bash
mkdir -p db/wiki-query/src
```

- [ ] **Step 2: 编写 Cargo.toml**

```toml
[package]
name = "wiki-query"
version = "0.1.0"
edition = "2021"

[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
```

- [ ] **Step 3: 编写 main.rs 骨架**

```rust
use std::env;

mod db;
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
```

- [ ] **Step 4: 验证项目可编译**

```bash
cd db/wiki-query && cargo check
```

Expected: 编译通过（可能有 warnings 关于未使用的模块）

- [ ] **Step 5: Commit**

```bash
git add db/wiki-query/
git commit -m "feat: 初始化 wiki-query Rust 项目"
```

---

### Task 2: 数据模型定义

**Files:**
- Create: `db/wiki-query/src/models.rs`

**Interfaces:**
- Produces: `Document` 结构体，供 db.rs 和 sync.rs 使用

- [ ] **Step 1: 编写 models.rs**

```rust
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Document {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub description: String,
    pub doc_type: String,
    pub updated: String,
    pub tags: String,
    pub content: String,
    pub word_count: i64,
    pub modified_at: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub description: String,
    pub doc_type: String,
    pub updated: String,
    pub tags: String,
}
```

- [ ] **Step 2: 验证编译**

```bash
cd db/wiki-query && cargo check
```

Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add db/wiki-query/src/models.rs
git commit -m "feat: 添加 Document 和 SearchResult 数据模型"
```

---

### Task 3: 数据库初始化与 Schema

**Files:**
- Create: `db/wiki-query/src/db.rs`

**Interfaces:**
- Consumes: `models::Document`, `models::SearchResult`
- Produces: `db::init()`, `db::search()`, `db::show()`

- [ ] **Step 1: 编写 db.rs 初始化逻辑**

```rust
use rusqlite::{Connection, Result};
use std::path::Path;
use crate::models::{Document, SearchResult};

const DB_PATH: &str = "db/wiki.db";

pub fn init() -> Result<Connection> {
    let db_dir = Path::new(DB_PATH).parent().unwrap();
    std::fs::create_dir_all(db_dir).ok();

    let conn = Connection::open(DB_PATH)?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY,
            path TEXT UNIQUE NOT NULL,
            name TEXT,
            description TEXT,
            type TEXT,
            updated TEXT,
            tags TEXT,
            content TEXT,
            word_count INTEGER,
            modified_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_name ON documents(name);
        CREATE INDEX IF NOT EXISTS idx_type ON documents(type);"
    )?;

    Ok(conn)
}
```

- [ ] **Step 2: 验证编译**

```bash
cd db/wiki-query && cargo check
```

Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add db/wiki-query/src/db.rs
git commit -m "feat: 添加数据库初始化与 Schema 创建"
```

---

### Task 4: 查询接口实现（search + show）

**Files:**
- Modify: `db/wiki-query/src/db.rs`

**Interfaces:**
- Consumes: `models::SearchResult`
- Produces: `db::search(query: &str)`, `db::show(path: &str)`

- [ ] **Step 1: 实现 search 函数**

```rust
pub fn search(query: &str) {
    let conn = init().expect("Failed to init database");

    let pattern = format!("%{}%", query);
    let mut stmt = conn
        .prepare(
            "SELECT path, name, description, type, updated, tags
             FROM documents
             WHERE name LIKE ?1
                OR description LIKE ?1
                OR tags LIKE ?1
                OR path LIKE ?1
             ORDER BY updated DESC
             LIMIT 20",
        )
        .expect("Failed to prepare statement");

    let results: Vec<SearchResult> = stmt
        .query_map(rusqlite::params![pattern], |row| {
            Ok(SearchResult {
                path: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                doc_type: row.get(3)?,
                updated: row.get(4)?,
                tags: row.get(5)?,
            })
        })
        .expect("Query failed")
        .filter_map(|r| r.ok())
        .collect();

    let json = serde_json::to_string_pretty(&results).unwrap();
    println!("{}", json);
}
```

- [ ] **Step 2: 实现 show 函数**

```rust
pub fn show(path: &str) {
    let conn = init().expect("Failed to init database");

    let result = conn.query_row(
        "SELECT id, path, name, description, type, updated, tags, content, word_count, modified_at
         FROM documents
         WHERE path = ?1",
        rusqlite::params![path],
        |row| {
            Ok(Document {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                doc_type: row.get(4)?,
                updated: row.get(5)?,
                tags: row.get(6)?,
                content: row.get(7)?,
                word_count: row.get(8)?,
                modified_at: row.get(9)?,
            })
        },
    );

    match result {
        Ok(doc) => {
            let json = serde_json::to_string_pretty(&doc).unwrap();
            println!("{}", json);
        }
        Err(_) => {
            eprintln!("Document not found: {}", path);
            std::process::exit(1);
        }
    }
}
```

- [ ] **Step 3: 验证编译**

```bash
cd db/wiki-query && cargo check
```

Expected: 编译通过

- [ ] **Step 4: Commit**

```bash
git add db/wiki-query/src/db.rs
git commit -m "feat: 实现 search 和 show 查询接口"
```

---

### Task 5: YAML Frontmatter 解析器

**Files:**
- Create: `db/wiki-query/src/sync.rs`

**Interfaces:**
- Produces: `sync::parse_frontmatter(content: &str) -> (Metadata, body)`

- [ ] **Step 1: 编写 frontmatter 解析逻辑**

```rust
use std::path::Path;
use chrono::Local;
use rusqlite::{Connection, params};
use crate::db::init;

#[derive(Debug)]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub doc_type: String,
    pub updated: String,
    pub tags: String,
}

pub fn parse_frontmatter(content: &str) -> (Metadata, String) {
    let mut meta = Metadata {
        name: String::new(),
        description: String::new(),
        doc_type: String::new(),
        updated: String::new(),
        tags: String::new(),
    };

    let lines: Vec<&str> = content.lines().collect();

    // 检查是否以 --- 开头
    if lines.is_empty() || lines[0].trim() != "---" {
        return (meta, content.to_string());
    }

    let mut end_idx = 0;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_idx = i;
            break;
        }
        // 解析 key: value
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            let value = line[colon_pos + 1..].trim();

            match key {
                "name" => meta.name = value.to_string(),
                "description" => meta.description = value.to_string(),
                "type" => meta.doc_type = value.to_string(),
                "updated" => meta.updated = value.to_string(),
                "tags" => meta.tags = value.to_string(),
                _ => {}
            }
        }
    }

    let body = if end_idx > 0 {
        lines[(end_idx + 1)..].join("\n")
    } else {
        content.to_string()
    };

    (meta, body)
}
```

- [ ] **Step 2: 验证编译**

```bash
cd db/wiki-query && cargo check
```

Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add db/wiki-query/src/sync.rs
git commit -m "feat: 添加 YAML frontmatter 手动解析器"
```

---

### Task 6: 文件扫描与增量同步

**Files:**
- Modify: `db/wiki-query/src/sync.rs`

**Interfaces:**
- Consumes: `parse_frontmatter()`, `db::init()`
- Produces: `sync::run()`

- [ ] **Step 1: 实现 sync::run() 函数**

```rust
pub fn run() {
    let conn = init().expect("Failed to init database");
    let root = Path::new(".");

    println!("Scanning MD files...");
    let mut count = 0;
    let mut updated = 0;

    scan_directory(root, &conn, &mut count, &mut updated);

    println!("Scan complete: {} files found, {} updated", count, updated);
}

fn scan_directory(dir: &Path, conn: &Connection, count: &mut i32, updated: &mut i32) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // 跳过隐藏目录
        if path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        if path.is_dir() {
            scan_directory(&path, conn, count, updated);
            continue;
        }

        // 只处理 .md 文件
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        *count += 1;

        let relative_path = path.strip_prefix("./").unwrap_or(&path);
        let path_str = relative_path.to_string_lossy().to_string();

        // 检查是否需要更新
        if needs_update(conn, &path_str, &path) {
            if sync_file(conn, &path_str, &path).is_ok() {
                *updated += 1;
            }
        }
    }
}

fn needs_update(conn: &Connection, path_str: &str, file_path: &Path) -> bool {
    let metadata = match std::fs::metadata(file_path) {
        Ok(m) => m,
        Err(_) => return true,
    };

    let modified = metadata.modified().unwrap();
    let modified_dt: chrono::DateTime<Local> = modified.into();
    let modified_str = modified_dt.format("%Y-%m-%d %H:%M:%S").to_string();

    let stored: String = conn
        .query_row(
            "SELECT modified_at FROM documents WHERE path = ?1",
            params![path_str],
            |row| row.get(0),
        )
        .unwrap_or_default();

    stored != modified_str
}

fn sync_file(conn: &Connection, path_str: &str, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let (meta, body) = parse_frontmatter(&content);

    let metadata = std::fs::metadata(file_path)?;
    let modified = metadata.modified().unwrap();
    let modified_dt: chrono::DateTime<Local> = modified.into();
    let modified_str = modified_dt.format("%Y-%m-%d %H:%M:%S").to_string();

    let word_count = body.split_whitespace().count() as i64;

    conn.execute(
        "INSERT INTO documents (path, name, description, type, updated, tags, content, word_count, modified_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(path) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            type = excluded.type,
            updated = excluded.updated,
            tags = excluded.tags,
            content = excluded.content,
            word_count = excluded.word_count,
            modified_at = excluded.modified_at",
        params![
            path_str,
            meta.name,
            meta.description,
            meta.doc_type,
            meta.updated,
            meta.tags,
            body,
            word_count,
            modified_str
        ],
    )?;

    Ok(())
}
```

- [ ] **Step 2: 验证编译**

```bash
cd db/wiki-query && cargo check
```

Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add db/wiki-query/src/sync.rs
git commit -m "feat: 实现增量同步逻辑"
```

---

### Task 7: 编译与功能验证

**Files:**
- Modify: 无（仅验证）

**Interfaces:**
- 验证所有功能正常工作

- [ ] **Step 1: 编译 Release 版本**

```bash
cd db/wiki-query && cargo build --release
```

Expected: 编译成功，生成 `target/release/wiki-query.exe`

- [ ] **Step 2: 复制二进制到项目目录**

```bash
cp db/wiki-query/target/release/wiki-query.exe db/wiki-query/wiki-query.exe
```

- [ ] **Step 3: 测试 sync 命令**

```bash
./db/wiki-query/wiki-query.exe sync
```

Expected: 扫描 MD 文件并输出统计

- [ ] **Step 4: 测试 search 命令**

```bash
./db/wiki-query/wiki-query.exe search "learning"
```

Expected: 返回包含 "learning" 的文档列表（JSON 格式）

- [ ] **Step 5: 测试 show 命令**

```bash
./db/wiki-query/wiki-query.exe show "sources/learning/learning-references.md"
```

Expected: 返回完整文档内容（JSON 格式）

- [ ] **Step 6: Commit**

```bash
git add db/wiki-query/
git commit -m "feat: 编译并验证 wiki-query 功能"
```

---

### Task 8: Git Hook 配置

**Files:**
- Create: `.git/hooks/post-commit`

**Interfaces:**
- 配置 Git 在 commit 后自动触发同步

- [ ] **Step 1: 创建 post-commit hook**

```bash
cat > .git/hooks/post-commit << 'EOF'
#!/bin/sh
cd "$(git rev-parse --show-toplevel)"
./db/wiki-query/wiki-query.exe sync
EOF
chmod +x .git/hooks/post-commit
```

- [ ] **Step 2: 测试 Hook**

```bash
git commit --allow-empty -m "test: verify post-commit hook"
```

Expected: 看到 sync 命令的输出

- [ ] **Step 3: Commit hook 配置说明**

```bash
git add docs/
git commit -m "docs: 添加 Git Hook 配置说明"
```

---

### Task 9: 集成测试与文档更新

**Files:**
- Create: `db/wiki-query/README.md`

**Interfaces:**
- 验证端到端流程

- [ ] **Step 1: 端到端测试**

```bash
# 1. 同步
./db/wiki-query/wiki-query.exe sync

# 2. 搜索
./db/wiki-query/wiki-query.exe search "agent"

# 3. 查看
./db/wiki-query/wiki-query.exe show "sources/agent/agent-architecture.md"
```

Expected: 所有命令正常执行，输出正确

- [ ] **Step 2: 创建项目 README**

```markdown
# wiki-query

Rust 实现的 SQLite 查询工具，用于快速查找 Obsidian 仓库中的 MD 文档。

## 命令

- `sync` - 同步 MD 文件到数据库
- `search <query>` - 搜索文档（返回 JSON）
- `show <path>` - 查看文档详情（返回 JSON）

## 构建

```bash
cd db/wiki-query
cargo build --release
```

## 使用

```bash
./db/wiki-query/wiki-query.exe sync
./db/wiki-query/wiki-query.exe search "关键词"
./db/wiki-query/wiki-query.exe show "path/to/file.md"
```
```

- [ ] **Step 3: Final Commit**

```bash
git add db/wiki-query/README.md
git commit -m "docs: 添加 wiki-query 使用文档"
```
