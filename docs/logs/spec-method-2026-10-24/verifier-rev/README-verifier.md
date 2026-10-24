# 独立验收脚本使用说明（verifier）

本目录是 `spec-method-2026-10-24` 的独立验收侧材料。脚本只依据冻结的
`../frozen/spec.md` 和 `../frozen/run-constraints.md` 编写，只启动调用方给出的
命令并通过 `127.0.0.1` 做 HTTP 验收；不读取实现源码、不访问外网、不安装依赖。
脚本已冻结，但**尚未对任何实现产物运行**。

2026-10-24 规格作者澄清 A1/A3/A4 后，脚本与说明已按澄清后的
`../frozen/spec.md` 更新并重新冻结；仍无验收运行。

## 1. 文件与冻结校验

| 文件 | 作用 |
|------|------|
| `acceptance.py` | 入口：81 项检查计划、参数、启动/停止、报告 |
| `acc_core.py` | HTTP、证据记录、服务进程管理、`/proc` 检查 |
| `acc_functional.py` | 创建、统计、跳转、编码、别名、点击并发、TTL |
| `acc_validation.py` | 非法输入、未知短码、错误体、超限请求 |
| `acc_restart.py` | SIGTERM / SIGKILL、重启持久化 |
| `acc_perf.py` | 1000 条预置、固定压测、p95、VmHWM |
| `ambiguities.md` | A1–A27 歧义与判定口径（A1 已解决，A3/A4 已固定）；验收报告必须引用 |
| `SCRIPT-SHA256.txt` | 上述脚本与两份说明文档的 SHA-256 |

冻结校验（在 verifier 目录执行）：

```bash
cd /tmp/opencode/spec-method-2026-10-24/verifier
sha256sum -c SCRIPT-SHA256.txt
```

每行应输出 `OK`。任一行失败，说明当前文件不是冻结版本，不得把运行结果与冻结
证据混用。运行时 `meta.json` 还会记录 `acceptance.py` 及各 `acc_*.py` 的
SHA-256，可与 `SCRIPT-SHA256.txt` 交叉核对。

## 2. 前置条件

- Linux（本项目为 WSL2），Python 3.12，仅标准库；不需要 pip 安装。
- 服务端产物已按实现方的 `START.md` 构建好，并且能以前台进程运行。
- 服务端必须读取脚本注入的 `PORT` 和 `DB_PATH`：
  - `PORT=0`，由系统分配端口；
  - `DB_PATH=<run-dir>/dbroot/nested/deeper/links.db`，其中
    `dbroot/nested/deeper` 在启动时故意不存在，用于检查父目录自动创建。
- 服务端必须向 stdout 打印一行并立即刷新：

  ```text
  listening on http://127.0.0.1:<实际端口>
  ```

  脚本对该行做完整匹配（允许行尾 CRLF），并从该行取得实际端口。
- 启动命令必须保持前台；脚本用单独的会话/进程组启动它，结束时先 SIGTERM，
  超时再 SIGKILL。不要用 `nohup`、`&`、`setsid` 或自行 daemonize。
- 服务端不得自己覆盖 `PORT` / `DB_PATH`；启动命令里也不要再写
  `PORT=...` / `DB_PATH=...` 前缀。

## 3. 运行方法

先阅读产物附带的 `START.md`，把其中的启动命令、工作目录和必要环境变量转换成
下面的参数。`--start-notes` 只是**记录**启动说明，不会解析或执行它。

```bash
cd /tmp/opencode/spec-method-2026-10-24/verifier

python3 acceptance.py \
  --start-cmd 'exec ./shortlink' \
  --start-cwd /path/to/artifact \
  --start-notes @/path/to/artifact/START.md \
  --run-id "20261024T120000Z-accept-1"
```

示例中的 `exec ./shortlink`、`/path/to/artifact` 必须替换为实际值。用 `exec`
可以避免多留一个包装 shell；如果二进制需要参数，把它们写进同一个单引号字符串。

如果不使用 `@文件`，`--start-notes` 也可以直接传一段说明文字。建议始终传
`--start-cwd`；省略时启动命令继承验收脚本当前目录。

### 参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--start-cmd` | 环境变量 `ACCEPT_START_CMD` | 必需；由 `/bin/sh -c` 执行 |
| `--start-cwd` | 当前目录 | 启动命令的工作目录；必须是已存在目录 |
| `--start-env KEY=VALUE` | 无 | 追加环境变量，可重复；空键或缺少 `=` 时退出 2 |
| `--start-notes` | 环境变量 `ACCEPT_START_NOTES` | 说明文字；`@path` 读取文件并记录 |
| `--startup-timeout` | `30.0` | 等待启动行的秒数；必须 > 0 |
| `--work-dir` | `<verifier>/runs` | 证据根目录 |
| `--run-id` | UTC 时间戳 + 进程号 | 本次证据子目录名；已存在则拒绝运行 |
| `--spec` | `../frozen/spec.md` | 冻结规格；必须存在 |
| `--constraints` | `../frozen/run-constraints.md` | 冻结运行约束；必须存在 |

