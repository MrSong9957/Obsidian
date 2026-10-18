---
name: Codex 记忆系统
description: Codex 记忆管线——两阶段提炼合并、SQLite+Markdown 双存储、摘要注入与按需文件搜索、全量配置项与 V2 差异；使用、排障与移植的入口
type: concept
updated: 2026-10-18
tags: [codex, agent-memory, memory, sqlite, markdown, progressive-disclosure]
source: "[[sources/agent/Codex记忆系统-框架实现与Agent执行手册.md]]"
---

# Codex 记忆系统

## 核心思想

Codex 把旧会话中的经验交给模型提炼，先写入 SQLite，再同步整理成 Markdown；新会话由程序自动把记忆摘要注入模型输入，模型发现相关主题后，再用文件工具搜索经验正文、按需追查来源。[^1]

真正让记忆参与当前任务的动作是**把文字送进模型输入**；存到磁盘或数据库只解决"以后还能取到"。[^2]

三个入口分工明确：原始会话历史用于恢复会话与追查；自动记忆（本页）从过去会话提炼经验、减少重复探索；`AGENTS.md` 等规则文件声明项目应当怎样工作。需要持续适用的团队规则应放规则文件，不依赖自动记忆。[^3]

所有结论依据 `openai/codex` 固定提交 `ce0323f`（2026-10-17）的静态源码核对，未编译或运行仓库测试；默认配置为 V1，记忆总开关 `features.memories` 默认关闭。[^4]

## 详细说明

### 阅读结构：三层概括、四级深度

| 层 | 内容 | 读取方式 |
|---|---|---|
| 记忆摘要 `memory_summary.md` | 用户概况、偏好、通用提示、主题导航 | 程序自动注入上下文 |
| 整理后的记忆文档 | `MEMORY.md` 经验手册、会话摘要、可复用流程 | 模型按需搜索和读取 |
| 原始会话记录 | 当时的消息、工具调用与输出 | 需要原始证据时再追查 |

进一步拆成四级阅读深度：`memory_summary.md` → `MEMORY.md` → `rollout_summaries/*.md` 与 `skills/*/SKILL.md`（并列来源）→ `rollout_path` 指向的原始会话。这是阅读深度而非目录层级，内容够用即可停止。[^5]

`MEMORY.md` **同时是来源索引和经验正文**：按任务组组织，含 `scope`、`applies_to`、`rollout_summary_files`、`keywords`、用户偏好、可复用知识与失败经验，不是"全部文档的目录"。[^6]

### 存储：SQLite 与 Markdown 双轨

默认 `CODEX_HOME` 为 `~/.codex`（环境变量可改，勿硬编码路径），核心位置：[^7]

| 位置 | 内容 |
|---|---|
| `memories/memory_summary.md` | V1 总摘要；合并 Agent 写、程序注入 |
| `memories/MEMORY.md` | V1 长期经验手册 |
| `memories/rollout_summaries/*.md` | 选入合并输入的会话摘要 |
| `memories/raw_memories.md` | 初步经验合集，程序从 SQLite 重建 |
| `memories/skills/<名称>/SKILL.md` | 可复用流程，可选 |
| `memories/extensions/ad_hoc/notes/*.md` | 用户明确要求的记忆修改便条 |
| `memories/.git/` 与 `phase2_workspace_diff.md` | 合并差异用的本地基线 |
| `memories_1.sqlite` / `state_5.sqlite` | 初步记忆、任务与合并进度 / 会话目录与状态 |

V2 使用独立的 `memories_v2/` 与 `memories_v2_1.sqlite`。[^8] 同一根目录**不按项目分库**：候选查询不限定工作目录，靠 `MEMORY.md` 的 `scope`、`applies_to` 和来源路径区分适用范围，因此经验必须注明适用的目录、分支或流程。[^9]

