# Wiki MCP 记忆系统实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 wiki-query.exe 从查询工具升级为 MCP 记忆系统中枢,让 AI agent 通过 MCP 读写 Obsidian 知识库。

**Architecture:** Markdown 权威源 + SQLite FTS5 派生索引 + Rust 核心(单一真相源)+ MCP stdio 前端(唯一 agent 入口)。CLI 仅保留 sync 运维命令。写入采用 Mem0 四态决策(ADD/UPDATE/NOOP/conflict)。

**Tech Stack:** Rust, rusqlite (bundled SQLite), FTS5 + CJK tokenizer (jieba-rs), rmcp (官方 Rust MCP SDK v0.5.0, tokio), serde_json。

**Spec:** `docs/superpowers/specs/2026-06-19-wiki-memory-system-design.md`

---

## 文件结构

```
db/wiki-query/
├── Cargo.toml                 # Modify: 加 rmcp/tokio/jieba-rs/tempfile 依赖
├── src/
│   ├── main.rs                # Modify: 加 serve 子命令(dispatch)
│   ├── lib.rs                 # Create: 核心库入口(re-export 公共 API)
│   ├── core.rs                # Create: 核心逻辑(search/read/write/patch/delete)
│   ├── fts.rs                 # Create: FTS5 建表/查询/CJK tokenizer 桥接
│   ├── frontmatter.rs         # Create: 从 sync.rs 抽出的 frontmatter 解析+校验
│   ├── policy.rs              # Create: path policy(sources/wiki/templates 权限)
│   ├── write_decision.rs      # Create: 四态决策(ADD/UPDATE/NOOP/conflict)
│   ├── mcp_server.rs          # Create: MCP server(rmcp #[tool] 定义)
│   ├── models.rs              # Modify: 加 WriteInput/WriteResult/ConflictCandidate
│   ├── db.rs                  # Modify: search 改 FTS5,加 fts 表初始化
│   ├── sync.rs                # Modify: 复用 frontmatter.rs,sync 时同步 FTS5
│   └── error.rs               # Create: 统一错误类型
└── tests/
    ├── frontmatter_test.rs    # frontmatter 解析/校验
    ├── write_decision_test.rs # 四态决策
    ├── fts_test.rs            # FTS5 中文检索
    ├── policy_test.rs         # path policy
    └── mcp_integration_test.rs# MCP stdio 端到端
```

**重构原则**:把 `sync.rs` 里的 frontmatter 解析抽到 `frontmatter.rs`(sync 和 write 共用)。核心逻辑进 `core.rs`,CLI(main.rs)和 MCP(mcp_server.rs)都调它。现有 `db.rs`/`sync.rs` 逻辑保留但重构为调 `core.rs`。

---

## 阶段 1:FTS5 检索升级(取代 LIKE)

> 这是独立可测试的起点,不依赖 write/MCP。先把检索质量提上去。

### Task 1.1: 加 CJK tokenizer 桥接

**Files:**
- Create: `db/wiki-query/src/fts.rs`
- Modify: `db/wiki-query/Cargo.toml`

- [ ] **Step 1: 加依赖到 Cargo.toml**

在 `db/wiki-query/Cargo.toml` 的 `[dependencies]` 末尾加:

```toml
jieba-rs = { version = "0.7", default-features = false }
```

> 注:FTS5 的 CJK 分词用 jieba-rs(纯 Rust,无外部依赖)。wangfenjin/simple 是 FTS5 tokenizer 扩展形态,jieba-rs 更易集成到 rusqlite。先验证 jieba-rs 可行,它是更稳妥的选择。

- [ ] **Step 2: 写 fts.rs 的 tokenizer 桥接(失败测试先行)**

Create `db/wiki-query/src/fts.rs`:

```rust
use jieba_rs::Jieba;
use std::sync::OnceLock;

static JIEBA: OnceLock<Jieba> = OnceLock::new();

fn jieba() -> &'static Jieba {
    JIEBA.get_or_init(Jieba::new)
}

/// 把输入文本用 jieba 分词,空格连接(FTS5 默认 whitespace tokenizer 友好)。
/// 中文 "记忆系统" → "记忆 系统";英文按空格保留。
pub fn tokenize_for_fts(text: &str) -> String {
    let words = jieba().cut(text, true);
    words.iter().map(|w| *w).collect::<Vec<_>>().join(" ")
}
```

- [ ] **Step 3: 在 main.rs 注册 module 并写单元测试验证分词**

在 `main.rs` 的 `mod` 声明区加 `mod fts;`。

在 `fts.rs` 末尾加测试:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_chinese() {
        let result = tokenize_for_fts("记忆系统设计");
        // jieba 会切成 "记忆"/"系统"/"设计",空格连接
        assert!(result.contains("记忆"));
        assert!(result.contains("系统"));
        assert!(result.contains("设计"));
    }

    #[test]
    fn tokenize_mixed() {
        let result = tokenize_for_fts("MCP 协议");
        assert!(result.contains("MCP"));
        assert!(result.contains("协议"));
    }
}
```

- [ ] **Step 4: 运行测试**

Run: `cd db/wiki-query && cargo test fts`
Expected: 2 tests PASS

- [ ] **Step 5: Commit**

```bash
git add db/wiki-query/src/fts.rs db/wiki-query/Cargo.toml db/wiki-query/src/main.rs
git commit -m "feat(wiki-query): 加 jieba-rs CJK tokenizer 桥接"
```

### Task 1.2: FTS5 虚拟表 + 索引同步

**Files:**
- Modify: `db/wiki-query/src/db.rs`(init 加 FTS5 表)
- Modify: `db/wiki-query/src/sync.rs`(sync_file 同步 FTS5)

- [ ] **Step 1: 在 db.rs 的 init() 加 FTS5 虚拟表**

在 `db.rs` 的 `init()` 函数的 `execute_batch` SQL 里,在现有 `documents` 表之后追加:

```rust
        // FTS5 外部内容表,复用 documents 正文
        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            path, name, description, type, tags, content,
            content='documents', content_rowid='rowid',
            tokenize='unicode61'  // unicode61 + 入库前 jieba 分词(见 Task 1.3)
        );
