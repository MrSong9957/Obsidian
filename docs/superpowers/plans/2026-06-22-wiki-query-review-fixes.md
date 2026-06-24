# wiki-query 代码审核修复 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复代码审核发现的 3 个 Critical + 4 个 Important + 2 个 Minor 问题，使测试套件编译通过、契约一致、查重功能真实可用。

**Architecture:** 抽取公共 `build_fts_query` 函数消除 C1/I3；将 FTS 连接词从 OR 改为 AND 实现契约一致（C2）；重写查重算法为真正的跨文档 BM25 比较（I1）；清理死代码（I2）；unwrap 改为安全处理（I5/I6）；对齐文档与实现（M2/M9）。

**Tech Stack:** Rust, rusqlite, FTS5, jieba_rs, rmcp

## Global Constraints

- 遵循现有代码风格：中文注释、`pub(crate)` 可见性、`Result<T, WikiError>` 错误处理
- TDD：先写/改测试，再改实现
- 每个 Task 结束时 `cargo test` 全绿
- 不改动 MCP 工具接口（5 个工具名称/参数不变）
- `documents` 表不存正文（v5 原则不变）

---

### Task 1: 抽取 `build_fts_query` 公共函数（C1 + I3）

**问题:** `core_edge_cases_test.rs` 调用不存在的 `core::build_fts_query`（C1）；FTS 查询构造逻辑在 `core.rs:24-28` 和 `sync.rs:190-195` 重复（I3）。

**Files:**
- Modify: `db/wiki-query/src/core.rs:24-28` — 提取函数
- Modify: `db/wiki-query/src/sync.rs:190-195` — 调用公共函数
- Modify: `db/wiki-query/tests/core_edge_cases_test.rs:1-57` — 修复测试引用
- Test: `db/wiki-query/tests/core_edge_cases_test.rs`

**Interfaces:**
- Produces: `pub fn build_fts_query(input: &str) -> String` — 接受原始查询字符串，返回 FTS5 MATCH 表达式（token 用 ` OR ` 连接，每个 token 加双引号）
- Consumed by: `core::search_with_scores`、`sync::fts_score_for_name`

- [ ] **Step 1: 在 `core.rs` 中添加 `build_fts_query` 函数**

在 `core.rs` 的 `search_with_scores` 函数之前添加：

```rust
/// 从原始查询字符串构造 FTS5 MATCH 表达式。
///
/// 流程:jieba 分词 → 过滤无效 token → 每个 token 加双引号 → OR 连接。
/// 返回空字符串表示无有效 token(调用方应返回空结果)。
pub fn build_fts_query(input: &str) -> String {
    use crate::fts::tokenize_for_fts;
    let tokenized = tokenize_for_fts(input);
    let tokens: Vec<&str> = tokenized
        .split_whitespace()
        .filter(|t| !t.contains('"') && t.chars().any(|c| c.is_alphanumeric()))
        .collect();
    if tokens.is_empty() {
        return String::new();
    }
    tokens
        .iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(" OR ")
}
```

- [ ] **Step 2: 修改 `search_with_scores` 使用 `build_fts_query`**

将 `core.rs:14-28` 替换为：

```rust
pub(crate) fn search_with_scores(query: &str) -> Result<Vec<SearchResult>, WikiError> {
    let fts_query = build_fts_query(query);
    if fts_query.is_empty() {
        return Ok(vec![]);
    }
    // ... 后续不变
```

- [ ] **Step 3: 修改 `sync::fts_score_for_name` 使用 `build_fts_query`**

将 `sync.rs:188-195` 替换为：

```rust
fn fts_score_for_name(conn: &Connection, path: &str, name: &str) -> Option<f64> {
    let fts_query = crate::core::build_fts_query(name);
    if fts_query.is_empty() {
        return None;
    }
    // ... 后续 SQL 不变
```

- [ ] **Step 4: 运行测试验证 C1 修复**

Run: `cd db/wiki-query && cargo test`
Expected: 所有 `build_fts_query_*` 测试通过，无 E0425 编译错误

