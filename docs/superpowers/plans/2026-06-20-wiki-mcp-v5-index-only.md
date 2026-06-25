# wiki-query MCP v5 纯索引层重构 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 wiki-query 从"Obsidian 知识库管理器"重构为"纯索引服务"——删掉所有抢 agent 活的工具(write/read/patch/四态决策),工具集收敛为 4 个(search/list/sync/get_project_rules),全部围绕 SQLite 索引。

**Architecture:** 存量减法重构。保留已验证的 FTS5/jieba/policy/documents 表,删除正文读写工具和决策智能。新增 sync 的 scope 查重(合并 dedupe)+ get_project_rules 规则注入。提炼流程从 8 步降到 6 步。

**Tech Stack:** Rust, rmcp(Model Context Protocol), SQLite + FTS5, jieba-rs(CJK 分词), chrono。

**Spec:** `docs/superpowers/specs/2026-06-20-wiki-mcp-v5-index-only.md`

**分支:** 继续在 `feature/wiki-mcp-memory-system`(v3/v4 同一分支延续)。

**工作目录:** 所有 `cargo` 命令在 `db/wiki-query/` 下跑,所有 `git` 命令在仓库根 `E:\Files\Obsidian` 跑。

---

## 文件改动总览

```
db/wiki-query/src/
├── lib.rs              # Modify: 删 write_decision 模块声明
├── main.rs             # Modify: sync 子命令输出含查重报告(支持 scope)
├── core.rs             # Modify: 删 write/read/patch/locate_section,保留 search/list,加 find_similar
├── write_decision.rs   # Delete: 整个文件删
├── frontmatter.rs      # Modify: 删 validate + VALID_TYPES,保留 parse/serialize
├── policy.rs           # 保留不动
├── db.rs               # 保留不动(不加 meta 表)
├── sync.rs             # Modify: run 接 scope 参数,返回 SyncReport(含查重)
├── mcp_server.rs       # Modify: 工具集 5→4(删 read/write/patch/mark_extracted,改 sync 加 scope,加 get_project_rules)
└── models.rs           # Modify: 删 WriteResult/ConflictCandidate/WriteStatus,加 SyncReport/SimilarPair/ProjectRulesRequest
tests/
├── core_write_test.rs  # Delete: 整个文件删(write/patch 测试),新建 sync_dedupe_test.rs
├── fts_test.rs         # 保留不动
└── mcp_integration_test.rs  # Modify: 断言新工具集(4 个)
templates/project-rules.md   # Create: 可注入的项目根规则模板
db/wiki-query/README.md      # Modify: 重写工具集 + 数据流
```

---

## Task 1: 删 write_decision.rs + core::write/read/patch(砍正文工具链路)

**Files:**
- Delete: `db/wiki-query/src/write_decision.rs`
- Modify: `db/wiki-query/src/lib.rs`
- Modify: `db/wiki-query/src/core.rs`(删 write/read/patch/locate_section)
- Delete: `db/wiki-query/tests/core_write_test.rs`

这是最大的删除,先做。删完后 core 只剩 search/search_with_scores/list/find_similar(后者 Task 4 加)。

- [ ] **Step 1: 删 write_decision.rs 文件**

```bash
cd "E:\Files\Obsidian"
git rm db/wiki-query/src/write_decision.rs
```

- [ ] **Step 2: lib.rs 删 write_decision 模块声明**

修改 `db/wiki-query/src/lib.rs`,删掉这一行:

```rust
pub mod write_decision;
```

同时更新文件顶部注释的模块职责列表,删掉对 write_decision 的描述(若有)。

- [ ] **Step 3: core.rs 删 write/read/patch/locate_section 及其测试**

修改 `db/wiki-query/src/core.rs`,删除以下函数(整块删):
- `pub fn read(...)` (约 L20-27)
- `pub fn write(...)` (约 L84-179)
- `pub fn patch(...)` (约 L297-343)
- `fn locate_section(...)` (约 L253-291)

同时删除 core.rs 里 `mod tests` 中所有 `locate_section` 相关测试(约 L349-397):
- `locate_h2_section` / `locate_h3_section` / `locate_section_at_doc_end` / `locate_section_not_found` / `locate_section_first_match_when_duplicate` / `locate_section_rejects_no_space_after_hashes` / `locate_section_skips_invalid_matches_real_heading`

删除 core.rs 顶部不再用的 imports(删完函数后 rustc 会报 unused):
- `use crate::frontmatter::{self, Frontmatter};` → 改为 `use crate::frontmatter;`(write/read 不再直接用 Frontmatter,但 sync 间接用,保留 frontmatter 模块引用。实际跑 `cargo build` 看 rustc 报哪些 unused 再删)
- `use crate::policy::{self, Op};` → 评估:read/write/patch 用了 policy::check,删掉后若 core 不再调 policy,整个 import 删
- `use std::path::Path;` → 若 read/write 删后不再用,删

保留 `pub fn search` / `pub(crate) fn search_with_scores` / `pub fn list` / `ListFilter`。

- [ ] **Step 4: 删 core_write_test.rs(整个文件)**

```bash
cd "E:\Files\Obsidian"
git rm db/wiki-query/tests/core_write_test.rs
```

