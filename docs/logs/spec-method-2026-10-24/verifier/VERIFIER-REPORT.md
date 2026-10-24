# 独立验收报告：短链接服务（spec-method-2026-10-24）

- 验收方角色：独立 verifier（全新会话，仅依据冻结材料与被测产物判定）
- 验收日期：2026-10-24（UTC）
- 结论：**未通过**（81 项中 75 PASS / 6 FAIL / 0 未执行；跳转与创建 p95 超阈值，TTL 组 4 项失败）
- 证据目录：`/tmp/opencode/spec-method-2026-10-24/verifier/runs/20261024T091922Z-accept-1/`

---

## 1. 材料与读取清单

本次会话实际读取/使用的材料，未读取其他任何上下文：

| 材料 | 用途 |
|------|------|
| `/tmp/opencode/spec-method-2026-10-24/frozen/spec.md` | 冻结规格（只读，通读全文） |
| `/tmp/opencode/spec-method-2026-10-24/frozen/run-constraints.md` | 冻结运行约束（只读，通读全文） |
| `verifier/README-verifier.md` | 验收脚本使用说明（通读全文） |
| `verifier/ambiguities.md` | A1–A27 歧义判定口径（通读全文） |
| `verifier/SCRIPT-SHA256.txt` | 冻结哈希校验 |
| `verifier/acc_functional.py` L655–700 | 仅用于解释 TTL-01..04 失败机制（重试判定 `exp - time.time() >= 0.4`），只读 |
| `implementer/artifact/START.md` | 启动说明（通过 `--start-notes @` 记录并人工通读） |
| `implementer/artifact/shortlink` | 被测产物：执行 + 记录 SHA-256；未读取、未反编译、未修改 |
| 本次会话收到的输入说明 | 验收任务指令（verifier role、执行步骤、汇报格式） |

另读取本次运行自身产生的证据文件：`report.txt`、`checks.jsonl`、`http-trace.jsonl`、`perf.json`、`meta.json`、`server-logs/boot-01.out`、`dbroot/` 目录列表。

明确未读：`implementer/build/`、`implementer/REPORT.md`、`implementer/selfcheck/`、`orchestrator/`、`/vault` 仓库、实现源码。未联网、未安装任何依赖。`acceptance.py` / `acc_core.py` / `acc_validation.py` / `acc_restart.py` / `acc_perf.py` 未逐行阅读，仅按冻结哈希校验后执行。

## 2. 脚本冻结校验与产物哈希

### 2.1 脚本哈希校验（运行前，在 verifier 目录）

`sha256sum -c SCRIPT-SHA256.txt` 结果，8/8 全部 OK：

```
acceptance.py: OK      acc_core.py: OK        acc_functional.py: OK
acc_validation.py: OK  acc_restart.py: OK     acc_perf.py: OK
ambiguities.md: OK     README-verifier.md: OK
```

运行时 `meta.json` 记录的 `script_sha256` / `component_sha256` 与 `SCRIPT-SHA256.txt` 逐项一致（交叉核对通过）：
`acceptance.py=1bbfe77f…`、`acc_core.py=ae611961…`、`acc_functional.py=34131ce6…`、`acc_validation.py=d87dfeb8…`、`acc_restart.py=a8c0ed54…`、`acc_perf.py=e848c3c2…`。

规格哈希（meta.json 记录）：`spec.md=392633596d58bd00307e45ff775b4807fd59125b232da312d1fedee52d227be4`；`run-constraints.md=157e40b76a732ca46ffc91b3b02abae356e695e7b405e0933a3cd10fa3bad122`。

### 2.2 被测产物 SHA-256

```
20780ce63419c1e7841ff86c281fd971f27a2b099206fdda0ce62345783811e7  /tmp/opencode/spec-method-2026-10-24/implementer/artifact/shortlink
```

产物大小 2,465,432 字节，可执行权限正常，未做任何修改。

## 3. 执行命令与退出码

仅运行一次验收，未重跑（退出码 1 为有效结果，非调用错误）：