```

> 决策:FTS5 用 `unicode61` tokenizer(默认),**入库前**在 sync 时用 jieba 预分词。这避免写 FTS5 自定义 tokenizer 函数(rusqlite 注册 fts5 tokenizer 较复杂),用"预分词"策略达到同等效果。

- [ ] **Step 2: 修改 sync.rs 的 sync_file,写入 FTS5 预分词后的内容**

在 `sync.rs` 的 `sync_file` 里,把现有 INSERT 改为同时写 FTS5。在现有 `conn.execute(INSERT INTO documents...)` 之后追加 FTS5 同步:

```rust
    // 同步 FTS5 索引(对 text 字段做 jieba 预分词)
    use crate::fts::tokenize_for_fts;
    let fts_name = tokenize_for_fts(&meta.name);
    let fts_desc = tokenize_for_fts(&meta.description);
    let fts_tags = tokenize_for_fts(&meta.tags);
    let fts_content = tokenize_for_fts(&body);
    // FTS5 content='documents' 是只读外部表,不能直接 INSERT。
    // 改用普通 FTS5 表(自有内容)。见 Step 3 修正。
```

- [ ] **Step 3: 修正 —— FTS5 用自有内容表(非 content='documents')**

撤回 Task 1.2 Step 1 的 `content='documents'`。FTS5 自有内容更简单,代价是正文存两份(documents.content + documents_fts.content),但几千篇规模无妨。

修正 `db.rs` init():

```rust
        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            path, name, description, type, tags, content,
            tokenize='unicode61'
        );
```

修正 `sync.rs`,在 documents INSERT 之后追加 FTS5 upsert:

```rust
    // FTS5 自有内容表,jieba 预分词
    use crate::fts::tokenize_for_fts;
    conn.execute(
        "INSERT INTO documents_fts (path, name, description, type, tags, content)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT DO NOTHING",  // FTS5 无 ON CONFLICT UPDATE,先删后插
        params![path_str, tokenize_for_fts(&meta.name), tokenize_for_fts(&meta.description),
                meta.doc_type, tokenize_for_fts(&meta.tags), tokenize_for_fts(&body)],
    )?;
    // 先删旧再插新(FTS5 更新的标准做法)
    conn.execute("DELETE FROM documents_fts WHERE path = ?1", params![path_str])?;
    conn.execute(
        "INSERT INTO documents_fts (path, name, description, type, tags, content)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![path_str, tokenize_for_fts(&meta.name), tokenize_for_fts(&meta.description),
                meta.doc_type, tokenize_for_fts(&meta.tags), tokenize_for_fts(&body)],
    )?;
```

> 修正:Step 3 是对 Step 2 的修正,最终代码是 Step 3 版本(先 DELETE 再 INSERT)。Step 2 的 ON CONFLICT 在 FTS5 无效,所以走 delete-then-insert。

- [ ] **Step 4: 写集成测试 —— sync 后 FTS5 可查**

Create `db/wiki-query/tests/fts_test.rs`:

```rust
use std::process::Command;

#[test]
fn sync_then_fts_search_finds_chinese() {
    // 先 sync(依赖现有 wiki 文档里有中文)
    let status = Command::new("./target/debug/wiki-query.exe")
        .arg("sync")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("failed to run sync");
    assert!(status.success());

    // 注意:本测试在 search 改 FTS5 后(Task 1.3)才完整生效
    // 此处仅验证 sync 不崩溃且 FTS5 表有数据
}
```

- [ ] **Step 5: 运行测试 + 手动验证 FTS5 表有数据**

Run: `cd db/wiki-query && cargo test fts && cargo run -- sync`
Expected: sync 成功

手动验证 FTS5 表:
```bash
sqlite3 db/wiki.db "SELECT count(*) FROM documents_fts;"
```
Expected: 数字 > 0

- [ ] **Step 6: Commit**

```bash
git add db/wiki-query/src/db.rs db/wiki-query/src/sync.rs db/wiki-query/tests/fts_test.rs
git commit -m "feat(wiki-query): FTS5 虚拟表 + jieba 预分词索引同步"
```

### Task 1.3: search 命令改用 FTS5 MATCH

**Files:**
- Modify: `db/wiki-query/src/db.rs`(search 函数)

- [ ] **Step 1: 写失败测试 —— FTS5 MATCH 查询**

在 `tests/fts_test.rs` 追加:

```rust
use std::process::Command;

