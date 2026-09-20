# TDD 修复日志：wiki-query P0 缺陷（风险盘点的 R1/R2/R3/R4/R7）

日期：2026-09-20 ｜ 基线：9b855cc ｜ 全量测试 133 → 161（+28，0 失败）

## 底层逻辑与修复

| 缺陷 | 根因 | 最小修复 |
|---|---|---|
| 错误 cwd 整库 prune | sync.rs run_with_scope 无条件 prune_orphans，错误 cwd 下 on_disk 不含真库路径→全删 | scan 后按 serve 同判据守卫：cwd 无 wiki/ 且无 sources/ → 跳过 prune 并 stderr 警告 |
| check_scope `..` 越上界假阳性 | normalize_path 的 pop 无根绑定，`../../../wiki/x.md` 解析成 `wiki/x.md` | normalize_path_checked 返回 (路径, escaped_root)，越根判 false；原 normalize_path 签名不变 |
| 绝对路径缺 vault_root 假阴性 | check_scope 仅用可选参数，无根可剥离 | vault_root 缺省回退 WIKI_VAULT_ROOT env（serve.sh 恒设置） |
| sync 提醒无前置条件 | hint 无条件"请删除源记忆文件" | 文案改为"先确认目标知识文件已成功写入，再删除；sync 成功不代表提炼完成" |
| sync_file 失败静默 | `.is_ok()` 丢弃错误，SyncReport 无字段 | SyncReport 加 `failed: i32`；失败 stderr 警告 + 计数 |
| 同秒修改漏更新 | mtime 存秒级字符串 | needs_update 与 sync_file 统一 `%Y-%m-%d %H:%M:%S%.3f`（存量库首次 sync 全量迁移一次后幂等恢复，实测 475 updated→0） |

## TDD 测试点（真实 RED → GREEN）

- tests/functional_sync_test.rs（新增 11）：F1 数据正确性 6 项（两表字段/更新/删除/幂等全表 dump/无 fm 回退/未闭合回退/深层路径）+ F2 两项 + 3 项 RED：同秒双写（对齐秒边界+jieba 预热后稳定复现）、failed 可观测、错误 cwd 不清库
- tests/functional_tools_test.rs（新增 10）：search 命中/字段一致/description 也索引/AND/OR/空与标点；list type 精确/tag 子串/updated DESC/limit 截断/组合 AND 与 SQL 口径一致
- sync_edge_cases_test.rs scope_tests（+2）：越根 false（RED→GREEN）、wiki2/sources-dev 非前缀回归
- mcp_e2e_full_test.rs（+5，helper 加 env 变体）：绝对路径 env 回退（RED→GREEN）、提醒前置条件（RED→GREEN）、failed 字段回归、limit 类型错误校验、模板缺失警告
- 测试陷阱修正：MCP text 内 JSON 引号转义为 `\"`，子串断言须用转义形式；jieba 分词破坏中文连续子串，FTS content 断言用英文整词

## 验证

- cargo test 全量 161/161（原 133 + 新 28），functional_sync_test 连续 3 次稳定全绿
- mcp-config.test.sh 9/10（T7b 为 1a718e9 存量失败，另行记录）
- build.sh 等价流程构建 + 改名法部署新 exe；vault 根 CLI sync 实测：475 updated（毫秒迁移）→ 再跑 0 updated/0 pruned（幂等恢复）
- diff 检查：6 改 + 2 新增均在任务范围；无无关格式/文档改动

## 遗留（范围外）

1. T7b 存量失败（AGENTS.md 构建说明被 1a718e9 移除）。
2. 查重报告阈值、跨平台 prune 分隔符、schema 迁移、project-rules 注入通道等 P2 缓暂项（见风险盘点）。
3. 本轮修复的 exe 已部署，但当前会话 MCP 进程仍为 9b855cc 版，重启后生效。
4. e2e wiki_sync 测试在错误 cwd（crate 目录）跑杂散库的既有现象未改（守卫已防 prune，建/更新仍会发生），属部署形态问题。