```bash
cd /tmp/opencode/spec-method-2026-10-24/verifier
python3 acceptance.py \
  --start-cmd 'exec ./shortlink' \
  --start-cwd /tmp/opencode/spec-method-2026-10-24/implementer/artifact \
  --start-notes @/tmp/opencode/spec-method-2026-10-24/implementer/artifact/START.md \
  --run-id "20261024T091922Z-accept-1"
```

- 退出码：**1**（至少一项 FAIL）
- 脚本汇总：`checks: 75 passed, 6 failed, 81 total`，`RESULT: FAIL`
- 服务三次启动（boot-01/02/03，重启阶段）stdout 均为单行 `listening on http://127.0.0.1:<port>`；stderr 全空
- 结束清理：`cleanup: SIGTERM stop elapsed=0.075s timed_out=False rc=0`
- 性能参数全部使用 `acc_perf.py` 顶部固定值，未做任何改动

## 4. 分组结果与 81 项差异说明

| 组 | 通过 | 失败 | 未执行 |
|----|------|------|--------|
| START | 7 | 0 | 0 |
| HTTP | 4 | 0 | 0 |
| CREATE | 15 | 0 | 0 |
| STATS | 7 | 0 | 0 |
| REDIR | 4 | 0 | 0 |
| ENC | 7 | 0 | 0 |
| ALIAS | 4 | 0 | 0 |
| TTL | 0 | 4 | 0 |
| RESTART | 6 | 0 | 0 |
| VALID | 15 | 0 | 0 |
| PERF | 6 | 2 | 0 |
| **合计** | **75** | **6** | **0** |

与预计 81 项无差异：`report.txt` 与 `checks.jsonl` 均恰好 81 条，无 NOT EXECUTED 项，无阶段异常（无 `errors.log`）。

## 5. 全部失败项

### 5.1 TTL-01 / TTL-02 / TTL-03 / TTL-04（TTL 到期组）

- 检查 ID：`TTL-01`（到期前 stats 200 / expired=false / 302 / clicks=1）、`TTL-02`（到期后 410）、`TTL-03`（stats 200 / expired=true / clicks 不变）、`TTL-04`（到期后跳转不再计数）
- 输入：`POST /api/links`，`{"url":"https://example.com/ttl-expiry/<0..4>","ttl_seconds":1}`，最多重试 5 次（A10），判定条件为样本剩余时间 `expires_at − now ≥ 0.4 s`（`acc_functional.py` L673）
- 预期：至少一次创建落在 wall-clock 秒的前 0.6 s 内，取得可用样本后执行四项断言
- 实际偏差：5 次创建（trace 标签 `create-ttl-expiry-0` … `-4`）全部返回 201 且 `expires_at` 均为 `2026-10-24T09:19:32Z`（创建时刻 09:19:31 秒内，服务端行为本身正确），但 5 次请求全部落在同一 wall-clock 秒的后 0.6 s（单次请求仅 ~0.2 ms，重试未跨秒），剩余时间恒 < 0.4 s，脚本未取得可用样本，四项检查按冻结逻辑记 FAIL（`checks.jsonl` detail：`could not obtain a TTL=1 link with enough time left`）
- 口径说明：这是 ambiguities.md **A10** 预先声明的已知边界（1 秒 TTL + 跨秒偏差无法用纯 API 消除，脚本最多重试 5 次）。本报告如实记录 FAIL；trace 显示服务端每次均正确返回 201 与一致的 `expires_at`，失败由验收脚本取样时机导致，而非观测到服务端到期行为错误。按冻结规则（失败就是失败、不得重跑改善数字），TTL 组计入失败，不在本次运行中复检。TTL 相关的间接证据另有：CREATE-08/09（`expires_at = created_at + ttl`、边界 1 与 31536000 接受）、RESTART-04（TTL 时间戳跨重启保留）均 PASS。

### 5.2 PERF-03（跳转 p95）