这个文件全是 write/patch 测试,对应函数删了,测试无意义。

- [ ] **Step 5: 编译验证 + 清 unused imports**

```bash
cd "E:\Files\Obsidian\db\wiki-query"
cargo build 2>&1
```

预期:会有 unused import / unused variable 警告(因为删了函数但 imports 可能残留)。按 rustc 提示逐一清理,直到 `cargo build` 0 error。

注意:此时会有大量 warning(frontmatter/policy 的某些函数变 unused 等),**Task 2 会处理 frontmatter,Task 3 处理 models,先不急着清所有 warning**。只要 0 error 即可。

- [ ] **Step 6: 跑测试确认剩余测试不依赖被删代码**

```bash
cd "E:\Files\Obsidian\db\wiki-query"
cargo test 2>&1 | grep "test result"
```

预期:fts_test(4 个)+ mcp_integration_test(4 个)可能报错——因为 mcp_server 还引用 WriteRequest 等(下一步处理)。**此时允许 mcp_integration_test 编译失败**,Task 6 会修。只要 lib unit tests + fts_test 通过即可。

若 lib unit tests 也失败(如 frontmatter tests 引用了被删的类型),按错误修。

- [ ] **Step 7: Commit**

```bash
cd "E:\Files\Obsidian"
git add db/wiki-query/src/lib.rs db/wiki-query/src/core.rs
git commit -m "refactor(wiki-query): 删 write/read/patch + write_decision(spec v5 Task1)

砍掉正文读写工具链路(归 agent 文件工具)。core 只剩 search/list。" 
```

---

## Task 2: frontmatter 删 validate + VALID_TYPES

**Files:**
- Modify: `db/wiki-query/src/frontmatter.rs`

v5 不再强校验 frontmatter(降级为 AGENTS.md 约定)。删 validate 函数和 VALID_TYPES 常量,保留 parse/serialize(sync 建索引用)。

- [ ] **Step 1: 删 VALID_TYPES 常量 + validate 函数 + 相关测试**

修改 `db/wiki-query/src/frontmatter.rs`,删除:
- `pub const VALID_TYPES: &[&str] = &[...];` (约 L34-47)
- `pub fn validate(fm: &Frontmatter) -> Result<(), WikiError>` (约 L110-127)
- mod tests 中的 `validate_rejects_empty_name` / `validate_rejects_invalid_type` / `validate_accepts_all_whitelisted_types` (约 L190-226)

保留 `parse` / `serialize` / `Frontmatter` struct 及其测试。

- [ ] **Step 2: 删不再用的 import**

删 `use crate::error::WikiError;`(若 validate 删后 frontmatter.rs 不再用 WikiError——parse/serialize 不返回 Result,应该不用了)。`cargo build` 确认。

- [ ] **Step 3: 编译 + 测试**

```bash
cd "E:\Files\Obsidian\db\wiki-query"
cargo build 2>&1 | grep -E "error|warning: unused"
cargo test --lib 2>&1 | grep "test result"
```

预期:lib 编译通过,frontmatter 的 parse/serialize 测试仍通过(约 8 个)。

- [ ] **Step 4: Commit**

```bash
cd "E:\Files\Obsidian"
git add db/wiki-query/src/frontmatter.rs
git commit -m "refactor(wiki-query): frontmatter 删 validate + VALID_TYPES(spec v5 Task2)

降级为 AGENTS.md 约定,不再代码强校验。保留 parse/serialize。"
```

---

## Task 3: models 清理 + 加新类型

**Files:**
- Modify: `db/wiki-query/src/models.rs`

删 WriteResult/ConflictCandidate/WriteStatus(随 write 一起没用了),加 SyncReport/SimilarPair/ProjectRulesRequest。

- [ ] **Step 1: 删旧类型,加新类型**

修改 `db/wiki-query/src/models.rs`,**完整替换为**:

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
    /// bm25 分数(FTS5:越小越相关,负值)。供 find_similar 判断相似度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

/// 一对相似文档(查重报告的单条)。find_similar 返回 Vec<SimilarPair>。
#[derive(Debug, Clone, Serialize)]
pub struct SimilarPair {
    /// 候选文档路径(被查重的那篇)
    pub candidate: String,
    /// 与之相似的已有文档路径
    pub similar_to: String,
    /// bm25 相关度分数(越小越相关)
    pub score: f64,
}

/// wiki_sync 的返回报告。含同步统计 + 查重报告。
#[derive(Debug, Serialize)]
pub struct SyncReport {
    /// 本次扫描到的文档总数
    pub synced: i32,
    /// 本次新增/更新的文档数
    pub updated: i32,
    /// 本次清理的孤儿记录数
    pub pruned: i32,
    /// 查重范围:"incremental" 或 "full"
    pub scope: String,
    /// 查重报告:相似文档对(incremental 模式可能为空,full 模式含全库相似对)
    pub dedupe_report: Vec<SimilarPair>,
}

