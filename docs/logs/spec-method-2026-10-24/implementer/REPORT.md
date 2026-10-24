# 实现报告：短链接服务（spec-method-2026-10-24）

- 状态：**待验收**（仅实现方自检，不构成验收结论）
- 实现会话：2026-10-24（独立会话，仅依据 `frozen/spec.md` 与 `frozen/run-constraints.md`）

## 1. 完成了什么

- 完整实现冻结规格的全部功能：
  - `POST /api/links` 创建（字段校验、别名规则、保留别名、TTL、错误分类 400/409、自动 7 位短码、同 URL 两次不同短码）；
  - `GET /api/links/<code>` 统计（含 `expired` 恰好到期边界：`expires_at <= now` 即过期）；
  - `GET /<code>` 跳转（302 / 410 / 404；原子 `UPDATE ... WHERE code=? AND (expires_at IS NULL OR expires_at > ?) RETURNING url` 同时判断过期并自增，无丢计数）；
  - Location 编码：已有 `%HH` 三元组按字节原样保留（大小写不变），仅对空格/非 ASCII/不安全字节做 UTF-8 百分号编码，保留/非保留字符原样；
  - HTTP/1.1 keep-alive、pipelining、固定 `Content-Length`、1 MiB 请求体上限（超限 400 且不崩溃）、UTF-8 JSON；
  - 启动：`PORT`（含 `PORT=0` 系统分配）、`DB_PATH` 父目录自动创建、启动行 `listening on http://127.0.0.1:<实际端口>` 立即刷新，`short_url` 使用实际端口；
  - SIGTERM：信号处理程序置原子标志，监控线程 50ms 轮询后 `exit(0)`，实测 0.003–0.032s 退出、exit code 0；已提交数据全部落盘（SQLite WAL + 每条写语句自动提交，201 返回前已提交）。
- 技术形态：Rust 1.99 单进程；64 个固定工作线程（256 KiB 小栈）+ 有界内存布局；单个共享 SQLite 连接（WAL、synchronous=NORMAL、busy_timeout=5000、page_size=512、cache_size=-256 KiB、wal_autocheckpoint=2000、`WITHOUT ROWID` 表、主键唯一性兜底并发别名）；依赖仅离线缓存 crate（rusqlite 0.40.2 bundled、httparse 1.10.1、serde_json 1、libc）。无 tokio、无多进程、无外部网络。
- TDD 自检：先写测试（单元测试 8 组 + 沿用/修复上一会话按冻结规格编写的 selftest.py 客户端），在空实现（stub）上运行保留红灯原始输出，再实现直至全绿。

## 2. 未完成什么

- 无功能缺口。规格第 1–2 节全部行为均已实现并自检覆盖。
- 未做：管理接口、删除、登录（规格明确不做）；多进程/只读副本等（规格禁止或不需要）。
- 压测之外的长时稳定性（如运行数天、万级 WAL 增长）未测——超出本次冻结验收范围。

## 3. 假设（规格歧义处的自行选择，均不改变外部行为要求）

