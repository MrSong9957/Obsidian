# wiki-query MCP 后端安全攻防测试报告

- 日期：2026-09-20
- 范围：`mcp/wiki-query`（Rust 本地 stdio MCP 索引服务，含 CLI `sync` / `serve` 与 5 个 MCP 工具）
- 方法：两步式——先静态审查源码定位攻击面，再用可运行攻击脚本对真实二进制（`target/release/wiki-query.exe`）发起 41 个攻击场景实测。攻击会话全部指向隔离 tempdir 环境，真实索引零接触。
- 结论总览：**未发现高危/中危可利用漏洞**；1 项低风险信息暴露、3 项加固建议。攻击者视角下核心数据资产（Markdown 真本与 SQLite 索引）未被任何场景越权触碰。

## 一、系统模型与八类攻击面适用性判定

| # | 攻击类别 | 判定 | 代码证据与说明 |
|---|---------|------|---------------|
| 1 | 数据越权（改单号/用户ID查改删他人数据） | **不适用** | 无用户/订单/多租户模型（`src/` 全模块无任何用户标识概念）；唯一删除路径 `prune_orphans` 只删 DB 行且有三重防护（归一比对/exists/父目录失联），不触碰 Markdown 真本 |
| 2 | 功能越权（普通用户调管理员接口） | **不适用** | 无角色/权限模型；v5 已删除 write 类工具，现存 5 工具均为只读或幂等索引操作。曾有"权限策略"错误变体，生产代码已无使用点 |
| 3 | 身份认证（不登录/伪造 token） | **不适用（附实测）** | 无认证层——信任边界为 OS 用户（stdio 本地进程）。实测伪造 `role/is_admin/token` 字段均被 serde 忽略，无认证旁路面 |
| 4 | 偷改字段提权（role→admin、价格→0、金额负数） | **不适用（附实测）** | 入参结构体无 role/price/金额字段；未知字段被 serde 忽略（实测 B1a）；类型混淆被 rmcp schema 拒绝（实测 A5b/A6b/B2，返回 -32602） |
| 5 | 恶意输入与注入 | **部分适用 → 实测** | SQL 注入被参数化阻断（`core.rs:66` `params![fts_query]`）；无 shell 调用（全模块无 Command/exec）→ 命令注入不适用；FTS5 MATCH 语法与 LIKE 通配符为剩余输入面 → 实测 A1/A2 系列 |
| 6 | 绕过前端限制 | **不适用** | 无前端；MCP 工具即最终接口，参数校验在服务端 rmcp schemars 层（实测 B2 缺必填被拒） |
| 7 | 敏感信息泄露 | **适用 → 实测发现 1 项 LOW** | 见 FINDING-1 |
| 8 | 资源滥用与防刷 | **适用（本地维度）→ 实测** | 无网络则无远程刷量；本地维度实测连发/并发/超大输入 → 见实测矩阵 C 组 |

## 二、发现清单

### FINDING-1（LOW）模板缺失警告回显服务器绝对路径

> **状态：已修复（2026-09-20，TDD）。** 修复方式：警告文案改为 `[警告:未找到 templates/project-rules.md,请检查 WIKI_VAULT_ROOT 配置]`，移除 `vault_root=<绝对路径>`，保留模板缺失提示与 agent_type 映射行为。复测：同场景响应含必要警告、泄露扫描（`C:\Users`/盘符路径/`vault_root=`）零命中（PASS）。回归：`wiki_get_project_rules_missing_template_warns_without_leaking_path`（新增，先 RED 后 GREEN）及全部既有 F7 测试通过。

- **类别**：敏感信息泄露（CWE-209 错误消息含敏感信息）
- **位置**：`src/mcp_server.rs:144-149`
- **后果**：`wiki_get_project_rules` 在模板缺失时，返回文案 `[警告:未找到 templates/project-rules.md,vault_root=C:\Users\...]`，暴露服务器绝对路径。当前调用方为本地 agent（本已知悉 WIKI_VAULT_ROOT），风险低；若未来 MCP 暴露至远程则升级为中危。
- **复现**：`WIKI_VAULT_ROOT` 指向无模板目录 → 调用 `wiki_get_project_rules` → 响应含 `vault_root=C:\\Users\\...`（实测命中泄露扫描正则）。
- **建议**：警告文案去除绝对路径，仅提示"模板缺失，请检查 WIKI_VAULT_ROOT 配置"。→ 已按此实施。

### FINDING-2（INFO/加固建议）FTS MATCH 语法 token 未完全过滤