SQLite 侧关键表：`stage1_outputs` 存单会话提炼结果（`raw_memory`、`rollout_summary`、使用次数、版本时间）；`jobs` 存两阶段任务状态（所有权令牌、租约、重试、输入水位）；`consolidation_progress` 存 V2 预热进度。[^10] 双轨分工原因（实现推断）：数据库便于按时间、状态、使用计数筛选并用事务协调并发；Markdown 便于模型用现成文件工具读写，文件差异可直接交给合并 Agent。[^11]

### 生成管线：触发 → 候选 → 两阶段

**触发点是一次符合条件的新一轮输入**，不是"会话结束"事件，也没有每会话闲置闹钟：app-server 在新轮启动后调用后台任务，再依次检查非临时会话、总开关、非根 Agent 来源、数据库可用与配额。[^12]

候选筛选（默认 V1）：[^13]

| 条件 | 规则 |
|---|---|
| 来源 | CLI、VS Code 及 `atlas`、`chatgpt` 自定义来源；归档会话不选 |
| 记忆模式 | `threads.memory_mode = 'enabled'` |
| 当前会话 | 排除触发本次扫描的会话 |
| 闲置与年龄 | 距最后更新 ≥6 小时，且不超过 10 天窗口 |
| 重复处理 | 当前源版本已成功处理则跳过 |
| 数量上限 | 数据库扫描上限 5,000；每轮默认领取 2 个 |

时间判断取数据库中的**会话更新时间**（`threads.updated_at_ms`，毫秒）。"已处理过当前版本"由 `stage1_outputs.source_updated_at` 与任务表 `last_success_watermark` 双重记录——空结果也算已检查，源更新后可重新提炼，不是"一生只总结一次"。[^14] 配额检查在能得到后端配额数据时要求剩余 ≥25%，无法判断时放行；`generate_memories = false` 只控制新会话登记，不是整条管线总开关。[^15]

**第一阶段（逐会话提炼）**：读取近期记录（处理后约 8 MiB、单条上限 1 MiB、模型输入预算为有效窗口的 70%、无窗口信息回退 150,000 tokens），清理注入说明、做秘密替换后，要求模型返回严格 JSON（V1：`raw_memory` / `rollout_summary` / 可空 `rollout_slug`）。[^16][^17] 是否值得记住由模型判断，代码不设轮数或字数门槛。三种结果：有内容则写入；空结果同样记成功并使旧结果失效；失败记错误并重试。并发上限 8、租约 3,600 秒、重试延迟 3,600 秒、默认重试 3 次。[^18]

**第二阶段（全局合并）**：先取全局锁（距上次成功 ≥6 小时冷却，与源会话闲置 6 小时是两个独立计时）；最多选 256 条来源（30 天保留窗口，优先按 `usage_count` 与使用、更新时间排序）；同步会话摘要、V1 重建 `raw_memories.md` 后，计算相对 Git 基线的文件差异——**无差异且输出有效则不调用模型直接记成功**，有差异才生成差异文件并启动合并 Agent。[^19][^20]

合并 Agent 以隔离配置运行：临时会话、关闭自身记忆注入与生成、移除 MCP；父会话用 Codex 托管沙箱时仅可写记忆根目录且断网（父会话关闭沙箱或用外部沙箱时保留相应策略，不是所有情况强制）。它负责合并重复、处理冲突与过时结论、应用用户便条、更新总摘要。[^21] 结束后程序还要校验输出（V1：`MEMORY.md` 是文件、总摘要首行 `v1`、符号链接检查）才重置基线；租约 3,600 秒、90 秒心跳续期；文件写入与数据库状态不是原子事务，失败不保证回滚。[^22]

### 新会话：注入与搜索

开启 `features.memories` 与 `memories.use_memories` 时，程序把记忆目录路径、查找说明和总摘要（约 2,500 tokens 内）组合成 **developer 级上下文**注入；摘要缺失或为空则跳过。[^23] 是否继续检索由当前任务模型按指引判断：先搜 `MEMORY.md`，再按需读 1—2 个会话摘要或技能，需要精确证据才追原始会话，无命中即停（建议 4—6 步内，是提示词预算非强制中断）。记忆只代表"过去记录如此"，依赖可变事实时仍要核验当前源码。[^24]