1. **“50 个并发客户端”按 50 个独立客户端进程理解**。实测（证据：`/tmp/floor` 实验，日志见第 6 节说明）：同一负载下用 Python 线程客户端时，对一个"只回固定 302 的空服务器"测得 p95 = 8.6–11.8ms（CPython GIL 抖动主导，与服务端无关）；用进程客户端时空服务器 p95 = 0.5–0.8ms。因此 selftest.py 压测部分把负载生成器从线程改为 50 个 `multiprocessing` 进程（负载参数不变：50 客户端 × 每轮 500/100 次、预热、3 轮、最近秩 p95、取中位数）。服务端行为不受此选择影响。
2. **scheme 匹配大小写敏感**：仅接受小写 `http://` / `https://` 前缀（规格未说 scheme 大小写不敏感）。
3. **错误优先级**（规格未规定多字段同时非法时的顺序）：请求形状（缺 `url`/类型错/TTL 非整数或越界 → `invalid_request`）→ URL 规则（`invalid_url`）→ 别名规则（`invalid_alias`）→ 数据库冲突（`alias_taken`）。
4. **未定义路由**：除三个接口外的一切方法/路径（含 `GET /api`、`GET /health`、`GET /api/links` 无 code、`GET /a/b` 多段）→ `404 {"error":"not_found"}`。保留别名 `api`/`health` 永不入库，因此按短码查也自然 404。
5. **重复 JSON 键**取最后一个（serde_json 语义）。
6. **防御性处理**：带 `Transfer-Encoding` 或多个不一致 `Content-Length` 的请求 → 400 `invalid_request` 并关闭连接；超 1 MiB 请求体 → 400 后先按声明长度排空（上限 16 MiB）再关闭，保证客户端能读到 400（实测 RST 会截断响应）。
7. **URL 细节**：主机名只要求非空、不含空格、不含控制字符（如 `http://a` 合法，`http://example.com:` 空 port 后缀按合法处理）；路径/查询/片段中的空格合法并按规格编码；`%` 全局必须跟两位十六进制。
8. **`GET` 路径匹配忽略查询串**（`?` 后部分）。
9. **内部兜底 500 `{"error":"internal"}`**：规格未定义 500，正常路径不可达（数据库异常兜底）。
10. **环境变量缺失/非法**（PORT 非 0–65535、DB_PATH 缺失或为空）：stderr 报错并 exit(1)。规格假定两者总是提供。
11. **SIGINT 一并处理**为与 SIGTERM 相同（便利性，规格只要求 SIGTERM）。
12. **数据库 schema**（规格未规定）：`links(code TEXT PRIMARY KEY, url TEXT NOT NULL, clicks INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL, expires_at INTEGER) WITHOUT ROWID`，时间为 Unix 整数秒。自检脚本以同名表/列直读 SQLite 验证提交。
13. **自动短码**：7 位 `[A-Za-z0-9]`，xorshift 随机 + 主键冲突重试（最多 64 次），唯一性由数据库主键兜底。

## 4. 与规格的偏差

- 对外行为无偏差。规格第 3 节"重要做法"为经验提示，除两点外全部采纳：
  - "固定大小线程池和有限队列"：采用固定 64 线程池；队列用 std mpsc（无界，但接受速率由内核 backlog 与 64 工作线程自然限流，内存不受连接数无界影响）——满足该提示的内存意图。
  - 未使用独立读写连接/分片（单连接 + 互斥锁已满足 p95 阈值）。
- 自检脚手架（selftest.py，上一会话产物，明确允许修改）修复了 4 处自身缺陷：`check_eq` 调用参数个数、`validate_create` 不接受别名短码、CSV 写入 API 误用、压测客户端线程→进程（见假设 1）。这些是自检工具修正，不是对冻结规格或验收脚本的修改（未读、未改 `frozen/` 与 `verifier/` 任何内容）。
- SQLite 用 rusqlite bundled 静态编译（环境无 `libsqlite3.so` 开发链接文件；规格允许 Rust + 离线缓存，bundled 属实现细节，外部行为不变）。

## 5. 证据（原始输出路径）

均在 `/tmp/opencode/spec-method-2026-10-24/implementer/` 下：

| 阶段 | 文件 | 说明 |
|------|------|------|
| 红灯（单元） | `selfcheck/logs/red-cargo-test.log` | stub 实现，`cargo test`：1 passed / **7 failed**，exit=101 |
| 红灯（客户端） | `selfcheck/logs/red-selftest-functional.log` | stub 服务（仅启动行+固定 201/404），功能自检 **121 FAIL**，exit=1 |
| 绿灯（单元） | `selfcheck/logs/green-cargo-test.log` | 实现 cargo test：**8/8 passed** |
| 绿灯（功能） | `selfcheck/logs/20261024T085837Z-functional.log` | 功能自检 349/349 |
| **最终全量** | `selfcheck/logs/final-selfcheck-all.log`（= `20261024T090504Z-all.log`；目录 `20261024T090504Z-all/` 含 `perf-summary.json`、`latencies.csv`） | 功能 349/349 + 压测 6 项全过，exit=0 |
| 性能迭代过程 | `selfcheck/logs/20261024T{090241,090335,090420}Z-perf.log` | 4 KiB 页→512 字节页→cache 收敛的调参轨迹 |

最终自检关键数字（3 轮取中位数；p95 最近秩）：

