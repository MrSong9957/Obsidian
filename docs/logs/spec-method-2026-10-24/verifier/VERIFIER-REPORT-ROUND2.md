# 第二轮独立验收报告：短链接服务（spec-method-2026-10-24）

- 验收方角色：独立 verifier（续首轮会话后的第二轮任务：诊断、脚本修订、独立复验）
- 日期：2026-10-24（UTC）
- 首轮结果 **75/81 未通过**与证据 `verifier/runs/20261024T091922Z-accept-1/` 原样保留，未被覆盖
- 被测产物（第二轮）：`implementer/artifact/shortlink`
  SHA-256 实测 `6cd95f435f0e18374b1a506a6e35694229a7e66b8eacff03cfd231272c164e2d`
  （与实现方声明一致；首轮产物为 `20780ce6…`，已更换）；`START.md` 未变（mtime 09:07 早于产物更新 09:35）
- 两轮复验结果：冻结版 **79/81 FAIL**（退出码 1）；修订版 **79/81 FAIL**（退出码 1）；均为仅 PERF-03/PERF-05 失败
- **最终判定：未通过**；其中 PERF-03/PERF-05 的阈值判定为“测量口径待决”（证据见任务 D，条件未放宽）

---

## 0. 材料与读取清单（本轮）

- 冻结材料：`frozen/spec.md`（sha256 `39263359…`）、`frozen/run-constraints.md`（`157e40b7…`，与首轮 meta.json 记录一致，未变）
- 验收侧：`verifier/` 8 个冻结文件（运行前后两次 `sha256sum -c` 全 OK）、`verifier/ambiguities.md`、`verifier/acc_perf.py` 与 `acc_core.py` 计时段（任务 C 只读分析）、`verifier/acc_functional.py` L655–700（任务 A 只读分析）
- 首轮证据：`verifier/runs/20261024T091922Z-accept-1/`（http-trace.jsonl、checks.jsonl、perf.json、meta.json）
- 产物：`implementer/artifact/shortlink`（执行 + 哈希）、`START.md`（哈希/时间戳核对）
- 本轮新建：`verifier-diag/`（诊断）、`verifier-rev/`（修订版脚本）、本报告
- 未读：实现源码、`implementer/build/`、`REPORT.md`、`selfcheck/`、`orchestrator/`、`/vault`；未联网、未装依赖
- 实现方声明仅作线索，所有判定以本方测量为准

## 1. 任务 A：TTL 取样诊断 —— 结论：取样器缺陷，非实现违约

数据源：首轮 `runs/20261024T091922Z-accept-1/http-trace.jsonl` rows 139–143
（trace `ts` 为响应完成后记录、秒级精度，acc_core.py `http_request` L143）：

| label | 响应 ts | status | RTT | expires_at |
|-------|---------|--------|-----|------------|
| create-ttl-expiry-0 | 09:19:31Z | 201 | 0.204ms | 2026-10-24T09:19:32Z |
| create-ttl-expiry-1 | 09:19:31Z | 201 | 0.238ms | 2026-10-24T09:19:32Z |
| create-ttl-expiry-2 | 09:19:31Z | 201 | 0.190ms | 2026-10-24T09:19:32Z |
| create-ttl-expiry-3 | 09:19:31Z | 201 | 0.197ms | 2026-10-24T09:19:32Z |
| create-ttl-expiry-4 | 09:19:31Z | 201 | 0.252ms | 2026-10-24T09:19:32Z |

重算：epoch 09:19:31=1792833571，expires_at=1792833572；接受条件
`expires_at − time.time() ≥ 0.4` ⟺ 检查时刻 ≤ …571.6。5 次均失败且响应
ts 均为 …571 秒 ⟹ 5 次创建全部落在该秒**最后 0.4 s（frac(now) > 0.6）**；
5 次 RTT 合计约 1 ms，不可能跨秒（若跨入 572 秒，expires_at 应为 573）。
首轮报告“同一秒 / 剩余恒 <0.4s / 重试未跨秒”与原始数据一致；“后 0.6 s”
的措辞精确化为“秒内位置 >0.6 s”，实质一致。