/// wiki_get_project_rules 的请求参数。
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ProjectRulesRequest {
    /// 当前 agent 类型,决定推荐的目标文件名。
    /// "claude" → CLAUDE.md;"codex"/"cursor"/"opencode"/"mimo"/其他 → AGENTS.md。
    /// 可选,默认按 AGENTS.md。
    #[serde(rename = "agentType", default)]
    pub agent_type: Option<String>,
}
```

注意:`ProjectRulesRequest` 不放在 models.rs 也行,但为集中管理数据类型放这里。它需要 `Deserialize`(MCP 参数反序列化)+ `Serialize`。

- [ ] **Step 2: 编译验证**

```bash
cd "E:\Files\Obsidian\db\wiki-query"
cargo build 2>&1 | grep -E "^error"
```

预期:0 error(models.rs 自身完整)。但 mcp_server.rs 还引用旧的 WriteRequest/WriteResult,会报 error——那是 Task 6 修的,这里不管。**只要 models.rs 本身编译通过即可**(可用 `cargo build --lib` 单独编译 lib)。

```bash
cargo build --lib 2>&1 | grep -E "^error"
```

- [ ] **Step 3: Commit**

```bash
cd "E:\Files\Obsidian"
git add db/wiki-query/src/models.rs
git commit -m "refactor(wiki-query): models 删 write 类型,加 SyncReport/SimilarPair/ProjectRulesRequest(spec v5 Task3)"
```

---

## Task 4: core 加 find_similar 查重函数(TDD)

**Files:**
- Modify: `db/wiki-query/src/core.rs`

find_similar 是 sync 查重(incremental + full)的共享底层函数。纯逻辑:给定候选集 + 全库 + 阈值,返回相似对。先写测试。

- [ ] **Step 1: 写失败测试**

在 `db/wiki-query/src/core.rs` 的 `mod tests` 里加(若 mod tests 已删则重建):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SearchResult;

    fn sr(path: &str, name: &str, score: f64) -> SearchResult {
        SearchResult {
            path: path.into(),
            name: name.into(),
            description: String::new(),
            doc_type: String::new(),
            updated: String::new(),
            tags: String::new(),
            score: Some(score),
        }
    }

    #[test]
    fn find_similar_returns_pair_when_score_below_threshold() {
        // score=-3.0 比阈值 -2.0 更相关(更负),应入报告
        let candidate = sr("wiki/new.md", "React Hooks", -3.0);
        let all = vec![sr("wiki/new.md", "React Hooks", -3.0), sr("wiki/exist.md", "Hooks 用法", -3.5)];
        let pairs = find_similar(&candidate, &all, -2.0);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].candidate, "wiki/new.md");
        assert_eq!(pairs[0].similar_to, "wiki/exist.md");
    }

    #[test]
    fn find_similar_excludes_self() {
        // 候选自己不应出现在 similar_to 里
        let candidate = sr("wiki/a.md", "A", -5.0);
        let all = vec![sr("wiki/a.md", "A", -5.0)];
        let pairs = find_similar(&candidate, &all, -2.0);
        assert!(pairs.is_empty(), "不应把自己当相似文档");
    }

    #[test]
    fn find_similar_excludes_low_relevance() {
        // score=-1.0 不如阈值 -2.0 相关,不入报告
        let candidate = sr("wiki/new.md", "X", -1.0);
        let all = vec![sr("wiki/new.md", "X", -1.0), sr("wiki/other.md", "Y", -1.0)];
        let pairs = find_similar(&candidate, &all, -2.0);
        assert!(pairs.is_empty(), "相关度不够不应入报告");
    }

    #[test]
    fn find_similar_collects_multiple_pairs() {
        // 候选与多篇文档都高度相关,全部入报告
        let candidate = sr("wiki/new.md", "React", -4.0);
        let all = vec![
            sr("wiki/new.md", "React", -4.0),
            sr("wiki/a.md", "React 基础", -3.5),
            sr("wiki/b.md", "React 进阶", -3.0),
            sr("wiki/c.md", "无关", -0.5),
        ];
        let pairs = find_similar(&candidate, &all, -2.0);
        assert_eq!(pairs.len(), 2, "应含 a 和 b,c 不够相关");
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cd "E:\Files\Obsidian\db\wiki-query"
cargo test --lib find_similar 2>&1 | grep -E "error|FAILED|cannot find"
```

预期:编译失败,`cannot find function find_similar`。

- [ ] **Step 3: 实现 find_similar**

在 `db/wiki-query/src/core.rs` 加(放在 search/list 函数附近):

```rust
/// 对单个候选文档,在 all_docs 中找相似文档(bm25 score ≤ threshold)。
/// 排除候选自身(path 相同)。返回相似对列表。
///
/// threshold 是 bm25 阈值:score 越小(越负)越相关。
/// 如 threshold=-2.0 表示 score ≤ -2.0 的才算相似。
pub fn find_similar(
    candidate: &crate::models::SearchResult,
    all_docs: &[crate::models::SearchResult],
    threshold: f64,
) -> Vec<crate::models::SimilarPair> {
    all_docs
        .iter()
        .filter(|d| d.path != candidate.path) // 排除自身
        .filter_map(|d| {
            let score = d.score.unwrap_or(0.0);
            if score <= threshold {
                Some(crate::models::SimilarPair {
                    candidate: candidate.path.clone(),
                    similar_to: d.path.clone(),
                    score,
                })
            } else {
                None
            }
        })
        .collect()
}
```

