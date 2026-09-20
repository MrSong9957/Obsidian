# 修复日志：安全审计 FINDING-1/4 绝对路径泄露

日期：2026-09-20 ｜ 基线：172/172 ｜ 完成后：174/174（+2 泄露检测测试）

## 缺陷与最小修复（TDD）

| Finding | 根因 | 修复 | 测试 |
|---|---|---|---|
| FINDING-1 | `wiki_get_project_rules` 模板缺失警告 `format!` 内嵌 `vault_root={}` | 文案改"[警告:未找到 templates/project-rules.md,请检查 WIKI_VAULT_ROOT 配置]"，移除路径 | e2e `wiki_get_project_rules_missing_template_warns_without_leaking_path`：断言警告存在 + 不含实际路径 + 不含 `vault_root=`（三重，防退化成删提示）。RED→GREEN |
| FINDING-4 | serve 启动校验 stderr 打印 `vault_root={}` | 文案改"当前目录未找到 wiki/ 或 sources/ 目录。可用 WIKI_VAULT_ROOT 环境变量指定…" | e2e `serve_startup_warning_does_not_leak_absolute_path`：spawn 错误 cwd 收集 stderr，断言警告存在 + 不含实际路径 + 不含 `vault_root=`。RED→GREEN |

## RED 归因

- F1：响应 text 含 `"vault_root=C:\\Users\\..."` → 命中"不得含 vault_root= 键值"断言
- F4：stderr 含 `vault_root=C:\Users\...\Temp\.tmpXXX 下未找到...` → 命中"不得包含 vault 根绝对路径"断言

## 复测与回归

- 泄露复测脚本（临时，验证后删除）：两场景警告均存在、4 类泄露模式（C:\Users/盘符/vault_root=/临时目录名）零命中 → ALL-PASS
- 既有 F7 测试（模板正常读取/claude 映射/标记/指引/缺失警告存在性）全部保持绿——行为无退化
- `cargo test` 全量 174/174；新 exe 已构建部署

## 改动

- `src/mcp_server.rs`：两处文案（去 `{}` 路径插值，各 +注释）
- `tests/mcp_e2e_full_test.rs`：+2 泄露检测测试
- `docs/security/reports/2026-09-20-wiki-query-mcp-security.md`：FINDING-1/4 标注已修复（保留历史发现与风险背景）
