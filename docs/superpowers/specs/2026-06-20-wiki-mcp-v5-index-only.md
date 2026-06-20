# wiki-query MCP v5:纯索引层重构

## 概述

将 wiki-query 从"Obsidian 知识库管理器"重构为"纯索引服务"。

**核心理念**:各 AI agent(Claude Code / Codex / OpenCode / MiMo Code / Cursor 等)都有自己的记忆系统和文件读写工具。凡是 agent 自己能干的(读写文件、编辑正文、智能决策),wiki-query 绝不重复。wiki-query 只做 agent 干不了的——**维护 SQLite 元数据索引,提供全文检索和查重报告**。

**一句话定位**:wiki-query 是 agent 的**记忆聚合索引层**——agent 用文件工具读写 vault 里的 Markdown,wiki-query 负责 SQLite 索引,让"文档越多,检索优势越大"成为现实。

**基线**:`2026-06-20-wiki-mcp-v4-converge.md`(v4,已实现但未合并 master)。

**性质**:存量减法重构。保留已验证的 FTS5 / jieba 分词 / policy / SQLite documents 表,删除抢 agent 活的工具和决策逻辑。

---

## 设计动机(为什么做这次转向)

v3/v4 把 wiki-query 设计成"全功能知识库管理器":既管索引,又管正文读写(write/read/patch),还内置四态决策智能。走查真实场景后暴露三个问题:

1. **职责重叠**:agent 本来就有文件读写工具,wiki_write/wiki_read/wiki_patch 在重复 agent 能力,违背"利用 agent 内置工具,避免重复造轮子"。
2. **智能放错层**:四态决策(ADD/UPDATE/NOOP/CONFLICT)用 Rust 启发式做查重(name 字符串相等),质量远不如 LLM 语义判断。该智能本该在 agent 层。
3. **SQLite 价值被稀释**:SQLite 只存元数据、不存正文,这才是 wiki-query 的不可替代价值(全文检索 + 文档越多优势越大)。但 v3/v4 把重心放在正文操作上,本末倒置。

v5 把这三点都纠正:砍掉正文工具、砍掉决策智能、把重心收回 SQLite 索引。

---

## 职责边界

| 谁负责 | 做什么 |
|--------|--------|
| **agent** | 读写自己的记忆系统(它最懂自己的格式)、读写 vault 的 Markdown(用 agent 自己的文件工具)、查重/去重/提炼/合并的智能决策、追加章节编辑正文、frontmatter 规范的自我约束(见 AGENTS.md 约定) |
| **wiki-query** | 维护 SQLite 元数据索引、FTS5 全文检索、索引同步时附查重报告、记录提炼状态时间戳 |

**铁律:凡是 agent 自己能干的(读写文件),wiki-query 绝不重复。**

### 不碰外部目录

wiki-query 只在配置的 vault 内读 Markdown 建索引,绝不跨目录扫描 agent 的记忆目录(`~/.claude/`、`.cursor/rules/`、`~/.codex/memories/` 等)。agent 自己读自己的记忆(各 agent 格式差异巨大:SQLite / 向量库 / logfmt / mdc / 纯 md,wiki-query 不可能统一适配),然后用自己的文件工具把提炼结果写入 vault。

### 与 agent 记忆系统独立

wiki-query 是 agent 记忆的**下游沉淀点**,不是上游来源。agent 不主动调用,wiki-query 静默;agent 主动选择性地把有价值的内容"倒进 vault",再调 wiki_sync 让索引跟上。

---

## 工具集(5 个 MCP 工具,全部围绕 SQLite + 规则分发)

| 工具 | 输入 | 输出 | 何时用 |
|------|------|------|--------|
| `wiki_search` | query 关键词 | top 10 元数据(已按相关度排序,**不含 score**,不含正文) | 找知识 / 查重 / 提炼前看库里有没有 |
| `wiki_list` | tag / type / dir(均可选) | 元数据列表 | 按结构化条件浏览某类文档 |
| `wiki_sync` | scope(可选,默认 `"incremental"`) | 同步报告 + **查重报告**(scope 决定查重范围) | agent 写完文件后调;用户说"去重"时调 `scope="full"` |
| `wiki_mark_extracted` | 无 | 确认 | 全量提炼完成后调,更新 SQLite meta 表的 last_extract 时间戳 |
| `wiki_get_project_rules` | agent_type(可选,如 `"claude"` / `"codex"` / `"cursor"`,默认通用) | 可注入的规则文本(含查重提示词) | agent 首次接入 wiki-query 时调,拿文本写入当前项目根的 CLAUDE.md / AGENTS.md |

### wiki_sync 的 scope 参数

