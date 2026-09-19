# 架构决策记录（ADR）

> **本文是设计决策与演进记录，不是操作规范。** 当前可执行规则以 [§6 当前权威入口](#6-当前权威入口) 所列文档为准。
> 前身为 2026-04-05 的 841 行三层架构设计蓝图（完整原文见 git 历史：`git show 573eb26:LLM-WIKI-ARCHITECTURE.md`）。

---

## 1. 背景与问题

让 LLM 长期使用一个知识库时，原始设计要解决三个问题：

- **效率低**：每次对话重新阅读和理解原始资料，浪费 token
- **缺关联**：资料间的隐含关系（关联、矛盾、互补）无法在一次查询中发现
- **不可靠**：每次重新理解可能产生不一致的结论

目标：LLM 像图书管理员一样维护知识——原始资料保证事实基础，LLM 管理结构化知识层，人负责裁决矛盾、修正错误、补充理解。

## 2. 核心决策：三层模型

```
Schema 层（规则与权限）    ← 由根目录 AGENTS.md 承担
Wiki 层（结构化知识）      ← wiki/
Sources 层（原始资料）     ← sources/
```

| 层 | 分工 | 设计理由 |
|----|------|----------|
| Schema | 规则与权限的唯一承载点，集中在 `AGENTS.md` | 人维护、LLM 遵守；规则集中一处，不散落在各目录，也不绑定单一 agent 工具 |
| Wiki | 面向任务的结构化知识入口 | LLM 与人均可读写，是日常使用的工作层 |
| Sources | 原始资料的归档层 | 保住事实基础，Wiki 的每条内容都可溯源到这里 |

各层的目录职责、权限矩阵与页面格式等现行规则见 [[AGENTS.md]]（权威文档总表见 §6）。

底层承载全部使用 Obsidian 原生设施——wikilinks、frontmatter、tags、脚注、内置图谱——**不引入额外数据库或向量库**。

## 3. 为什么这样设计

- **Obsidian 本身就是 Wiki 层的基础设施**：交叉引用（wikilinks）、知识图谱、元数据（frontmatter）、检索均有原生能力，自建存储只会增加部署复杂度。
- **溯源优先**：每条 Wiki 内容必须能追溯到原始资料的精确位置（frontmatter `source` 字段 + 正文脚注 + `L{起}-{止}` 行号索引）。这是整个架构的可靠性基石，机制详见 [[AGENTS.md]]。
- **人是核心角色**：矛盾裁决、错误修正、上下文补充由人完成；LLM 只做整理与提炼，不做最终裁决。
- **零部署**：文件系统 + Markdown + SQLite（本地索引），不依赖任何常驻服务。

## 4. 方案取舍

2026-04 设计期对比了三个开源项目，真正影响决策的借鉴：

| 借鉴来源 | 采纳的思想 | 落地方式 |
|----------|-----------|----------|
| Semiont | append-only / 事件溯源，内容不可丢失 | 简化为「sources/ 禁止删除、仅标记 deprecated」 |
| Semiont | W3C Annotation 溯源思想 | 简化为 `source` 字段 + 脚注行号索引 |
| Mem0 | ADD/UPDATE/DELETE/NOOP 四操作记忆模型 | 演进为今天的「提炼工作流」（见 §5） |
| Khoj | 验证 Obsidian 作为 AI 知识前端可行 | 直接采用纯 Obsidian 方案，零客户端开发 |

明确**不采纳**的方向（当时已识别，至今成立）：向量数据库（百级至千级文档用全文索引足够）、自动知识合成（先做好摘要与交叉引用）、JSON 替代 Markdown。

## 5. 架构演进

> 下表「最初设计」仅陈述当时方案；「采纳的决定与关系」为当时作出的决定及其与其他决定的关系，不代表现行规则。现行规则以 §6 权威文档为准。

| 最初设计（2026-04） | 采纳的决定与关系 | 原因 |
|---------------------|------------------|------|
| Schema 放在 `.claude/CLAUDE.md` | 协议集中到根目录 `AGENTS.md`，`CLAUDE.md` 改为单行指针 | 规则对人和所有 agent 工具通用，不绑定单一工具目录 |
| sources/ append-only：创建❌ 编辑❌ | 放宽为可创建、可编辑，仅禁止删除——修订了最初照搬 Semiont 的 append-only | 个人 vault 中 append-only 过严，补 frontmatter、修错字都需要写权限；「禁删」保住了原始事实基础 |
| 三个模板：concept / summary / contradiction | 合并为 `templates/wiki.md`（`type` 字段区分 concept 与 contradiction）+ `templates/sources-reference.md` | 摘要页职责由 sources 文件自身承担，无需独立页型；减少模板数量 |
| `wiki/contradictions/` 子目录存放矛盾页 | 取消独立目录，用 `type: contradiction` 字段标识 | Obsidian 按目录组织导航，混排更利于 wikilink 图谱 |
| confidence 0-1 自评 + <0.7 警示 | 未采纳 | LLM 自评置信度不可靠，改为人工审阅把关 |
| 手工更新引擎：ADD/UPDATE/CONTRADICT/NOOP | 引擎被本地索引服务替代：检索与查重交给 wiki-query（Rust + SQLite FTS5 + jieba 分词），提炼保持手动触发；四操作判定保留在单次写入流程中（AGENTS.md「通用写入流程」） | 手工流程无法规模化到 440+ 文档；检索/查重交给本地索引服务，提炼手动触发防止噪声入库 |
| 搜索增强路线：Omnisearch / sqlite-vss（Phase 2） | 放弃插件增强路线，自建 wiki-query 的 FTS5 + bm25 检索 | 自建索引一步到位，不再依赖插件 |
| 自动化路线：hooks 监听 sources/（Phase 3） | 放弃文件监听，改为 `wiki-query sync` 挂 git post-commit hook；提炼保持手动触发 | 提交即同步，无需文件监听；自动提炼会灌入未定型噪声 |
| sources/ 平铺存放 paper-*.md | 按主题建子目录（agent / claude-code / dev / learning / projects / security / tools / DeepSeek-Harness），根目录仅保留 `_index.md` | 文档量增长后按主题归类才可浏览、可维护 |

## 6. 当前权威入口

| 文档 | 负责什么 |
|------|----------|
| [[AGENTS.md]] | 库内现行操作规范：目录权限与分类、建页格式、溯源与引用、写入与提炼流程 |
| [[templates/wiki.md]]、[[templates/sources-reference.md]] | 页面与详细文档模板 |
| [[templates/project-rules.md]] | 外部项目接入说明——经 wiki-query 注入外部项目，供外部 Agent 独立沉淀记忆使用；与库内规范的重叠部分以 [[AGENTS.md]] 为准 |
| [[mcp/wiki-query/README.md]]、[[mcp/wiki-query/mcp-integration.md]] | wiki-query 检索工具的构建、部署与 MCP 接入 |
| [[sources/_index.md]]、[[wiki/_index.md]] | 资料索引 / 知识导航 |
| [[agents/security-testing-gate.md]] | 安全测试子代理 |

库内日常操作遵循 [[AGENTS.md]]；[[templates/project-rules.md]] 仅服务外部项目接入，不作为库内操作依据。

与 agent 自带记忆（CLAUDE.md / .claude/ / .zcode/ 等）的分工原则：会话级的放 agent 记忆，跨会话、需溯源的进本知识库——详见 [[AGENTS.md]]。
