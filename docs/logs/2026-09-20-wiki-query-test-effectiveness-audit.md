# 测试有效性审计日志（mutation 验证）

日期：2026-09-20 ｜ 基线/终态：172/172 ｜ mutation：12 个（15 个测试组）｜ 假测试：0

## 方法

对每条核心生产链路注入"业务上错误但技术上合法"的最小 mutation → 对应测试必须 RED 且 RED 断言为该业务错误 → 精确反向 Edit 恢复（工作区有未提交合法改动，禁用 git checkout）→ 同测试重新 GREEN → 全文确认 MUTATION 标记零残留。

## Mutation 记录

| # | 生产链路 | 注入点与错误 | 捕获测试（RED） | 恢复后 |
|---|---|---|---|---|
| M1a | F1 入库字段 | sync_file INSERT name/description 互换 | sync_new_file_writes…（"name 应取自 frontmatter"） | GREEN |
| M1b | F1 正文入 FTS | FTS content 置空 | update/without_frontmatter/same_second 3 条 RED | GREEN |
| M2 | F1 FTS 一致性 | 更新时跳过 FTS 旧记录 DELETE | sync_update_refreshes（残留旧正文） | GREEN |
| M3 | F2 prune 删除链 | prune_orphans 入口恒返回 0 | sync_delete_prunes / sync_moved_file 2 条 RED（prune_keeps/wrong_cwd 保持绿=正交契约正确） | GREEN |
| M4a | F3 AND 语义 | connector 恒 " OR " | 集成 search_multi_word + 单元 lowercase_or 双层 RED | GREEN |
| M4b | F3 top10 | LIMIT 10→5 | search_caps_results_at_ten | GREEN |
| M5 | F4 排序 | DESC→ASC | list_orders + list_limit_takes 2 条 RED | GREEN |
| M6 | F5 越根契约 | escaped_root 检测禁用 | escape_root_dots_not_in_scope | GREEN |
| M7a | F6 报告数值 | updated 虚报 +1 | new_file/idempotent 等 4 条 DB 核对 RED | GREEN |
| M7b | F6 提醒契约 | 提醒回退无前置条件文案 | hint_requires_confirmation RED；旧宽断言测试不红（证明补强必要） | GREEN |
| M8 | F7 目标映射 | claude 恒映射 AGENTS.md | claude_targets_claude_md | GREEN |
| M9 | F8 工具注册 | list_tools truncate(4) | mcp_tools_list_exposes_exactly_five | GREEN |
| M10 | P0 失联保护 | `if false` 禁用 | subtree_unavailable/recovery 2 条 RED | GREEN |
| M11 | P0 schema 校验 | 校验结果忽略 | rejects_incompatible/search_on_incompatible 2 条 RED | GREEN |
| M12 | P1 分隔符归一 | 入库 replace 取消 | mixed_separator/forward_slash 2 条 RED | GREEN |

## 结论

- 15 个测试组全部有效，**未发现假测试**；0 个测试需要重写。
- 两次弱断言说明（非假测试，已有强断言覆盖同一链路）：旧 `hint_remembers`（M7b 下不红）与 e2e 字段存在性冒烟断言——同链路的强契约测试（M7b/functional_sync DB 级核对）均能捕获对应 mutation，宽断言保留为冒烟层。
- M3 首次注入失误（双循环导致真实 prune 仍执行）重做后验证——属 mutation 工艺问题，非测试缺陷。
- 全量 172/172；`MUTATION` 标记残留 0；git diff 与审计前一致（374 = 上轮 372 + .gitignore 补行 2）。