性能参数**没有**命令行覆盖项，全部固定在 `acc_perf.py` 顶部；测试开始后不得修改。

## 4. 证据目录

每次运行创建 `<work-dir>/<run-id>/`，脚本不删除它：

| 路径 | 内容 |
|------|------|
| `report.txt` | 汇总；每个检查一行 `PASS`/`FAIL`，末尾 `RESULT: PASS/FAIL` |
| `checks.jsonl` | 每个检查一行 JSON，逐行 flush，含时间、ID、ok、详情 |
| `http-trace.jsonl` | 功能阶段 HTTP 请求/响应；原始头、状态、响应体（截断到 2000 字符）、耗时 |
| `meta.json` | 参数、命令、环境、平台、CPU/内存、spec/constraints/脚本 SHA-256、检查计划 |
| `perf.json` | 固定参数、跳转/创建每轮 500/100 个延迟样本、每轮 p95、中位数、内存快照 |
| `server-logs/boot-01.out/.err` … | 服务每次启动的原始 stdout/stderr（重启会生成 boot-02、boot-03） |
| `dbroot/nested/deeper/links.db` | 本次隔离数据库；留给人工复查 |
| `errors.log` | 仅在某阶段抛异常时出现，含 traceback |

`report.txt` 和 `checks.jsonl` 都应恰好有 81 条检查；任何未执行的检查会在结束时
显式写成 `FAIL ... NOT EXECUTED`，不会静默跳过。阶段异常不会中断后续阶段；
唯一例外是重启阶段异常：此时不会再跑性能阶段，剩余检查同样显式记为未执行。

## 5. 退出码与结论

| 退出码 | 含义 |
|--------|------|
| `0` | 81 项全部 PASS |
| `1` | 至少一项 FAIL，或有检查未执行；服务无法启动也属于此类 |
| `2` | 命令行/环境变量/路径/规格文件等调用错误，未开始验收 |

`RESULT: PASS` 只是脚本的原始判定；按运行约束，最终验收报告必须由验收方写成
“已通过”或“未通过”，并列出每项结果、原始证据路径、歧义、环境、命令。不得只引用
实现方自检，也不得在未运行时写“已通过”。

## 6. 覆盖范围（81 项）

| 组 | 数量 | 主要内容 |
|----|------|----------|
| `START` | 7 | `PORT=0` 启动行/实际端口/临时端口范围、TCP 可连、父目录与 DB 创建、SQLite 头、单进程、启动行可读耗时 |
| `HTTP` | 4 | HTTP/1.1、同连接 keep-alive 两次请求、所有已检查响应的 `Content-Length`、成功响应 `Content-Type` |
| `CREATE` | 15 | 201 对象字段、`short_url` 实际端口、7 位自动码、`expires_at`、alias（含 `null`）、TTL 边界/`null`、未知字段、同 URL 两次、20 个唯一码、恰好 1 MiB、alias 字符/长度、Content-Type 大小写与 charset |
| `STATS` | 7 | 六字段对象、URL、clicks、created_at、expires_at/expired、code 回显、未知码 404 |
| `REDIR` | 4 | 302/Location/CL=0、点击 +1、未知码 404、50 并发点击恰好 +50 |
| `ENC` | 7 | `%2F`、`%2f`、`%20` 原样保留；中文/空格示例严格 201 + 原始 `Location` 精确匹配（路径/查询/片段空格合法）；原始头比较；中文 URL 编码；stats 返回原 UTF-8 URL |
| `ALIAS` | 4 | 20 并发 1×201/19×409、大小写敏感、重复 alias 409、并发失败体 |
| `TTL` | 4 | 到期前 200/302/clicks=1；到期后 410；stats 200/expired=true/点击数不变；再次跳转不增计数 |
| `RESTART` | 6 | SIGTERM 5 秒内退出并释放端口、链接/URL/点击/时间戳持久化、重启后继续计数、TTL 时间戳保留、201 已提交（SIGKILL）、脏停后一致性 |
| `VALID` | 15 | 缺/错 Content-Type、坏 JSON、非对象、url 类型/缺失、scheme/host 带空格、控制字符 0x00–0x1F/0x7F、UTF-8 长度 2048/2049、非法 `%`、TTL 类型/范围、alias 非法/保留/重复、错误体单字段、超 1 MiB → 400 invalid_request、超限后仍可服务 |
| `PERF` | 8 | 1000 条预置、种子抽查、跳转 3 轮/预热 50、跳转 p95、创建 3 轮/预热 20、创建 p95、VmHWM、单进程 |

