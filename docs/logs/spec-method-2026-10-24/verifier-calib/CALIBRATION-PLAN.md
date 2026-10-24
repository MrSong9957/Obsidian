# 验收测法校准计划（verifier-calib）

- 角色：独立验收方 verifier（第三轮任务：测法校准 + 验收脚本 v3 + 完整独立验收）
- 日期：2026-10-24（UTC）
- 状态：定稿（本文件 SHA-256 记录于 `CALIBRATION-PLAN.sha256`，此后不得修改）
- 材料范围：只读 `frozen/`、`verifier/`、`verifier-rev/`、`verifier-diag/`、`implementer/artifact/`（产物只执行不读取内容）；新建可写仅 `verifier-calib/`、`verifier-v3/`。不读 `implementer/build|REPORT.md|selfcheck/`、`orchestrator/`、`/vault`；不联网、不装依赖；不改 `frozen/`、`verifier/`、`verifier-rev/` 既有文件（新增 `verifier/VERIFIER-REPORT-ROUND3.md` 除外）。

## 0. 契约与实现分离分析（第 0 步）

依据 `frozen/spec.md` §1.4（L59–68）与 `frozen/run-constraints.md` "测量参数"（L32–40）逐项划分。

### 0.1 规格契约（校准与 v3 均不得改变）

| # | 契约项 | 出处 |
|---|--------|------|
| C1 | 预置 1000 条：`seed0001`–`seed1000`，URL `https://example.com/seed/<短码>`，压测前完成 | spec L63 / rc L34 |
| C2 | 跳转：50 个并发客户端 × 每轮 500 次（每客户端 10 次）；预热 50；3 轮 | spec L64 / rc L35 |
| C3 | 创建：50 个并发客户端 × 每轮 100 次（每客户端 2 次）；预热 20；3 轮；每轮 URL 不同 | spec L65 / rc L36 |
| C4 | p95 = `sorted(latencies)[ceil(0.95*n)-1]`，毫秒，保留两位；先每轮取 p95 再取 3 轮中位数 | rc L37 / spec L64 |
| C5 | 阈值：跳转 p95 ≤ 10.00 ms；创建 p95 ≤ 15.00 ms；不得改阈值/并发数/轮次/取数方式 | rc L39 / spec L67 |
| C6 | 内存：压测结束后读服务进程 `/proc/<pid>/status` VmHWM ≤ 10240 KiB；被测服务单进程；客户端进程不计 | spec L66/L11 / rc L38,40 |
| C7 | 服务端与压测客户端是两个独立进程；本机 `127.0.0.1`；不访问外网 | spec L61 / rc L12 |
| C8 | 没有预热、轮次或结果选取方式的数据不算通过（不得挑轮次） | spec L67 |

### 0.2 验收脚本客户端实现选择（不属契约；v2 的具体化，可被替代）

| # | v2 实现选择 | 说明 |
|---|-------------|------|
| I1 | "50 个并发客户端"载体 = 单个 Python 进程内 50 个 OS 线程 + `threading.Barrier` | 契约只说"50 个并发客户端"，未规定线程/进程载体 |
| I2 | HTTP 客户端 = `http.client`，每客户端一条持久连接，建连在 barrier 前完成、不计入计时 | 契约未规定连接复用细节 |
| I3 | 计时起止 = `time.perf_counter()` 包住 `conn.request()` → `getresponse()` → `read()`（请求写入→响应读完的客户端观测往返） | 冻结文本未明说；rev 任务 C 判定与冻结定义相符；**v3 保持不变** |
| I4 | 预热形态 = 单连接串行（ambiguities A20） | 验收方口径，非规格原文 |
| I5 | 预置 1000 条以 20 并发线程创建（A21） | fixture 实现细节 |
| I6 | 错误请求的延迟也计入样本（错误另行记录） | 契约未规定；替代客户端按本任务规定改为**错误不入样本** |
| I7 | 跳转短码选取公式 `(r*131+i*17+s*29)%n`、创建 URL 命名 `perf/create/r{r}/c{i}/i{s}`、预热 URL `perf/warmup/{i}` | fixture 细节，v3 原样保留 |
| I8 | "保留两位"用 Python `round()` | 实现细节，v3 原样保留 |

