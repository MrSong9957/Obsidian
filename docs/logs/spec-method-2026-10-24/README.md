# spec-method-2026-10-24 验证证据归档

- 测试编号：`spec-method-2026-10-24`
- 性质：按修订版《规格文档实战法》重建短链接样例的新一轮盲测（旧样例与旧证据已丢失，不与旧结果合并）
- 仓库：`/vault` @ `test/spec-doc-method`（归档时 HEAD `7777a72`）
- 本目录为只读证据归档；复现命令见文末

## 结论（三轮验收，均独立判定）

| 轮次 | 被测产物 SHA-256 | 脚本 | 结果 | 失败项 |
|------|------------------|------|------|--------|
| 首轮 accept-1 | `20780ce6…83811e7` | 冻结版 | **未通过**（75/81） | TTL-01..04（取样失败）、PERF-03 跳转 p95 15.36ms、PERF-05 创建 p95 17.95ms；VmHWM 8196 KiB 达标 |
| 二轮 accept-2-frozen | `6cd95f43…c164e2d` | 冻结版 | **未通过**（79/81） | 仅 PERF-03 跳转 p95 17.01ms、PERF-05 创建 18.24ms；TTL 4/4 过（相位幸运）；VmHWM 7548 KiB 达标 |
| 二轮 accept-2-rev | `6cd95f43…c164e2d` | 修订版（仅 TTL 取样器） | **未通过**（79/81） | 仅 PERF-03/05（17.89 / 18.44ms）；TTL 4/4 确定性取得样本并实测通过 |

要点：

1. **TTL 争议已了结**：首轮 TTL 组失败经 trace 重算证实为验收取样器缺陷（重试 ~1ms 无相位多样性，无法跨秒，单次成功率 60%），服务端行为符合冻结规格（`expires_at = floor(now)+ttl` 向下取整）。修订版脚本（对齐到秒开始再创建）确定性取得样本，到期前 200/`expired=false`/302/clicks=1、到期后 410/`expired=true`/clicks 不变，全部实测通过。
2. **PERF-03/05 判“测量口径待决”**：验收方独立基线诊断（`verifier-diag/`）显示，零服务端成本（最小 302 回显服务器）下同形态 50 线程客户端 p95 基线即 15.88/14.94ms——高于跳转 10ms 阈值、达到创建 15ms 阈值，即冻结口径下跳转阈值对任何服务端物理不可达。是否维持口径/换客户端/调阈值须规格作者裁决后重新冻结重测；验收方未放宽任何条件。服务端真实贡献：进程客户端口径 p95 跳转 1.74ms（实现方自测）；第二轮成组提交优化后各轮延迟中位数（冻结客户端口径）跳转 6.2/5.6/6.2ms、创建 8.3–10.1ms。
3. 首轮 75/81 与二轮两份 79/81 各自独立保留，互不覆盖；三轮均单次运行、未为改善数字重跑。

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
verifier-rev/acc_functional.py 8989931967e009eb66c99dff1539b2db2f7e2f429f7d12145baf88a0550c4d3a（唯一改动文件）
implementer/artifact/shortlink（二轮产物）6cd95f435f0e18374b1a506a6e35694229a7e66b8eacff03cfd231272c164e2d
implementer/build/src/main.rs d51189a857d4ff1d6691968a2459553aa3194fd78126d5e15fe988f25154f1ee
首轮产物（未留存，见“证据缺口”）20780ce63419c1e7841ff86c281fd971f27a2b099206fdda0ce62345783811e7
```

其余哈希：`verifier/scripts/SCRIPT-SHA256.txt`、`verifier-rev/SCRIPT-SHA256-rev.txt`、各 run 目录 `meta.json` 的 `script_sha256`/`component_sha256` 可交叉核对。本目录内 `.gitattributes` 声明 `* -text`，禁止 git 对证据文件做换行规范化，保证校验和与原件逐字节一致。

## 会话角色与隔离说明

- **编排方**（本会话）：冻结材料核验、会话编排、假设核对、归档。未代做实现或验收判定。
- **实现方**：两次独立后台会话（一轮实现 + 一轮修复接续），输入仅为冻结规格、运行约束、环境信息与同角色脚手架（probe/、selfcheck/）+ 第二轮的失败清单（输入/预期/实际偏差）。未接触验收脚本、方法文档、编排对话。**实现输入包含既有自检脚手架，非“只看规格”的严格盲测。**
- **验收方**：两次独立后台会话，输入为冻结规格、运行约束、冻结验收脚本、可运行产物与 START.md；未读实现源码、实现报告与自检结论。二轮诊断（任务 A–D）由验收方独立完成。

## 证据缺口（如实记录）

1. 首轮被测产物二进制（`20780ce6…`）在实现方二轮迭代时被原路径覆盖，cargo deps 缓存同名文件亦被覆盖，**无法恢复**；其身份由首轮报告与 `runs/…accept-1/meta.json` 固定的 SHA-256 及完整行为证据承载。首轮实现源码同样无版本化快照（源码现为二轮版本）。
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

# 4. 客户端基线诊断
cd verifier-diag && python3 baseline_server.py & python3 baseline_client.py

# 5. 重新构建产物（Rust 1.99，离线缓存）
cd implementer/build && cargo build --release --offline
```

环境：Linux 6.18.33.2-microsoft-standard-WSL2 x86_64，16 vCPU，约 15.5 GiB 内存；rustc/cargo 1.99.0；Python 3.12.3；仅 127.0.0.1。测量参数固定于 `acc_perf.py` 顶部（1000 预置、50 并发、跳转 500×3 轮预热 50、创建 100×3 轮预热 20、p95 最近秩取三轮中位、VmHWM ≤10240 KiB）。