`wiki_sync` 合并了原 v5 设计的 wiki_dedupe 功能。索引同步永远是全量扫描 vault(SQLite ← Markdown),`scope` **只控制查重范围**:

| scope | 索引同步 | 查重范围 | 用途 |
|-------|---------|---------|------|
| `"incremental"`(默认) | 全量扫,更新索引 | **本次新入库/更新的文档** 对比全库 | 写入后即时反馈,抓"刚写的有没有撞车" |
| `"full"` | 全量扫,更新索引 | **全库两两**对比 | 用户说"去重一下"时,扫历史积压的重复 |

两种 scope 共享底层查重函数(`core::find_similar`),只是喂入的候选集不同。查重永远不拦截(文件已写),只返回报告让 agent 决策。

### 设计要点

- **search 不返回 score**:bm25 原始值(负数)对 agent 是噪声。只返回排好序的结果,agent 信任顺序。
- **search 返回 top 10**:避免一次检索淹没 agent 上下文。需要更多用 wiki_list 分页。
- **sync 是唯一的索引入口**:agent 写完文件调 sync,索引跟上 + 拿查重报告,一次调用两件事。scope 切换查重范围,不切换索引同步方式(永远全量扫,简单可靠)。
- **查重不自动合并**:去重的合并动作(保留哪篇、丢弃哪篇、正文怎么融合)是智能决策,归 agent。sync 只输出"哪几对文档高度相似"的报告。
- **每个工具的 description 就是它的场景提示词**:不额外做场景识别机制。工具说明写清楚,agent 自然知道何时用。

---

## 核心数据流:全量提炼

提炼是 wiki-query 最核心的使用场景。**提炼是批量全量动作**(不是单文档),因为"提炼后删除 agent 记忆源文件"是常态,不删源则后续提炼越来越重。

```
用户:"提炼一下我的记忆"
    │
    ▼
1. agent 全量读自己记忆系统
   │   Claude Code: ~/.claude/<项目>/MEMORY.md(索引页,前200行进上下文)
   │                + MEMORY.md 链接的关联 .md(具体内容)
   │   Codex:       ~/.codex/memories/*.md
   │   Cursor:      .cursor/rules/*.mdc
   │   OpenCode:    .opencode/memory/*(格式依插件而定)
   │   (agent 内置文件工具,wiki-query 不碰)
   ▼
2. agent (可选)判断"距上次提炼多久"
   │   → 方式待定(见下方「last_extract 读取方式」待决项)
   │   → 判断间隔,太近且记忆无变化则告诉用户"刚提炼过,无需重复"
   ▼
3. agent 对每篇记忆内容,调 wiki_search 逐篇查重(固定流程,不可省)
   │   → 相似的:agent 用文件工具读 vault 文档全文 → 决定合并/追加
   │   → 无相似的:agent 用文件工具新建 vault 文档
   ▼
4. agent 用文件工具读写 vault Markdown(新建/覆盖/追加章节)
   │   (agent 内置文件工具,wiki-query 不碰。frontmatter 按 AGENTS.md 约定)
   ▼
5. agent 调 wiki_sync()
   │   → wiki-query 扫描 vault,SQLite 索引跟上 Markdown 变化
   │   → 返回查重报告:本次新入库文档与已有文档的相似对
   │   → agent 看报告,若有意外重复,用文件工具处理
   ▼
6. agent 删除自己记忆系统里的源文件(提炼后删源,避免重复提炼)
   │   (agent 侧 housekeeping,wiki-query 不管)
   ▼
7. agent 调 wiki_mark_extracted()  [若选方案 C 则免此步]
   │   → wiki-query 写 SQLite meta 表:last_extract = now
   ▼
8. agent 告诉用户"已提炼 N 篇,删源 M 个"
```

### 为什么提炼后删源

记忆会不断增加,不删已提炼的源文档,后面每次提炼都要重新处理一遍旧内容,越来越重。删源让"提炼"始终是对增量记忆的处理。删源是 agent 侧 housekeeping,wiki-query 不参与。

### 查重的两道防线

1. **agent 主动查重(第 3 步)**:agent 每篇都先 wiki_search,自己判断要不要新建。这是主防线,智能在 LLM。
2. **wiki_sync 报告兜底(第 5 步)**:sync 时对新入库文档做相似检测,返回报告。agent 若第 3 步漏了,sync 还能兜住。

---

## frontmatter 约定(降级,不再强制)

v3/v4 用代码强校验 frontmatter(5 字段非空 + type 白名单)。v5 删除 wiki_write 后没有写入入口,无法强制。降级为**约定**,通过规则注入机制分发(见「wiki_get_project_rules 工具」):

