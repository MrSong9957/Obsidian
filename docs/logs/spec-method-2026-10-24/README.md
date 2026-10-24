# spec-method-2026-10-24 验证证据归档

- 测试编号：`spec-method-2026-10-24`
- 性质：按修订版《规格文档实战法》重建短链接样例的新一轮盲测（旧样例与旧证据已丢失，不与旧结果合并）
- 仓库：`/vault` @ `test/spec-doc-method`（归档时 HEAD `7777a72`）
- 本目录为只读证据归档；复现命令见文末

## 结论（三轮验收，均独立判定）

| 轮次 | 被测产物 SHA-256 | 脚本 | 结果 | 失败项 |
|------|------------------|------|------|--------|
| 首轮 accept-1 | `20780ce6…83811e7` | 冻结版 v2 | **未通过**（75/81） | TTL-01..04（取样失败）、PERF-03 跳转 p95 15.36ms、PERF-05 创建 p95 17.95ms；VmHWM 8196 KiB 达标 |
| 二轮 accept-2-frozen | `6cd95f43…c164e2d` | 冻结版 v2 | **未通过**（79/81） | 仅 PERF-03 跳转 p95 17.01ms、PERF-05 创建 18.24ms；TTL 4/4 过（相位幸运）；VmHWM 7548 KiB 达标 |
| 二轮 accept-2-rev | `6cd95f43…c164e2d` | 修订版（仅 TTL 取样器） | **未通过**（79/81） | 仅 PERF-03/05（17.89 / 18.44ms）；TTL 4/4 确定性取得样本并实测通过 |
| 三轮 accept-3-v3 | `6cd95f43…c164e2d` | **v3（校准后客户端）** | **已通过**（81/81） | 无；跳转各轮 p95 4.16/3.06/2.66（中位 3.06ms ≤10.00）、创建 4.21/4.49/2.68（中位 4.21ms ≤15.00）、每轮 0 错误；VmHWM 7436 KiB 达标 |

要点：

1. **TTL 争议已了结**：首轮 TTL 组失败经 trace 重算证实为验收取样器缺陷（重试 ~1ms 无相位多样性，无法跨秒，单次成功率 60%），服务端行为符合冻结规格（`expires_at = floor(now)+ttl` 向下取整）。修订版脚本（对齐到秒开始再创建）确定性取得样本，到期前 200/`expired=false`/302/clicks=1、到期后 410/`expired=true`/clicks 不变，全部实测通过。
2. **PERF-03/05 判“测量口径待决”**：验收方独立基线诊断（`verifier-diag/`）显示，最小回显对照（302 回显服务器）下同形态 50 线程客户端 p95 基线即 15.88/14.94ms——本环境、本次回显对照基线已超过跳转 10ms 阈值、达到创建 15ms 阈值，提示测量开销可能主导结果，尚不能证明物理下界。是否维持口径/换客户端/调阈值须规格作者裁决后重新冻结重测；验收方未放宽任何条件。另一客户端口径参考（进程客户端，实现方自测，仅适用于该测试条件）p95 中位数跳转 1.74ms；第二轮成组提交优化后各轮延迟中位数（冻结客户端口径）跳转 6.2/5.6/6.2ms、创建 8.3–10.1ms。
3. **第三轮（测法校准 + v3）**：按预先定稿并记哈希（`36cbd365…`）的校准计划执行 2×2 矩阵（线程/进程客户端 × 独立 C 回显服务器/被测产物，跳转+创建协议，每格 3 次完整冻结协议重复，12 次 0 错误）：跳转 p95 中位 T-E 15.20 / P-E 1.42 / T-A 17.24 / P-A 2.77 ms——线程客户端对最小回显对照仍 ~15ms（复现 diag 基线；本次对照显示客户端实现明显影响测得延迟，具体开销来源尚未单独定位），进程客户端有真实区分度。按预写采纳判据（P-E 跳转中位 ≤5.00ms 且极差 ≤30%；实测 1.42ms、2.8%）采纳替代客户端（50 个独立单线程进程）为 v3。v3 与 v2 的差异仅为客户端载体，规格契约（并发数/请求量/预热/轮次/p95 算法/阈值）不变，记为验收脚本新版本/实验新版本。同一产物 `6cd95f43…` 在 v3 口径下 **81/81 已通过**；**v2 冻结口径的未通过结果保持不变**，四轮分列不拼接；是否以 v3 为规格最终裁决口径留规格作者决定（已裁决采纳，见下节）。

