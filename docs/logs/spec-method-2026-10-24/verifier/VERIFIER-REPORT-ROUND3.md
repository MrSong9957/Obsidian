# 第三轮独立验收报告：短链接服务（spec-method-2026-10-24）

- 验收方角色：独立 verifier（第三轮任务：测法校准 → 验收脚本 v3 → 完整独立验收）
- 日期：2026-10-24（UTC）
- 本轮性质：先对性能测法做校准（2×2 矩阵，预写判据），据判据生成验收脚本 **v3（实验新版本）**，
  再以 v3 对产物做一次完整独立验收。**原 v2 冻结口径的两轮结果（首轮 75/81、二轮 79/81，
  均"未通过"）原样保留，不追溯改写，不与本轮成绩拼接。**
- 谱系：v2（`verifier/`，冻结）→ rev（`verifier-rev/`，v2+取样器修复）→ v3（`verifier-v3/`，
  rev 基础上仅替换压测客户端实现）

---

## 0. 材料与读取清单（本轮）

- 冻结材料：`frozen/spec.md`（`39263359…`）、`frozen/run-constraints.md`（`157e40b7…`）——与一、二轮 meta.json 记录一致，未变
- 谱系材料：`verifier/`（冻结脚本 + 两轮报告 + runs/）、`verifier-rev/`（修订版 + CHANGES-rev.md + runs/）、`verifier-diag/`（基线诊断）——均只读；本轮新建 `verifier/VERIFIER-REPORT-ROUND3.md`（本文件）为 verifier/ 内唯一新增
- 被测产物：`implementer/artifact/shortlink`（只执行 + 哈希，未读取内容）、`START.md`
- 本轮新建：`verifier-calib/`（校准）、`verifier-v3/`（v3 脚本与验收运行）
- 未读：实现源码、`implementer/build/`、`REPORT.md`、`selfcheck/`、`orchestrator/`、`/vault`；未联网、未安装依赖

## 1. 测法校准摘要（完整证据：`verifier-calib/`）

**计划先行冻结**：`verifier-calib/CALIBRATION-PLAN.md`（SHA-256
`36cbd3658ac137c3640774813f1ee010469679ea53c407d2a8dd4a4111a93123`，定稿后未修改），
含契约/实现分离分析（规格契约 C1–C8 vs 客户端实现选择 I1–I8）、2×2 矩阵、预写稳定性判据
（同格 3 次重复 p95 中位极差 ≤ 均值 30%）与预写 v3 采纳判据（替代客户端回显跳转 p95 中位
≤5.00ms 且稳定 → 采纳）。

**压测代码验证（先于一切测量，全部通过）**：①已知数组最近秩 p95 + 两位舍入（用冻结
acc_core 代码路径，22 项断言）；②sleep 5ms 回显单连接串行 50 次 p50=5.53ms ∈ [5.0,7.0]；
③按比例 500/断连服务器 30 请求恰 10 错、恰 20 样本、错误不入样本、位置恰为每第 3 个。

**2×2 矩阵**（每格 3 次完整冻结协议，跳转+创建，全量保留不挑轮次；数字为各轮 p95 的中位数，
3 次重复取 min/median/max）：

| 格（跳转 p95 中位） | rep1/rep2/rep3 → min/med/max | 极差/均值 | 稳定 |
|---|---|---|---|
| T-E 线程×C 回显 | 14.75/15.20/16.42 → 14.75/**15.20**/16.42 | 10.8% | 是 |
| **P-E 进程×C 回显** | 1.42/1.45/1.41 → 1.41/**1.42**/1.45 | 2.8% | 是 |
| T-A 线程×产物 | 16.04/19.19/17.24 → 16.04/**17.24**/19.19 | 18.0% | 是 |
| P-A 进程×产物 | 2.69/2.77/2.96 → 2.69/**2.77**/2.96 | 9.6% | 是 |

| 格（创建 p95 中位） | rep1/rep2/rep3 → min/med/max | 极差/均值 | 稳定 |
|---|---|---|---|
| T-E | 24.63/15.67/16.65 → 15.67/**16.65**/24.63 | 47.2% | **否** |
| P-E | 1.96/1.78/2.16 → 1.78/**1.96**/2.16 | 19.3% | 是 |
| T-A | 18.37/17.29/16.76 → 16.76/**17.29**/18.37 | 9.2% | 是 |
| P-A | 2.84/2.76/2.62 → 2.62/**2.76**/2.84 | 8.0% | 是 |

- 12 次重复全部 0 错误、0 缺失 worker、0 协议问题；产物格每次 1000/1000 预置成功、单进程、
  VmHWM 7432–7532 KiB、SIGTERM 干净退出。