默认 `dedicated_tools = false` 时，模型用通用文件或命令工具（如 `rg`）搜索 Markdown 正文。[^25] 开启后注册专用工具：`memories.list` / `memories.read` / `memories.search` / `memories.add_ad_hoc_note`；`search` 接受 `queries` 数组与三种匹配模式（`any` / `all_on_same_line` / `all_within_lines`）。[^26] 本地实现是**逐行文字包含比较**——不查 SQLite 元数据、无 FTS、无向量检索；专用工具限定在记忆根目录内（拒绝越界路径、隐藏路径与符号链接）。[^27]

### 修改、引用与遗忘

用户明确要求"记住/忘记/纠正"时，当前任务模型写便条到 `extensions/ad_hoc/notes/`（文件名 `YYYY-MM-DDTHH-MM-SS-<slug>.md`），由后续合并 Agent 应用并标记 `[ad-hoc note]`；便条写入成功不等于总摘要已更新。[^28]

使用计数靠**记忆引用反馈**：模型在回答中附 `<oai-mem-citation>` 块，程序从 `rollout_ids` 解析有效会话 ID 后给 `usage_count` 加一——工具读取指标与引用计数是两种记录，"搜到文件"不等于"使用次数加一"。[^29]

遗忘分步生效：后台按保留窗口清理旧初步记录（每批 ≤200），第二阶段排除窗口外来源，文件同步删除对应摘要，差异交给合并 Agent 删除仅由失效来源支持的结论；[^30] 扩展资源（不含用户便条）另有 14 天清理；显式重置（如 `codex debug clear-memories`）清理数据库与记忆目录，但保留原始会话。[^31]

### 配置项

全局配置通常位于 `~/.codex/config.toml`，默认值按固定提交的执行代码填写：[^32]

| 配置 | 默认值 | 生效范围或含义 |
|---|---:|---|
| `features.memories` | `false` | 记忆总功能开关 |
| `memories.version` | `"v1"` | 选择读取和普通写入使用的版本 |
| `memories.dual_write` | `false` | 同时为 V1、V2 运行写入管线 |
| `memories.generate_memories` | `true` | 新建会话是否可作为生成输入 |
| `memories.use_memories` | `true` | 是否加入记忆读取上下文 |
| `memories.dedicated_tools` | `false` | 是否提供专用记忆工具 |
| `memories.disable_on_external_context` | `false` | 外部上下文事件是否排除该来源 |
| `memories.max_rollouts_per_startup` | `2` | 每轮最大领取数，限制 1—128 |
| `memories.max_rollout_age_days` | `10` | 源会话更新时间窗口，限制 0—90 天 |
| `memories.min_rollout_idle_hours` | `6` | 源会话最少闲置，限制 1—48 小时 |
| `memories.min_rate_limit_remaining_percent` | `25` | 剩余配额门槛，限制 0—100 |
| `memories.max_raw_memories_for_consolidation` | `256` | 合并最大来源数，限制 1—4096 |
| `memories.max_unused_days` | `30` | 保留窗口，限制 0—365 天 |
| `memories.extract_model` / `memories.consolidation_model` | 未指定 | 由供应商接口提供默认提炼/合并模型 |

开关分工：开启整套功能用 `features.memories`；停止注入用 `use_memories`；新会话不再作为来源用 `generate_memories`（不重写旧会话状态，不是总开关）；清除记忆用显式重置，关开关本身不是删除。[^33]

最小配置：

```toml
[features]
memories = true

[memories]
version = "v1"
generate_memories = true
use_memories = true
dedicated_tools = false
```