- 规则文本(vault 内 `templates/project-rules.md`)写明:vault 内文档 frontmatter 应有 `name` + `description`(检索展示用),`type` / `tags` 建议填写。
- agent 首次接入时调 wiki_get_project_rules 拿规则,注入到当前项目根的 CLAUDE.md / AGENTS.md,之后在该项目里遵守。
- wiki_sync 索引时,缺 name/description 的文档,对应字段存空字符串。search 时这类文档相关度靠正文命中,不致命。
- type 不再校验白名单。各 agent 提炼过来的文档 type 五花八门,强制白名单反而拒收。

**为什么不保留部分强制(name/description 必填)?** 因为没有 write 工具了,没有拦截点。即便 agent 偶尔不遵守,后果只是该文档 search 时元数据不全,不影响索引和检索的可用性。约定 + agent 自觉够用。

---

## 删掉的现有功能(对比 v4)

| v4 功能 | v5 处置 | 理由 |
|---------|---------|------|
| `wiki_write` 工具 | **删** | 写文件是 agent 的活,wiki-query 只 sync 索引 |
| `wiki_read` 工具 | **删** | 读文件是 agent 的活 |
| `wiki_patch` 工具 | **删** | v4 已删,v5 延续(局部编辑归 agent 文件工具) |
| `write_decision.rs` 四态决策 | **删** | 查重交给 agent + sync 报告兜底 |
| `core::write` / `core::read` / `core::patch` | **删** | 对应工具删了 |
| frontmatter 强校验(type 白名单 + 字段必填) | **删/降级为约定** | 没有 write 了,拦不住;降级为 AGENTS.md 约定 |
| extras 保留逻辑(UPDATE 时保留 source/aliases) | **删** | 没有 write/UPDATE 了,agent 自己用文件工具管 frontmatter |
| `core::list` / `core::search` / `core::search_with_scores` | **保留** | search/list 的核心,文档越多越值钱 |
| `db::search` | **保留** | search_with_scores 复用 |
| FTS5 + jieba 分词 | **保留** | 核心价值 |
| policy 三层目录权限 | **保留**(sync 时校验路径) | 安全底线 |
| 路径穿越防御 | **保留** | 安全底线 |
| ServerInfo instructions | **保留**(更新内容) | 告诉 agent 新工具用法 |
| `wiki-query sync` CLI 命令 | **保留**(git hook 用,功能含查重) | 运维入口 |

---

## 新增功能

### 1. SQLite meta 表

key-value 结构,存全局状态。schema:

```sql
CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

初始 key:`last_extract`(ISO 8601 时间戳,记录上次全量提炼时间)。

写入:wiki_mark_extracted 工具。

#### ⚠️ 待决项:last_extract 的读取方式

agent 删了 wiki_read 后,无法用 MCP 工具读 SQLite meta 表。判断"距上次提炼多久"有三个方案:

- **方案 A**:加 wiki_status 工具(读 meta 表返回 last_extract)。干净,但多一个工具。
- **方案 B**:last_extract 双写——SQLite meta 表(主)+ vault 内 `.wiki/last-extract.json`(副,供 agent 文件工具读)。不加工具,但引入非 Markdown 文件。
- **方案 C**:放弃时间戳判断。agent 不读时间戳,每次提炼都全量(反正源已删,重复提炼的是增量)。

**建议:方案 C。** 源文件删了,全量提炼处理的就是增量,时间戳判断的价值有限,不值得为它加工具或引入双写复杂度。若后续证明需要,再加 wiki_status。

> 若选方案 C,wiki_mark_extracted 工具也随之失去意义(写了没人读),可一并去掉,工具集降为 4 个(search/list/sync/get_project_rules)。

### 2. wiki_sync 含 scope 查重报告

`wiki_sync` 合并了原设计的 wiki_dedupe,通过 `scope` 参数切换查重范围。返回值:

```json
{
  "synced": 375,
  "updated": 12,
  "pruned": 2,
  "scope": "incremental",
  "dedupe_report": [
    {"new_doc": "wiki/foo.md", "similar_to": "wiki/bar.md", "score": -3.2},
    ...
  ]
}
```

`scope="full"` 时,`dedupe_report` 是全库两两对比的相似对(双向去重,a/b 与 b/a 只报一次);`scope="incremental"`(默认)时,只对本次新入库/更新的文档查重。

#### 查重算法(共享底层函数)

```rust
/// 对给定候选文档集,每个用其 name 做 FTS5 search,收集相似对。
/// incremental 模式:候选集 = 本次 sync 新增/更新的文档
/// full 模式:候选集 = 全库所有文档
fn find_similar(candidates: &[DocMeta], all_docs: &[DocMeta], threshold: f64) -> Vec<SimilarPair>
```

bm25 score 越小(越负)越相关。设相关度阈值(如 score ≤ -2.0),相关度高于阈值的纳入报告。阈值在实现时用真实 vault 数据实测调优。

查重永远**不自动合并**——合并(保留哪篇、丢弃哪篇、正文怎么融合)是智能决策,归 agent。sync 只输出报告。

### 3. wiki_mark_extracted 工具

```rust
fn wiki_mark_extracted(&self) -> Result<CallToolResult, McpError>
```

无参数。写 SQLite meta 表 `last_extract = chrono::now()`。返回确认信息。

> 注意:此工具依赖 last_extract 待决项的结论。若选方案 C(放弃时间戳),本工具删除。

### 4. wiki_get_project_rules 工具 + 规则注入机制

**场景**:agent 在**别的项目**(如 `E:\项目X\`)干活时,通过 MCP 连上 wiki-query。为了让 agent 知道"这个项目接了 wiki-query、vault 在哪、怎么提炼记忆",需要把提炼规则注入到**项目根**的指令文件(CLAUDE.md / AGENTS.md)。

**铁律**:wiki-query **不碰外部目录**。注入由 **agent 执行**(它有文件工具),wiki-query 只通过 `wiki_get_project_rules` 工具**返回规则文本**,agent 拿到后自己写入项目根。

#### 工具签名

```rust
/// 返回可注入到项目根指令文件的规则文本。
/// agent_type 决定目标文件名提示(Claude→CLAUDE.md,其他→AGENTS.md)。
fn wiki_get_project_rules(
    &self,
    Parameters(req): Parameters<ProjectRulesRequest>,
) -> Result<CallToolResult, McpError>