- [ ] **Step 5: Commit**

```bash
cd db/wiki-query && git add src/core.rs src/sync.rs tests/core_edge_cases_test.rs
git commit -m "fix(core): 抽取 build_fts_query 公共函数，修复测试编译失败 (C1+I3)"
```

---

### Task 2: 修复 AND/OR 契约矛盾（C2）

**问题:** 实现用 OR 连接多词（`core.rs:28`），但 MCP 工具描述和 README 都说"多词默认 AND"。实测搜 "rust 编程" 返回 20 条，15 条两个词都没有。

**Files:**
- Modify: `db/wiki-query/src/core.rs` — `build_fts_query` 改为 AND 连接
- Modify: `db/wiki-query/tests/core_edge_cases_test.rs:40-44` — 修复 OR 测试断言
- Test: `db/wiki-query/tests/core_edge_cases_test.rs`

**Interfaces:**
- `build_fts_query` 返回值：token 间连接词从 ` OR ` 改为 ` AND `

- [ ] **Step 1: 修改 `build_fts_query` 使用 AND 连接**

将 `core.rs` 中 `build_fts_query` 的 `.join(" OR ")` 改为 `.join(" AND ")`：

```rust
    tokens
        .iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(" AND ")
```

- [ ] **Step 2: 修复 `build_fts_query_multiple_or` 测试**

`core_edge_cases_test.rs:40-44` 当前断言 `contains(" OR ")`。这个测试的语义是"用户显式写 OR 时应保持 OR"。但 `build_fts_query` 不解析用户输入的 OR——它对所有 token 统一连接。需要重新设计这个测试。

将 `tests/core_edge_cases_test.rs:40-44` 替换为：

```rust
#[test]
fn build_fts_query_multiple_tokens_use_and() {
    // 多个 token 应用 AND 连接(全部命中)
    let q = build_fts_query("rust go python");
    assert!(q.contains(" AND "), "多 token 应用 AND 连接: {}", q);
    assert!(!q.contains(" OR "), "不应含 OR: {}", q);
}
```

同时删除 `tests/core_edge_cases_test.rs:53-57` 的 `build_fts_query_or_with_chinese` 测试（OR 语义不再由 `build_fts_query` 处理）。

- [ ] **Step 3: 运行测试验证**

Run: `cd db/wiki-query && cargo test`
Expected: 全绿

- [ ] **Step 4: 实测 MCP 搜索验证行为正确**

```bash
cd "E:\Files\Obsidian" && echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"wiki_search","arguments":{"query":"rust 编程"}}}' | ./db/wiki-query/target/release/wiki-query.exe serve 2>/dev/null | tail -1
```

Expected: 返回结果应同时包含 "rust" 和 "编程"（AND 语义），数量应少于 20

- [ ] **Step 5: Commit**

```bash
cd db/wiki-query && git add src/core.rs tests/core_edge_cases_test.rs
git commit -m "fix(core): FTS 查询多词改为 AND 连接，与文档契约一致 (C2)"
```

---

### Task 3: 重写查重算法（I1）

**问题:** `fts_score_for_name` 的 SQL `WHERE MATCH ?1 AND path = ?2` 把 BM25 分数锁在文档自己身上。`find_similar` 比较的是"文档 A 对自己的 BM25"和"文档 B 对自己的 BM25"，不是相似度。实测 full 查重返回 75,993 对（393 篇文档的 98.7%），形同虚设。

**Files:**
- Modify: `db/wiki-query/src/sync.rs:140-206` — 重写 `load_all_docs_with_scores` 和 `fts_score_for_name`
- Modify: `db/wiki-query/src/core.rs:62-89` — 修改 `find_similar` 签名
- Modify: `db/wiki-query/src/models.rs` — `SimilarPair` 可能需要调整
- Test: `db/wiki-query/tests/core_edge_cases_test.rs` — 更新 find_similar 测试