### 0.3 校准问题

verifier-diag（任务 D）：50 Python 线程客户端对最小 302 回显服务器的跳转 p95 中位 = 15.88 / 14.94 ms——客户端机制自身开销可能已超过 10.00 ms 跳转阈值，使 PERF-03 在该口径下对任何服务端不可达。本校准检验：把 I1 的载体从"50 线程"换成"50 个独立单线程进程（multiprocessing，无 GIL 争用）"，其余实现选择（I2–I8 的连接/计时/形态）与冻结口径一致时，回显基线是否降到阈值之下（测量开销不再主导）。**只替代 I1（及随之的 I6），其余契约与实现口径全部不变。**

## 1. 2×2 校准矩阵

| | 最小回显服务器 | 被测产物 shortlink |
|---|---|---|
| **线程客户端**（复用 `verifier/acc_perf.py` 现有代码路径 `_jump_round`/`_create_round`，只读 import） | 格 T-E | 格 T-A |
| **替代客户端**（50 个独立单线程进程，multiprocessing fork，持久连接，计时=请求写入→响应读完，与冻结口径一致；错误不入延迟样本） | 格 P-E | 格 P-A |

- 每格都测**跳转与创建**两个协议（均为冻结形态：预热 50/20 + 3 轮 × 500/100）。
- **每格重复 3 次完整冻结协议**，全部结果保留；不挑轮次、不从被测结果减去基线。
- 报每格 3 次重复的"p95 中位数"（每次重复 = 其 3 轮 p95 的中位数）的 min / median / max。

### 1.1 组件定义

- **线程客户端（T）**：`import verifier/acc_perf.py` 的 `_jump_round(port, codes, r)` / `_create_round(port, r)`（`sys.dont_write_bytecode=True`，不向 `verifier/` 写入任何文件）。轮内行为、计时、样本语义 = 冻结版逐字节。
- **替代客户端（P）**：`verifier-calib/alt_client.py`。每轮 fork 50 个独立单线程子进程；每子进程 `http.client` 一条持久连接，先 `connect()` 再 `multiprocessing.Barrier` 同步；子进程内 `perf_counter` 包 `conn.request→getresponse→read`（与冻结相同的计时起止）；跳转校验 302+Location+协议（HTTP/1.1、Content-Length），创建校验 201+协议；非预期状态或异常记错误，**该请求延迟不入样本**；结果经 Queue 回收。
- **最小回显服务器**：C 写就（gcc 13.3.0，`verifier-calib/echo_server.c`），accept 后 fork 每连接一子进程、阻塞 I/O；GET → 预构 302（`Location: https://example.com/seed<path>`，回显路径），POST（按 Content-Length 读完请求体）→ 预构 201 JSON；无解析/存储成本。选 C 而非 Python（verifier-diag 用的 Python 版）的原因：(a) Python 回显在 50 并发下自身 GIL 序列化会混入"客户端下限"测量；(b) run-constraints"不许用 Python 写服务端"的适用争议。回显服务器是诊断工具，不是被测服务，不属 C6 单进程/VmHWM 约束对象。
- **被测产物**：`implementer/artifact/shortlink`（SHA-256 必须等于 `6cd95f435f0e18374b1a506a6e35694229a7e66b8eacff03cfd231272c164e2d`）。每次重复用 `PORT=0` + 全新 `DB_PATH` 启动新实例，读启动行取端口。
- **fixture（T/P 两排一致，使格间只差被测负载客户端）**：预热 = 单连接串行（同 A20 形态，两种客户端共用同一段串行代码）；预置 1000 条 = 20 线程创建（同 A21 形态）；跳转码选取/创建 URL 公式 = 冻结版逐字节（I7）。回显格无持久化，"预置"对回显无意义，跳过（如实记录；不影响两排可比性——fixture 不参与计时）。

### 1.2 每格流程（一次重复）

1. 记录 `/proc/loadavg`（前）。
2. [产物格] 启动新实例（新 DB）→ 预置 1000 条并校验 201+code。[回显格] 启动回显服务器。
3. 跳转：预热 50（串行）→ 3 轮 `_jump_round`/替代客户端轮 → 每轮 p95（两位舍入）→ 3 轮中位。
4. 创建：预热 20（串行）→ 3 轮 → 同上。
5. [产物格] 读服务进程 VmHWM 与进程数；SIGTERM 停止。
6. 记录 `/proc/loadavg`（后）。