## 最终裁决（2026-10-24）

依据规格作者 2026-10-24 指令，**v3 正式采纳为测试 `spec-method-2026-10-24` 的正式验收口径**。

- 采纳范围：仅本案例规格（`frozen/spec.md` v2 的业务行为与阈值不变；采纳的是验收脚本测法 v3），不外推到其他规格或环境。
- 采纳依据：校准计划预写判据（`verifier-calib/CALIBRATION-PLAN.md`，定稿 SHA-256 `36cbd365…`，测后未改）——P-E（进程客户端 × 最小回显对照）跳转 p95 中位 1.42ms ≤ 5.00ms 且极差 2.8% ≤ 30%，判定过程未参考被测产物成绩。
- 被测产物：`implementer/artifact/shortlink`，SHA-256 `6cd95f435f0e18374b1a506a6e35694229a7e66b8eacff03cfd231272c164e2d`。
- 测法文档：`verifier-v3/MEASUREMENT-v3.md`（谱系 v2→rev→v3，唯一差异为压测客户端载体，脚本哈希 `verifier-v3/SCRIPT-SHA256-v3.txt`）。
- 验收证据：本目录 `verifier-v3/runs/20261024T114349Z-accept-3-v3/`（单次运行，81/81 通过，退出码 0）。
- 验证提交：`f5937ed`（含 v3 验收与校准全部证据；本裁决记录为其后的独立文档提交）。
- **结论限定：本产物在本环境、v3 测法下通过。** 原 v2／rev 各轮“未通过”结果与全部冻结件保持原样、未被覆盖。
4. 首轮 75/81 与后续各轮各自独立保留，互不覆盖；正式验收均单次运行、未为改善数字重跑。

## 目录结构

```
frozen/          冻结材料：method.md、spec.md（v2）、run-constraints.md、CHANGELOG.md、versions.txt
verifier/        验收方材料
  scripts/       8 个冻结脚本与说明 + SCRIPT-SHA256.txt（sha256sum -c 全 OK）
  VERIFIER-REPORT.md        首轮验收报告（结论：未通过 75/81）
  VERIFIER-REPORT-ROUND2.md 二轮报告（诊断 A–D + 两轮复验 + 最终判定）
  runs/20261024T091922Z-accept-1/          首轮完整证据（report/checks/http-trace/perf/meta/server-logs/dbroot）
  runs/20261024T094842Z-accept-2-frozen/   二轮冻结版完整证据
verifier-rev/    修订版验收脚本（8 文件副本，仅 TTL 取样器改动）
  CHANGES-rev.md、sampler-fix.diff、SCRIPT-SHA256-rev.txt
  runs/20261024T094857Z-accept-2-rev/      二轮修订版完整证据
verifier-calib/  第三轮测法校准（2026-10-24）
  CALIBRATION-PLAN.md + .sha256（定稿后未改）、CALIBRATION-REPORT.md
  calib_run.py、alt_client.py、echo_server.c、calib_validation.py
  validation/（压测代码三组验证原始输出）、data/（2×2×3 重复全部原始数据）
verifier-v3/     验收脚本 v3（rev 谱系 + 校准采纳的进程客户端）
  MEASUREMENT-v3.md（与 v2 口径差异、谱系 v2→rev→v3）、SCRIPT-SHA256-v3.txt
  accept-3-v3-console.log、runs/20261024T114349Z-accept-3-v3/（完整证据）
verifier/VERIFIER-REPORT-ROUND3.md  第三轮验收报告（四轮对照 + 校准摘要 + v3 判定）
verifier-diag/   客户端基线诊断（baseline_server/client + 两次原始结果 + DIAGNOSTICS.md）
implementer/     实现方材料
  artifact/      shortlink 二进制（二轮产物）+ START.md
  build/         源码（src/main.rs、Cargo.toml/lock；不含 179M target 构建缓存）
  REPORT.md      实现报告（含 13 条假设、二轮迭代记录）
  selfcheck/     自检脚手架 selftest.py + run_selfcheck.sh + logs/（TDD 红绿灯、功能 349/349、二轮红绿灯）
  probe/         Rust 内存探针源码（h.rs、run.sh）
orchestrator/    编排方记录：resume-log.md（全过程时间线+断点）、assumption-check.md（13 条假设核对+脚手架修改记录）
```