**Interfaces:**
- 旧: `find_similar(candidate, all_docs, threshold)` — candidate 和 all_docs 都带 self-score
- 新: `find_similar(candidate, all_docs, threshold)` — candidate 用自己的 name 做 FTS 查询，在 all_docs 中找 BM25 分数最好的匹配

**核心思路:** 查重应该是"用候选文档的 name 去搜全库，看哪些文档的 BM25 分数足够低（相关）"。不是"每个文档对自己的 name 做自评分"。

- [ ] **Step 1: 重写 `fts_score_for_name` → `search_similar_by_name`**

将 `sync.rs` 中的 `fts_score_for_name`（187-206 行）替换为：

```rust
/// 用 candidate_name 做 FTS5 查询，在全库中找 BM25 分数最低（最相关）的文档。
/// 返回 Vec<(path, score)>，排除 candidate 自身的 path。
/// 这是真正的跨文档相似度计算。
fn search_similar_by_name(
    conn: &Connection,
    candidate_path: &str,
    candidate_name: &str,
) -> Vec<(String, f64)> {
    let fts_query = crate::core::build_fts_query(candidate_name);
    if fts_query.is_empty() {
        return vec![];
    }
    let mut stmt = match conn.prepare(
        "SELECT d.path, bm25(documents_fts) AS score
         FROM documents_fts f
         JOIN documents d ON d.path = f.path
         WHERE documents_fts MATCH ?1
         ORDER BY score
         LIMIT 20",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map(rusqlite::params![fts_query], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f64>(1)?,
        ))
    })
    .ok()
    .map(|iter| {
        iter.filter_map(|r| r.ok())
            .filter(|(path, _)| path != candidate_path)
            .collect()
    })
    .unwrap_or_default()
}
```

- [ ] **Step 2: 重写 `load_all_docs_with_scores` → `load_all_docs`**

将 `sync.rs:142-183` 的 `load_all_docs_with_scores` 替换为只加载元数据（不再算分）：

```rust
/// 从 documents 表加载所有文档的元数据（不含 score，score 由查重时实时计算）。
fn load_all_docs(conn: &Connection) -> Vec<crate::models::SearchResult> {
    let mut stmt = match conn.prepare(
        "SELECT path, name, description, type, updated, tags FROM documents",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map([], |row| {
        Ok(crate::models::SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            doc_type: row.get(3)?,
            updated: row.get(4)?,
            tags: row.get(5)?,
            score: None,
        })
    })
    .ok()
    .map(|iter| iter.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}
```

- [ ] **Step 3: 重写 `build_dedupe_report`**

将 `sync.rs:104-138` 替换为：

```rust
fn build_dedupe_report(
    scope: &str,
    updated_paths: &[String],
    conn: &Connection,
) -> Vec<SimilarPair> {
    let threshold = -2.0;
    let all_docs = load_all_docs(conn);

    let candidates: Vec<&crate::models::SearchResult> = if scope == "full" {
        all_docs.iter().collect()
    } else {
        updated_paths
            .iter()
            .filter_map(|p| all_docs.iter().find(|d| &d.path == p))
            .collect()
    };

    let mut pairs = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for candidate in &candidates {
        let similar = search_similar_by_name(conn, &candidate.path, &candidate.name);
        for (similar_path, score) in similar {
            if score > threshold {
                continue; // 不够相关
            }
            let key = if candidate.path < similar_path {
                format!("{}|{}", candidate.path, similar_path)
            } else {
                format!("{}|{}", similar_path, candidate.path)
            };
            if seen.insert(key) {
                pairs.push(SimilarPair {
                    candidate: candidate.path.clone(),
                    similar_to: similar_path,
                    score,
                });
            }
        }
    }
    pairs
}
```

- [ ] **Step 4: 更新 `core::find_similar` 单元测试**

`core.rs` 中的 `find_similar` 函数本身逻辑不变（它只是过滤），但 `core_edge_cases_test.rs` 的测试用例需要确认仍然适用。运行现有测试即可。

- [ ] **Step 5: 运行测试验证**

Run: `cd db/wiki-query && cargo test`
Expected: 全绿