#[test]
fn search_returns_json_results() {
    let output = Command::new("./target/debug/wiki-query.exe")
        .args(["search", "记忆"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run search");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // 应返回 JSON 数组(可能为空,但格式正确)
    assert!(stdout.trim_start().starts_with('[') || stdout.trim_start().starts_with('{'));
}
```

- [ ] **Step 2: 运行测试确认当前(LIKE)实现仍通过**

Run: `cd db/wiki-query && cargo test search_returns_json`
Expected: PASS(LIKE 也能返回 JSON)

- [ ] **Step 3: 改 db.rs 的 search 用 FTS5 MATCH**

替换 `db.rs` 的 `search` 函数体:

```rust
pub fn search(query: &str) {
    let conn = init().expect("Failed to init database");

    // FTS5 查询:jieba 分词后用 OR 连接的 phrase 查询
    use crate::fts::tokenize_for_fts;
    let tokens: Vec<&str> = tokenize_for_fts(query).split_whitespace().collect();
    if tokens.is_empty() {
        println!("[]");
        return;
    }
    // 每个 token 加引号做 phrase,OR 连接
    let fts_query: String = tokens
        .iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(" OR ");

    let mut stmt = conn
        .prepare(
            "SELECT path, name, description, type, tags
             FROM documents_fts
             WHERE documents_fts MATCH ?1
             ORDER BY bm25(documents_fts)
             LIMIT 20",
        )
        .expect("Failed to prepare statement");

    let results: Vec<SearchResult> = stmt
        .query_map(rusqlite::params![fts_query], |row| {
            Ok(SearchResult {
                path: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                doc_type: row.get(3)?,
                updated: String::new(),  // FTS5 表无 updated,留空或 JOIN documents
                tags: row.get(4)?,
            })
        })
        .expect("Query failed")
        .filter_map(|r| r.ok())
        .collect();

    let json = serde_json::to_string_pretty(&results).unwrap();
    println!("{}", json);
}
```

> 注:FTS5 表无 `updated` 列(我们没存)。MVP 暂留空。Task 2.x 重构时从 documents JOIN 取。

- [ ] **Step 4: 运行测试验证 FTS5 检索**

Run: `cd db/wiki-query && cargo test fts && cargo run -- search "记忆"`
Expected: 返回含中文"记忆"相关文档的 JSON

- [ ] **Step 5: Commit**

```bash
git add db/wiki-query/src/db.rs db/wiki-query/tests/fts_test.rs
git commit -m "feat(wiki-query): search 改用 FTS5 MATCH + bm25 排序"
```

---

## 阶段 2:核心库重构 + frontmatter 抽取

> 把现有同步逻辑重构成可被 MCP 复用的库。这是 write 和 MCP 的地基。

### Task 2.1: 抽取 frontmatter 模块

**Files:**
- Create: `db/wiki-query/src/frontmatter.rs`
- Modify: `db/wiki-query/src/sync.rs`(复用 frontmatter.rs)
- Modify: `db/wiki-query/src/main.rs`(加 mod frontmatter)

- [ ] **Step 1: 写 frontmatter.rs(从 sync.rs 的 parse_frontmatter 迁移+加校验)**

Create `db/wiki-query/src/frontmatter.rs`:

```rust
use crate::error::WikiError;

/// frontmatter 字段集合
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Frontmatter {
    pub name: String,
    pub description: String,
    pub doc_type: String,
    pub updated: String,
    pub tags: String,
}

/// 合法 type 白名单
pub const VALID_TYPES: &[&str] = &["concept", "reference", "contradiction"];

/// 解析 frontmatter + 正文。返回 (Frontmatter, body)。
/// 无 frontmatter(不以 --- 开头)返回空 Frontmatter + 原文。
pub fn parse(content: &str) -> (Frontmatter, String) {
    let mut fm = Frontmatter::default();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0].trim() != "---" {
        return (fm, content.to_string());
    }

    let mut end_idx = 0;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_idx = i;
            break;
        }
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            let value = line[colon_pos + 1..].trim();
            match key {
                "name" => fm.name = value.to_string(),
                "description" => fm.description = value.to_string(),
                "type" => fm.doc_type = value.to_string(),
                "updated" => fm.updated = value.to_string(),
                "tags" => fm.tags = value.to_string(),
                _ => {}
            }
        }
    }

    let body = if end_idx > 0 {
        lines[(end_idx + 1)..].join("\n")
    } else {
        content.to_string()
    };

    (fm, body)
}

/// 校验 frontmatter 5 字段非空 + type 合法。失败返回错误。
pub fn validate(fm: &Frontmatter) -> Result<(), WikiError> {
    if fm.name.trim().is_empty() {
        return Err(WikiError::InvalidFrontmatter("name 不能为空".into()));
    }
    if fm.description.trim().is_empty() {
        return Err(WikiError::InvalidFrontmatter("description 不能为空".into()));
    }
    if !VALID_TYPES.contains(&fm.doc_type.as_str()) {
        return Err(WikiError::InvalidFrontmatter(format!(
            "type 必须是 {:?} 之一,实际: {}",
            VALID_TYPES, fm.doc_type
        )));
    }
    if fm.tags.trim().is_empty() {
        return Err(WikiError::InvalidFrontmatter("tags 不能为空".into()));
    }
    Ok(())
}

/// 把 Frontmatter + body 重组为完整 Markdown(frontmatter + 正文)
pub fn serialize(fm: &Frontmatter, body: &str) -> String {
    let updated = if fm.updated.is_empty() {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    } else {
        fm.updated.clone()
    };
    format!(
        "---\nname: {}\ndescription: {}\ntype: {}\nupdated: {}\ntags: {}\n---\n\n{}",
        fm.name, fm.description, fm.doc_type, updated, fm.tags, body
    )
}
```

- [ ] **Step 2: 写 error.rs(前置依赖)**

Create `db/wiki-query/src/error.rs`:

```rust
use std::fmt;

#[derive(Debug)]
pub enum WikiError {
    InvalidFrontmatter(String),
    PathTraversal(String),
    PolicyViolation(String),
    Io(String),
    Db(String),
}

impl fmt::Display for WikiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WikiError::InvalidFrontmatter(msg) => write!(f, "frontmatter 校验失败: {}", msg),
            WikiError::PathTraversal(p) => write!(f, "路径穿越拒绝: {}", p),
            WikiError::PolicyViolation(msg) => write!(f, "权限策略: {}", msg),
            WikiError::Io(msg) => write!(f, "IO 错误: {}", msg),
            WikiError::Db(msg) => write!(f, "数据库错误: {}", msg),
        }
    }
}

impl std::error::Error for WikiError {}

impl From<std::io::Error> for WikiError {
    fn from(e: std::io::Error) -> Self {
        WikiError::Io(e.to_string())
    }
}

impl From<rusqlite::Error> for WikiError {
    fn from(e: rusqlite::Error) -> Self {
        WikiError::Db(e.to_string())
    }
}
```

- [ ] **Step 3: 在 main.rs 注册 mod,改 sync.rs 复用 frontmatter.rs**

在 `main.rs` mod 区加:
```rust
mod error;
mod frontmatter;
```

在 `sync.rs` 把现有 `Metadata`/`parse_frontmatter` 删除,改为 `use crate::frontmatter::{Frontmatter, parse};`。把 `parse_frontmatter(&content)` 调用改为 `parse(&content)`,字段访问从 `meta.name` 等保持一致(Frontmatter 字段名相同)。

- [ ] **Step 4: 写 frontmatter 单元测试**

Create `db/wiki-query/tests/frontmatter_test.rs`:

```rust
use wiki_query::frontmatter::{parse, validate, serialize, Frontmatter};

#[test]
fn parse_valid_frontmatter() {
    let content = "---\nname: 测试\ndescription: 描述\ntype: concept\nupdated: 2026-06-20\ntags: [a, b]\n---\n\n正文";
    let (fm, body) = parse(content);
    assert_eq!(fm.name, "测试");
    assert_eq!(fm.doc_type, "concept");
    assert!(body.contains("正文"));
}

#[test]
fn validate_rejects_empty_name() {
    let fm = Frontmatter {
        name: "".into(),
        description: "d".into(),
        doc_type: "concept".into(),
        updated: "2026-06-20".into(),
        tags: "t".into(),
    };
    assert!(validate(&fm).is_err());
}

#[test]
fn validate_rejects_invalid_type() {
    let fm = Frontmatter {
        name: "n".into(),
        description: "d".into(),
        doc_type: "invalid".into(),
        updated: "2026-06-20".into(),
        tags: "t".into(),
    };
    assert!(validate(&fm).is_err());
}

