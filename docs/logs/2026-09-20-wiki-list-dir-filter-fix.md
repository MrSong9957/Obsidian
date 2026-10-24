# 修复日志：wiki_list(dir) Windows 路径过滤失效

日期：2026-09-20 ｜ 仓库：E:\Files\Obsidian ｜ 组件：mcp/wiki-query

## 底层逻辑

- sync 入库时 `scan_directory` 只做 `strip_prefix("./")`（sync.rs L317），Windows 上 path 以 `\` 存入 SQLite（如 `wiki\_index.md`）。
- `core::list` 的 dir 过滤是 `path LIKE 'wiki/%'`（core.rs L201-204 修复前），分隔符不匹配 → 前缀永远对不上 → 静默返回空。
- 修复：比较时两侧统一分隔符——SQL 端 `REPLACE(path, '\', '/')` 归一化 DB 旧数据（含混合格式），Rust 端 `d.replace('\\', "/")` 归一化 dir 参数。不删库、不重建索引，旧数据直接兼容。
- 前缀语义不变：`sources/dev/` → `LIKE 'sources/dev/%'`，精确到路径段，不混入 `sources/development/`。

## TDD 测试点

新文件 `mcp/wiki-query/tests/list_dir_filter_test.rs`（16 用例，GLOBAL_TEST_MUTEX + WIKI_DB_PATH 临时库模式，沿用 sync_edge_cases_test.rs 惯例）：

1. 反斜杠 DB × dir `wiki/`（修复前 RED）
2. dir `wiki/` 与 `wiki\` 结果一致（RED）
3. dir `sources/dev/` 命中深层子目录（RED）
4. 混合分隔符 DB 全命中（RED）
5. 回归保护：正斜杠 DB 行为不变、`sources/dev/` 不混入相邻目录、无 dir 返回全部

## 失败原因（RED 实录）

`cargo test --test list_dir_filter_test`：4 failed / 4 passed——反斜杠路径 × 正斜杠 dir 全部落空（返回 `[]`），与真实库 `wiki_list(dir:"wiki/")` 返回空的复现一致。其中 1 例初判失败系测试内期望向量未按字节序排序（`\` 后 `_`(0x5F) < `a`(0x61)），修正测试后转绿，非实现问题。

## 验证

- `cargo test` 全量：133 passed / 0 failed（列表过滤 16/16；e2e 31/31）。首次全量曾现 3 个 `wiki_sync_*` 失败，单独运行及重跑全量均通过，原因待查，与本次改动无调用关系。
- `bash tests/mcp-config.test.sh`：9/10。T7b（AGENTS.md 构建说明）为存量失败：`1a718e9` 文档精简移除了 `bash mcp/wiki-query/build.sh`，早于本任务，边界外未修。
- 构建：`bash build.sh` 编译成功；exe 被运行中 MCP 锁定，用改名法完成部署（新 exe 已就位，旧 `.old` 已删）。
- 真实库 stdio 实测（新二进制，cwd=vault 根）：`wiki/`=94、`wiki\`=94、`sources/dev/`=4、`sources\dev\`=4，与 SQL 基准（REPLACE 归一化直查 db/wiki.db）逐项一致。
- 会话内 `mcp__wiki__wiki_list(dir:"wiki/")` 仍返回 `[]`：本会话 MCP 进程为旧代码，**待下次 MCP 进程重启加载新 exe 后生效**。

## 改动文件

- `mcp/wiki-query/src/core.rs`：dir 过滤 4 行（+注释 2 行）
- `mcp/wiki-query/tests/list_dir_filter_test.rs`：新增 16 用例

## 遗留

1. ~~会话内 MCP 重启前仍是旧行为~~ 2026-09-20 重启 ZCode 后实测：`wiki/` 与 `wiki\` 返回一致完整列表，`sources/dev/` 与 `sources\dev\` 一致返回 4 条，与索引基准（wiki=94、sources/dev=4）吻合，修复生效。
2. T7b 存量失败待另行裁决（恢复 AGENTS.md 构建说明或改测试断言）。
3. e2e sync 测试曾偶发失败（见验证段），原因待查，未在本任务范围。
4. `git status` 中 `wiki/Rust 程序设计语言/` 为外部新增目录，非本任务产物，未触碰。