- [ ] **Step 6: 实测查重验证 I1 修复**

```bash
cd "E:\Files\Obsidian" && echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"wiki_sync","arguments":{"scope":"full"}}}' | ./db/wiki-query/target/release/wiki-query.exe serve 2>/dev/null | tail -1
```

Expected: 查重 pairs 数量应显著少于 75,993（合理范围：< 500）

- [ ] **Step 7: Commit**

```bash
cd db/wiki-query && git add src/sync.rs src/core.rs tests/core_edge_cases_test.rs
git commit -m "fix(sync): 重写查重算法为真正的跨文档 BM25 比较 (I1)"
```

---

### Task 4: 清理 CheckScopeRequest 重复定义（I2）

**问题:** `CheckScopeRequest` 在 `models.rs:54-62` 和 `mcp_server.rs:62-68` 重复定义。`models.rs` 那份是死代码，但测试还在用它。

**Files:**
- Modify: `db/wiki-query/src/models.rs:54-62` — 删除死代码
- Modify: `db/wiki-query/src/mcp_server.rs:62-68` — 保留为唯一定义
- Modify: `db/wiki-query/tests/core_edge_cases_test.rs:400-414` — 测试改为引用 `mcp_server::CheckScopeRequest`

**Interfaces:**
- `CheckScopeRequest` 只在 `mcp_server` 模块中定义

- [ ] **Step 1: 删除 `models.rs` 中的 `CheckScopeRequest`**

删除 `models.rs:53-62`（`/// wiki_check_scope 的请求参数。` 到 `pub vault_root: Option<String>,` 的整个结构体）。

- [ ] **Step 2: 更新测试引用**

将 `core_edge_cases_test.rs:400-414` 的两处 `wiki_query::models::CheckScopeRequest` 改为 `wiki_query::mcp_server::CheckScopeRequest`。

- [ ] **Step 3: 运行测试验证**

Run: `cd db/wiki-query && cargo test`
Expected: 全绿，无 dead_code 警告

- [ ] **Step 4: Commit**

```bash
cd db/wiki-query && git add src/models.rs tests/core_edge_cases_test.rs
git commit -m "refactor: 删除 models.rs 中重复的 CheckScopeRequest (I2)"
```

---

### Task 5: 修复 unwrap panic 风险（I5 + I6）

**问题:** `sync.rs:335,368` 的 `metadata.modified().unwrap()` 在某些文件系统会 panic；`db.rs:21` 的 `Path::parent().unwrap()` 在 `WIKI_DB_PATH=wiki.db` 时 panic。

**Files:**
- Modify: `db/wiki-query/src/sync.rs:329-370` — `needs_update` 和 `sync_file` 中的 unwrap
- Modify: `db/wiki-query/src/db.rs:21` — `parent().unwrap()`

**Interfaces:**
- 无接口变更

- [ ] **Step 1: 修复 `sync.rs:335` 的 `metadata.modified().unwrap()`**

将 `sync.rs:335` 替换为：

```rust
    let modified = match metadata.modified() {
        Ok(m) => m,
        Err(_) => return true, // 无法获取修改时间，假定需要更新
    };
```

- [ ] **Step 2: 修复 `sync.rs:368` 的 `metadata.modified().unwrap()`**

将 `sync.rs:368` 替换为：

```rust
    let modified = match metadata.modified() {
        Ok(m) => m,
        Err(_) => return Err("无法获取文件修改时间".into()),
    };
```

- [ ] **Step 3: 修复 `db.rs:21` 的 `Path::parent().unwrap()`**

将 `db.rs:21` 替换为：

```rust
    let db_dir = Path::new(&path).parent().unwrap_or(Path::new("."));
```

- [ ] **Step 4: 运行测试验证**

Run: `cd db/wiki-query && cargo test`
Expected: 全绿

- [ ] **Step 5: Commit**

```bash
cd db/wiki-query && git add src/sync.rs src/db.rs
git commit -m "fix: unwrap 改为安全处理，防止特殊文件系统/路径 panic (I5+I6)"
```