格顺序固定：T-E → P-E → T-A → P-A；每格连做 3 次重复。原始数据（每轮全部延迟样本、错误、协议问题、loadavg、p95）写入 `verifier-calib/data/<格>-rep<N>.json`。

## 2. 预写判据（定稿后不可改）

- **稳定性判据**：同格 3 次重复的 p95 中位数极差（max−min）≤ 其均值的 30% 为"稳定"。
- **v3 采纳判据（只看回显基线格）**：替代客户端格 P-E 的**跳转** p95 中位数（3 次重复取中位）≤ 5.00 ms 且按上条稳定 → v3 采用替代客户端并跑完整验收；否则不建 v3，本轮交付"测法待决"诊断。
- 判据只引用 P-E 跳转；P-E 创建、T-* 格、P-A 格数字全部如实报告，不参与采纳判定。
- 当前环境常驻守护进程负载 ~4.5–5.3 属正常背景，loadavg 如实记录，不作修正、不重跑。

## 3. 压测代码验证（先于一切测量；不过不得进入校准）

写入 `verifier-calib/validation/`，保留真实输出：

1. **① 已知延迟数组的最近秩 p95 + 两位舍入**：用冻结 `acc_core.nearest_rank_p95`（只读 import）对 n=20/100/500 已知数组断言 `sorted[ceil(0.95n)-1]` 精确取值；对若干延迟值断言 `round(x,2)` 结果（含 Python 银行家舍入语义，与冻结脚本同一 `round` 代码路径）。全部断言通过 = 过。
2. **② sleep 5ms 回显基线**：每请求 `time.sleep(0.005)` 后回 302 的服务器；替代客户端计时原语单连接串行 50 次；p50（`sorted[ceil(0.50n)-1]`）∈ [5.0, 7.0] ms = 过（计时起止正确、无系统性偏差）。
3. **③ 按比例 500/断连**：每连接第 3k 个请求回 500 后关闭连接、其余 200 keep-alive 的服务器；替代客户端串行会话发 30 次：总请求 = 30、错误 = 恰 10、延迟样本 = 恰 20（错误不入样本）、错误位置恰为每第 3 个 = 过。

验证 ①②③ 针对替代客户端的计时/取样/错误计数原语（校准与 v3 复用同一原语）；multiprocessing 轮机制由矩阵本身检验。

## 4. 产物与路径

- 校准数据：`verifier-calib/data/`；验证输出：`verifier-calib/validation/`
- 校准报告：`verifier-calib/CALIBRATION-REPORT.md`（每格结果、稳定性判定、按第 2 节判据的 v3 决定）
- 若采纳：`verifier-v3/`（基于 `verifier-rev/` 全套 8 文件，仅替换 `acc_perf.py` 客户端实现为替代客户端；`MEASUREMENT-v3.md`、`SCRIPT-SHA256-v3.txt`；复核 `verifier/`、`verifier-rev/` 哈希未动；产物哈希复算相符后在 `verifier-v3/` 跑一次完整验收，`--run-id` = 当前 UTC 时间戳 + `-accept-3-v3`，单次有效运行不重跑）
- 第三轮报告：`verifier/VERIFIER-REPORT-ROUND3.md`（新增文件；四列对照 v2 首轮 75/81、v2 二轮 79/81、rev 二轮 79/81、v3 本轮；原 v2 冻结口径结果保持"未通过"不变，不拼接不同版本成绩）

## 5. 环境基线（计划时点实测）

- 主机 WSL2 内核 6.18.33.2-microsoft-standard-WSL2，16 vCPU，内存 16,236,180 KiB
- Python 3.12.3（仅标准库）、gcc 13.3.0
- `/proc/loadavg`（计划时点）：5.10 5.44 4.94（常驻后台负载，正常背景）
- 产物 SHA-256（计划时点复算）：`6cd95f435f0e18374b1a506a6e35694229a7e66b8eacff03cfd231272c164e2d`（与二轮验收记录一致）

费用：功能本身无外部付费调用；模型/会话用量验收侧拿不到，按约束记"未测"。
