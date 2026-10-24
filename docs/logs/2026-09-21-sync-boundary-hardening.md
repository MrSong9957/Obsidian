# 修复日志：sync 仓库边界确认 + 守卫日志脱敏 + 分隔符归一平台感知

日期：2026-09-21 ｜ 仓库：E:\Files\Obsidian ｜ 组件：mcp/wiki-query ｜ 基线：64c27b6

## 底层逻辑

- R1 边界：`run_with_scope` 原守卫只看 cwd 是否含 wiki//sources/，且守卫位置在 init/迁移/扫描之后——错误 cwd（恰为 vault 形状）+ WIKI_DB_PATH 指向真库时照样污染/清理。修复：`confirm_sync_boundary()` 提到一切 DB 访问之前，双判据 = cwd 为仓库形状（与 serve 启动校验同判据）**且** 索引库位于 cwd 之内（相对路径经 `..` 解析后须仍位于 cwd 内；绝对路径做分隔符/`..`归一 + Windows 忽略大小写的前缀比较）。任一不成立 → failed=-1 整体拒绝，不 init（不产生 WAL 文件）、不迁移、不扫描、不 prune，三阶段边界一致。
- R2 日志：原守卫警告打印 `cwd={:?}` 绝对路径；拒绝文案重写为无路径 + 保留排查指引（仓库根/WIKI_DB_PATH）。
- R3 FTS 缺口：**不成立，原发现关闭**。`needs_update`（sync.rs，ff3c492 引入）对 documents 有行而 FTS 缺行的文档强制重写，不依赖 mtime；迁移在扫描之前执行 → 迁移（含 dup 分支）或历史中断（ce64184 之前无事务时代）留下的缺口在同一轮 sync 内自愈。测试实证：人工构造缺口与跨平台 dup 残迹后，文件内容/mtime 不变，检索恢复命中。
- R4 平台范围：Unix 属支持范围（serve.sh/build.sh 按 Git Bash/macOS/Linux 设计，WSL 除外是有意决策）。修复：分隔符归一改为按 `MAIN_SEPARATOR` 语义——Windows 替换 '\'→'/'；Unix 原样保留（'\' 是合法文件名字符，Path 输出本就是 '/'）。迁移/prune/入库三处同口径；Unix 下迁移的 `LIKE '%\%'` 虽会扫到合法 '\' 文件名，但 `new==old` 跳过，无副作用。

## TDD 测试点

新文件 `tests/sync_boundary_test.rs` + `src/sync.rs` 单元测试：

1. 错误 cwd（vault 形状）+ 外部真库 → 拒绝（failed=-1/synced=0），真库 2 行原样、无 evil 行、错误 repo 不建 db/（RED）
2. vault 形状 cwd + 库在 cwd 外 → 拒绝，外部库不得被污染或 prune（RED）
3. 库在 cwd 内但无 wiki//sources/ → 拒绝，DB 文件不被创建（RED）
4. 子进程 stderr：含"拒绝同步"+指引词，不含 cwd/DB 绝对路径；stdout 汇总含 failed=-1（RED）
5. R3 证明×2：中断型缺口自愈（updated=1、检索命中、modified_at 不变）；dup+FTS 仅旧行 → 收敛单行可检索（证明性测试，锁行为）
6. R4 单元：`normalize_separators_with` Windows 语义替换/Unix 语义原样；`is_path_under_cwd` 子目录/同级前缀/`..` 逃逸/大小写

## 失败原因（RED 实录）

`cargo test --test sync_boundary_test`（实现前）：4 failed / 2 passed——
- 三个边界拒绝用例 failed=0（实际照常 init/扫描/迁移）：vault 形状错误 cwd 把 `wiki/evil.md` 写进真库并按错误扫描集 prune；库在 cwd 外同被污染。
- stderr 用例:守卫警告回显 `cwd={:?}` 绝对路径，且无 `failed=-1` 汇总。
- 2 passed 为 R3 证明性测试（FTS 自愈已存在），RED 阶段即绿，符合"锁行为"定位。
- 首轮断言缺陷 1 例：CLI 汇总语序为 `-1 failed`，测试误找 `failed=-1`，修正断言后全绿（非实现问题）。

## 第二轮收口（同日，两个验收缺口）

### 缺口 1：正常 MCP 同步无真实业务结果证明

