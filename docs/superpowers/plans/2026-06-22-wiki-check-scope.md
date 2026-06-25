# wiki_check_scope Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `wiki_check_scope` MCP tool that determines whether a file path is within the wiki's managed scope (`sources/`, `wiki/`, `templates/`), enabling AI to avoid redundant distillation of files already inside the knowledge base.

**Architecture:** Add `is_in_wiki_scope` pure function in `sync.rs` for path scope checking, expose it as the 5th MCP tool `wiki_check_scope` in `mcp_server.rs`, add `CheckScopeRequest` request struct with schemars JSON Schema support, return structured JSON response with `in_scope`, `scope_dir`, and `suggestion` fields.

**Tech Stack:** Rust, rmcp (MCP server framework), serde/schemars (JSON Schema), rusqlite (existing)

## Global Constraints

- Follow existing code style: Chinese comments, snake_case functions, PascalCase types
- Path normalization must handle Windows backslashes (`\`) and Unix forward slashes (`/`)
- All paths use forward slashes internally (matching existing `sync.rs` convention)
- New tool must integrate with existing `#[tool_router]` macro system
- TDD: write failing tests first, then implement

---

### Task 1: Add `is_in_wiki_scope` function to `sync.rs`

**Files:**
- Modify: `db/wiki-query/src/sync.rs` — add `is_in_wiki_scope` and `normalize_path` functions
- Modify: `db/wiki-query/tests/sync_edge_cases_test.rs` — add scope checking tests

**Interfaces:**
- Consumes: none (pure function)
- Produces: `pub fn is_in_wiki_scope(path: &str) -> bool` — later used by Task 2's MCP tool

- [ ] **Step 1: Write failing tests for `is_in_wiki_scope`**

Add to `db/wiki-query/tests/sync_edge_cases_test.rs`:

```rust
// ============================================================
// is_in_wiki_scope 测试
// ============================================================

mod scope_tests {
    use super::*;

    #[test]
    fn in_scope_sources() {
        assert!(wiki_query::sync::is_in_wiki_scope("sources/dev/rust.md"));
    }

    #[test]
    fn in_scope_wiki() {
        assert!(wiki_query::sync::is_in_wiki_scope("wiki/claude-code.md"));
    }

    #[test]
    fn in_scope_templates() {
        assert!(wiki_query::sync::is_in_wiki_scope("templates/wiki.md"));
    }

    #[test]
    fn in_scope_sources_subdir() {
        assert!(wiki_query::sync::is_in_wiki_scope("sources/agent/agent-arch-p1.md"));
    }

    #[test]
    fn out_scope_claude_memory() {
        assert!(!wiki_query::sync::is_in_wiki_scope(
            ".claude/projects/E--Files-Obsidian/memory/rust-gnu-toolchain-setup.md"
        ));
    }

    #[test]
    fn out_scope_root_file() {
        assert!(!wiki_query::sync::is_in_wiki_scope("CLAUDE.md"));
    }

    #[test]
    fn out_scope_git() {
        assert!(!wiki_query::sync::is_in_wiki_scope(".git/config"));
    }

    #[test]
    fn path_with_leading_dot_slash() {
        assert!(wiki_query::sync::is_in_wiki_scope("./sources/dev/rust.md"));
    }

    #[test]
    fn path_with_leading_slash() {
        assert!(wiki_query::sync::is_in_wiki_scope("/sources/dev/rust.md"));
    }

    #[test]
    fn path_with_backslashes() {
        assert!(wiki_query::sync::is_in_wiki_scope("sources\\dev\\rust.md"));
    }

    #[test]
    fn path_with_vault_root_prefix() {
        assert!(wiki_query::sync::is_in_wiki_scope(
            "E:\\Files\\Obsidian\\sources\\dev\\rust.md",
            "E:\\Files\\Obsidian"
        ));
    }

    #[test]
    fn empty_path() {
        assert!(!wiki_query::sync::is_in_wiki_scope(""));
    }

    #[test]
    fn exact_scope_dir() {
        assert!(wiki_query::sync::is_in_wiki_scope("sources"));
        assert!(wiki_query::sync::is_in_wiki_scope("wiki"));
        assert!(wiki_query::sync::is_in_wiki_scope("templates"));
    }

    #[test]
    fn path_traversal_not_in_scope() {
        // ../etc/passwd doesn't start with sources/ wiki/ or templates/
        assert!(!wiki_query::sync::is_in_wiki_scope("../etc/passwd"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd E:\Files\Obsidian\db\wiki-query && cargo test --test sync_edge_cases_test scope_tests 2>&1`
Expected: FAIL with "function `is_in_wiki_scope` not found in `wiki_query::sync`"

- [ ] **Step 3: Implement `normalize_path` and `is_in_wiki_scope`**

Add to `db/wiki-query/src/sync.rs` (at the end, after existing functions):

```rust
/// 判断文件路径是否在 wiki 管辖范围内(sources/ wiki/ templates/)。
///
/// AI 提炼记忆时调用:在范围内 → 跳过(已是 wiki 的一部分);
/// 不在范围内 → 需要提炼到 wiki。
///
/// 路径规范化:去除 vault root 前缀、统一为正斜杠、去除前导 ./ 或 /。
pub fn is_in_wiki_scope(path: &str) -> bool {
    let normalized = normalize_path(path);
    if normalized.is_empty() {
        return false;
    }
    normalized.starts_with("sources/")
        || normalized.starts_with("wiki/")
        || normalized.starts_with("templates/")
        || normalized == "sources"
        || normalized == "wiki"
        || normalized == "templates"
}

/// 规范化路径:统一为正斜杠,去除前导 ./ 和 /。
fn normalize_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let normalized = normalized.strip_prefix("./").unwrap_or(&normalized);
    let normalized = normalized.strip_prefix('/').unwrap_or(normalized);
    // 去除尾部斜杠(但保留文件名)
    let normalized = normalized.trim_end_matches('/');
    normalized.to_string()
}

/// 去除路径中的 vault root 前缀(如果路径以 vault_root 开头)。
/// 返回去除前缀后的相对路径(正斜杠,无前导 /)。
pub fn strip_vault_root(path: &str, vault_root: &str) -> String {
    let normalized_path = path.replace('\\', "/");
    let normalized_root = vault_root.replace('\\', "/");
    
    // 尝试去除 vault root 前缀
    let stripped = normalized_path
        .strip_prefix(&normalized_root)
        .unwrap_or(&normalized_path);
    
    // 去除前导分隔符
    let stripped = stripped.strip_prefix('/').unwrap_or(stripped);
    stripped.to_string()
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd E:\Files\Obsidian\db\wiki-query && cargo test --test sync_edge_cases_test scope_tests 2>&1`
Expected: All 14 scope tests PASS

- [ ] **Step 5: Run full test suite to verify no regressions**

Run: `cd E:\Files\Obsidian\db\wiki-query && cargo test 2>&1`
Expected: All tests PASS (existing 102 + new 14 = 116)

- [ ] **Step 6: Commit**

```bash
cd E:\Files\Obsidian\db\wiki-query
git add src/sync.rs tests/sync_edge_cases_test.rs
git commit -m "feat(sync): add is_in_wiki_scope path scope checking function"
```

---

### Task 2: Add `CheckScopeRequest` struct to `models.rs`

**Files:**
- Modify: `db/wiki-query/src/models.rs` — add request struct
- Modify: `db/wiki-query/tests/core_edge_cases_test.rs` — add serialization test

**Interfaces:**
- Consumes: none
- Produces: `CheckScopeRequest` struct — used by Task 3's MCP tool

- [ ] **Step 1: Write failing test for `CheckScopeRequest` serialization**

Add to `db/wiki-query/tests/core_edge_cases_test.rs`:

```rust
#[test]
fn check_scope_request_deserializes() {
    let json = r#"{"path": "sources/dev/rust.md"}"#;
    let req: wiki_query::models::CheckScopeRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.path, "sources/dev/rust.md");
}

#[test]
fn check_scope_request_with_vault_root() {
    let json = r#"{"path": "E:\\Files\\Obsidian\\sources\\dev\\rust.md", "vault_root": "E:\\Files\\Obsidian"}"#;
    let req: wiki_query::models::CheckScopeRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.vault_root, Some("E:\\Files\\Obsidian".to_string()));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd E:\Files\Obsidian\db\wiki-query && cargo test --test core_edge_cases_test check_scope 2>&1`
Expected: FAIL with "CheckScopeRequest not found"

- [ ] **Step 3: Add `CheckScopeRequest` to `models.rs`**

Add to `db/wiki-query/src/models.rs` (after `ProjectRulesRequest`):

```rust
/// wiki_check_scope 的请求参数。
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CheckScopeRequest {
    /// 要检查的文件路径(相对路径或绝对路径均可)。
    #[schemars(description = "要检查的文件路径(如 'sources/dev/rust.md' 或 '.claude/projects/xxx/memory/file.md')。")]
    pub path: String,
    /// 可选:vault 根路径。如果传入,会先去除路径中的 vault root 前缀再判断。
    #[schemars(description = "可选:vault 根路径(如 'E:\\Files\\Obsidian')。传入后会自动去除路径中的 vault root 前缀再判断。")]
    pub vault_root: Option<String>,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd E:\Files\Obsidian\db\wiki-query && cargo test --test core_edge_cases_test check_scope 2>&1`
Expected: All check_scope tests PASS

- [ ] **Step 5: Commit**

```bash
cd E:\Files\Obsidian\db\wiki-query
git add src/models.rs tests/core_edge_cases_test.rs
git commit -m "feat(models): add CheckScopeRequest struct for wiki_check_scope tool"
```

---

### Task 3: Add `wiki_check_scope` MCP tool to `mcp_server.rs`

**Files:**
- Modify: `db/wiki-query/src/mcp_server.rs` — add tool implementation
- Modify: `db/wiki-query/tests/mcp_e2e_full_test.rs` — add end-to-end tests

**Interfaces:**
- Consumes: `sync::is_in_wiki_scope`, `sync::strip_vault_root`, `models::CheckScopeRequest`
- Produces: 5th MCP tool `wiki_check_scope` callable via JSON-RPC

- [ ] **Step 1: Write failing end-to-end tests**

Add to `db/wiki-query/tests/mcp_e2e_full_test.rs`:

```rust
// ============================================================
// wiki_check_scope 工具测试
// ============================================================

#[test]
fn wiki_check_scope_in_scope_sources() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":70,"method":"tools/call","params":{"name":"wiki_check_scope","arguments":{"path":"sources/dev/rust.md"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":70"#), "应返回 id:70");
    assert!(stdout.contains("true"), "sources/ 下应返回 in_scope=true: {}", &stdout[..stdout.len().min(500)]);
    assert!(stdout.contains("sources"), "应指明 scope_dir=sources");
}

#[test]
fn wiki_check_scope_in_scope_wiki() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":71,"method":"tools/call","params":{"name":"wiki_check_scope","arguments":{"path":"wiki/claude-code.md"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":71"#));
    assert!(stdout.contains("true"), "wiki/ 下应返回 in_scope=true");
}

#[test]
fn wiki_check_scope_out_scope() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":72,"method":"tools/call","params":{"name":"wiki_check_scope","arguments":{"path":".claude/projects/E--Files-Obsidian/memory/rust-gnu-toolchain-setup.md"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":72"#));
    assert!(stdout.contains("false"), ".claude/ 下应返回 in_scope=false: {}", &stdout[..stdout.len().min(500)]);
}

#[test]
fn wiki_check_scope_with_vault_root() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":73,"method":"tools/call","params":{"name":"wiki_check_scope","arguments":{"path":"E:\\Files\\Obsidian\\sources\\dev\\rust.md","vault_root":"E:\\Files\\Obsidian"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":73"#));
    assert!(stdout.contains("true"), "带 vault_root 前缀应正确判断: {}", &stdout[..stdout.len().min(500)]);
}

#[test]
fn wiki_check_scope_empty_path() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":74,"method":"tools/call","params":{"name":"wiki_check_scope","arguments":{"path":""}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":74"#));
    assert!(stdout.contains("false"), "空路径应返回 in_scope=false");
}

#[test]
fn wiki_check_scope_suggestion_text() {
    let (init, notif) = mcp_handshake();
    let call = r#"{"jsonrpc":"2.0","id":75,"method":"tools/call","params":{"name":"wiki_check_scope","arguments":{"path":"sources/dev/rust.md"}}}"#;
    let stdout = run_mcp_session(&[init, notif, call]);
    assert!(stdout.contains(r#""id":75"#));
    // 在范围内的建议应提示"不需要提炼"
    assert!(stdout.contains("不需要提炼") || stdout.contains("跳过"), "应提示不需要提炼: {}", &stdout[..stdout.len().min(500)]);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd E:\Files\Obsidian\db\wiki-query && cargo test --test mcp_e2e_full_test wiki_check_scope 2>&1`
Expected: FAIL — tool not found (wiki_check_scope not registered)

- [ ] **Step 3: Add `wiki_check_scope` tool to `mcp_server.rs`**

Add the request struct and tool implementation. Insert after the `SyncRequest` struct (around line 59):

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CheckScopeRequest {
    #[schemars(description = "要检查的文件路径(如 'sources/dev/rust.md' 或 '.claude/projects/xxx/memory/file.md')。")]
    pub path: String,
    #[schemars(description = "可选:vault 根路径(如 'E:\\Files\\Obsidian')。传入后会自动去除路径中的 vault root 前缀再判断。")]
    pub vault_root: Option<String>,
}
```

Add the tool method inside the `#[tool_router] impl WikiServer` block (after `wiki_get_project_rules`, around line 154):

```rust
    #[tool(description = "判断文件路径是否在 wiki 管辖范围内(sources/ wiki/ templates/)。AI 提炼记忆前调用:在范围内 → 跳过(已是 wiki 的一部分);不在范围内 → 需要提炼。支持相对路径和绝对路径,自动处理 Windows/Unix 路径分隔符差异。")]
    fn wiki_check_scope(
        &self,
        Parameters(req): Parameters<CheckScopeRequest>,
    ) -> Result<CallToolResult, McpError> {
        // 如果提供了 vault_root,先去除前缀
        let path_to_check = match &req.vault_root {
            Some(root) => crate::sync::strip_vault_root(&req.path, root),
            None => req.path.clone(),
        };

        let in_scope = crate::sync::is_in_wiki_scope(&path_to_check);

        let (scope_dir, suggestion) = if in_scope {
            // 找出匹配的 scope 目录
            let dir = if path_to_check.starts_with("sources/") || path_to_check == "sources" {
                "sources"
            } else if path_to_check.starts_with("wiki/") || path_to_check == "wiki" {
                "wiki"
            } else {
                "templates"
            };
            (
                Some(dir.to_string()),
                "文件已在 wiki 范围内,不需要提炼。如果内容需要更新,直接编辑即可。".to_string(),
            )
        } else {
            (
                None,
                "文件不在 wiki 范围内,需要提炼到 sources/ 或 wiki/ 目录。".to_string(),
            )
        };

        let result = serde_json::json!({
            "in_scope": in_scope,
            "path": path_to_check,
            "scope_dir": scope_dir,
            "suggestion": suggestion,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".into()),
        )]))
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd E:\Files\Obsidian\db\wiki-query && cargo test --test mcp_e2e_full_test wiki_check_scope 2>&1`
Expected: All 6 wiki_check_scope tests PASS

- [ ] **Step 5: Run full test suite**

Run: `cd E:\Files\Obsidian\db\wiki-query && cargo test 2>&1`
Expected: All tests PASS (no regressions)

- [ ] **Step 6: Commit**

```bash
cd E:\Files\Obsidian\db\wiki-query
git add src/mcp_server.rs tests/mcp_e2e_full_test.rs
git commit -m "feat(mcp): add wiki_check_scope tool for path scope checking"
```

---

### Task 4: Update documentation and tool count

**Files:**
- Modify: `db/wiki-query/src/mcp_server.rs` — update doc comment (4 tools → 5 tools)
- Modify: `db/wiki-query/src/lib.rs` — update module doc if present
- Modify: `E:\Files\Obsidian\db\wiki-query\README.md` — add wiki_check_scope to tools table
- Modify: `E:\Files\Obsidian\CLAUDE.md` — update MCP tools list

**Interfaces:**
- Consumes: all previous tasks completed
- Produces: documentation consistent with implementation

- [ ] **Step 1: Update `mcp_server.rs` module doc comment**

Change line 3-4 from:
```rust
//! v5 重构:纯索引服务。工具集 4 个:
//!   wiki_search(FTS5 检索)/ wiki_list(结构化浏览)/
//!   wiki_sync(索引同步 + 查重报告)/ wiki_get_project_rules(规则注入)。
```
To:
```rust
//! v5 重构:纯索引服务。工具集 5 个:
//!   wiki_search(FTS5 检索)/ wiki_list(结构化浏览)/
//!   wiki_sync(索引同步 + 查重报告)/ wiki_get_project_rules(规则注入)/
//!   wiki_check_scope(路径范围判断)。
```

- [ ] **Step 2: Update `mcp_server.rs` `get_info` instructions**

In the `get_info()` method (around line 167), add wiki_check_scope to the tools description:

Change:
```
工具:wiki_search(FTS5 检索,返回 top10 元数据,不含正文)/ \
wiki_list(按 tag/type/dir 浏览)/ \
wiki_sync(同步索引 + 查重报告,scope=incremental/full)/ \
wiki_get_project_rules(获取注入到项目根的使用规则文本)。
```
To:
```
工具:wiki_search(FTS5 检索,返回 top10 元数据,不含正文)/ \
wiki_list(按 tag/type/dir 浏览)/ \
wiki_sync(同步索引 + 查重报告,scope=incremental/full)/ \
wiki_get_project_rules(获取注入到项目根的使用规则文本)/ \
wiki_check_scope(判断文件是否在 wiki 范围内,sources/wiki/templates)。
```

- [ ] **Step 3: Update README.md tools table**

In `E:\Files\Obsidian\db\wiki-query\README.md`, add to the tools table:

```markdown
| `wiki_check_scope` | 判断文件是否在 wiki 范围内 | AI 提炼记忆前，先判断要不要提炼 |
```

- [ ] **Step 4: Update CLAUDE.md MCP tools list**

In `E:\Files\Obsidian\CLAUDE.md`, update the MCP tools section (around the "4 个 MCP 工具" heading) to mention 5 tools and add wiki_check_scope.

- [ ] **Step 5: Run full test suite**

Run: `cd E:\Files\Obsidian\db\wiki-query && cargo test 2>&1`
Expected: All tests PASS

- [ ] **Step 6: Commit**

```bash
cd E:\Files\Obsidian\db\wiki-query
git add src/mcp_server.rs
cd E:\Files\Obsidian
git add db/wiki-query/README.md CLAUDE.md
git commit -m "docs: update tool count to 5, add wiki_check_scope documentation"
```

---

### Task 5: Integration verification

**Files:**
- None (verification only)

- [ ] **Step 1: Build release binary**

Run: `cd E:\Files\Obsidian\db\wiki-query && cargo build --release 2>&1`
Expected: Successful build

- [ ] **Step 2: Verify tools/list returns 5 tools**

Run manually via MCP or check test output from Task 3.

- [ ] **Step 3: Verify real-world scenario**

Test with actual path:
```
wiki_check_scope(path=".claude/projects/E--Files-Obsidian/memory/rust-gnu-toolchain-setup.md")
→ in_scope: false, suggestion: "需要提炼"

wiki_check_scope(path="sources/dev/rust.md")
→ in_scope: true, scope_dir: "sources", suggestion: "不需要提炼"
```

- [ ] **Step 4: Run full test suite one final time**

Run: `cd E:\Files\Obsidian\db\wiki-query && cargo test 2>&1`
Expected: All tests PASS