- 预置 1000 条（seed0001–seed1000）耗时 0.13s；
- 跳转 p95 轮次 [5.83, 5.74, 6.99] ms，中位数 **5.83ms**（≤ 10.00 ✓）；
- 创建 p95 轮次 [1.20, 2.60, 2.44] ms，中位数 **2.44ms**（≤ 15.00 ✓）；
- 服务进程 VmHWM 最终 **8284 KiB**（≤ 10240 ✓，压测后读 `/proc/<pid>/status`）；
- 规格第 5 节 7 项预测全部在最终日志中逐条实测：并发别名 1×201+19×409、50 并发跳转点击数恰 +50、TTL=1s 到期（410 / 200 expired=true / 点击数不变）、`%2F`/`%2f`/中文/空格 Location 原样保留、重启持久化（含直读 SQLite 校验 clicks 提交）、同 URL 两次不同短码、预置 1000 条后的 p95 与内存。

## 6. 启动方式

见 `artifact/START.md`：在 `artifact/` 目录 `PORT=0 DB_PATH=<路径> exec ./shortlink` 前台启动；从 stdout 启动行读取实际端口。产物 SHA-256：`20780ce63419c1e7841ff86c281fd971f27a2b099206fdda0ce62345783811e7`。

## 7. 费用

功能本身无外部付费调用。本次模型/会话用量由运行平台记录，实现侧拿不到具体数字：**未测**（不写 0）。

---

# 第二轮迭代（验收反馈处理后，2026-10-24）

背景：独立验收方对第一轮产物（SHA-256 `20780ce6…83811e7`）单次验收结论"未通过"：81 项中 75 过 / 6 失败（TTL-01~04、PERF-03 跳转 p95、PERF-05 创建 p95）。以下逐项处理。

## 1. TTL-01~04：判定"服务端无合法改进空间"，产物行为不变

事实链：

1. 验收方自述：5 次创建全部 201 且 `expires_at` 一致（2026-10-24T09:19:32Z），"服务端行为本身正确"；失败原因全部是 "could not obtain a TTL=1 link with enough time left"。
2. 时序数学：`created_at = floor(now)`、`expires_at = created_at + 1`（冻结语义），剩余时间 = `1 − frac(now)`。要满足验收判定条件"剩余 ≥ 0.4s"，创建时刻必须落在该 wall-clock 秒的前 0.6s（概率 60%）。验收方 5 次重试总计约 1ms（单次 ~0.2ms），不可能跨秒，5 次实为同一时间抽签 → 单次验收运行有 40% 概率整体取不到合格样本。本次验收正落在该 40%。
3. 服务端可选项逐一排除：改 `created_at` 取整方式违反冻结的"向下取整"；人为延迟创建响应（等跨秒）属题目明令禁止的投机行为；其余任何服务端动作都无法改变验收客户端的取样窗口。
4. 我方对照证据：两轮全量自检中 TTL 生命周期（等待到期法：到期前 302/clicks=1 → 到期后 410、stats `expired=true`、clicks 不变、`expires_at = created_at + ttl` 精确差 1 秒）全部通过（`final-selfcheck-all.log`、`final-selfcheck-round2-all.log` 各 349/349）。

结论：不改产物、不改行为。该项失败源于验收脚本的取样时序，非服务端缺陷。

## 2. PERF-03 / PERF-05：服务端存在真实优化空间，已实施（group commit 成组提交）

### 2.1 延迟分解（判断差距来源）

| 测量场景 | 轮中位数 | p95 | 数据来源 |
|---|---|---|---|
| 空服务器（固定 302，无 DB）× 50 **线程**客户端 | 4.0–4.3ms | 10.0–10.4ms | `floor-client-overhead.log` |
| 空服务器 × 50 **进程**客户端 | 0.07ms | 0.5–1.1ms | 同上 |
| 我方第一轮产物 × 50 进程客户端 | 1.0–1.5ms | 5.7–7.0ms | `20261024T090504Z-all/latencies.csv` |
| 验收方实测（其客户端） | 5.45–6.28ms | 13.98–15.50ms | 验收报告 |

验收方中位数 ≈ 我方服务中位数（1.0–1.5）+ 线程客户端 GIL 中位数增量（~4.0）；其 p95 ≈ 我方服务 p95（5.7–7.0）+ GIL 尾部增量（~8–9.5）。即：验收客户端是 50 线程模型，其测得的 p95 中约 8–9.5ms 是客户端自身 GIL 抖动（对照实验证明这与被测服务无关——空服务器也测得 p95≈10ms）。服务端可合法压缩的是自己那部分：第一轮为每请求一次独立事务提交，50 并发在单连接上串行排队（~115µs/请求 × 50 ≈ 6ms）。