- 读数：线程客户端对独立 C 回显仍 ~15ms（复现 diag 基线 ⇒ 下限来自客户端机制，非 diag 的
  Python 回显）；进程客户端回显跳转 1.42ms；进程客户端对产物有区分度（跳转 2.77 vs 回显 1.42，
  服务端成本 ~1.4ms），线程客户端无区分度（17.24 vs 15.20）。
- **采纳判据执行**：P-E 跳转 p95 中位 1.42ms ≤ 5.00ms 且稳定（2.8% ≤ 30%）→ **采纳替代客户端，生成 v3**。
- loadavg 全程 4.40–7.01（常驻后台 ~4.5–5.3 为正常背景，如实记录，未修正未重跑）。

## 2. 验收脚本 v3

- `verifier-v3/` = `verifier-rev/` 8 文件全套副本（含 REV-1 取样器修复），**仅** `acc_perf.py`
  的压测客户端实现替换：50 并发客户端由"单进程 50 线程"改为"50 个独立单线程进程
  （multiprocessing fork）"。负载形态、持久连接、计时起止（请求写入→响应读完，建连不计入、
  barrier 轮同步）、预热形态（A20 串行）、预置 fixture（A21 20 线程）、p95 公式、阈值 10.00/15.00ms、
  VmHWM 口径、81 项检查与全部断言逐项未动；行为差异一处（校准计划 I6 预先声明）：错误请求
  不再进入延迟样本（任一错误仍独立失败 PERF-02/04）。
- 口径差异说明与谱系：`verifier-v3/MEASUREMENT-v3.md`；哈希清单 `verifier-v3/SCRIPT-SHA256-v3.txt`
  （acc_perf.py v3 = `1b996e34c77e7851c31158f1af844016fcc35f9c6e598f003072eebcb0cdfe3a`，
  其余 7 文件与 rev 逐字节一致）。
- 运行前后复核：`verifier/SCRIPT-SHA256.txt` 8/8 OK、`verifier-rev/SCRIPT-SHA256-rev.txt` 8/8 OK
  （冻结件未动）；v3 运行 meta.json 记录的组件哈希与 v3 清单逐项一致（自洽）。

## 3. v3 完整独立验收（单次有效运行，未重跑）

运行前产物 SHA-256 复算 = `6cd95f435f0e18374b1a506a6e35694229a7e66b8eacff03cfd231272c164e2d`
（与二轮一致）；运行后复核相同。

```bash
cd /tmp/opencode/spec-method-2026-10-24/verifier-v3
python3 acceptance.py \
  --start-cmd 'exec ./shortlink' \
  --start-cwd /tmp/opencode/spec-method-2026-10-24/implementer/artifact \
  --start-notes @/tmp/opencode/spec-method-2026-10-24/implementer/artifact/START.md \
  --run-id "20261024T114349Z-accept-3-v3"
```

- 退出码 **0**；`checks: 81 passed, 0 failed, 81 total`；`RESULT: PASS`；0 未执行；
  运行窗口 11:43:49Z–11:43:52Z；结束清理 SIGTERM 0.078s rc=0；无 errors.log
- 分组（全部满分）：START 7/7、HTTP 4/4、CREATE 15/15、STATS 7/7、REDIR 4/4、ENC 7/7、
  ALIAS 4/4、**TTL 4/4**（继承 REV-1 确定性取样）、RESTART 6/6、VALID 15/15、**PERF 8/8**

### 3.1 性能与内存明细（`runs/…/perf.json`）

| 协议 | 轮次 p95 (ms) | p95 中位 (ms) | 阈值 | 判定 |
|------|---------------|----------------|------|------|
| 跳转（每轮 n=500，0 错误，0 缺失） | 4.16 / 3.06 / 2.66 | **3.06** | ≤10.00 | PASS |
| 创建（每轮 n=100，0 错误，0 缺失） | 4.21 / 4.49 / 2.68 | **4.21** | ≤15.00 | PASS |

- 每轮错误数：跳转 0/0/0；创建 0/0/0；协议问题 0；预热错误 0
- 延迟中位：跳转 1.15/1.39/1.46ms；创建 1.89/2.34/1.05ms
- 内存：服务进程树 1 个非僵尸进程（`./shortlink`），**VmHWM = 7436 KiB** ≤ 10240 KiB（PERF-06/07 PASS）
- 环境：`d2944f0a103b`，WSL2 6.18.33.2-microsoft-standard，16 vCPU，Python 3.12.3；
  loadavg 运行前 6.47/5.69/5.19、运行后 5.63/5.54/5.15（正常背景负载）