- **类别**：输入处理
- **位置**：`src/core.rs:21`（仅过滤含 `"` 的 token 与非字母数字 token）
- **现状**：`*`、`NEAR(...)`、`-`、`^`、`OR` 大写等 FTS5 语法字符可进入 MATCH 表达式。实测 10 个语法炸弹全部安全返回（参数化+错误行过滤 `.filter_map(r.ok())` 兜底），无崩溃、无注入。若未来 FTS 错误改为直接透传，语法细节可能成为探测面。
- **建议**：保持现状可接受；后续可加 FTS 语法字符白名单。

### FINDING-3（INFO/已知行为）`wiki_list` dir 的 LIKE 通配符未转义

- **位置**：`src/core.rs:201-206`
- **现状**：`dir: "%"` 会宽匹配全部行（LIKE 通配语义）。无注入（参数化），仅枚举粒度问题；调用方本可全量 list。记录为已知契约。

### FINDING-4（LOW）serve 启动校验 stderr 打印 vault_root 绝对路径

> **状态：已修复（2026-09-20，TDD）。** 修复方式：启动校验警告改为 `当前目录未找到 wiki/ 或 sources/ 目录。可用 WIKI_VAULT_ROOT 环境变量指定正确的 vault 路径`，不再打印实际路径；警告本身保留、启动契约不变。复测：同场景 stderr 含必要警告、泄露扫描零命中（PASS）。回归：`serve_startup_warning_does_not_leak_absolute_path`（新增，先 RED 后 GREEN）通过。

- **位置**：`src/mcp_server.rs:277-283`
- **现状**：与 FINDING-1 同类（CWE-209），输出通道为 stderr（本地可见），风险低于 FINDING-1。

## 三、实测矩阵（41 场景，全部真实 stdio 请求）

**A 注入面（29 场景）**
- SQL 注入经典串/`DROP TABLE`（A1a/A1b）：正常返回，参数化阻断 ✓
- FTS 语法炸弹 ×8（引号不闭合/NEAR/列过滤/括号/悬空 AND 等）：正常返回，无崩溃 ✓
- `wiki_list` dir 通配符/穿越/绝对路径（A2a-d）：宽匹配或空结果，无越权数据带出 ✓
- `wiki_list` limit 负数/超 i64 精度/null：安全处理 ✓；浮点 `1.5` 被 schema 拒绝（-32602）✓
- `wiki_check_scope` 穿越/空字节/全角伪装/超长(20KB)/URL 伪装/盘符/混合穿越 ×9：全部安全判定，无崩溃、无栈溢出、fail-closed ✓
- `wiki_sync` scope 大小写回退 ✓；类型混淆（对象）被拒 ✓
- `wiki_get_project_rules` agent_type 超长(100KB) 安全 ✓；类型混淆（数组）被拒 ✓

**B 字段篡改（3 场景）**：未知提权字段被忽略 ✓；空 `vault_root` 回退环境变量 ✓；缺必填被拒（-32602）✓

**C 资源滥用（5 场景）**：1MB 超长 query 201ms 安全处理 ✓；5000 层穿越无栈溢出 ✓；limit=10⁹ 空库即时返回 ✓；连发 10 次 `wiki_sync` 平均 17ms（幂等无写放大）✓；8 并发进程握手 8/8（252ms）✓

**D 信息泄露（1 场景）**：48 条响应/45KB 扫描 → 命中 1 处（FINDING-1）；无密码/密钥/token 类泄露 ✓

**E 认证/越权佐证（并入 B 组）**：伪造 `role/is_admin/token` 字段全部被忽略，无任何认证旁路面 ✓

## 四、恢复契约与既有防护确认（本轮实测复核）

- cwd 守卫、父目录失联保护、schema 校验拒绝、分隔符归一（此前 P0/P1 修复）在攻击会话中均处于启用状态，攻击全程未破坏索引一致性。
- Markdown 真本在整个测试过程中零接触（攻击会话使用独立 tempdir vault）。

## 五、建议汇总（按优先级）

1. FINDING-1/4：错误与警告文案去除绝对路径（LOW，改动数行）。
2. FINDING-2/3：作为已知契约记录；若未来引入远程传输层，需优先补 FTS 语法白名单与 LIKE 转义。

## 附：测试环境与方法说明

- 攻击脚本：Node.js spawn 真实二进制 `serve`，逐场景发 JSON-RPC 并按行解析响应；15s 超时判定挂起；进程存活监控判定崩溃。
- 隔离：攻击会话 cwd 与 `WIKI_VAULT_ROOT` 均指向 tempdir；`db/wiki.db` 真实索引未被读写。
- 脚本与原始结果为一次性工件，验证后已删除，未纳入本目录。