### 2.2 改动内容（`build/src/main.rs`）

1. **group commit**：新增唯一 DB 写线程。工作线程解析请求后把操作投入队列并等待；写线程被唤醒后一次性排空当前队列（上限 256 个操作，无任何人造延迟——只在自然到达时成组），放进**一个**事务执行，一次提交；提交完成后才唤醒各请求构造响应。持久性语义与逐请求自动提交完全一致（201/302 返回前必已提交；批内任一 INSERT 主键冲突仅回滚该语句，SQLite ABORT 语义，其余操作与错误分类 409/400 不受影响）。
2. `PRAGMA wal_autocheckpoint` 2000 → 8000 页（512B 页 ≈ 4MB）：把 checkpoint 停顿摊薄到极少批次。中间试过 400 页（更差，已弃）与 2000 页（create 轮内偶发 7.9ms 离群，见 `20261024T093424Z-all.log`）。
3. 其余（HTTP 层、校验、编码、信号、启动、schema、其余 pragma）与第一轮完全相同。

### 2.3 第二轮 TDD 证据（`selfcheck/logs/`）

- 红灯：`red2-cargo-test.log` —— 新增 4 个批处理语义测试（批内 20 同名别名恰 1 Created+19 AliasTaken；批内 50 跳转 clicks 恰 +50；TTL 批内到期边界 302→Expired、clicks 冻结；批内 100 自动短码唯一）先对 stub 运行：8 passed / **4 failed**。
- 绿灯单元：`green2-cargo-test.log` —— **12/12 passed**。
- 绿灯全量：`final-selfcheck-round2-all.log`（目录 `20261024T093631Z-all/`，含 `perf-summary.json`、`latencies.csv`）—— 功能 **349/349**，压测全过，exit=0。
- 过程日志：`20261024T093424Z-all.log`（group commit 初版）、`20261024T093503Z-perf.log`、`20261024T093611Z-perf.log`（checkpoint 调参）。

### 2.4 第二轮自检数字（同冻结测量参数，50 进程客户端）

| 指标 | 第一轮 | 第二轮 | 阈值 |
|---|---|---|---|
| 跳转 p95 三轮 | 5.83 / 5.74 / 6.99 → 中位 5.83ms | 1.74 / 2.42 / 1.67 → **中位 1.74ms** | ≤10.00 |
| 创建 p95 三轮 | 1.20 / 2.60 / 2.44 → 中位 2.44ms | 1.76 / 2.69 / 4.06 → **中位 2.69ms** | ≤15.00 |
| VmHWM | 8284 KiB | **7540 KiB** | ≤10240 |
| 跳转轮中位数 | 1.0–1.5ms | 0.5–0.9ms | — |

### 2.5 对验收重跑的诚实预测

若验收客户端仍为 50 线程（GIL 尾部 ~8–9.5ms 不变），按上表分解，PERF-05（≤15ms）预计可通过；PERF-03 预计落在 ~9.5–11.5ms，仍在 10.00ms 阈值边缘——且对照实验显示即使"零耗时服务端"在该客户端下 p95 也 ≈10.0–10.4ms（`floor-client-overhead.log`），即该项在纯线程客户端下接近不可达。服务端已把自身贡献从 ~6ms 压到 ~1.7ms，这是不引入任何人造延迟前提下的全部合法空间。若验收客户端为进程/低 GIL 模型，两项均有充分余量。

## 3. 产物更新

- 新二进制：`implementer/artifact/shortlink`，SHA-256 `6cd95f435f0e18374b1a506a6e35694229a7e66b8eacff03cfd231272c164e2d`（第一轮为 `20780ce6…83811e7`，保留记录）。
- START.md 无需变更（启动方式、环境变量、接口均不变）。
- 冒烟复验：artifact 目录 `PORT=8978 DB_PATH=新路径 ./shortlink` 启动行/创建/TTL/404/父目录创建/SIGTERM 退出均正常。

## 4. 费用

功能本身无外部付费调用；本轮模型/会话用量由平台记录，实现侧拿不到具体数字：**未测**。

- 状态：**待验收**（第二轮；实现方自检不构成验收结论）