- 输入：1000 条预置（`seed0001`–`seed1000`，20 并发创建，A21）后，50 并发客户端 × 每轮 500 次 × 3 轮跳转，预热 50（单连接串行，A20）
- 预期：3 轮 p95 的中位数 ≤ 10.00 ms
- 实际：3 轮 p95 = [13.98, 15.36, 15.50] ms，中位数 **15.36 ms**，超阈值 53.6%（`checks.jsonl` detail：`median p95=15.36ms rounds=[13.98, 15.36, 15.5] limit=10.00ms`）

### 5.3 PERF-05（创建 p95）

- 输入：50 并发客户端 × 每轮 100 次 × 3 轮创建（每轮不同 URL），预热 20
- 预期：3 轮 p95 的中位数 ≤ 15.00 ms
- 实际：3 轮 p95 = [19.60, 17.95, 13.90] ms，中位数 **17.95 ms**，超阈值 19.7%（`checks.jsonl` detail：`median p95=17.95ms rounds=[19.6, 17.95, 13.9] limit=15.00ms`）

## 6. 性能与内存原始测量

数据源：`runs/20261024T091922Z-accept-1/perf.json`（每轮完整 500/100 个延迟样本已存档）。

### 6.1 跳转（每轮 n=500）

| 轮次 | p95 (ms) | 中位数 (ms) |
|------|----------|-------------|
| 1 | 13.98 | 5.45 |
| 2 | 15.36 | 5.85 |
| 3 | 15.50 | 6.28 |
| **3 轮 p95 中位数** | **15.36** | — |

判定：15.36 ms > 10.00 ms → **不达标**。

### 6.2 创建（每轮 n=100）

| 轮次 | p95 (ms) | 中位数 (ms) |
|------|----------|-------------|
| 1 | 19.60 | 9.02 |
| 2 | 17.95 | 8.44 |
| 3 | 13.90 | 7.95 |
| **3 轮 p95 中位数** | **17.95** | — |

判定：17.95 ms > 15.00 ms → **不达标**。

### 6.3 内存（压测后）

- 服务进程树：1 个非僵尸进程（pid 2677576，`cmdline=./shortlink`），监听端口 45597（A5/A16 口径：以监听 socket 进程树计）
- `VmHWM = 8196 KiB`（VmRSS 同值）
- 判定：8196 KiB ≤ 10240 KiB → **达标**（PERF-06、PERF-07 PASS）

### 6.4 功能阶段请求耗时参考

功能/校验阶段单请求 elapsed 多在 0.2–0.35 ms；并发压测下 p95 显著抬高，说明瓶颈在并发路径。

## 7. 歧义判定口径引用（ambiguities.md）

- **A1（已解决）**：`ENC-04` 按澄清后口径执行——规格示例 `https://example.com/路径?x=中 文#片段` 与路径/查询/片段空格 URL `https://example.com/a b?c=d e#f g` 均要求 201 + 302 + 原始 `Location` 精确匹配。实测 PASS（6 cases，含 `a%20b?c=d%20e#f%20g`）。
- **A3（已固定）**：超 1 MiB 请求体必须 `400 {"error":"invalid_request"}`。`VALID-14` PASS（1,048,577 字节 → 400 精确错误体），`VALID-15` PASS（超限后 stats 仍 200）。
- **A4（已固定）**：SIGTERM ≤ 5.0 s 退出并释放端口。`RESTART-01` PASS（3 cases）；结束清理实测 SIGTERM 停止耗时 0.075 s，rc=0。
- **A14（已明确）**：1 MiB 含上界。`CREATE-13` PASS（恰好 1,048,576 字节接受），`VALID-14` 见上。
- **A5/A16**：单进程口径以监听 socket 进程树计。实测进程数 1，VmHWM 求和 = 8196 KiB。
- **A10**：TTL=1 到期边界无法用纯 API 精确构造，脚本最多重试 5 次取剩余 ≥ 0.4 s 样本——本次 5 次均未取得，TTL 组 FAIL（详见 §5.1，失败机制即本条已知边界）。
- **A15**：每轮 p95 先四舍五入到两位再取中位数。实测跳转 [13.98, 15.36, 15.50] → 15.36；创建 [19.60, 17.95, 13.90] → 17.95。
- **A20/A21**：预热为单连接串行；1000 条预置用 20 并发客户端创建。均按此执行。
- **A26**：`PORT=0` 实际端口 33167 落在内核临时端口范围 32768–60999 内（`START-01` PASS，`kernel_range=(32768, 60999)`）。
- 其余口径（A2、A6–A9、A11–A13、A17–A19、A22–A25、A27）按 ambiguities.md 原文执行，无争议项。