#[test]
fn serialize_roundtrip() {
    let fm = Frontmatter {
        name: "测试".into(),
        description: "d".into(),
        doc_type: "reference".into(),
        updated: "".into(),  // 空,应自动填今天
        tags: "t".into(),
    };
    let md = serialize(&fm, "正文");
    assert!(md.starts_with("---\n"));
    assert!(md.contains("name: 测试"));
    assert!(md.contains("正文"));
}
```

- [ ] **Step 5: 加 lib.rs 暴露公共 API(error.rs/fmt 需要让测试可见)**

Create `db/wiki-query/src/lib.rs`:

```rust
pub mod frontmatter;
pub mod error;
pub mod policy;
pub mod write_decision;
pub mod core;
```

> 注:lib.rs 暴露模块供集成测试用。binary(main.rs)通过 `mod` 声明内部模块,integration test 通过 `use wiki_query::...` 访问。为此 Cargo.toml 需声明 `[lib]` + `[[bin]]`。

- [ ] **Step 6: 改 Cargo.toml 声明 lib + bin**

在 `Cargo.toml` 加:

```toml
[lib]
name = "wiki_query"
path = "src/lib.rs"

[[bin]]
name = "wiki-query"
path = "src/main.rs"
```

- [ ] **Step 7: 运行测试**

Run: `cd db/wiki-query && cargo test frontmatter`
Expected: 4 tests PASS

- [ ] **Step 8: Commit**

```bash
git add db/wiki-query/src/frontmatter.rs db/wiki-query/src/error.rs db/wiki-query/src/lib.rs db/wiki-query/src/main.rs db/wiki-query/src/sync.rs db/wiki-query/Cargo.toml db/wiki-query/tests/frontmatter_test.rs
git commit -m "refactor(wiki-query): 抽取 frontmatter 模块 + error 类型 + lib/bin 分离"
```

### Task 2.2: path policy 模块

**Files:**
- Create: `db/wiki-query/src/policy.rs`

- [ ] **Step 1: 写 policy.rs**

Create `db/wiki-query/src/policy.rs`:

```rust
use crate::error::WikiError;

/// 目录权限:sources(读+写,禁删)/ wiki(读+写+deprecated)/ templates(只读)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Read,
    Write,
    Delete,
}

/// 检查 path 相对路径的操作权限。
/// path 如 "wiki/agent.md"、"sources/dev/rust.md"。
/// 含 ".." 直接拒绝(路径穿越)。
pub fn check(path: &str, op: Op) -> Result<(), WikiError> {
    // 路径穿越防御
    for segment in path.split('/') {
        if segment == ".." {
            return Err(WikiError::PathTraversal(path.into()));
        }
    }
    if path.starts_with('/') {
        return Err(WikiError::PathTraversal(path.into()));
    }

    let is_sources = path.starts_with("sources/");
    let is_wiki = path.starts_with("wiki/");
    let is_templates = path.starts_with("templates/");

    if !is_sources && !is_wiki && !is_templates {
        return Err(WikiError::PolicyViolation(format!("路径必须在 sources/wiki/templates 下: {}", path)));
    }

    match op {
        Op::Read => Ok(()),
        Op::Write => {
            if is_templates {
                Err(WikiError::PolicyViolation("templates 只读".into()))
            } else {
                Ok(())
            }
        }
        Op::Delete => {
            if is_sources {
                Err(WikiError::PolicyViolation("sources 禁止删除".into()))
            } else if is_wiki {
                // wiki 删除实际是 deprecated 标记,允许
                Ok(())
            } else {
                Err(WikiError::PolicyViolation("templates 禁止删除".into()))
            }
        }
    }
}
```

- [ ] **Step 2: 写 policy 测试**

Create `db/wiki-query/tests/policy_test.rs`:

```rust
use wiki_query::policy::{check, Op};

#[test]
fn allows_read_everywhere() {
    assert!(check("wiki/x.md", Op::Read).is_ok());
    assert!(check("sources/x.md", Op::Read).is_ok());
    assert!(check("templates/x.md", Op::Read).is_ok());
}

#[test]
fn blocks_write_templates() {
    assert!(check("templates/x.md", Op::Write).is_err());
    assert!(check("wiki/x.md", Op::Write).is_ok());
    assert!(check("sources/x.md", Op::Write).is_ok());
}

#[test]
fn blocks_delete_sources() {
    assert!(check("sources/x.md", Op::Delete).is_err());
    assert!(check("wiki/x.md", Op::Delete).is_ok());  // wiki 允许(deprecated)
    assert!(check("templates/x.md", Op::Delete).is_err());
}

#[test]
fn blocks_path_traversal() {
    assert!(check("../etc/passwd", Op::Read).is_err());
    assert!(check("wiki/../../etc", Op::Read).is_err());
    assert!(check("/absolute", Op::Read).is_err());
}

#[test]
fn blocks_outside_vault_dirs() {
    assert!(check("random/x.md", Op::Read).is_err());
}
```

- [ ] **Step 3: 运行测试**

Run: `cd db/wiki-query && cargo test policy`
Expected: 5 tests PASS

- [ ] **Step 4: Commit**

```bash
git add db/wiki-query/src/policy.rs db/wiki-query/tests/policy_test.rs
git commit -m "feat(wiki-query): path policy 模块(sources/wiki/templates 权限矩阵)"
```

---

## 阶段 3:write 四态决策

> 核心写入逻辑。先写决策函数(纯函数,易测),再接落盘。

### Task 3.1: 四态决策纯函数

**Files:**
- Create: `db/wiki-query/src/write_decision.rs`
- Modify: `db/wiki-query/src/models.rs`(加 WriteResult/ConflictCandidate)

- [ ] **Step 1: 在 models.rs 加写入相关类型**

在 `models.rs` 追加:

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WriteStatus {
    Add,
    Update,
    Noop,
    Conflict,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConflictCandidate {
    pub path: String,
    pub name: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WriteResult {
    pub status: WriteStatus,
    pub path: String,
    pub synced: bool,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates: Option<Vec<ConflictCandidate>>,
}
```

- [ ] **Step 2: 写 write_decision.rs(纯决策,不落盘)**

Create `db/wiki-query/src/write_decision.rs`:

```rust
use crate::frontmatter::Frontmatter;
use crate::models::{WriteStatus, ConflictCandidate};

/// 四态决策的输入(已有文档信息)
pub struct DecisionInput<'a> {
    pub target_path: &'a str,
    pub new_frontmatter: &'a Frontmatter,
    /// FTS5 search 结果(按 name/tags 查),已按 bm25 排序
    pub existing_matches: Vec<ExistingDoc>,
    /// target_path 是否已存在文件
    pub path_exists: bool,
}

#[derive(Debug, Clone)]
pub struct ExistingDoc {
    pub path: String,
    pub name: String,
    pub score: f64,
}

/// 决策:返回 status + 候选(若 conflict)
pub fn decide(input: &DecisionInput) -> (WriteStatus, Option<Vec<ConflictCandidate>>) {
    // path 已存在 → UPDATE
    if input.path_exists {
        return (WriteStatus::Update, None);
    }

    // 无任何匹配 → ADD
    if input.existing_matches.is_empty() {
        return (WriteStatus::Add, None);
    }

    // 有匹配:判断是否 NOOP(等价)或 conflict
    // NOOP:存在 name 完全相同的文档(大小写不敏感)
    let exact = input.existing_matches.iter().find(|d| {
        d.name.to_lowercase() == input.new_frontmatter.name.to_lowercase()
    });
    if let Some(_exact_doc) = exact {
        return (WriteStatus::Noop, None);
    }

    // 否则 conflict:有相似但不完全相同的文档
    let candidates: Vec<ConflictCandidate> = input
        .existing_matches
        .iter()
        .take(5)
        .map(|d| ConflictCandidate {
            path: d.path.clone(),
            name: d.name.clone(),
            score: d.score,
        })
        .collect();
    (WriteStatus::Conflict, Some(candidates))
}
```

- [ ] **Step 3: 写决策测试(表驱动)**

Create `db/wiki-query/tests/write_decision_test.rs`:

```rust
use wiki_query::frontmatter::Frontmatter;
use wiki_query::write_decision::{decide, DecisionInput, ExistingDoc};
use wiki_query::models::WriteStatus;

fn fm(name: &str) -> Frontmatter {
    Frontmatter {
        name: name.into(),
        description: "d".into(),
        doc_type: "concept".into(),
        updated: "2026-06-20".into(),
        tags: "t".into(),
    }
}

#[test]
fn add_when_no_match_and_path_new() {
    let input = DecisionInput {
        target_path: "wiki/new.md",
        new_frontmatter: &fm("新概念"),
        existing_matches: vec![],
        path_exists: false,
    };
    let (status, _) = decide(&input);
    assert_eq!(status, WriteStatus::Add);
}

#[test]
fn update_when_path_exists() {
    let input = DecisionInput {
        target_path: "wiki/exists.md",
        new_frontmatter: &fm("任意"),
        existing_matches: vec![],
        path_exists: true,
    };
    let (status, _) = decide(&input);
    assert_eq!(status, WriteStatus::Update);
}

#[test]
fn noop_when_exact_name_match() {
    let input = DecisionInput {
        target_path: "wiki/different-path.md",
        new_frontmatter: &fm("Agent"),
        existing_matches: vec![ExistingDoc {
            path: "wiki/agent.md".into(),
            name: "Agent".into(),  // 大小写不敏感匹配
            score: -1.5,
        }],
        path_exists: false,
    };
    let (status, _) = decide(&input);
    assert_eq!(status, WriteStatus::Noop);
}

#[test]
fn conflict_when_similar_but_different() {
    let input = DecisionInput {
        target_path: "wiki/agent-pattern.md",
        new_frontmatter: &fm("Agent Pattern"),
        existing_matches: vec![ExistingDoc {
            path: "wiki/agent.md".into(),
            name: "Agent".into(),  // 相似但不完全相同
            score: -2.0,
        }],
        path_exists: false,
    };
    let (status, candidates) = decide(&input);
    assert_eq!(status, WriteStatus::Conflict);
    assert!(candidates.is_some());
    assert_eq!(candidates.unwrap().len(), 1);
}
```

- [ ] **Step 4: 运行测试**

Run: `cd db/wiki-query && cargo test write_decision`
Expected: 4 tests PASS

- [ ] **Step 5: Commit**

```bash
git add db/wiki-query/src/write_decision.rs db/wiki-query/src/models.rs db/wiki-query/tests/write_decision_test.rs
git commit -m "feat(wiki-query): write 四态决策纯函数(ADD/UPDATE/NOOP/CONFLICT)"
```

### Task 3.2: core.rs 写入落盘 + 即时索引

**Files:**
- Create: `db/wiki-query/src/core.rs`

- [ ] **Step 1: 写 core.rs(search/read/write 集成)**

Create `db/wiki-query/src/core.rs`:

```rust
use crate::error::WikiError;
use crate::frontmatter::{self, Frontmatter};
use crate::models::{WriteResult, WriteStatus, SearchResult};
use crate::policy::{self, Op};
use crate::write_decision::{self, DecisionInput, ExistingDoc};
use rusqlite::{Connection, params};
use std::path::Path;

// 注:此处 Connection import 实际由 db.rs 内部使用,core 调 db::init 拿连接

/// 读单文档:返回完整 Markdown
pub fn read(vault_root: &str, path: &str) -> Result<String, WikiError> {
    policy::check(path, Op::Read)?;
    let abs = Path::new(vault_root).join(path);
    Ok(std::fs::read_to_string(&abs)?)
}

/// FTS5 检索:返回元数据列表
pub fn search(query: &str) -> Result<Vec<SearchResult>, WikiError> {
    let conn = crate::db::init()?;
    use crate::fts::tokenize_for_fts;
    let tokens: Vec<&str> = tokenize_for_fts(query).split_whitespace().collect();
    if tokens.is_empty() {
        return Ok(vec![]);
    }
    let fts_query: String = tokens
        .iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(" OR ");

    let mut stmt = conn.prepare(
        "SELECT path, name, description, type, tags
         FROM documents_fts
         WHERE documents_fts MATCH ?1
         ORDER BY bm25(documents_fts)
         LIMIT 20",
    )?;
    let results = stmt
        .query_map(params![fts_query], |row| {
            Ok(SearchResult {
                path: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                doc_type: row.get(3)?,
                updated: String::new(),
                tags: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(results)
}

/// 写入:四态决策 + frontmatter 校验 + 落盘 + 即时索引
pub fn write(
    vault_root: &str,
    path: &str,
    new_fm: &Frontmatter,
    body: &str,
) -> Result<WriteResult, WikiError> {
    policy::check(path, Op::Write)?;
    frontmatter::validate(new_fm)?;

    let abs = Path::new(vault_root).join(path);
    let path_exists = abs.exists();

    // 查 FTS5 看有无相似文档(用 name 做 query)
    let existing = search(&new_fm.name).unwrap_or_default();
    let existing_docs: Vec<ExistingDoc> = existing
        .iter()
        .map(|r| ExistingDoc {
            path: r.path.clone(),
            name: r.name.clone(),
            score: 0.0,  // bm25 分数从 search 取(需扩展 search 返回 score,MVP 暂略)
        })
        .collect();

    let input = DecisionInput {
        target_path: path,
        new_frontmatter: new_fm,
        existing_matches: existing_docs,
        path_exists,
    };
    let (status, candidates) = write_decision::decide(&input);

    // conflict:不落盘,返回候选
    if status == WriteStatus::Conflict {
        return Ok(WriteResult {
            status,
            path: path.into(),
            synced: false,
            reason: "检测到相似文档,需人工决策".into(),
            candidates,
        });
    }
    // noop:不落盘
    if status == WriteStatus::Noop {
        return Ok(WriteResult {
            status,
            path: path.into(),
            synced: false,
            reason: "已存在等价文档,未写入".into(),
            candidates: None,
        });
    }

    // ADD / UPDATE:落盘
    let content = frontmatter::serialize(new_fm, body);
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&abs, &content)?;

    // 即时索引该文件(复用 sync 的单文件逻辑)
    sync_single_file(path)?;

    Ok(WriteResult {
        status,
        path: path.into(),
        synced: true,
        reason: if status == WriteStatus::Add { "新建".into() } else { "更新".into() },
        candidates: None,
    })
}

/// 单文件即时入索引(写后调用)
fn sync_single_file(path: &str) -> Result<(), WikiError> {
    let conn = crate::db::init()?;
    // 复用 sync.rs 的逻辑 —— 需把 sync_file 改为 pub(crate)
    crate::sync::sync_file_by_path(&conn, path)?;
    Ok(())
}
```

- [ ] **Step 2: 把 sync.rs 的 sync_file 暴露为 pub(crate) 且按 path 调用**

在 `sync.rs` 把 `sync_file` 改签名,加一个按相对 path 的包装:

```rust
/// 按相对路径同步单文件(供 core.rs write 后即时索引)
pub(crate) fn sync_file_by_path(conn: &Connection, path_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = std::path::Path::new(path_str);
    if !file_path.exists() {
        return Ok(());
    }
    sync_file(conn, path_str, file_path)
}
```

- [ ] **Step 3: 写 core 写入集成测试**

Create `db/wiki-query/tests/core_write_test.rs`:

```rust
use wiki_query::core::write;
use wiki_query::frontmatter::Frontmatter;
use wiki_query::models::WriteStatus;
use std::fs;

#[test]
fn write_adds_new_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = "wiki/test-add.md";
    let fm = Frontmatter {
        name: "测试新增".into(),
        description: "测试".into(),
        doc_type: "concept".into(),
        updated: "2026-06-20".into(),
        tags: "test".into(),
    };
    let result = write(tmp.path().to_str().unwrap(), path, &fm, "# 正文").unwrap();
    assert_eq!(result.status, WriteStatus::Add);
    assert!(result.synced);
    // 验证文件落盘
    let content = fs::read_to_string(tmp.path().join(path)).unwrap();
    assert!(content.contains("name: 测试新增"));
}
```

> 注:需加 `tempfile` 测试依赖。在 Cargo.toml `[dev-dependencies]` 加 `tempfile = "3"`。

- [ ] **Step 4: 运行测试**

Run: `cd db/wiki-query && cargo test core_write`
Expected: 1 test PASS(需确保 db init 用临时库或测试隔离)

> 注意:db.rs 的 DB_PATH 硬编码 `"db/wiki.db"`。测试会写真实库。Task 3.3 处理测试隔离。

- [ ] **Step 5: Commit**

```bash
git add db/wiki-query/src/core.rs db/wiki-query/src/sync.rs db/wiki-query/tests/core_write_test.rs db/wiki-query/Cargo.toml
git commit -m "feat(wiki-query): core.rs 写入落盘 + 即时索引 + 四态集成"
```

### Task 3.3: 测试隔离(DB_PATH 可配置)

**Files:**
- Modify: `db/wiki-query/src/db.rs`

- [ ] **Step 1: 改 DB_PATH 支持环境变量覆盖**

在 `db.rs` 把 `const DB_PATH` 改为函数:

```rust
fn db_path() -> String {
    std::env::var("WIKI_DB_PATH").unwrap_or_else(|_| "db/wiki.db".to_string())
}

pub fn init() -> Result<Connection> {
    let path = db_path();
    let db_dir = Path::new(&path).parent().unwrap();
    std::fs::create_dir_all(db_dir).ok();
    let conn = Connection::open(&path)?;
    // ... 原有建表
}
```

- [ ] **Step 2: 测试里设 WIKI_DB_PATH 指向 tempdir**

在 `tests/core_write_test.rs` 顶部加:

```rust
use std::sync::Once;

static INIT: Once = Once::new();

fn setup_test_db(dir: &std::path::Path) -> String {
    let db_path = dir.join("test.db");
    let db_path_str = db_path.to_str().unwrap().to_string();
    std::env::set_var("WIKI_DB_PATH", &db_path_str);
    db_path_str
}
```

并在每个测试开头调用 `setup_test_db`。

- [ ] **Step 3: 运行全部测试验证隔离**

Run: `cd db/wiki-query && cargo test`
Expected: 所有测试 PASS,真实 db/wiki.db 不被测试污染

- [ ] **Step 4: Commit**

```bash
git add db/wiki-query/src/db.rs db/wiki-query/tests/core_write_test.rs
git commit -m "refactor(wiki-query): DB_PATH 支持 WIKI_DB_PATH 环境变量,测试隔离"
```

---

## 阶段 4:MCP server(rmcp)

> 核心 API 已就绪,包 MCP 前端。手动应用 mcp-builder 技能原则:
> - 清晰工具描述(Claude 靠 description 决定何时调用)
> - 输入校验(zod/schema)
> - 错误处理返回有意义信息
> - 幂等性(write 的 NOOP 即幂等保障)

