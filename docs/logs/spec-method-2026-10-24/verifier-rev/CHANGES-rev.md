# verifier-rev 修订说明（第二轮验收）

来源：`../verifier/` 8 个冻结文件的逐字节副本，仅 `acc_functional.py` 的
TTL 取样器加一处对齐等待（REV-1）。2026-10-24。

## 为什么只改取样器

首轮运行（runs/20261024T091922Z-accept-1）TTL-01..04 失败的诊断结论
（见 ../verifier-diag/DIAGNOSTICS.md 任务 A）：冻结取样器在 TTL=1 样本
剩余 <0.4 s 时背靠背重试 5 次，但 5 次请求合计仅约 1 ms，全部落在同一
wall-clock 秒的同一秒内位置，实质是同一个样本，无法取得"剩余 ≥0.4 s"
的样本（均匀到达下失败率 40%）。服务端 5 次创建行为均正确（201 +
expires_at = floor(now)+1，符合规格 L34）。属取样器缺陷，非实现违约。

## 修改内容（REV-1，唯一改动）

`acc_functional.py` `phase_ttl()`：创建样本前 `time.sleep(1.05 - (time.time() % 1.0))`
把发送时刻对齐到下一 wall-clock 秒开始约 0.05 s 处，使剩余时间 ≈ 1.0 s − RTT，
`exp - time.time() >= 0.4` 的接受条件必然成立。

统一 diff（`diff -u verifier/acc_functional.py verifier-rev/acc_functional.py`）：

```diff
--- verifier/acc_functional.py  2026-10-24 03:12:53
+++ verifier-rev/acc_functional.py  2026-10-24 09:47:49
@@ -664,6 +664,15 @@
     attempts = 0
     for attempt in range(5):
         attempts = attempt + 1
+        # REV-1 (sampler fix only, 2026-10-24 round-2): sleep into the start of
+        # the next wall-clock second before creating the TTL=1 sample, so the
+        # remaining time is ~1.0s - RTT and the >= 0.4s acceptance condition
+        # below is guaranteed to hold. Round-1 diagnosis showed the previous
+        # back-to-back retries (span ~1ms) all sample the same frac(now) and
+        # fail together when the first attempt lands in the last 0.4s of a
+        # second. Assertions, thresholds and the acceptance condition itself
+        # are unchanged.
+        time.sleep(1.05 - (time.time() % 1.0))
         url = f"https://example.com/ttl-expiry/{attempt}"
         r = post_json(rec, port, {"url": url, "ttl_seconds": 1}, label=f"create-ttl-expiry-{attempt}")
         obj = r.json_body()
```

## 未被削弱的部分

- TTL-01..04 的全部业务断言（到期前 200/expired=false/302/clicks=1；到期后
  410；stats 200/expired=true/clicks 不变；再次跳转不增计数）逐字未动；
- 接受条件 `exp - time.time() >= 0.4`、重试上限 5 次、A10 的 `expires_at + 0.35 s`
  断言时机均未改动（对齐只是让条件可被满足，不放松条件本身）；
- 其余 7 个文件与冻结版逐字节一致（见 SCRIPT-SHA256-rev.txt，7 项哈希与
  ../verifier/SCRIPT-SHA256.txt 相同，仅 acc_functional.py 变化）；
- 所有性能参数、阈值、81 项检查计划无任何变化。

## 哈希

见 `SCRIPT-SHA256-rev.txt`。acc_functional.py：
冻结版 `34131ce65b9182275a0a186fdfec4259830465527e00ee1c77f5fb345cec255b`
→ 修订版 `8989931967e009eb66c99dff1539b2db2f7e2f429f7d12145baf88a0550c4d3a`。