---

### Task 6: 对齐文档与实现（M2 + M9）

**问题:** `LIMIT 20` vs 文档"前 10 条"（M2）；测试注释说"4 个工具"实际 5 个（M9）。

**Files:**
- Modify: `db/wiki-query/src/core.rs:38` — LIMIT 20 → LIMIT 10
- Modify: `db/wiki-query/tests/mcp_integration_test.rs:87` — 注释修正
- Modify: `db/wiki-query/tests/mcp_e2e_full_test.rs:1` — 注释修正

- [ ] **Step 1: 修改 LIMIT 20 → LIMIT 10**

将 `core.rs:38` 的 `LIMIT 20` 改为 `LIMIT 10`。

- [ ] **Step 2: 修正 `mcp_integration_test.rs:87` 注释**

将 `// v5 工具集 4 个` 改为 `// v5 工具集 5 个`。

- [ ] **Step 3: 修正 `mcp_e2e_full_test.rs:1` 注释**

将 `//! MCP 端到端全面测试:覆盖所有 4 个工具的完整调用路径。` 改为 `//! MCP 端到端全面测试:覆盖所有 5 个工具的完整调用路径。`

- [ ] **Step 4: 运行测试验证**

Run: `cd db/wiki-query && cargo test`
Expected: 全绿

- [ ] **Step 5: Commit**

```bash
cd db/wiki-query && git add src/core.rs tests/mcp_integration_test.rs tests/mcp_e2e_full_test.rs
git commit -m "docs: 对齐 LIMIT 值和工具数量注释 (M2+M9)"
```

---

### Task 7: 更新 README 文档

**问题:** README 宣传"BM25 算相似度"但查重逻辑有误；搜索行为描述需与 AND 语义一致。

**Files:**
- Modify: `db/wiki-query/README.md:358` — 查重描述修正
- Modify: `db/wiki-query/README.md:61` — wiki_search 描述确认

- [ ] **Step 1: 修正 README 查重描述**

将 `README.md:358` 的：
```
- **查重**：用 BM25 算法算相似度，阈值 -2.0 以下算重复
```
改为：
```
- **查重**：用候选文档的 name 做 FTS5 查询，在全库中找 BM25 分数 ≤ -2.0 的相似文档
```

- [ ] **Step 2: 确认 wiki_search 描述**

`README.md:61` 写 `搜关键词，返回最相关的 10 篇文档`，与修正后的 LIMIT 10 一致，无需修改。

- [ ] **Step 3: Commit**

```bash
cd db/wiki-query && git add README.md
git commit -m "docs: 修正 README 查重描述 (I1 文档同步)"
```

---

### Task 8: 全量验证

- [ ] **Step 1: 运行全部测试**

```bash
cd db/wiki-query && cargo test 2>&1
```

Expected: 全绿，无编译错误，无 E0425

- [ ] **Step 2: 运行 clippy**

```bash
cd db/wiki-query && cargo clippy 2>&1
```

Expected: 无 error，无 warning（或只有已知的 dead_code 警告）

- [ ] **Step 3: 实测 MCP 搜索**

```bash
cd "E:\Files\Obsidian" && echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"wiki_search","arguments":{"query":"rust 编程"}}}' | ./db/wiki-query/target/release/wiki-query.exe serve 2>/dev/null | tail -1
```

Expected: 返回结果同时含 "rust" 和 "编程"，数量 ≤ 10

- [ ] **Step 4: 实测 MCP 查重**

```bash
cd "E:\Files\Obsidian" && echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"wiki_sync","arguments":{"scope":"full"}}}' | ./db/wiki-query/target/release/wiki-query.exe serve 2>/dev/null | tail -1
```

Expected: 查重 pairs 数量合理（< 500），不再出现 75,993

- [ ] **Step 5: 最终 Commit（如有遗漏修复）**

```bash
cd db/wiki-query && git add -A && git commit -m "chore: 全量验证通过，审核问题全部修复"
```