非配置项的关键常量：两阶段并发上限 8、租约与重试延迟 3,600 秒、第二阶段心跳 90 秒、第一阶段推理强度 Low / 第二阶段 Medium、注入摘要 2,500 tokens、专用读取 20,000 tokens、专用搜索返回 ≤200、列目录 ≤2,000、差异文件 ≤4 MiB；完整常量表见来源文。[^34]

### V2：同一管线里的精简分支

V2 共用入口与两阶段流程，但不生成 `raw_memories.md`、不要求 `MEMORY.md` 与技能层，总摘要更直接指向会话摘要；[^35] 提炼输出只有 `rollout_summary` 与 `rollout_slug`（裁至 9,000 字节内）；总摘要硬边界为首行 `v1`、严格小于 10,000 UTF-8 字节、四个必需标题（`User Profile` / `User preferences` / `General Tips` / `What's in Memory`）。[^36] 双写时两套目录与数据库隔离（旧便条不自动复制）；V2 就绪检查默认要求历史成功合并 ≥20 个源会话——**20 是预热阈值，不是生成门槛**。[^37]

### 常见误解修正

| 常见说法 | 准确说法 |
|---|---|
| `MEMORY.md` 只是文档目录 | 来源索引 + 完整可复用经验正文 |
| 会话结束就开始总结 | 由后续新一轮输入触发扫描 |
| 过了 6 小时一定已总结 | 只是获得资格，还需执行机会与其他条件 |
| SQLite 只存任务状态 | `raw_memory` 与 `rollout_summary` 正文也在库中 |
| 检索走 SQLite 元数据或向量 | 默认与专用工具都搜 Markdown 正文，逐行文字匹配 |
| 被读取一次使用次数就加一 | 只有记忆引用反馈才计数 |
| 多层摘要就是 RAPTOR、有链接就是 GraphRAG | 无递归聚类建树，也无实体关系图；结构上更接近文档式 Wiki |
| V2 满 20 个会话才生成记忆 | 20 是就绪检查默认阈值，候选筛选无此条件 |

更多修正与从零开始的时间线见来源文。[^38]

### 使用、排障与移植入口

- **查记忆的 Agent 规程**：判断历史是否影响当前任务 → 读已注入摘要 → 选关键词 → 先搜 `MEMORY.md` → 确认 `scope` / `applies_to` → 经验够用直接使用，否则读最相关的 1—2 个会话摘要 → 需要精确证据才追原始会话 → 无命中即停 → 回答区分"过去记录如此"与"本轮已验证"。[^39]
- **移植**：来源文提供输入/输出/边界表、最少状态清单、两阶段伪代码、10 步实施验收与可直接下发的任务指令模板；[^40] 配套 A01—A32 验收矩阵与已静态核对的测试位置。[^41]
- **排障**：先看触发和资格，再看模型结果，最后看文件与状态是否成功提交；详细定位表见来源文。[^42]
- **证据边界**：不要把本地 Codex 的时间参数套到 ChatGPT 网页版；在线文档与固定提交存在默认值、路径与说明文字差异，以执行代码为准。[^43] 源码与公开资料的完整索引（S01—S64）见来源文。[^44]

## 相关概念

- [[mimocode-memory-system]] — MiMo Code 三层记忆文件 + FTS5 搜索的另一套实现；Codex 是 SQLite+Markdown 双轨、逐行文字匹配
- [[Agent 记忆分层与四种存储格式]] — 记忆分层与存储格式的通用框架，Codex 是其工程化实例
- [[用户记忆与知识库]] — 用户记忆与知识库分工的通用原则
- [[可执行记忆与记忆内容的三种类型]] — 情景/语义/程序三类记忆内容，可与 Codex 各文件对应
- [[Agent Skills]] — `memories/skills/` 可选技能层与 Skill 渐进式加载同一思想
- [[工具调用闭环：Codex复用与参考范围]] — 同一 Codex 仓库工具机制的源码复用参考
- [[how-openai-uses-codex]] — OpenAI 团队的 Codex 使用实践

## 引用索引