规格核对（spec.md L34）：`expires_at = created_at + ttl_seconds`，时间 Unix 秒
**向下取整**。服务端对 571 秒内的创建返回 572，与 floor 语义一致，5 次创建
（201 + 一致 expires_at）均正确。

**判定**：背靠背重试共享同一秒内相位，取样失败率 = 单次尝试落在秒尾 0.4 s 的
概率（均匀到达下 40%），首轮命中。修复属取样器侧；实现方“TTL 无服务端修复”
的声明与本诊断一致。首轮 FAIL 按冻结口径保留不改。

## 2. 任务 B：修订版脚本（verifier-rev/）

- `verifier-rev/` = `verifier/` 8 文件逐字节副本 + 唯一改动 REV-1
  （`acc_functional.py` `phase_ttl()`：创建样本前
  `time.sleep(1.05 - (time.time() % 1.0))` 对齐到下一 wall-clock 秒开始，
  使剩余 ≈ 1.0s − RTT，接受条件必然满足）。完整 diff 与说明：
  `verifier-rev/CHANGES-rev.md`、`verifier-rev/sampler-fix.diff`。
- 业务断言、接受条件（≥0.4s）、重试上限、A10 断言时机、81 项计划、全部性能
  参数与阈值逐字未动；其余 7 文件与冻结版逐字节一致（cmp 验证）。
- `verifier-rev/SCRIPT-SHA256-rev.txt`：仅 acc_functional.py 变化
  （`34131ce65b91…` → `8989931967e0…`），另 7 项与冻结清单相同。
- 原 `verifier/` 复核：任务 B 后 `sha256sum -c SCRIPT-SHA256.txt` 8/8 OK，
  冻结件未动。修订版运行时 meta.json 记录的 acc_functional.py 哈希 =
  `8989931967e0…`，自洽。

## 3. 任务 C：性能计时口径核对（只读）—— 结论：与冻结定义相符

- 计时范围（acc_perf.py L131–135 / L177–182）：`time.perf_counter()` 包住
  `conn.request()` → `getresponse()` → `read()`，即**客户端观测的单次完整往返**
  （含 http.client 序列化/解析与客户端线程调度开销）；TCP 建连不计入
  （50 客户端先 connect 再 barrier 同步，整轮一条持久连接）；预热不计入；
  单调钟。
- p95：`sorted[ceil(0.95*n)−1]`（acc_core.py L49–54），每轮先两位小数取整
  再取三轮中位数（L237、L256–257、L261），与 run-constraints.md 测量参数及
  A15 一致。并发/轮次/预热/预置（20 并发建 1000 条，A21）/VmHWM 口径逐项
  相符；阈值 10.00/15.00 ms 未动。
- 唯一冻结文本未明说之处：计时是否含客户端自身开销。脚本按“含”执行，对所有
  实现一视同仁；该开销量级由任务 D 量化。**结论：脚本与冻结测量定义相符。**

## 4. 任务 D：客户端基线 —— 结论：零服务端成本下 p95 基线 ≈ 14.9–15.9 ms

- 材料：`verifier-diag/baseline_server.py`（原生 socket、每连接一线程、不解析
  请求、固定 302 字节串的最廉价 keep-alive 应答器）、`verifier-diag/baseline_client.py`
  （逐行复刻 acc_perf.py 跳转负载形态与计时范围：50 线程、barrier、每线程 10 次、
  一条持久连接、预热 50 单连接串行、3 轮、最近秩 p95 先取整再中位）。
- 原始数据：`baseline-result.json`、`baseline-result2.json`（诊断性运行，两次都保留）。

| 运行 | 各轮 p95 (ms) | p95 中位 (ms) | 各轮延迟中位 (ms) |
|------|---------------|----------------|-------------------|
| 1（09:45:51Z） | 15.84 / 15.88 / 15.88 | **15.88** | 5.28 / 5.42 / 5.50 |
| 2（09:46:27Z） | 14.94 / 15.50 / 13.97 | **14.94** | 5.59 / 5.06 / 5.23 |