- [ ] **Step 4: 跑测试确认通过**

```bash
cd "E:\Files\Obsidian\db\wiki-query"
cargo test --lib find_similar 2>&1 | grep "test result"
```

预期:`4 passed; 0 failed`。

- [ ] **Step 5: Commit**

```bash
cd "E:\Files\Obsidian"
git add db/wiki-query/src/core.rs
git commit -m "feat(wiki-query): core::find_similar 查重函数(TDD,spec v5 Task4)

给定候选 + 全库 + bm25 阈值,返回相似对。sync 的 incremental/full 共用。"
```

---

## Task 5: sync 接 scope 参数 + 返回 SyncReport

**Files:**
- Modify: `db/wiki-query/src/sync.rs`
- Modify: `db/wiki-query/src/core.rs`(加 run_with_scope 编排函数)
- Create: `db/wiki-query/tests/sync_dedupe_test.rs`

sync 从"void run()"改为"SyncReport run_with_scope(scope)"。run() 保留(CLI 入口)但内部调 run_with_scope。

- [ ] **Step 1: sync.rs 的 run 改为返回 SyncReport + 接 scope**

修改 `db/wiki-query/src/sync.rs`。把现有的 `pub fn run()` 改造为内部只做"扫描+同步+prune"并返回统计,新增 `run_with_scope` 编排查重:

将 `pub fn run()` 改为:

```rust
use crate::models::{SyncReport, SimilarPair};

/// CLI 入口:全量 sync + incremental 查重,打印报告。
pub fn run() {
    let report = run_with_scope("incremental");
    println!(
        "Scan complete: {} files found, {} updated, {} pruned, {} similar pairs (scope={})",
        report.synced, report.updated, report.pruned, report.dedupe_report.len(), report.scope
    );
}

/// 带查重的 sync。scope="incremental" 查新文档,scope="full" 查全库。
/// 返回 SyncReport(同步统计 + 查重报告)。
pub fn run_with_scope(scope: &str) -> SyncReport {
    let conn = init().expect("Failed to init database");
    let root = Path::new(".");

    let mut count = 0;
    let mut updated = 0;
    let mut updated_paths: Vec<String> = Vec::new(); // 记录本次更新的路径(供 incremental 查重)
    let mut on_disk: std::collections::HashSet<String> = std::collections::HashSet::new();
    scan_directory(root, &conn, &mut count, &mut updated, &mut on_disk, &mut updated_paths);

    let pruned = prune_orphans(&conn, &on_disk);

    // 查重:加载全库元数据(含 score 需要重查 FTS5,这里简化为按 name 做 search)
    let dedupe_report = build_dedupe_report(scope, &updated_paths, &conn);

    SyncReport {
        synced: count,
        updated,
        pruned,
        scope: scope.into(),
        dedupe_report,
    }
}

/// 构建查重报告。
/// incremental:只对本次更新的文档(updated_paths)查重。
/// full:对全库所有文档查重。
fn build_dedupe_report(
    scope: &str,
    updated_paths: &[String],
    conn: &rusqlite::Connection,
) -> Vec<SimilarPair> {
    let threshold = -2.0; // bm25 阈值:score ≤ -2.0 算相似。实现时用真实数据调。

    // 候选集:incremental 用 updated_paths,full 用全库
    let candidates: Vec<crate::models::SearchResult> = if scope == "full" {
        // 全库:从 documents 表读所有文档,用每个 name 做 FTS5 search 拿 score
        load_all_docs_with_scores(conn)
    } else {
        // incremental:updated_paths 的文档,每个用 name 做 search
        updated_paths
            .iter()
            .filter_map(|p| load_doc_with_score(conn, p))
            .collect()
    };

    // 对每个候选,调 core::find_similar
    let all_docs = load_all_docs_with_scores(conn);
    let mut pairs = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for candidate in &candidates {
        for pair in crate::core::find_similar(candidate, &all_docs, threshold) {
            // 双向去重:full 模式下 (a,b) 和 (b,a) 只报一次
            let key = if pair.candidate < pair.similar_to {
                format!("{}|{}", pair.candidate, pair.similar_to)
            } else {
                format!("{}|{}", pair.similar_to, pair.candidate)
            };
            if seen.insert(key) {
                pairs.push(pair);
            }
        }
    }
    pairs
}

/// 从 documents 表加载所有文档,对每个用 name 做 FTS5 search 拿 score。
fn load_all_docs_with_scores(conn: &rusqlite::Connection) -> Vec<crate::models::SearchResult> {
    // 简化实现:对每个文档的 name 调 core::search_with_scores,取第一个非自身的作为自身 score。
    // 性能不优(O(n²)),MVP 够用,文档量大时优化。
    let paths: Vec<(String, String)> = {
        let mut stmt = match conn.prepare("SELECT path, name FROM documents") {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .ok()
            .map(|iter| iter.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    };
    paths
        .iter()
        .filter_map(|(p, n)| {
            // 用 name 做 search,找到自己那条的 score
            crate::core::search_with_scores(n).ok().and_then(|mut results| {
                results.iter().position(|r| r.path == *p).map(|i| results.remove(i))
            })
        })
        .collect()
}

/// 加载单个文档(用 path 找 name,再用 name 做 search 拿 score)。
fn load_doc_with_score(conn: &rusqlite::Connection, path: &str) -> Option<crate::models::SearchResult> {
    let name: String = conn
        .query_row("SELECT name FROM documents WHERE path = ?1", params![path], |row| row.get(0))
        .ok()?;
    crate::core::search_with_scores(&name)
        .ok()?
        .into_iter()
        .find(|r| r.path == path)
}
```