pub struct ProjectRulesRequest {
    /// 当前 agent 类型,决定返回文本里推荐的目标文件名。
    /// "claude" → CLAUDE.md;"codex"/"cursor"/"opencode"/"mimo"/其他 → AGENTS.md。
    /// 可选,默认按 AGENTS.md(最通用约定)。
    pub agent_type: Option<String>,
}
```

返回内容包含:
1. **规则文本**(从 vault 内模板文件读取,见下)
2. **注入指引**:目标文件名(根据 agent_type)、查重标记(避免重复注入)、注入步骤提示词

#### 规则文本来源:vault 内模板文件

规则文本**不硬编码在 Rust 里**(改规则要重编译),而是放在 vault 内的模板文件:

```
<vault>/templates/project-rules.md    ← 权威规则模板(wiki-query 读它返回)
```

放 `templates/` 的理由:该目录语义就是"模板",且 policy 里只读(防误改)。wiki-query 启动时读这个文件,通过工具返回。

**模板文件内容由 4 块组成**(对应 spec 扫描出的需要 agent 遵守的约定):

1. **frontmatter 约定**:vault 内文档 frontmatter 应有 `name` + `description`(检索展示用),`type` / `tags` 建议填写。
2. **全量提炼工作流**:8 步流程(读记忆 → wiki_search 查重 → 文件工具写 vault → wiki_sync → 删源 → wiki_mark_extracted)。
3. **提炼后删源**:避免重复提炼(记忆不断增加,不删源越来越重)。
4. **查重是固定流程 + 合并归 agent**:每篇先 wiki_search;sync 的查重报告只提示,合不合并是 agent 自己的智能决策。

#### 注入流程(agent 执行)

```
agent 首次接入 wiki-query(或在项目里第一次想提炼时):
  │
  ▼
1. agent 调 wiki_get_project_rules(agent_type="claude")
   → wiki-query 读 templates/project-rules.md + 注入指引,返回
   ▼
2. agent 读自己当前项目根,检查 CLAUDE.md(或 AGENTS.md)是否已含标记
   → 标记约定:文件中含 "<!-- wiki-query-rules -->" 则已注入
   → 已含:跳过,不重复注入
   → 未含:继续
   ▼
3. agent 用文件工具把规则文本追加到项目根 CLAUDE.md(或 AGENTS.md)
   → 追加,不覆盖已有内容
   → 顶部带标记 "<!-- wiki-query-rules -->"
   ▼