**核实（启动链证据）**：
- 受支持的同步启动方式 = 仓库根 `.mcp.json`：`{"command":"bash","args":["mcp/wiki-query/serve.sh"],"cwd":"."}`（serve.sh 头注释同款）——进程 cwd 固定为 vault 根，边界确认放行。
- 用户级 config.json 为 `command=exe + env WIKI_VAULT_ROOT`、**无 cwd**：服务器 cwd 继承 CLI 启动目录。sync 历史上从不消费 WIKI_VAULT_ROOT（只被 check_scope/get_project_rules 消费），非 vault cwd 下的 sync 正是已发生过的"错误 cwd 整库 prune"事故形态——拒绝它是修复目标而非禁止有效配置；该形态下 check_scope 等只读工具不受影响。
- 旧 e2e wiki_sync 测试（cwd=mcp/wiki-query，非 vault）实际走的是拒绝路径，但原断言只查字段名，failed=-1 也通过。

**修复（测试层）**：新文件 `tests/mcp_sync_e2e_test.rs`——
- `mcp_sync_indexes_temp_files_and_search_hits`（正常业务路径）：受支持形态（cwd=temp vault、库在 cwd 内）spawn 真实 serve 子进程，经 wiki_sync 工具同步临时文件，断言 `"failed": 0` 且 `"synced": 2`（区分成功/拒绝）；再开独立会话经 wiki_search 以**正文唯一词"屠龙刀"**（证明 FTS content 列）与**描述唯一词"玄铁令"**（证明元数据列）检索命中 `wiki/mcp.md`；另直查 DB 复核两表落库。
- `mcp_sync_from_non_vault_cwd_refuses_and_leaves_db_intact`（错误路径对照）：非 vault cwd + 库指向目标仓库 → `"failed": -1` 且目标库 doc_paths 原样。
- 旧 e2e 5 个 wiki_sync 测试补 `"failed": -1` 断言，显式锁定拒绝契约，与新正常路径形成对照。

**过程中发现（重要，非 wiki-query 缺陷）**：rmcp **并发处理 tool calls**（响应乱序实测 `id:1→32→31→30`），同一会话串行发送的 sync 与 search 之间无顺序保证——首轮业务断言失败（search 空/半成品）即此因。有依赖的调用必须拆会话，与真实使用一致。已写入测试注释。

**断言写法注意**：report JSON 作为 MCP content 内嵌字符串被转义（`\"failed\": 0`），contains 断言须用转义形态。

### 缺口 2：`../` 相对路径逃逸

**核实结论：实现已正确，设计表述有误，以证据关闭。** `confirm_sync_boundary` 对相对路径 `cwd.join` 后经 `normalize_path` 解析 `..` 再做前缀比较——`../sibling/db/wiki.db` 解析后落在 cwd 外 → 拒绝。新增集成测试 `relative_db_path_with_dotdot_escape_refused` 实证（RED 前置为测试自身漏建 `sibling/wiki` 目录，修正后绿）：拒绝、目标库逻辑内容原样、cwd 下无 db 生成。配套补 `relative_db_path_inside_cwd_allowed`（`data/nested/wiki.db` 普通相对路径放行）防误伤。绝对路径两形态已有覆盖：在内（既有全部 setup_vault 系测试）、在外（`vault_shaped_cwd_with_db_outside_cwd_refused`）。

**日志更正**：第一轮"相对路径必然满足（cwd 内）"的设计表述不准确，实现语义为"相对路径经 `..` 解析后须仍位于 cwd 内"。

### 外置数据库支持约定核实

`WIKI_DB_PATH` 的设计用途（db.rs 注释，review Important-4）是测试隔离与运维显式指定，且"非默认路径打日志防生产误设指向错误库"。边界确认允许库在 cwd 内的任意形态（默认相对/子目录相对/绝对），仅拒绝"库在 cwd 之外"的跨仓库形态——与既有"防误设"意图一致，未悄悄禁止任何原有有效配置。

## 失败原因（RED 实录，第二轮）

- 业务测试首轮失败 2 例：①断言未按 MCP 内嵌 JSON 转义形态书写；②同一会话内 sync+search 被 rmcp 并发执行（见上）。均为测试编排/断言缺陷，wiki-query 实现无改动。
- `..` 逃逸测试首轮失败 1 例：测试自身漏建 `sibling/wiki` 目录（fs::write 不建父目录）。实现行为自始正确（设计表述有误已更正）。

## 验证