**客户端机制（50 Python 线程 + GIL/调度 + http.client）自身的 p95 基线即
15 ms 量级，已超过 10.00 ms 跳转阈值、达到并超过 15.00 ms 创建阈值。**
在该测量口径下，任何服务端（包括零成本回显）都不可能测得 ≤10 ms 的 p95；
p95 样本主要由客户端机制主导。首轮 shortlink 实测（跳转 p95 中位 15.36，
延迟中位 5.45–6.28）与零成本基线几乎重合，佐证 p95 未能反映服务端成本。

## 5. 任务 E：独立复验（新产物，两轮分开报告）

两轮命令相同，仅目录与 run-id 不同：

```bash
cd <verifier 或 verifier-rev>
python3 acceptance.py \
  --start-cmd 'exec ./shortlink' \
  --start-cwd /tmp/opencode/spec-method-2026-10-24/implementer/artifact \
  --start-notes @/tmp/opencode/spec-method-2026-10-24/implementer/artifact/START.md \
  --run-id "<UTC>-accept-2-frozen | <UTC>-accept-2-rev"
```

### 5.1 冻结版（verifier/，run-id `20261024T094842Z-accept-2-frozen`）

- 退出码 **1**；`checks: 79 passed, 2 failed, 81 total`；`RESULT: FAIL`；0 未执行
- 失败项：`PERF-03`（跳转 p95 中位 17.01 ms > 10.00，轮次 16.49/17.44/17.01）、
  `PERF-05`（创建 p95 中位 18.24 ms > 15.00，轮次 18.18/18.24/19.18）
- 分组：START 7/7、HTTP 4/4、CREATE 15/15、STATS 7/7、REDIR 4/4、ENC 7/7、
  ALIAS 4/4、**TTL 4/4**、RESTART 6/6、VALID 15/15、PERF 6/8
- TTL：本次冻结版取样落在秒相位前 0.6 s（60% 概率分支），第 1 次尝试即取得
  样本（`ttl-timing: attempt=1, waited=1.146s`），TTL-01..04 全过——与任务 A
  “40% 取样失败率”的诊断一致（首轮失败、本轮成功均为同一随机过程）
- 性能：跳转各轮 p95 16.49/17.44/17.01，延迟中位 6.18/5.64/6.24；
  创建各轮 p95 18.18/18.24/19.18，延迟中位 10.09/8.81/8.26；
  **VmHWM = 7548 KiB**（≤10240，PASS），单进程
- meta.json 记录 acc_functional.py 哈希 = 冻结版 `34131ce6…`

### 5.2 修订版（verifier-rev/，run-id `20261024T094857Z-accept-2-rev`）

- 退出码 **1**；`checks: 79 passed, 2 failed, 81 total`；`RESULT: FAIL`；0 未执行
- 失败项：`PERF-03`（跳转 p95 中位 17.89 ms > 10.00，轮次 17.89/**39.40**/16.92，
  第 2 轮含环境噪声离群）、`PERF-05`（创建 p95 中位 18.44 ms > 15.00，
  轮次 16.29/22.13/18.44）
- 分组：与冻结版相同，仅 PERF 6/8；**TTL 4/4（REV-1 生效：对齐后第 1 次尝试
  即取得样本，`ttl-timing: attempt=1, waited=1.308s`，确定性取得）**
- TTL 实测时间线（http-trace）：09:48:59 创建（expires 09:49:00）→ 到期前
  stats 200 / expired=false、跳转 302、clicks=1；09:49:00 到期后跳转 **410**、
  stats 200 / expired=true、clicks 不变、再次跳转仍 410 不增计数——服务端到期
  行为完整通过全部断言
- 性能：跳转各轮 p95 17.89/39.40/16.92，延迟中位 6.22/8.14/6.12；
  创建各轮 p95 16.29/22.13/18.44，延迟中位 8.84/10.83/10.88；
  **VmHWM = 7540 KiB**（PASS），单进程