### Task 4.1: 加 rmcp 依赖 + serve 子命令骨架

**Files:**
- Modify: `db/wiki-query/Cargo.toml`
- Modify: `db/wiki-query/src/main.rs`
- Create: `db/wiki-query/src/mcp_server.rs`

- [ ] **Step 1: 加 rmcp + tokio 依赖**

在 `Cargo.toml` `[dependencies]` 加:

```toml
rmcp = { version = "0.5", features = ["server", "transport-io"] }
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 2: 在 main.rs 加 serve dispatch**

在 `main.rs` 的 match 分支加:

```rust
        "serve" => {
            // 启动 MCP stdio server(阻塞)
            wiki_query::mcp_server::run().expect("MCP server failed");
        }
```

并在 lib.rs 加 `pub mod mcp_server;`。

- [ ] **Step 3: 写 mcp_server.rs 最小骨架(先跑通 stdio)**

Create `db/wiki-query/src/mcp_server.rs`:

```rust
use rmcp::{
    ServerHandler,
    model::*,
    service::*,
    transport::stdio,
    Error as McpError,
};
use std::sync::Arc;
use tokio::io::{stdin, stdout};

/// MCP server 状态(持有 vault root 路径)
#[derive(Clone)]
pub struct WikiServer {
    pub vault_root: String,
}

// 最小骨架:先实现 ServerHandler trait,工具在 Task 4.2 加
#[impl_tool Toolbox]  // 注:rmcp 用宏生成,具体语法见 rmcp 文档
impl WikiServer {
    // 工具定义在 Task 4.2 补充
}

impl ServerHandler for WikiServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "wiki-mcp".into(),
                version: "0.1.0".into(),
            },
            capabilities: ServerCapabilities::default(),
            ..Default::default()
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let server = WikiServer {
            vault_root: std::env::current_dir()?
                .to_str()
                .ok_or("invalid path")?
                .to_string(),
        };
        let service = Server::serve(stdio(stdin(), stdout()), server).await?;
        service.waiting().await?;
        Ok(())
    })
}
```

> **重要说明**:rmcp 的 `#[impl_tool]` 宏语法以 rmcp 官方文档为准。Task 4.2 实现工具时,以 [docs.rs/rmcp](https://docs.rs/rmcp) 当前示例为准。上面是结构示意,不是最终代码 —— 实现时必须查 rmcp 实际 API。

- [ ] **Step 4: 运行 cargo build 确认编译**

Run: `cd db/wiki-query && cargo build`
Expected: 编译通过(可能需按 rmcp 实际 API 调整)

- [ ] **Step 5: Commit**

```bash
git add db/wiki-query/Cargo.toml db/wiki-query/src/main.rs db/wiki-query/src/mcp_server.rs db/wiki-query/src/lib.rs
git commit -m "feat(wiki-query): MCP server 骨架(rmcp + stdio serve 子命令)"
```

### Task 4.2: 实现 6 个 MCP 工具

**Files:**
- Modify: `db/wiki-query/src/mcp_server.rs`

- [ ] **Step 1: 实现 wiki_search / wiki_read(读工具)**

按 rmcp 实际 API(查 docs.rs/rmcp)实现。每个工具:
- 清晰 description(Claude 靠这个决定何时调用)
- 参数 schema 校验
- 调 `wiki_query::core::search` / `read`
- 错误返回有意义的 text

```rust
// 示意(以 rmcp 实际宏 API 为准):
#[tool(description = "搜索 Obsidian 知识库文档(FTS5 全文检索,支持中文)。返回匹配文档的元数据列表。当需要查找某个主题、概念、技术的相关知识时使用。")]
async fn wiki_search(
    &self,
    #[tool(param)] query: String,
) -> Result<CallToolResult, McpError> {
    match wiki_query::core::search(&query) {
        Ok(results) => {
            let json = serde_json::to_string_pretty(&results)
                .unwrap_or_else(|_| "[]".into());
            Ok(CallToolResult::success(vec![Content::text(json)]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    }
}

#[tool(description = "读取 Obsidian 知识库中指定文档的完整内容(Markdown 原文)。参数 path 是相对路径,如 wiki/agent.md。")]
async fn wiki_read(
    &self,
    #[tool(param)] path: String,
) -> Result<CallToolResult, McpError> {
    match wiki_query::core::read(&self.vault_root, &path) {
        Ok(content) => Ok(CallToolResult::success(vec![Content::text(content)])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    }
}
```

- [ ] **Step 2: 实现 wiki_write(四态决策写)**

```rust
#[tool(description = "向 Obsidian 知识库写入或更新文档。写入前自动决策:新建(ADD)/更新(UPDATE)/跳过(NOOP,已存在等价文档)/冲突(CONFLICT,有相似文档需人工裁决)。参数 path 为相对路径,name/description/type/tags 为 frontmatter 字段,body 为正文。")]
async fn wiki_write(
    &self,
    #[tool(param)] path: String,
    #[tool(param)] name: String,
    #[tool(param)] body: String,
    #[tool(param)] description: String,
    #[tool(param)] doc_type: String,
    #[tool(param)] tags: String,
) -> Result<CallToolResult, McpError> {
    let fm = wiki_query::frontmatter::Frontmatter {
        name, description, doc_type, updated: String::new(), tags,
    };
    match wiki_query::core::write(&self.vault_root, &path, &fm, &body) {
        Ok(result) => {
            let json = serde_json::to_string_pretty(&result).unwrap();
            Ok(CallToolResult::success(vec![Content::text(json)]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    }
}
```

- [ ] **Step 3: 实现 wiki_list / wiki_patch / wiki_delete**

`wiki_list`:按 tag/type/dir 过滤(查 documents 表 LIKE)。
`wiki_patch`:section 级编辑(读文件 → 替换指定 section → 写回)。MVP 可先实现"按 heading 替换"。
`wiki_delete`:受 policy,wiki 层改为 deprecated 标记而非真删。

> 这三个工具的实现模式与 search/read/write 一致(调 core 对应函数)。具体代码在实现时按 core API 补全。此处给框架,避免在计划里写未验证的 rmcp API 细节。

- [ ] **Step 4: 实现 core.rs 的 list/patch/delete 对应函数**