## 关键哈希

```
frozen/method.md        f3f2ae9a3c9368b9db073b27a375af81e56f33ae298c3226064b2e85792b50c8
frozen/spec.md (v2)     392633596d58bd00307e45ff775b4807fd59125b232da312d1fedee52d227be4
frozen/run-constraints.md 157e40b76a732ca46ffc91b3b02abae356e695e7b405e0933a3cd10fa3bad122
frozen/CHANGELOG.md     622982d22a30ab1ca3bef6d06ab5047f513f57ec7c7e351d6d02c888bd7747e6
verifier/scripts/acceptance.py 1bbfe77f78e8212653859bf9eef954da77c1619e067a4b5fd5d625e779cb8b60
verifier/scripts/acc_functional.py 34131ce65b9182275a0a186fdfec4259830465527e00ee1c77f5fb345cec255b
verifier-rev/acc_functional.py 8989931967e009eb66c99dff1539b2db2f7e2f429f7d12145baf88a0550c4d3a（取样器修复版唯一改动）
verifier-v3/acc_perf.py 1b996e34c77e7851c31158f1af844016fcc35f9c6e598f003072eebcb0cdfe3a（v3 唯一改动，其余与 rev 相同）
verifier-calib/CALIBRATION-PLAN.md 36cbd3658ac137c3644774813f1ee010469679ea53c407d2a8dd4a4111a93123（定稿后未改）
verifier-calib/CALIBRATION-REPORT.md f6425a5e6b1fe31b4216e88aa7feecb35a4592e80a6616f883ab7cc29affa926
verifier/VERIFIER-REPORT-ROUND3.md f023002b47d14bccbe3bb438a92dc2b3bd8dd1c0ef700226576773b0e638eeed
implementer/artifact/shortlink（二轮产物）6cd95f435f0e18374b1a506a6e35694229a7e66b8eacff03cfd231272c164e2d
implementer/build/src/main.rs d51189a857d4ff1d6691968a2459553aa3194fd78126d5e15fe988f25154f1ee
首轮产物（未留存，见“证据缺口”）20780ce63419c1e7841ff86c281fd971f27a2b099206fdda0ce62345783811e7
```

其余哈希：`verifier/scripts/SCRIPT-SHA256.txt`、`verifier-rev/SCRIPT-SHA256-rev.txt`、各 run 目录 `meta.json` 的 `script_sha256`/`component_sha256` 可交叉核对。本目录内 `.gitattributes` 声明 `* -text`，禁止 git 对证据文件做换行规范化，保证校验和与原件逐字节一致。

## 会话角色与隔离说明

- **编排方**（本会话）：冻结材料核验、会话编排、假设核对、归档。未代做实现或验收判定。
- **实现方**：两次独立后台会话（一轮实现 + 一轮修复接续），输入仅为冻结规格、运行约束、环境信息与同角色脚手架（probe/、selfcheck/）+ 第二轮的失败清单（输入/预期/实际偏差）。未接触验收脚本、方法文档、编排对话。**实现输入包含既有自检脚手架，非“只看规格”的严格盲测。**
- **验收方**：三次独立后台会话（一轮冻结验收 + 二轮诊断复验 + 三轮测法校准与 v3 验收，原会话失效后每次均以无上下文新会话接续），输入为冻结规格、运行约束、冻结/谱系验收脚本、可运行产物与 START.md；未读实现源码、实现报告与自检结论。诊断与校准均由验收方独立完成。