其中 `PERF-SEED-SPOT` 只抽查 `seed0001` 和 `seed1000`，不是把 1000 条全部读一遍；
`PERF-01` 要求 1000 条全部创建成功。

## 7. 固定性能口径

- 预置：`seed0001`–`seed1000`，URL `https://example.com/seed/<code>`；用 20 个并发
  客户端创建（约束未规定创建并发，见 A21）。
- 跳转：50 个并发客户端 × 每客户端 10 次 = 每轮 500 次；预热 50 次；3 轮。
  每轮 p95 用最近秩 `sorted[ceil(0.95*n)-1]`；取 3 轮 p95 的中位数；≤ 10.00 ms。
- 创建：50 个并发客户端 × 每客户端 2 次 = 每轮 100 次；预热 20 次；3 轮；每轮
  URL 不同；同样取 p95 中位数；≤ 15.00 ms。
- 内存：压测结束后对“持有监听 socket 的进程及其后代”求和 `VmHWM`，要求非僵尸
  进程数为 1，且总和 ≤ 10240 KiB。客户端进程不计入。
- 预热是单连接串行（A20）；正式轮次每个客户端使用一条持久连接。
- 脚本先把每轮 p95 四舍五入到两位，再取中位数（A15）。
- 阈值、并发数、轮次、预热数、取数方式均不得在测试后调整；失败就是失败。

## 8. 歧义与判定口径

完整列表见 `ambiguities.md`（A1–A27），验收报告必须引用。执行前至少确认：

- **A1（已解决）**：规格作者已澄清：URL 只禁止 ASCII 控制字符（0x00–0x1F、0x7F）
  和 scheme/host 中的空格；路径、查询、片段中的普通空格合法，必须按 1.3 编码。
  `ENC-04` 对规格示例和 `https://example.com/a b?c=d e#f g` 都要求 201 + 302 +
  原始 `Location` 精确匹配，不再接受 400。`VALID-05` 测 scheme/host 空格，
  `VALID-07` 测控制字符；路径空格合法由 `ENC-04` 覆盖。
- **A3（已固定）**：超过 1 MiB 请求体必须返回 `400 {"error":"invalid_request"}`；
  其他 4xx、连接关闭、超时、崩溃均失败；`VALID-15` 确认服务仍可响应。
- **A4（已固定）**：SIGTERM 后 5 秒内退出；脚本以 ≤5.0 s 实测并检查端口释放，
  实际值写入证据。
- **A14**：1 MiB 按包含上界理解：恰好 1 MiB 接受，超过 1 MiB 返回
  `400 invalid_request`。
- **A5/A16**：服务进程按监听 socket 的进程树计；包装 shell 若在树外不计内存，也
  不算服务进程。
- **A10**：TTL 用 1 秒等待到期；脚本在 `expires_at + 0.35 s`（仍处在到期那一秒）
  断言，正常情况下检查的是 `<=` 边界，但无法完全消除调度跨秒。
- **A26**：`PORT=0` 的启动端口会检查是否落在
  `/proc/sys/net/ipv4/ip_local_port_range` 内；这是排除“忽略 PORT 绑固定端口”的
  启发式，不能证明一定来自 `bind(0)`。
- **A27**：“立即刷新”没有独立数值阈值；`START-06` 记录启动行实际读取耗时。

## 9. 已知边界

- 脚本不读实现源码，不做代码审查；代码审查另做。
- 不测试规格未定义的路由、方法、HEAD/OPTIONS、尾斜杠、HTTP pipelining、JSON
  重复键/BOM、IPv6/主机名边界等（见 A12/A25）。
- 不监控服务是否向外发起网络请求；验收环境本身不联网。
- 不并发启动第二个服务实例来证明 `PORT=0` 的端口冲突行为（A26）。
- 不计算服务产物的 SHA-256；按运行约束，验收方应在运行目录或报告中另行记录
  产物哈希/最小快照。
- 依赖 Linux `/proc` 读取监听 inode、进程树和 `VmHWM`；非 Linux 环境不适用。

## 10. 当前状态

脚本已按规格澄清更新、完成静态检查并重新冻结哈希，**尚未运行**。收到可运行
产物和 `START.md` 后，由验收方按第 3 节执行；在此之前没有验收结论。