## 8. 环境与测量参数

- 主机：`d2944f0a103b`；内核 `6.18.33.2-microsoft-standard-WSL2`（WSL2，x86_64，glibc 2.39）
- CPU：16 vCPU；内存：16,236,180 KiB（约 15.5 GiB）——与冻结运行约束一致
- Python：3.12.3（仅标准库，验收客户端/脚本用）
- 启动注入：`PORT=0`、`DB_PATH=<run-dir>/dbroot/nested/deeper/links.db`（父目录故意不存在）；启动命令 `exec ./shortlink`，工作目录 `implementer/artifact`；前台进程
- 测量参数（`acc_perf.py` 固定，测试前后未改动）：
  - 预置：`seed0001`–`seed1000` → `https://example.com/seed/<code>`，20 并发创建
  - 跳转：50 客户端 × 500/轮 × 3 轮，预热 50；阈值 p95 中位数 ≤ 10.00 ms
  - 创建：50 客户端 × 100/轮 × 3 轮（每轮不同 URL），预热 20；阈值 p95 中位数 ≤ 15.00 ms
  - p95：`sorted[ceil(0.95*n)-1]`，先两位小数四舍五入再取中位数
  - 内存：监听进程树非僵尸进程数须为 1，`VmHWM` 总和 ≤ 10240 KiB

## 9. 结论

**未通过。**

依据（逐项）：

1. 冻结脚本判定 `RESULT: FAIL`，退出码 1；81 项中 75 PASS / 6 FAIL / 0 未执行（`report.txt`、`checks.jsonl`）。
2. 规格 §1.4 / 运行约束的硬性阈值两项不达标：跳转 p95 中位数 15.36 ms（≤10.00 ms 要求，超 53.6%）；创建 p95 中位数 17.95 ms（≤15.00 ms 要求，超 19.7%）（`perf.json`）。
3. TTL 组 4 项 FAIL：由 A10 已知边界（TTL=1 样本剩余时间不足 0.4 s，5 次重试未跨秒）导致；trace 显示服务端每次创建均正确返回，但按冻结口径计为失败，且规则禁止为改善结果重跑。
4. 达标项：内存 VmHWM 8196 KiB ≤ 10240 KiB、单进程；功能、编码、别名并发、重启持久化、非法输入等 71 项功能检查全部通过。
5. 完成条件（功能、边界、编码、重启、速度和内存全部通过）未满足：速度（跳转/创建 p95）与 TTL 到期验证未通过。

证据路径：

- 汇总：`/tmp/opencode/spec-method-2026-10-24/verifier/runs/20261024T091922Z-accept-1/report.txt`
- 逐项结果：`.../runs/20261024T091922Z-accept-1/checks.jsonl`
- HTTP 原始轨迹（226 条）：`.../runs/20261024T091922Z-accept-1/http-trace.jsonl`
- 性能与内存：`.../runs/20261024T091922Z-accept-1/perf.json`
- 参数/环境/哈希：`.../runs/20261024T091922Z-accept-1/meta.json`
- 服务日志：`.../runs/20261024T091922Z-accept-1/server-logs/`（boot-01/02/03）
- 隔离数据库：`.../runs/20261024T091922Z-accept-1/dbroot/nested/deeper/links.db`

费用：功能本身无外部付费调用；本次模型/会话用量未在验收侧取得，按约束记“未测”。