## 勘误（2026-10-24 校准轮追加）

- `verifier/VERIFIER-REPORT-ROUND2.md` 与本 README 初版中“PERF-03 在该口径下对任何服务端**物理不可达**”属过度结论，更正为：“本环境、本次回显对照基线超过跳转阈值，提示测量开销可能主导结果，尚不能证明物理下界。”报告原文按原始证据保留不动。
- “服务端真实贡献：进程客户端口径 p95 跳转 1.74ms”更正为：“另一客户端口径（进程客户端，实现方自测）下测得的 p95 中位数，仅适用于该测试条件。”
- 首轮证据状态明确为：“原始日志可审阅，原产物及源码缺失，无法精确复验。”
- 归因措辞收窄（2026-10-24 裁决轮）：早前解释性文字中的“零成本回显／零服务端成本”更正为“最小回显对照”；“下限来自客户端（调度）机制”更正为“本次对照显示客户端实现明显影响测得延迟，具体开销来源尚未单独定位”。角色报告（VERIFIER-REPORT-ROUND2/ROUND3、CALIBRATION-REPORT 等）与原始日志中的旧措辞按原始证据保留不动，以本勘误为准。

## 证据缺口（如实记录）

1. 首轮被测产物二进制（`20780ce6…`）在实现方二轮迭代时被原路径覆盖，cargo deps 缓存同名文件亦被覆盖，**无法恢复**；其身份由首轮报告与 `runs/…accept-1/meta.json` 固定的 SHA-256 及完整行为证据承载。首轮实现源码同样无版本化快照（源码现为二轮版本）。首轮原始日志可审阅，但原产物及源码缺失，无法精确复验。
2. 实现方对自检脚手架 selftest.py 的 4 处修改（参数个数、别名短码校验、CSV API、压测客户端线程→进程）无修改前快照，无法逐行 diff；描述以实现报告为准，佐证为 `floor-client-overhead.log`。
3. `implementer/selfcheck/work/`（自检临时运行目录，25M）与 `probe/h`（探针编译产物）未归档：可由源码与日志复现，非证据本体。
4. 模型/会话费用：平台侧用量未取得，按约束记“未测”。

## 复现命令

```bash
# 1. 脚本冻结校验
cd verifier/scripts && sha256sum -c SCRIPT-SHA256.txt

# 2. 冻结版验收（二轮产物）
cd verifier/scripts   # 或复制到独立目录并指向 frozen/
python3 acceptance.py \
  --start-cmd 'exec ./shortlink' \
  --start-cwd <implementer/artifact 绝对路径> \
  --start-notes @<implementer/artifact/START.md 绝对路径> \
  --run-id "<UTC时间戳>-accept"

# 3. 修订版验收（仅 TTL 取样器不同）
cd verifier-rev && python3 acceptance.py <同上参数>

# 3b. v3 验收（校准后进程客户端；先读 MEASUREMENT-v3.md）
cd verifier-v3 && python3 acceptance.py <同上参数>

# 3c. 测法校准矩阵复现（详见 verifier-calib/CALIBRATION-REPORT.md）
gcc -O2 -Wall -Wextra -o echo_server verifier-calib/echo_server.c
python3 verifier-calib/calib_run.py   # 2×2×3 重复，参数见 CALIBRATION-PLAN.md

# 4. 客户端基线诊断
cd verifier-diag && python3 baseline_server.py & python3 baseline_client.py

# 5. 重新构建产物（Rust 1.99，离线缓存）
cd implementer/build && cargo build --release --offline
```

环境：Linux 6.18.33.2-microsoft-standard-WSL2 x86_64，16 vCPU，约 15.5 GiB 内存；rustc/cargo 1.99.0；Python 3.12.3；仅 127.0.0.1。测量参数固定于 `acc_perf.py` 顶部（1000 预置、50 并发、跳转 500×3 轮预热 50、创建 100×3 轮预热 20、p95 最近秩取三轮中位、VmHWM ≤10240 KiB）。