- meta.json 记录 acc_functional.py 哈希 = 修订版 `8989931967e0…`

### 5.3 跨轮观察

- 新产物两轮 p95（跳转 17.01/17.89，创建 18.24/18.44）与首轮旧产物
  （15.36/17.95）及零成本基线（15.88/14.94）同处一个带宽内，轮内/轮间波动
  （如修订版 39.40 ms 离群轮）与机器噪声相当；本测量口径无法分辨实现方
  “成组提交”优化的效果，也未观测到劣化。
- VmHWM 由首轮 8196 KiB 降至 7548/7540 KiB（始终达标）。
- 两轮功能面 79 项全过（含 1 MiB 精确边界、编码、别名并发、重启持久化、
  SIGKILL 提交性、脏停一致性）。

## 6. 最终判定

**未通过**（冻结版与修订版各自独立判定均为 FAIL，退出码 1，仅 PERF-03/PERF-05 超限）。逐项依据：

1. **冻结版口径（原始、不可修改的判定基础）**：79/81，PERF-03 跳转 p95 中位
   17.01 ms > 10.00 ms，PERF-05 创建 p95 中位 18.24 ms > 15.00 ms。
   按冻结规则“失败就是失败、不得改阈值”，判 FAIL。
2. **修订版口径（仅修取样器，断言未削弱）**：79/81，PERF-03 17.89 ms、
   PERF-05 18.44 ms，同样判 FAIL。
3. **TTL 组**：首轮 4 项 FAIL 经任务 A 诊断为取样器缺陷（非实现违约）；本轮
   冻结版（随机成功相位）与修订版（确定性取样）均 4/4 通过，服务端到期行为
   （到期前 200/302/clicks=1，到期后 410/expired=true/clicks 不变）实测符合
   规格。TTL 争议就此了结：以修订版证据为准，TTL 组通过。
4. **性能阈值：测量口径待决（如实记录，不放宽条件）。** 任务 D 证明该冻结
   客户端机制自身的零成本基线 p95 ≈ 14.9–15.9 ms，已高于 10.00 ms 跳转阈值
   并达到 15.00 ms 创建阈值；即 PERF-03 阈值在此口径下对任何服务端均不可达，
   PERF-05 阈值处于基线水平。PERF-03/05 的 FAIL 是原始事实（两轮、两口径
   一致），但是否构成“实现不合格”取决于测量口径裁决：需规格作者/编排方
   决定（a）维持现口径 ⇒ 阈值物理不可达，任何实现都无法通过；或（b）更换
   压测客户端或调整阈值 ⇒ 需重新冻结测量参数并重测。验收方无权单方面放宽。
5. 首轮 75/81 未通过结论与 `runs/20261024T091922Z-accept-1/` 保留不变；本报告
   为第二轮记录，不追溯改写首轮。

### 证据路径

- 冻结版复验：`/tmp/opencode/spec-method-2026-10-24/verifier/runs/20261024T094842Z-accept-2-frozen/`
- 修订版复验：`/tmp/opencode/spec-method-2026-10-24/verifier-rev/runs/20261024T094857Z-accept-2-rev/`
- 首轮（保留）：`/tmp/opencode/spec-method-2026-10-24/verifier/runs/20261024T091922Z-accept-1/`
- 诊断：`/tmp/opencode/spec-method-2026-10-24/verifier-diag/`（脚本 ×2、结果 ×2、DIAGNOSTICS.md）
- 修订版脚本与哈希：`/tmp/opencode/spec-method-2026-10-24/verifier-rev/`（CHANGES-rev.md、sampler-fix.diff、SCRIPT-SHA256-rev.txt）
- 本报告：`/tmp/opencode/spec-method-2026-10-24/verifier/VERIFIER-REPORT-ROUND2.md`
- 首轮报告（保留）：`/tmp/opencode/spec-method-2026-10-24/verifier/VERIFIER-REPORT.md`

费用：功能本身无外部付费调用；模型/会话用量验收侧未取得，按约束记“未测”。