4. 之后 agent 在该项目里就知道:有 wiki-query 可用、vault 路径、怎么提炼
```

#### 为什么不硬编码在 Rust

规则文本是最常迭代的(提炼流程微调、frontmatter 要求调整、不同 agent 的格式差异)。硬编码意味着每次改规则要改 Rust + 重编译。放 vault 内模板文件,改规则直接改 markdown,符合"Markdown 是权威源"的精神。wiki-query 只是透传。

#### 为什么不写进 ServerInfo instructions

instructions 在每次 initialize 连接时全量传给 agent,适合放"工具列表 + 一句话定位"这种稳定的、必须每次都知道的信息。提炼规则文本较长(4 块),写进 instructions 会每次连接都传,浪费上下文。规则注入是一次性动作,用专门的工具按需获取更合适。

---

## 与 v4 的文件改动对照

```
db/wiki-query/src/
├── main.rs              # Modify: sync 子命令输出含查重报告(支持 scope)
├── core.rs              # Modify: 删 write/read/patch,保留 search/list,加 find_similar 查重函数
├── write_decision.rs    # Delete: 四态决策整个删
├── frontmatter.rs       # Modify: 删 validate(白名单校验),保留 parse/serialize(sync 建索引用)
├── policy.rs            # 保留(几乎不动)
├── db.rs                # Modify: 加 meta 表 schema + 读写 last_extract(待决项 C 选定则不加)
├── sync.rs              # Modify: sync 接 scope 参数,调 find_similar 返回查重报告
├── mcp_server.rs        # Modify: 工具集收敛为 5 个(删 read/write/patch/dedupe,保留 search/list,改 sync 加 scope,加 mark_extracted + get_project_rules)
└── models.rs            # Modify: 删 WriteResult/ConflictCandidate,加 SyncReport/SimilarPair/ProjectRulesRequest
tests/
├── core_write_test.rs   # Delete/重写: write/patch 测试删,加 find_similar 查重测试
├── fts_test.rs          # 保留
└── mcp_integration_test.rs  # Modify: 断言新工具集(5 个)
AGENTS.md                # Modify: frontmatter 约定 + 提炼工作流
db/wiki-query/README.md  # Modify: 重写工具集 + 数据流
templates/project-rules.md  # Create: 可注入的项目根规则模板(wiki_get_project_rules 读它)
```

---

## 不在本轮范围

- **文件 watcher**:vault 变动自动 sync。推迟(违背"agent 主动调 sync"的模型,且 Obsidian 多源编辑会引入一致性问题)。
- **向量语义检索**:当前用 FTS5 + jieba 关键词检索够用,且 SQLite-only 的定位下不引入向量依赖。
- **知识图谱**(wikilink 关系查询):nice-to-have,推迟。
- **wiki_status 工具**(读 meta 表):本轮只写 last_extract。读需求出现再加。
- **自动合并去重**:sync 的查重报告只提示相似对,不自动合并。合并智能归 agent。

---

## 测试策略

- **FTS5 / jieba / policy**:保留现有测试,确保重构不破坏检索能力。
- **sync 查重(incremental)**:新建文档后调 wiki_sync(默认 scope),断言返回值含相似对(构造重复 name 的两个文档)。
- **sync 查重(full)**:构造已知相似对,调 wiki_sync(scope="full"),断言全库报告包含该对。
- **find_similar 阈值**:单元测试验证 score 低于阈值的不入报告(避免噪声)。
- **wiki_mark_extracted**:调用后读 SQLite meta 表断言 last_extract 已更新(若选方案 C 则免)。
- **wiki_get_project_rules**:断言返回值含规则文本的 4 块(frontmatter 约定 / 提炼流程 / 删源 / 查重);断言 agent_type="claude" 时推荐 CLAUDE.md,其他推荐 AGENTS.md;断言文本来自 templates/project-rules.md(改文件后返回值跟着变)。
- **MCP 集成测试**:tools/list 断言 5 个工具(search/list/sync/mark_extracted/get_project_rules),initialize 断言 instructions 含新工具。
- **删除的功能**:对应测试一并删除,不留孤儿测试。

---

## 复杂度估计

| 工作 | 估计 |
|------|------|
| 删 write/read/patch 工具 + core 函数 + write_decision.rs | 1h |
| frontmatter 删 validate,policy 微调 | 0.5h |
| db.rs 加 meta 表 + 读写(若选方案 C 则免) | 0.5h |
| core.rs 加 find_similar 查重函数(共享给 incremental/full) | 1h |
| sync.rs 接 scope 参数 + 返回查重报告 | 1h |
| mcp_server 工具集重写(5 个)+ instructions 更新 | 1h |
| wiki_mark_extracted 工具(若选方案 C 则免) | 0.5h |
| wiki_get_project_rules 工具 + templates/project-rules.md 模板 | 1h |
| 测试重写 + MCP 集成测试 | 1.5h |
| README + AGENTS.md 文档更新 | 0.5h |
| **总计** | **~8.5h**(方案 C 则 ~7.5h) |