- `cargo test` 全量：**190 passed / 0 failed**（上轮 186 + 本轮新增 mcp_sync_e2e 2 + sync_boundary 2）。实现代码本轮零改动（两缺口均为测试/文档层收口）。
- `bash tests/mcp-config.test.sh`：9/10，唯一失败 T7b 为 `1a718e9` 存量（AGENTS.md 构建说明），边界外未动。
- `mcp_sync_e2e_test + sync_boundary_test` 复跑 5 次全绿。
- 上轮偶发失败改记：**原因未明**（当时详情未捕获，无法归因环境/实现，留证如下：耗时 4.09s 显著长于常态 1.6s，单例，复跑不现）。本轮必要验证（MCP e2e 5 连跑）中未再出现；若再现将保留完整输出定位，不以重复跑绿代替。
- 真实库零触碰：全部验证使用 tempfile 仓库 + 独立 DB。
- 未提交改动保持原样：`wiki/_index.md`、`wiki/Docker容器开发方案.md`（用户改动）。未 commit、未推送。

## 第三轮：最终交付收口（同日，独立 review + 修复 + 终验）

**验证入口说明**：仓库未定义名为 verify:full/L3 的脚本入口（根文档与 README 均无）；按历次修复日志的既定口径执行 `cargo test` 全量 + `bash mcp/wiki-query/tests/mcp-config.test.sh`。

**独立 code-review**（子代理隔离审最终 diff，并实跑三个测试套件复核）：结论"可交付"，2 Medium + 2 Low 在范围内，已全部修复：
- [Medium] `refusal_output_guides_without_absolute_path` 子进程测试用 `current_dir().join("target/...")` 定位 exe 且不持锁，与本二进制内切 cwd 的测试并行有竞态 → 改用 cargo 注入的 `env!("CARGO_BIN_EXE_wiki-query")` 绝对路径。
- [Medium] `confirm_sync_boundary` 主文档残留已推翻的"相对路径必然满足"表述（实现实际解析 `..`，测试注释与日志均已更正但源码未改）→ 改为"`..` 解析后须仍在 cwd 内"，并将"与 serve 启动校验同判据"精确化（serve 可受 WIKI_VAULT_ROOT 指向他处且用 exists；sync 仅看 cwd 且要求 is_dir，更严格）。
- [Low] `wiki_sync_report_has_dedupe_report`(id:34) 补拒绝契约断言（scope 系 5 个 wiki_sync e2e 现全部锁定 failed=-1）。
- [Low] `SyncReport.failed` 契约文档补 -1 语义（-1 = 整体失败/被拒，索引未触碰）。
- [Info 记录不修]：MCP 拒绝时原因仅走 stderr、agent 只见 failed=-1（行为增强，超本轮范围）；is_path_under 为文本比对不解析 symlink、macOS 大小写盘可能误拒（对误配置防护威胁模型可接受）。

**T7b 存量归因（完整证据链）**：T7b 断言 AGENTS.md 含 `bash mcp/wiki-query/build.sh`；`1a718e9`（2026-09-10 "精简 AGENTS.md"）恰好把该串从 1 处删至 0 处（`git show 1a718e9~1:AGENTS.md | grep -c` = 1，`git show 1a718e9:AGENTS.md | grep -c` = 0）；工作树 AGENTS.md 与 HEAD 无差异（`git diff HEAD -- AGENTS.md` 为空），本任务变更文件均非 T7b 检查对象 → 与本任务无关的存量失败。本轮完整输出 10 项：9 PASS / 1 FAIL(T7b)。

**最终验证（review 修复后）**：`cargo test` 全量 190 passed / 0 failed；`mcp-config.test.sh` 9/10（唯一非绿 T7b，归因见上）。

## 交付清单（最终 diff 范围）

| 文件 | 性质 |
|---|---|
| mcp/wiki-query/src/sync.rs | 边界确认前置 + 日志脱敏 + 平台感知归一 + 注释更正（第一轮实现 + review 修正） |
| mcp/wiki-query/src/models.rs | SyncReport.failed 契约文档补 -1 语义（review 修正，+2 行） |
| mcp/wiki-query/tests/sync_boundary_test.rs | 新增 9 用例（边界/日志/平台归一/FTS 自愈证明/路径三形态） |
| mcp/wiki-query/tests/mcp_sync_e2e_test.rs | 新增 2 用例（MCP 入口真实业务同步+检索 / 拒绝对照） |
| mcp/wiki-query/tests/mcp_e2e_full_test.rs | scope 系 5 个 wiki_sync e2e 补拒绝契约断言 |
| logs/2026-09-21-sync-boundary-hardening.md | 本日志 |

用户文件 `wiki/_index.md`、`wiki/Docker容器开发方案.md` 为用户自有改动，未触碰、不纳入本任务。未 commit、未 push。
