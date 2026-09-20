# TDD 修复日志：wiki-query 暂缓风险 P0/P1

日期：2026-09-20 ｜ 基线：166/166 ｜ 完成后：172/172（+6，零回归）

## 缺陷、根因与最小修复

| 风险 | 根因 | 最小修复 |
|---|---|---|
| P0-1 整子树失联批量 prune | prune_orphans 只做文件级 exists()，父目录失联时整个子树 exists 同假→被当正常删除批量清空 | 父目录失联保护：候选行的父目录不存在→疑似失联，保守保留 + stderr 警告（父目录存在的单文件删除仍正常 prune，不退化成永不 prune） |
| P0-2 不兼容 schema 半坏 | CREATE IF NOT EXISTS 静默跳过旧表→sync_file 逐文件失败(failed 计数)但库保持旧 schema，甚至静默补建缺失表 | db.rs init() 开库即校验 documents/documents_fts 列集合，不符→明确 Err(提示删库重建)；run_with_scope 捕获→报告 failed=-1(库不可用)+stderr，不写任何行 |
| P1 跨平台分隔符不一致 | 入库保留 OS 分隔符(Windows `\`)，prune 比对/on_disk 为字符串精确匹配→跨形态产生同文件双索引或漏比对 | 统一 `/` 口径：入库时归一；存量行一次性迁移(目标已存在则删旧行去重，两表同步，事务)；prune 候选先归一再比对 |

## TDD（tests/risk_p0_p1_test.rs，6 用例，全部真实 RED → GREEN）

- `sync_subtree_unavailable_keeps_rows_and_others_intact`：RED(误清 2 行)→GREEN；验证 DB 行数/FTS 保留/未受影响目录
- `sync_subtree_recovery_resumes_updates`：失联保留→恢复→变更正常更新、无重复
- `sync_rejects_incompatible_schema_observably`：RED(failed=1 半坏)→GREEN(failed=-1 整体拒绝+0 写入)
- `search_on_incompatible_db_returns_error`：RED(静默空结果)→GREEN(Err 可观察)
- `sync_mixed_separator_db_migrates_without_duplicates`：RED(双索引)→GREEN(全库 `/`、无重复、search/list 无重复 path)
- `sync_forward_slash_db_rows_not_pruned`：RED(`/`行+Windows 扫描产生重复行)→GREEN(不误删)

## 实施中的测试捕获

- schema 校验初版期望列漏了 `documents.id` 主键 → 正常库被误拒，被正常路径场景立即暴露（mixed 测试前置失败），修正期望集合。教训：校验类改动必须有正常路径用例。

## 口径与契约

- schema 契约：不兼容→整体拒绝+failed=-1+stderr 重建指引；不引入迁移框架（Markdown 唯一真本、DB 可重建，明确拒绝优于静默半迁移）。
- 路径口径：DB 全库正斜杠；磁盘访问走 Path API（join 吸收 `/`）；入库归一、prune 归一比对、存量一次性迁移。普通文件名与相邻目录语义不受影响（仅替换分隔符）。

## 验证

- risk_p0_p1_test 6/6（3 次稳定）；cargo test 全量 **172/172**（166 基线零回归）
- build.sh 等价构建 + 改名法部署；vault 根真实库迁移实测：476 行迁移、**0 pruned**、二次 sync 0 updated/0 pruned（幂等）；DB 反斜杠路径残留 0
- mcp-config.test.sh 9/10（T7b 为 1a718e9 存量，另行记录）

## 遗留（范围外）

- P2：查重阈值调优、serve.sh 部署一致性、跨平台真实 Linux 环境端到端（本轮仅逻辑层统一）。
- T7b 存量失败待裁决。