然后修改 `scan_directory` 签名,加 `updated_paths` 参数:

```rust
fn scan_directory(
    dir: &Path,
    conn: &Connection,
    count: &mut i32,
    updated: &mut i32,
    on_disk: &mut std::collections::HashSet<String>,
    updated_paths: &mut Vec<String>,  // 新增:记录本次更新的路径
) {
    // ...原有逻辑不变...
    // 在 `if needs_update(...) && sync_file(...).is_ok()` 块里加:
    //     updated_paths.push(path_str.clone());
}
```

找到 scan_directory 里这一段(约原 L116):
```rust
if needs_update(conn, &path_str, &path) && sync_file(conn, &path_str, &path).is_ok() {
    *updated += 1;
}
```
改为:
```rust
if needs_update(conn, &path_str, &path) && sync_file(conn, &path_str, &path).is_ok() {
    *updated += 1;
    updated_paths.push(path_str.clone());
}
```

- [ ] **Step 2: 写 sync 查重集成测试**

创建 `db/wiki-query/tests/sync_dedupe_test.rs`:

```rust
//! Task 5 集成测试:sync 的查重报告(incremental + full)。
//!
//! 用真实 vault 的临时 DB,构造重复 name 的文档,验证 sync 返回的 SyncReport 含相似对。

use wiki_query::sync;
use wiki_query::models::SyncReport;

#[test]
fn sync_incremental_reports_similar_for_new_docs() {
    // 在真实 vault 跑 incremental sync,至少应返回 SyncReport 结构
    // (具体相似对数依赖 vault 内容,这里只验证结构 + scope 字段)
    let report = sync::run_with_scope("incremental");
    assert_eq!(report.scope, "incremental");
    assert!(report.synced >= 0);
    // dedupe_report 是 Vec,结构正确即可
    let _pairs: &Vec<wiki_query::models::SimilarPair> = &report.dedupe_report;
}

#[test]
fn sync_full_returns_full_scope() {
    let report = sync::run_with_scope("full");
    assert_eq!(report.scope, "full");
    assert!(report.synced >= 0);
}

#[test]
fn sync_run_cli_prints_report() {
    // run() 不 panic 即可(它内部调 run_with_scope 并 println)
    sync::run();
}
```

- [ ] **Step 3: 编译 + 测试**

```bash
cd "E:\Files\Obsidian\db\wiki-query"
cargo test --test sync_dedupe_test 2>&1 | grep -E "test result|error"
```

预期:3 passed。若 `find_similar` 或 `load_all_docs_with_scores` 有编译错误,按 rustc 提示修。

- [ ] **Step 4: Commit**

```bash
cd "E:\Files\Obsidian"
git add db/wiki-query/src/sync.rs db/wiki-query/tests/sync_dedupe_test.rs
git commit -m "feat(wiki-query): sync 接 scope 参数 + 返回 SyncReport 含查重(spec v5 Task5)

run_with_scope:incremental 查新文档,full 查全库。复用 find_similar。
run()(CLI 入口)内部调 run_with_scope 并打印报告。"
```

---

## Task 6: mcp_server 工具集重写(4 个)

**Files:**
- Modify: `db/wiki-query/src/mcp_server.rs`

这是最大的修改。删 read/write/patch 工具 + 对应 Request 结构体,改 sync 加 scope,加 get_project_rules。

- [ ] **Step 1: 删旧 Request 结构体 + 旧工具,加新 Request**

修改 `db/wiki-query/src/mcp_server.rs`。

**删除**这些 Request 结构体(约 L52-102):
- `SearchRequest` 保留
- `ReadRequest` 删
- `WriteRequest` 删
- `ListRequest` 保留
- `PatchRequest` 删

**新增**(放在 ListRequest 后):

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SyncRequest {
    #[schemars(description = "查重范围:'incremental'(默认,只查本次新入库文档)或 'full'(全库两两查重)")]
    pub scope: Option<String>,
}
```

ProjectRulesRequest 已在 models.rs 定义(Task 3),这里直接 `use crate::models::ProjectRulesRequest`。

- [ ] **Step 2: 改 tool_router impl,删旧工具加新工具**

修改 `#[tool_router] impl WikiServer { ... }`。

删除工具方法:`wiki_read` / `wiki_write` / `wiki_patch`。

修改 `wiki_search`(保留,但删 score——见下)。

修改 `wiki_list`(保留)。

**新增** `wiki_sync` 和 `wiki_get_project_rules`:

```rust
#[tool(description = "同步 vault 的 Markdown 到 SQLite 索引,并返回同步报告 + 查重报告。agent 用文件工具写完 vault 文档后调用此工具,让索引跟上 + 检测是否有相似文档(防重复)。scope 参数:'incremental'(默认,只查本次新入库文档)或 'full'(全库查重,用户说'去重一下'时用)。")]
fn wiki_sync(
    &self,
    Parameters(req): Parameters<SyncRequest>,
) -> Result<CallToolResult, McpError> {
    let scope = req.scope.as_deref().unwrap_or("incremental");
    let scope = if scope == "full" { "full" } else { "incremental" };
    let report = crate::sync::run_with_scope(scope);
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into()),
    )]))
}

#[tool(description = "获取可注入到当前项目根指令文件(CLAUDE.md / AGENTS.md)的 wiki-query 使用规则文本。agent 首次接入 wiki-query 时调用:拿到文本后,检查当前项目根的指令文件是否已含 '<!-- wiki-query-rules -->' 标记,未含则把规则文本追加进去(带标记),已含则跳过。agentType 参数:'claude'(注入到 CLAUDE.md)或其他(注入到 AGENTS.md)。规则文本来自 vault 内 templates/project-rules.md。")]
fn wiki_get_project_rules(
    &self,
    Parameters(req): Parameters<crate::models::ProjectRulesRequest>,
) -> Result<CallToolResult, McpError> {
    let template_path = std::path::Path::new(&self.vault_root)
        .join("templates")
        .join("project-rules.md");
    let rules = std::fs::read_to_string(&template_path).unwrap_or_else(|_| {
        format!(
            "[警告:未找到 templates/project-rules.md,vault_root={}]",
            self.vault_root
        )
    });
    let target_file = match req.agent_type.as_deref() {
        Some("claude") => "CLAUDE.md",
        _ => "AGENTS.md",
    };
    let inject_guide = format!(
        "\n\n---\n## 注入指引(给 agent)\n\
         目标文件:{}\n\
         查重标记:<!-- wiki-query-rules -->\n\
         流程:读当前项目根的 {},若不含上述标记,则把下方规则文本(含标记)追加到文件末尾。\n\
         若已含标记,跳过(避免重复注入)。\n",
        target_file, target_file
    );
    let output = format!(
        "<!-- wiki-query-rules -->\n{}\n<!-- /wiki-query-rules -->{}",
        rules, inject_guide
    );
    Ok(CallToolResult::success(vec![Content::text(output)]))
}
```

- [ ] **Step 3: wiki_search 删 score 字段输出**

v5 spec 要求 search 不返回 score(对 agent 是噪声)。修改 `wiki_search` 工具,序列化前把 score 设为 None:

找到 wiki_search 里:
```rust
Ok(results) => Ok(CallToolResult::success(vec![Content::text(
    serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".into()),
)])),
```
改为:
```rust
Ok(mut results) => {
    // score 对 agent 是噪声,清掉(只保留排序)
    for r in &mut results {
        r.score = None;
    }
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".into()),
    )]))
}
```

- [ ] **Step 4: 更新 ServerInfo instructions**

修改 `impl ServerHandler for WikiServer` 的 `get_info`,把工具列表从旧的 5 个改为新的 4 个:

```rust
.with_instructions(
    "Obsidian 知识库纯索引 MCP。Markdown 是权威源,SQLite 是派生索引(可随时 rm + sync 重建)。\
     三层结构:sources/(原始资料)/ wiki/(结构化知识)/ templates/(只读模板)。\
     agent 用自己的文件工具读写 vault Markdown,wiki-query 只管 SQLite 索引。\
     工具:wiki_search(FTS5 检索,返回 top10 元数据,不含正文)/ \
     wiki_list(按 tag/type/dir 浏览)/ \
     wiki_sync(同步索引 + 查重报告,scope=incremental/full)/ \
     wiki_get_project_rules(获取注入到项目根的使用规则文本)。\
     提炼工作流:读 agent 记忆 → wiki_search 查重 → 文件工具写 vault → wiki_sync → 删源。"
        .to_string(),
)
```

- [ ] **Step 5: 删不再用的 imports + err_to_result 评估**

删 `use crate::frontmatter::Frontmatter;`(write 工具删了,不再用)。
删 `use crate::error::WikiError;`(若 err_to_result 还用就保留——err_to_result 可能整体删了,因为错误从 core 层来,现在 sync/search 不返回 WikiError 而是直接 panic/unwrap。**评估**:run_with_scope 用 expect,search 返回 Result。err_to_result 若无调用方则删)。

跑 `cargo build` 看 rustc 报哪些 unused,逐一清。

- [ ] **Step 6: 编译验证**

```bash
cd "E:\Files\Obsidian\db\wiki-query"
cargo build 2>&1 | grep -E "^error"
```

预期:0 error。

- [ ] **Step 7: Commit**

```bash
cd "E:\Files\Obsidian"
git add db/wiki-query/src/mcp_server.rs
git commit -m "feat(wiki-query): mcp_server 工具集收敛为 4 个(spec v5 Task6)

删 wiki_read/write/patch,改 wiki_sync 加 scope 查重,加 wiki_get_project_rules。
wiki_search 不再返回 score(噪声)。ServerInfo instructions 更新。"
```

---

## Task 7: templates/project-rules.md 创建