[^1]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L26-30 — 总述：提炼入 SQLite、整理成 Markdown、新会话先读摘要再按需检索
[^2]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L84-92 — 记忆生效的直接动作是把文字送进模型输入
[^3]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L139-147 — 自动记忆、会话历史与 AGENTS.md 的三入口分工
[^4]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L36-44 — 固定提交、默认 V1、总开关默认关闭、静态核对未运行测试
[^5]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L151-172 — 三层概括表与四级阅读深度
[^6]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L174-189 — MEMORY.md 双重身份（来源索引 + 经验正文）
[^7]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L240-261 — 运行时数据目录表与 Git 基线含义
[^8]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L265-275 — V2 独立位置（memories_v2/ 与 memories_v2_1.sqlite）
[^9]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L279-287 — 同一根目录不按项目分库，靠 scope/applies_to 区分
[^10]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L411-455 — stage1_outputs、jobs、consolidation_progress 表结构
[^11]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L457-467 — SQLite 与 Markdown 双轨分工原因（实现推断）
[^12]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L474-495 — 触发点为符合条件的新一轮输入及后台检查项
[^13]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L510-527 — 候选筛选规则表
[^14]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L529-548 — 时间取 threads.updated_at_ms；双重记录避免重复总结
[^15]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L550-568 — 配额 25% 检查的适用条件与 generate_memories 边界
[^16]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L579-609 — 输入准备顺序与预算边界（8 MiB / 1 MiB / 70% / 150,000 tokens）
[^17]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L611-654 — 模型保留/舍弃判断与 V1 严格 JSON 输出合同
[^18]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L656-686 — 三种结果（空结果也算成功）、并发与重试参数
[^19]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L690-726 — 全局锁、6 小时冷却、来源选择规则（256 / 30 天 / usage_count）
[^20]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L728-747 — 文件差异驱动合并；无差异不调用模型
[^21]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L749-779 — 合并 Agent 职责与隔离配置（含沙箱例外）
[^22]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L781-810 — 结束确认、写后校验、心跳与部分失败无原子回滚
[^23]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L813-837 — 读取开关与 developer 级注入（约 2,500 tokens）
[^24]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L839-861 — 读取指引（4—6 步预算）与记忆不是当前事实证明
[^25]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L863-895 — 默认用通用工具搜索 Markdown 正文
[^26]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L897-944 — 专用工具四件套、参数与三种匹配模式
[^27]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L946-977 — 逐行 contains 实现（非 FTS/向量）与专用工具路径边界
[^28]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L980-1016 — ad_hoc 便条流程与文件名格式
[^29]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1018-1046 — 记忆引用反馈与工具读取指标是两种记录
[^30]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1048-1062 — 30 天保留分步生效，清理每批 ≤200
[^31]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1064-1098 — 删除来源、14 天扩展资源清理与显式重置
[^32]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1112-1138 — 可配置参数表（默认值与允许范围）
[^33]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1140-1178 — 开关分工表与最小配置示例
[^34]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1180-1216 — 硬编码常量表与两个 6 小时的区分
[^35]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1218-1233 — V1/V2 对比表
[^36]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1235-1271 — V2 输出合同与总摘要硬边界（9,000 / 10,000 字节）
[^37]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1273-1290 — 双写隔离与就绪检查（20 非生成门槛）
[^38]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1292-1339 — 从零开始时间线与常见误解修正表
[^39]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1342-1418 — Agent 使用与修改记忆的执行规程
[^40]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1420-1597 — 移植实施规格（边界表、最少状态、伪代码、任务指令）
[^41]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1599-1677 — 验收矩阵 A01—A32 与已静态核对的测试位置
[^42]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1679-1700 — 故障定位表（触发→模型→提交）
[^43]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1702-1728 — ChatGPT 网页版边界与在线说明差异
[^44]: [[sources/agent/Codex记忆系统-框架实现与Agent执行手册]] L1756-1918 — 源码索引 S01—S64 与公开资料索引