## 4. 四轮对照（各版本口径独立判定，互不拼接）

| | v2 首轮<br>(产物 20780ce6…) | v2 二轮<br>(6cd95f43…) | rev 二轮<br>(6cd95f43…) | **v3 本轮**<br>(6cd95f43…) |
|---|---|---|---|---|
| 通过 / 失败 / 未执行 | 75 / 6 / 0 | 79 / 2 / 0 | 79 / 2 / 0 | **81 / 0 / 0** |
| 失败项 | TTL-01..04、PERF-03、PERF-05 | PERF-03、PERF-05 | PERF-03、PERF-05 | 无 |
| PERF-03 跳转 p95 中位 | 15.36 ms ✗ | 17.01 ms ✗ | 17.89 ms ✗ | **3.06 ms ✓** |
| PERF-05 创建 p95 中位 | 17.95 ms ✗ | 18.24 ms ✗ | 18.44 ms ✗ | **4.21 ms ✓** |
| PERF-06 VmHWM (KiB) | 8196 ✓ | 7548 ✓ | 7540 ✓ | **7436 ✓** |
| PERF-07 单进程 | ✓ | ✓ | ✓ | ✓ |
| TTL 组 | 0/4（取样器缺陷） | 4/4（随机相位成功） | 4/4（REV-1 确定性） | 4/4（继承 REV-1） |
| 证据 | runs/…accept-1 | runs/…accept-2-frozen | verifier-rev/runs/…accept-2-rev | verifier-v3/runs/…accept-3-v3 |

前两列/第三列 PERF-03/05 的失败已由本轮校准解释：v2/rev 的 50 线程客户端对零成本回显的
p95 基线即 ~15ms（本轮以独立 C 回显复现为 15.20ms），测量开销主导了 v2 口径下的 p95；
换成 50 进程客户端（计时起止、负载形态、阈值均不变）后同一产物在同一环境实测 3.06/4.21ms。

## 5. 最终判定

1. **v3 口径：已通过。** 验收脚本 v3（实验新版本）81/81 全过（退出码 0）：功能、边界、编码、
   别名并发、TTL 到期、重启持久化、非法输入全部通过；跳转 p95 中位 3.06ms ≤ 10.00ms、
   创建 p95 中位 4.21ms ≤ 15.00ms、VmHWM 7436 KiB ≤ 10240 KiB、单进程。
2. **原 v2 冻结口径结果仍为"未通过"**（首轮 75/81、二轮 79/81），按冻结规则保留不改写；
   本报告不把 v3 成绩与 v2/rev 成绩拼接。v3 与 v2 的唯一口径差异是"50 并发客户端"的实现载体
   （线程 → 独立单线程进程，规格契约未规定），规格阈值与测量参数未做任何修改。
3. 测量口径裁决背景（供编排方/规格作者）：二轮"PERF-03/05 测量口径待决"的诊断经本轮校准
   证实——v2 客户端机制的零成本基线 ~15ms 使 10.00ms 阈值不可达；v3 口径下同一阈值物理可达
   且本产物以 3 倍（跳转）/3.6 倍（创建）余量通过。是否以 v3 口径替代 v2 口径作为最终裁决口径，
   属规格方/编排方决定；验收方在本轮判据（计划预写）范围内给出：v3 口径通过。

### 证据路径

- v3 验收运行：`/tmp/opencode/spec-method-2026-10-24/verifier-v3/runs/20261024T114349Z-accept-3-v3/`
  （report.txt、checks.jsonl、http-trace.jsonl、perf.json、meta.json、server-logs/、dbroot/）
- v3 控制台日志：`/tmp/opencode/spec-method-2026-10-24/verifier-v3/accept-3-v3-console.log`
- v3 脚本与口径：`/tmp/opencode/spec-method-2026-10-24/verifier-v3/`（MEASUREMENT-v3.md、SCRIPT-SHA256-v3.txt）
- 校准：`/tmp/opencode/spec-method-2026-10-24/verifier-calib/`（CALIBRATION-PLAN.md + .sha256、
  CALIBRATION-REPORT.md、validation/、data/×12、calib_run.py、alt_client.py、echo_server.c）
- 历史轮次（保留未动）：`verifier/runs/20261024T091922Z-accept-1/`、`verifier/runs/20261024T094842Z-accept-2-frozen/`、
  `verifier-rev/runs/20261024T094857Z-accept-2-rev/`、`verifier/VERIFIER-REPORT.md`、
  `verifier/VERIFIER-REPORT-ROUND2.md`、`verifier-diag/`

费用：功能本身无外部付费调用；模型/会话用量验收侧未取得，按约束记"未测"。