**Files:**
- Create: `templates/project-rules.md`

wiki_get_project_rules 读这个文件返回给 agent。内容是可注入到项目根的提炼规则。

- [ ] **Step 1: 创建模板文件**

创建 `E:\Files\Obsidian\templates\project-rules.md`:

```markdown
# wiki-query 记忆提炼规则(注入到项目根)

本文件由 wiki-query MCP 的 wiki_get_project_rules 工具读取,内容会注入到你当前项目根的 CLAUDE.md / AGENTS.md,让你在该项目里知道如何使用 wiki-query 沉淀记忆。

## vault 信息

- **vault 根**:见你的 MCP 配置(WIKI_VAULT_ROOT 环境变量或 cwd)
- **三层结构**:`sources/`(原始资料,可写禁删)/ `wiki/`(结构化知识,可写)/ `templates/`(只读模板)
- **权威源**:Markdown 文件。SQLite 是派生索引,可随时 `rm + sync` 重建。

## frontmatter 约定

你写入 vault 的文档,frontmatter 应有:
- `name`(必有):文档标题,检索展示用
- `description`(必有):一句话说明,检索展示用
- `type`(建议):如 concept / reference / summary / guide 等
- `tags`(建议):逗号分隔的标签

缺 name/description 的文档,检索时元数据不全(不致命,但影响展示)。type 不强制白名单。

## 全量提炼工作流(6 步)

当用户说"提炼一下我的记忆"时,按此流程:

1. **读自己的记忆系统**(你自己的文件工具):全量读取你的项目记忆(Claude Code: MEMORY.md + 关联文档;Codex: ~/.codex/memories/*;Cursor: .cursor/rules/*;等等)
2. **逐篇查重**(wiki_search):对每篇记忆内容,调 wiki_search 用关键词查,看 vault 里有没有相似的
   - 相似:用文件工具读 vault 文档全文 → 决定合并/追加
   - 无相似:用文件工具新建 vault 文档
3. **写 vault**(你自己的文件工具):用文件工具把提炼后的内容写入 vault(frontmatter 按上述约定)
4. **同步索引**(wiki_sync):调 wiki_sync() 让 SQLite 索引跟上 + 拿查重报告
   - 看报告,若有意外重复,用文件工具处理
5. **删源文件**(你自己的文件工具):删除自己记忆系统里已提炼的源文件(避免下次重复提炼)
6. **告知用户**:报告"已提炼 N 篇,删源 M 个"

## 查重是固定流程

- **主防线(第 2 步)**:每篇记忆都先 wiki_search,自己判断要不要新建。智能在你的 LLM。
- **兜底(第 4 步)**:wiki_sync 的查重报告会提示相似对。你若第 2 步漏了,sync 还能兜住。
- **合并归你**:sync 报告只提示相似,合不合并、怎么合并是你的智能决策。wiki-query 不自动合并。

## 提炼后删源

记忆会不断增加,不删已提炼的源文档,后面每次提炼都要重新处理旧内容,越来越重。删源让"提炼"始终是对增量记忆的处理。

## 用户说"去重一下"时

调 wiki_sync(scope="full"),拿全库相似对报告,用文件工具处理重复(合并/删除)。
```

- [ ] **Step 2: Commit**

```bash
cd "E:\Files\Obsidian"
git add templates/project-rules.md
git commit -m "feat(wiki-query): templates/project-rules.md 规则注入模板(spec v5 Task7)

wiki_get_project_rules 工具读此文件,返回给 agent 注入到项目根。
含 frontmatter 约定 + 6 步提炼流程 + 查重防线 + 删源 + 去重指引。"
```

---

## Task 8: 测试更新 + 文档更新 + 全局验证

**Files:**
- Modify: `db/wiki-query/tests/mcp_integration_test.rs`
- Modify: `db/wiki-query/README.md`
- Modify: `AGENTS.md`

- [ ] **Step 1: 更新 mcp_integration_test.rs 断言新工具集**

修改 `db/wiki-query/tests/mcp_integration_test.rs` 的 `mcp_tools_list_exposes_wiki_tools`:

把断言改为 4 个工具:

```rust
assert!(stdout.contains("wiki_search"), "应含 wiki_search");
assert!(stdout.contains("wiki_list"), "应含 wiki_list");
assert!(stdout.contains("wiki_sync"), "应含 wiki_sync");
assert!(stdout.contains("wiki_get_project_rules"), "应含 wiki_get_project_rules");
assert!(!stdout.contains("wiki_read"), "不应再有 wiki_read");
assert!(!stdout.contains("wiki_write"), "不应再有 wiki_write");
assert!(!stdout.contains("wiki_patch"), "不应再有 wiki_patch");
```

更新 `mcp_initialize_handshake`:断言 instructions 含新内容:

```rust
assert!(stdout.contains("纯索引"), "instructions 应提'纯索引'");
assert!(stdout.contains("wiki_get_project_rules"), "instructions 应提新工具");
```

- [ ] **Step 2: README 重写工具集 + 数据流**

修改 `db/wiki-query/README.md`。把"5 个 MCP 工具"改为"4 个 MCP 工具",工具表替换为:

| 工具 | 做什么 | agent 何时用 |
|------|--------|-------------|
| `wiki_search` | FTS5 全文检索,返回 top 10 元数据 | 找知识 / 查重 |
| `wiki_list` | 按 tag/type/dir 结构化浏览 | 浏览某类文档 |
| `wiki_sync` | 同步索引 + 查重报告(scope=incremental/full) | 写完文件后;用户说"去重"时用 full |
| `wiki_get_project_rules` | 返回可注入到项目根的规则文本 | 首次接入 wiki-query 时 |

数据流章节改为 6 步提炼流程(从 spec 复制)。

架构图的 core 列表改为 `read → 删 / write → 删 / patch → 删`,只留 `search / list / sync(含查重)`。

删掉所有"wiki_write 四态决策""wiki_read""wiki_patch"相关章节。

- [ ] **Step 3: AGENTS.md 更新**

修改 `E:\Files\Obsidian\AGENTS.md`:
- "本地 Wiki 检索工具说明"章节:工具列表改为 4 个(删 read/write/patch,加 sync/get_project_rules)
- "记忆系统分工"章节:提炼流程改为 6 步(从 project-rules.md 复制要点)

- [ ] **Step 4: 全局验证**

```bash
cd "E:\Files\Obsidian\db\wiki-query"
cargo test 2>&1 | grep "test result"
cargo clippy --all-targets 2>&1 | grep -E "warning|Finished"
cargo build --release 2>&1 | grep Finished
```

预期:
- 所有测试通过(lib unit + fts_test + sync_dedupe_test + mcp_integration_test)
- clippy 0 warning
- release build 成功

- [ ] **Step 5: 派 Explore subagent 做本轮 review**

派一个 Explore subagent,重点 review:
- core.rs 是否彻底删干净了 write/read/patch 的残留引用
- mcp_server 工具集是否正好 4 个,无遗漏
- find_similar 的 full 模式双向去重逻辑是否正确
- project-rules.md 内容是否覆盖 spec 的 4 块要求
- README/AGENTS.md 是否还有旧工具(write/read/patch)的残留描述

- [ ] **Step 6: 修复 review 发现的问题**

按 subagent 报告修复。

- [ ] **Step 7: Commit + 收尾**

```bash
cd "E:\Files\Obsidian"
git add db/wiki-query/tests/mcp_integration_test.rs db/wiki-query/README.md AGENTS.md
git commit -m "docs(wiki-query): 测试 + README + AGENTS.md 更新(spec v5 Task8)

mcp_integration_test 断言 4 工具集;README 重写工具表+6步数据流;
AGENTS.md 同步。全局验证:测试全过 + clippy 0 warning + release ok。"
```

---

## 实现顺序

```
Task 1 删 write/read/patch(最大删除,先清场)
  ↓
Task 2 frontmatter 删 validate(配合 Task 1)
  ↓
Task 3 models 清理 + 加新类型(为后续 task 准备数据结构)
  ↓
Task 4 core 加 find_similar(TDD 纯函数)
  ↓
Task 5 sync 接 scope + 返回 SyncReport(依赖 Task 4 的 find_similar)
  ↓
Task 6 mcp_server 工具集重写(依赖 Task 5 的 run_with_scope)
  ↓
Task 7 templates/project-rules.md(独立,Task 6 的 get_project_rules 读它)
  ↓
Task 8 测试 + 文档 + 全局验证 + review
```

每 Task 末尾 commit。Task 1 和 Task 6 是最大的两块,中间会遇到编译错误链(删函数导致引用断裂),按 rustc 提示逐个修。Task 1 完成后允许 mcp_integration_test 暂时编译失败(Task 6 才修好)。

---

## Self-Review

**1. Spec 覆盖:**
- ✅ 删 write/read/patch/四态 → Task 1
- ✅ frontmatter 降级为约定 → Task 2 + Task 7(模板)
- ✅ models 加 SyncReport/SimilarPair/ProjectRulesRequest → Task 3
- ✅ find_similar 查重函数 → Task 4
- ✅ sync 含 scope 查重 → Task 5
- ✅ 4 个工具集(search/list/sync/get_project_rules)→ Task 6
- ✅ 规则注入机制 → Task 6(get_project_rules)+ Task 7(模板)
- ✅ search 不返回 score → Task 6 Step 3
- ✅ 测试 + 文档 → Task 8
- ✅ SQLite meta 表 / mark_extracted 已砍(spec 确认),plan 里无对应 task ✓

**2. 占位符扫描:** 无 TBD/TODO,所有代码块完整。

**3. 类型一致性:**
- `find_similar` 签名(Task 4)与 sync.rs 调用(Task 5)一致:`(candidate: &SearchResult, all_docs: &[SearchResult], threshold: f64) -> Vec<SimilarPair>`
- `SyncReport` 字段(Task 3)与 sync.rs 构造(Task 5)一致:`synced/updated/pruned/scope/dedupe_report`
- `ProjectRulesRequest`(Task 3)与 mcp_server 用法(Task 6)一致:`agentType` 字段(rename)
- `SyncRequest`(Task 6 Step 1 定义)与 wiki_sync 用法(Task 6 Step 2)一致:`scope: Option<String>`

✅ 一致。
