# 冻结版本变更记录

测试编号：`spec-method-2026-10-24`

## v1（2026-10-24T02:05:56Z）

- 方法文档：`f3f2ae9a3c9368b9db073b27a375af81e56f33ae298c3226064b2e85792b50c8`
- 规格：`4cde133ec7b9f80fb7421dda55fbcc5913f8372b6e51f12db418da4987c45a1d`
- 运行约束：`157e40b76a732ca46ffc91b3b02abae356e695e7b405e0933a3cd10fa3bad122`
- 验收脚本：`acceptance.py` = `4e5e13dc7b3aa22ce21917cdd3bce9a5152221919c46c925e426a6eb1a389c60`

验收方在写脚本时发现 A1：规格同时要求“URL 不能含 ASCII 空白”和“含空格示例编码为 `%20`”，无法同时满足。尚未接收产物，也未开始验收。

## v2（2026-10-24，澄清 A1 后）

- 方法文档不变：`f3f2ae9a3c9368b9db073b27a375af81e56f33ae298c3226064b2e85792b50c8`
- 规格：`392633596d58bd00307e45ff775b4807fd59125b232da312d1fedee52d227be4`
- 运行约束不变：`157e40b76a732ca46ffc91b3b02abae356e695e7b405e0933a3cd10fa3bad122`
- 验收脚本：`acceptance.py` = `1bbfe77f78e8212653859bf9eef954da77c1619e067a4b5fd5d625e779cb8b60`

澄清内容：

1. URL 只禁止 ASCII 控制字符和 scheme/host 里的空格；路径、查询、片段里的普通空格合法，按第 1.3 节编码。
2. 超过 1 MiB 请求体固定为 `400 {"error":"invalid_request"}`。
3. SIGTERM 固定为 5 秒内退出。
4. 非法 URL 验收改为“scheme/host 带空格或含控制字符”；路径空格必须成功编码。

验收脚本已由验收方重新冻结，尚未运行；没有产物、没有验收结论。