补全 `core.rs`:
- `list(filter: ListFilter) -> Vec<SearchResult>`:按 tag/type/dir 查 documents 表
- `patch(path, section_heading, new_content)`:读文件 → 找 `## {heading}` section → 替换内容 → 写回
- `delete(path)`:policy 检查;wiki 层 → 在 frontmatter 加 `deprecated: true`;sources/templates 拒绝

- [ ] **Step 5: 写 MCP 集成测试(stdio 端到端)**

Create `db/wiki-query/tests/mcp_integration_test.rs`:

```rust
use std::process::{Command, Stdio};
use std::io::Write;

#[test]
fn mcp_tools_list_returns_wiki_tools() {
    let mut child = Command::new("./target/debug/wiki-query.exe")
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start serve");

    // 发 initialize + tools/list(JSON-RPC over stdio)
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#;
    let tools_list = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = writeln!(stdin, "{}", init);
        let _ = writeln!(stdin, "{}", tools_list);
    }

    let output = child.wait_with_output().expect("failed to wait");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // 验证 tools/list 响应包含 wiki_ 前缀工具
    assert!(stdout.contains("wiki_search"), "stdout: {}", stdout);
    assert!(stdout.contains("wiki_write"));
}
```

- [ ] **Step 6: 运行 MCP 测试**

Run: `cd db/wiki-query && cargo test mcp_integration`
Expected: PASS(tools/list 返回 wiki_* 工具)

- [ ] **Step 7: 用 MCP Inspector 手动验证**

Run: `npx @anthropics/mcp-inspector ./target/debug/wiki-query.exe serve`
Expected: 在浏览器 Inspector 里看到 6 个 wiki_* 工具,能调用 wiki_search

- [ ] **Step 8: Commit**

```bash
git add db/wiki-query/src/mcp_server.rs db/wiki-query/src/core.rs db/wiki-query/tests/mcp_integration_test.rs
git commit -m "feat(wiki-query): MCP server 6 工具实现(search/read/write/list/patch/delete)"
```

---

## 阶段 5:接入配置 + 验证

### Task 5.1: 各工具 MCP 接入文档

**Files:**
- Create: `docs/mcp-integration.md`

- [ ] **Step 1: 写接入指南**

Create `docs/mcp-integration.md`,包含:
- Claude Code:`.mcp.json` 配置(`{"mcpServers":{"wiki":{"command":"E:/Files/Obsidian/db/wiki-query/wiki-query.exe","args":["serve"]}}}`)
- MiMo-Code:`mimocode.jsonc` 的 `mcp.wiki` 配置
- Cursor / Cline / ZCode:各自 MCP 配置格式
- 验证步骤:用 MCP Inspector

- [ ] **Step 2: Commit**

```bash
git add docs/mcp-integration.md
git commit -m "docs: MCP 各工具接入配置指南"
```

### Task 5.2: git post-commit hook 更新(保持 sync)

**Files:**
- Modify: `.git/hooks/post-commit`(或文档化)

- [ ] **Step 1: 确认 post-commit hook 调 wiki-query sync**

现有 hook 应已是 `./db/wiki-query/wiki-query.exe sync`。确认它存在且可执行。若无,创建(参考 spec 基线文档)。

- [ ] **Step 2: 验证 commit 后索引自动更新**

手动改一个 wiki 文件 → commit → 跑 search → 验证新内容可查。

- [ ] **Step 3: Commit(如有 hook 改动)**

```bash
git add .git/hooks/post-commit 2>/dev/null || echo "hook 不跟踪,文档化即可"
git commit -m "chore: post-commit hook 维持 wiki-query sync" --allow-empty
```

---

## 阶段 6:验证 + Code Review + 收尾

### Task 6.1: verification-before-completion

- [ ] 运行全部测试:`cd db/wiki-query && cargo test` —— 全绿
- [ ] 运行 cargo clippy:`cargo clippy -- -D warnings` —— 无警告
- [ ] 手动端到端:serve → MCP Inspector 调 wiki_search/wiki_write/wiki_read 全通
- [ ] 验证 sync 仍正常:`cargo run -- sync` 不崩
- [ ] 验证 git hook:`git commit` 触发 sync

### Task 6.2: requesting-code-review

- [ ] 触发 superpowers:requesting-code-review 技能,对 feature 分支做 review

### Task 6.3: finishing-a-development-branch

- [ ] 触发 superpowers:finishing-a-development-branch,决定合并到 master / 开 PR / 保留分支

---

## 实现顺序总结

```
阶段1 FTS5 (Task 1.1-1.3)     ← 独立可测,先提升检索
   ↓
阶段2 核心重构 (Task 2.1-2.2)  ← frontmatter/policy 抽取
   ↓
阶段3 write 四态 (Task 3.1-3.3)← 写入逻辑 + 测试隔离
   ↓
阶段4 MCP (Task 4.1-4.2)       ← 包 MCP 前端
   ↓
阶段5 接入 (Task 5.1-5.2)      ← 配置 + hook
   ↓
阶段6 验证收尾 (Task 6.1-6.3)  ← 完整 superpowers 流程
```

每阶段产出可测试软件。每 Task 末尾 commit。

---

## Out of Scope(MVP 推迟项)

以下原计划/spec 列出但 MVP 未实现,留作演进:

- **wiki_patch(section 级编辑)**:Task 4.2 原列,推迟。agent 可用 wiki_read + wiki_write 整篇更新替代,精度差但够用。
- **wiki_delete(含 deprecated 标记工作流)**:Task 4.2 原列,推迟。spec 的矛盾页 + deprecate 工作流也同期推迟。`policy::Op::Delete` 已实现权限矩阵但无 core/MCP 入口调用——保留为基础设施,待 delete 工具实现时直接接入。sources 禁删/wiki deprecated 标记的语义已在 policy 层就绪。
- **write 的 mode 参数(auto/add/update)**:spec 写了 `mode` 字段,实现硬编码 auto。CONFLICT 时 agent 无法强制写入,需先 read 候选再对指定 path 发 write 触发 UPDATE 绕过。
- **search 返回 snippet**:spec 数据流写"返回元数据+snippet",实现只返回元数据(工具描述已声明"需另调 wiki_read")。
- **FTS5 external content 表(`content='documents'`)**:Task 1.2 Step 2 曾尝试,因更新陷阱改回自有内容表(正文存两份),数千篇规模可接受。

这些不阻断 MVP 合并,记录供后续迭代。

