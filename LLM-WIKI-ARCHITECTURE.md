# LLM Wiki 三层架构设计

> 本文档是搭建 Obsidian 原生 LLM Wiki 的完整参考方案。
> 源自 2026-04-05 的架构讨论，整合了 Semiont、Mem0、Khoj 三个开源项目的设计经验。

---

## 目录

1. [问题与动机](#1-问题与动机)
2. [核心模型](#2-核心模型)
3. [Obsidian 原生方案](#3-obsidian-原生方案)
4. [三层详细规格](#4-三层详细规格)
5. [更新引擎](#5-更新引擎)
6. [模板定义](#6-模板定义)
7. [完整工作流](#7-完整工作流)
8. [质量保障](#8-质量保障)
9. [开源项目对比与借鉴](#9-开源项目对比与借鉴)
10. [演进路线](#10-演进路线)
11. [实施清单](#11-实施清单)

---

## 1. 问题与动机

### 痛点

LLM 每次对话都要重新阅读和理解原始资料，存在三个问题：

- **效率低**：相同的资料反复处理，浪费 token
- **缺关联**：资料之间的隐含关系（关联、矛盾、互补）无法在一次查询中发现
- **不可靠**：每次重新理解可能产生不一致的结论

### 目标

让 LLM 像一个"图书管理员"一样，主动整理和维护知识：

- 原始资料只读不可变（保证事实基础）
- LLM 管理结构化知识层（摘要、交叉引用、矛盾标注）
- 人的参与：裁决矛盾、修正错误、补充理解

---

## 2. 核心模型

### 三层架构

```
┌────────────────────────────────────────────────┐
│  Schema 层（CLAUDE.md / SYSTEM.md）             │
│  定义 LLM 的角色、权限、行为规则                   │
│  LLM 只读，人可改                                 │
├────────────────────────────────────────────────┤
│  Wiki 层（Obsidian 笔记）                        │
│  结构化知识：摘要、交叉引用、矛盾标注               │
│  LLM 可读写，人可读写                              │
├────────────────────────────────────────────────┤
│  原始资料层（sources/）                           │
│  论文、文档、报告等原始材料                         │
│  LLM 只读，人负责添加                              │
└────────────────────────────────────────────────┘
```

### 各层职责

| 层 | 内容 | 谁写入 | 谁读取 | 可变性 |
|---|------|--------|--------|--------|
| Schema | 规则、权限、模板路径 | 人 | LLM + 人 | 人可随时修改 |
| Wiki | 概念页、摘要、交叉引用、矛盾标注 | LLM（主要）+ 人 | LLM + 人 | LLM 和人均可编辑 |
| 原始资料 | 原始文档 | 人（放入文件） | LLM + 人 | append-only，可标记 deprecated |

---

## 3. Obsidian 原生方案

### 为什么选择 Obsidian

Obsidian 的原生能力恰好覆盖了 Wiki 层的核心需求：

| Wiki 层需求 | Obsidian 原生能力 |
|------------|------------------|
| 交叉引用 | `[[wikilinks]]` 双向链接 |
| 知识图谱 | 内置图谱视图（自动基于 wikilinks 生成） |
| 元数据 | YAML frontmatter |
| 引用原文 | `> [!quote]` callout 语法 |
| 标签分类 | `#tag` 标签系统 |
| 搜索 | 内置搜索 + Omnisearch 插件 |
| 人工编辑 | 直接在 Obsidian 中编辑 Markdown |

这意味着不需要额外的数据库、图数据库或向量库——**Obsidian 本身就是 Wiki 层的基础设施**。

### 仓库目录结构

```
你的 Obsidian 仓库/
│
├── .claude/
│   └── CLAUDE.md                  ← Schema 层（LLM 行为规则）
│
├── sources/                       ← 原始资料层（只读）
│   ├── _index.md                  # 资料清单（可选，用于追踪）
│   ├── paper-attention-is-all-you-need.md
│   ├── paper-bert.md
│   ├── api-doc-react-hooks.md
│   └── ...
│
├── wiki/                          ← Wiki 层（LLM 可读写）
│   ├── _index.md                  # Wiki 总览（可选）
│   ├── attention-mechanism.md     # 概念页
│   ├── transformer-architecture.md
│   ├── learning-rate-scheduling.md
│   ├── ...
│   └── contradictions/            # 矛盾标注
│       ├── lr-scheduling-debate.md
│       └── ...
│
├── templates/                     ← 模板（LLM 生成新页时遵循）
│   ├── concept.md                 # 概念页模板
│   ├── summary.md                 # 摘要页模板
│   └── contradiction.md           # 矛盾页模板
│
└── .obsidian/                     ← Obsidian 配置（不动）
```

### 设计约束

- **sources/ 是 append-only**：文件只追加，不修改不删除；需要"撤回"时在 `_index.md` 中标记 `deprecated`
- **wiki/ 里只用 Obsidian 原生语法**：wikilinks、frontmatter、callout、tags——不引入自定义格式
- **人和 LLM 用相同格式编辑**：不做区分，LLM 的产出和人手写的笔记结构完全一致

---

## 4. 三层详细规格

### 4.1 原始资料层（sources/）

#### 文件命名

- 使用有意义的英文短名 + 序号：`paper-attention-is-all-you-need.md`
- 保持原始文件名可读，不做 hash 命名（Obsidian 环境下可读性优先）

#### 文件内容

原始资料直接以 Markdown 存放。如果是 PDF/网页，由人在放入前转换为 Markdown。

```markdown
---
title: "Attention Is All You Need"
authors: ["Vaswani, A.", "..."]
date: 2017-06-12
type: paper
url: https://arxiv.org/abs/1706.03762
tags: [深度学习, Transformer, 注意力机制]
---

# Attention Is All You Need

（原始论文的 Markdown 全文）
```

#### 资料索引（_index.md，可选）

```markdown
---
type: index
---

# 原始资料索引

| ID | 文件名 | 类型 | 状态 | 加入日期 |
|----|--------|------|------|----------|
| src-001 | [[paper-attention-is-all-you-need]] | paper | active | 2026-04-05 |
| src-002 | [[paper-bert]] | paper | active | 2026-04-05 |
| src-003 | [[blog-rag-explained]] | blog | deprecated | 2026-04-01 |
```

`deprecated` 状态表示资料已过时或有误，但文件保留不删除。LLM 读到 deprecated 资料时会在 Wiki 页面中标注。

### 4.2 Wiki 层（wiki/）

Wiki 层有三种页面类型：**概念页**、**摘要页**、**矛盾页**。

#### 概念页（concept）

核心页面类型。每个"知识概念"对应一个概念页。

```markdown
---
type: concept
sources:
  - "[[paper-attention-is-all-you-need]]"
  - "[[paper-bert]]"
confidence: 0.95
updated: 2026-04-05
status: active
tags: [深度学习, 序列模型, 注意力]
---

# 注意力机制

## 核心思想

不再用固定窗口看输入序列，而是让模型"决定"每个位置该关注哪些其他位置。[^1]

## 变体

- [[自注意力]] — 同一序列内部的关系
- [[交叉注意力]] — 两个不同序列之间
- [[多头注意力]] — 并行多组注意力

## 相关概念

- [[Transformer架构]] ← 使用了注意力机制
- [[位置编码]] ← 解决注意力缺少位置信息的问题
- [[BERT]] ← 基于注意力的双向编码器

## 待解决的问题

- ⚠️ 关于计算效率的争论 → [[attention-efficiency-debate]]

## 引用索引

[^1]: [[paper-attention-is-all-you-need]] `L42-58` — 注意力函数的定义
```

**字段说明：**

| Frontmatter 字段 | 含义 | 示例 |
|-----------------|------|------|
| `type` | 页面类型 | `concept` |
| `sources` | 溯源到哪些原始资料 | `["[[paper-xxx]]"]` |
| `confidence` | LLM 对摘要准确度的自评（0-1） | `0.95` |
| `updated` | 最后更新日期 | `2026-04-05` |
| `status` | 页面状态 | `active` / `deprecated` |
| `tags` | Obsidian 标签 | `[深度学习, 注意力]` |

**内容区块：**

| 区块 | 作用 | 必需 |
|------|------|------|
| 核心思想 | 一段话概括这个概念 | 是 |
| `[^n]` 脚注标记 | 标注每个要点的来源位置 | 是 |
| 变体 / 分类 | 概念的子类或变体 | 按需 |
| 相关概念 | `[[wikilinks]]` 指向关联概念 | 是 |
| 待解决的问题 | 指向矛盾页或待研究问题 | 按需 |
| 引用索引 | 脚注定义：文件名 + 行号范围 + 内容摘要 | 是 |

#### 摘要页（summary）

对单篇原始资料的结构化摘要。一篇原始资料对应一个摘要页。

```markdown
---
type: summary
source: "[[paper-attention-is-all-you-need]]"
confidence: 0.9
updated: 2026-04-05
tags: [论文摘要, Transformer]
---

# 摘要：Attention Is All You Need

## 一句话总结

提出了 Transformer 架构，完全基于注意力机制，抛弃了 RNN 和 CNN。

## 核心贡献

1. 提出自注意力（Self-Attention）机制 [^1]
2. 证明纯注意力模型可以在翻译任务上超越所有已有方法 [^2]
3. 训练速度大幅提升（并行化） [^3]

## 关键数据

- WMT 2014 英德翻译：BLEU 28.4（当时 SOTA） [^4]
- 8 个 P100 GPU 上训练 3.5 天 [^5]

## 涉及的概念

- [[注意力机制]]
- [[多头注意力]]
- [[位置编码]]
- [[Transformer架构]]

## 引用索引

[^1]: [[paper-attention-is-all-you-need]] `L42-58` — 自注意力机制定义
[^2]: [[paper-attention-is-all-you-need]] `L89-112` — 翻译任务实验结果
[^3]: [[paper-attention-is-all-you-need]] `L15-23` — 并行化优势说明
[^4]: [[paper-attention-is-all-you-need]] `L95-98` — BLEU 分数对比表
[^5]: [[paper-attention-is-all-you-need]] `L120-125` — 训练配置细节
```

#### 矛盾页（contradiction）

记录原始资料之间或概念之间的矛盾。

```markdown
---
type: contradiction
involved:
  - "[[paper-cosine-lr]]"
  - "[[paper-warmup-lr]]"
related_concepts:
  - "[[学习率调度]]"
severity: high
status: open
updated: 2026-04-05
tags: [待裁决, 学习率]
---

# 学习率调度策略之争

## [[paper-cosine-lr]] 的观点

Cosine annealing 在大多数场景下表现更稳定，学习率平滑下降到 0。 [^1]

## [[paper-warmup-lr]] 的观点

Linear warmup 对大 batch 训练必不可少，先线性增后线性减。 [^2]

## 分析

两者并不完全矛盾：cosine annealing 可以结合 warmup 使用。
实际上很多实践采用 warmup + cosine decay 的组合策略。

## 结论（待填写）

<!-- 人在此填写裁决结果，填写后将 status 改为 resolved -->

## 引用索引

[^1]: [[paper-cosine-lr]] `L67-82` — cosine annealing 效果对比实验
[^2]: [[paper-warmup-lr]] `L34-51` — 无 warmup 训练发散的实验数据
```

**字段说明：**

| 字段 | 含义 |
|------|------|
| `involved` | 涉及矛盾的原始资料 |
| `related_concepts` | 相关概念页 |
| `severity` | `high`（关键矛盾）/ `medium` / `low` |
| `status` | `open`（待裁决）/ `resolved`（已裁决）/ `accepted`（双方都不完全对） |

### 4.3 Schema 层（CLAUDE.md）

Schema 层定义 LLM 的行为规则，嵌入在 Obsidian 仓库的 `.claude/CLAUDE.md` 中。

```markdown
# LLM Wiki 管理协议

## 你的角色

你是本 Obsidian 仓库的知识管理员。
仓库是一个三层结构的知识库：

- `sources/` = 原始资料（你只能读）
- `wiki/` = 结构化知识（你可以读写）
- `templates/` = 页面模板（创建新页面时遵循）

## 权限矩阵

| 操作 | sources/ | wiki/ | templates/ |
|------|----------|-------|------------|
| 读取 | ✅ | ✅ | ✅ |
| 创建 | ❌ | ✅ | ❌ |
| 编辑 | ❌ | ✅ | ❌ |
| 删除 | ❌ | 仅标记 deprecated | ❌ |

## 创建新页面规则

1. 概念页使用 `templates/concept.md` 模板
2. 摘要页使用 `templates/summary.md` 模板
3. 矛盾页使用 `templates/contradiction.md` 模板
4. 每个页面的 frontmatter 必须完整填写
5. 每个 `[[wikilink]]` 必须指向已存在的页面，如果目标页面不存在，先创建再链接
6. `sources` 字段必须溯源到 `sources/` 中的文件

## 更新规则

当新资料加入 sources/ 时，执行以下判断：

1. **ADD**：发现新概念 → 创建新的概念页
2. **UPDATE**：补充已有概念 → 编辑已有的 wiki 页面
3. **CONTRADICT**：发现矛盾 → 创建矛盾页
4. **NOOP**：无新信息 → 不做任何操作

## 质量规则

- 每条 Wiki 内容必须标注来源（`sources` 字段 + 正文脚注 `[^n]`）
- 每个脚注必须在"引用索引"区块中定义，格式为 `[[文件名]] L起始行-结束行 — 简要说明`
- `confidence < 0.7` 的内容必须在页面中标注 `⚠️ 需要人工确认`
- `severity: high` 的矛盾页必须提示人工裁决
- 禁止捏造不存在的 `[[wikilink]]`

## 行号索引规则

行号索引是 Wiki 层的核心溯源机制，确保 AI 和人都能直达原文：

- **格式**：`L{起始行}-{结束行}`，例如 `L42-58`
- **每个要点都要标注**：正文中用 `[^1]`、`[^2]` 等脚注标记，底部"引用索引"区块中定义
- **行号是原文的实际行号**：LLM 在读取 sources/ 文件时记录关键信息所在行，生成 Wiki 页时写入
- **行号会随原文变化失效**：如果原始资料被替换（新版本），相关 Wiki 页的行号需要重新标注

示例：
```
正文中：提出了自注意力机制 [^1]
引用索引中：[^1]: [[paper-attention-is-all-you-need]] `L42-58` — 自注意力机制定义
```

## 格式规则

- 使用 Obsidian 原生语法：wikilinks、frontmatter、callout、tags、脚注
- 不使用 HTML 标签
- 不使用自定义语法
- Markdown 格式遵循 CommonMark 规范
```

---

## 5. 更新引擎

### 借鉴来源

更新引擎的核心思路来自 **Mem0 的四操作模型**（ADD / UPDATE / DELETE / NOOP），
适配到 Obsidian + Markdown 的文件环境。

### 触发条件

| 事件 | 触发的动作 |
|------|-----------|
| 新文件加入 `sources/` | 执行完整更新流程 |
| 资料标记为 `deprecated` | 检查关联的 wiki 页面，标注"来源已过时" |
| 用户请求"重新分析 [资料]" | 对指定资料重新执行更新流程 |
| 用户请求"更新 [概念]" | 对指定概念页重新检查相关资料 |

### 更新流程

```
新资料进入 sources/
       │
       ▼
┌──────────────────────────────┐
│ Step 1: 读取原始资料           │
│ 提取关键概念、论点、数据        │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Step 2: 生成/更新摘要页        │
│                              │
│ 检查 wiki/ 是否已有该资料的     │
│ 摘要页：                      │
│  - 无 → ADD（创建 summary 页） │
│  - 有 → UPDATE（刷新内容）     │
│  - 无新信息 → NOOP            │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Step 3: 概念提取与关联         │
│                              │
│ 从摘要中识别关键概念：          │
│  - 新概念 → ADD（创建 concept │
│    页 + wikilinks）           │
│  - 已有概念 → UPDATE（补充    │
│    新信息到已有页面）           │
│  - 概念已充分覆盖 → NOOP      │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Step 4: 矛盾检测              │
│                              │
│ 对比新资料与已有资料：          │
│  - 发现矛盾 → ADD（创建       │
│    contradiction 页）         │
│  - 已有矛盾已解决 → UPDATE    │
│  - 无矛盾 → NOOP             │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Step 5: 一致性检查             │
│                              │
│ - 检查所有 wikilinks 指向      │
│   的页面是否存在               │
│ - 检查 sources 字段是否指向    │
│   存在的原始资料               │
│ - 检查每个脚注的行号范围是否    │
│   在原文实际行数内             │
│ - 检查 confidence 标注         │
└──────────────────────────────┘
```

### 增量更新策略

不做全量重读，而是：

1. **新资料**只与**已有摘要的语义相似资料**对比（通过标题、tags、wikilinks 预筛）
2. **概念提取**只在新资料涉及新概念时触发
3. **矛盾检测**只在同一概念下有多个来源时触发

这避免了 O(n²) 的全量比较，将复杂度控制在可接受范围。

---

## 6. 模板定义

### concept.md（概念页模板）

```markdown
---
type: concept
sources:
  - "[[在此填入原始资料]]"
confidence:
updated: {date}
status: active
tags: []
---

# {概念名称}

## 核心思想

（用一段话概括这个概念是什么、为什么重要）[^1]

## 详细说明

（分点展开说明概念的关键细节，每个要点都标注脚注编号）

## 相关概念

- [[相关概念A]] — 关系说明
- [[相关概念B]] — 关系说明

## 待解决的问题

（如果存在矛盾或未解决的问题，在此指向矛盾页）

## 引用索引

[^1]: [[原始资料名]] `L起始行-结束行` — 这段内容的简要说明
```

> **行号格式**：`L{起始行}-{结束行}`，例如 `L42-58`。
> 多个来源时使用多个脚注：`[^1]`、`[^2]`、`[^3]`...
> 每个脚注在"引用索引"区块中对应一行，标注文件名、行号范围、内容摘要。

### summary.md（摘要页模板）

```markdown
---
type: summary
source: "[[原始资料文件名]]"
confidence:
updated: {date}
tags: []
---

# 摘要：{原始资料标题}

## 一句话总结

（用一句话概括这篇资料的核心内容）

## 核心贡献 / 要点

1. （第一个要点）[^1]
2. （第二个要点）[^2]
3. （第三个要点）[^3]

## 关键数据

（如果有具体数据，在此列出，标注脚注）[^4]

## 涉及的概念

- [[概念A]]
- [[概念B]]

## 引用索引

[^1]: [[原始资料文件名]] `L起始行-结束行` — 第一个要点的出处
[^2]: [[原始资料文件名]] `L起始行-结束行` — 第二个要点的出处
[^3]: [[原始资料文件名]] `L起始行-结束行` — 第三个要点的出处
[^4]: [[原始资料文件名]] `L起始行-结束行` — 关键数据的出处
```

### contradiction.md（矛盾页模板）

```markdown
---
type: contradiction
involved:
  - "[[资料A]]"
  - "[[资料B]]"
related_concepts:
  - "[[相关概念]]"
severity: medium
status: open
updated: {date}
tags: [待裁决]
---

# {矛盾标题}

## [[资料A]] 的观点

（资料A 对这个问题的说法）[^1]

## [[资料B]] 的观点

（资料B 对这个问题的说法）[^2]

## 分析

（LLM 的初步分析：矛盾是否真实存在、可能的解释）

## 结论（待填写）

<!-- 人在此填写裁决结果，填写后将 status 改为 resolved -->

## 引用索引

[^1]: [[资料A]] `L起始行-结束行` — 资料A 中支撑其观点的原文位置
[^2]: [[资料B]] `L起始行-结束行` — 资料B 中支撑其观点的原文位置
```

---

## 7. 完整工作流

### 场景 A：添加新资料

```
用户操作：将 paper-rag-survey.md 放入 sources/

LLM 自动执行：
1. 读取 paper-rag-survey.md
2. 创建 wiki/summary-rag-survey.md（摘要页）
3. 识别出新概念"RAG"，创建 wiki/rag.md（概念页）
4. 识别出已有概念"向量数据库"，更新 wiki/向量数据库.md（补充新信息）
5. 检查是否与已有资料矛盾，如无矛盾则结束

用户在 Obsidian 中看到：
- 图谱中出现"RAG"节点，连着"向量数据库"、"嵌入模型"等
- 可以直接点击链接浏览相关知识
```

### 场景 B：发现矛盾

```
LLM 执行过程中发现：
- paper-rag-survey.md 说 "RAG always improves accuracy"
- blog-rag-failures.md 说 "RAG can degrade accuracy with noisy retrieval"

LLM 自动执行：
1. 创建 wiki/contradictions/rag-accuracy-debate.md
2. 在 wiki/rag.md 的"待解决的问题"区块中添加链接
3. severity: medium（因为不是关键矛盾，有合理解释）

用户在 Obsidian 中看到：
- wiki/rag.md 底部出现矛盾提示
- 打开矛盾页，看到双方观点和 LLM 的初步分析
- 用户在"结论"区块填写裁决，将 status 改为 resolved
```

### 场景 C：用户手动修正

```
用户操作：在 Obsidian 中打开 wiki/attention-mechanism.md，
修改了一段摘要，添加了一个 [[局部注意力]] 的 wikilink。

LLM 下次运行时：
1. 发现 attention-mechanism.md 被修改（通过 updated 时间戳）
2. 发现 [[局部注意力]] 指向的页面不存在
3. 创建 wiki/局部注意力.md 的初始页面，标注 confidence: 0.5（因为是人提到但 LLM 尚未深入分析）
4. 在页面中标注"⚠️ 需要人工确认"
```

### 场景 D：资料被标记为过时

```
用户操作：在 sources/_index.md 中将 blog-rag-explained 标记为 deprecated

LLM 下次运行时：
1. 读取 _index.md，发现资料状态变更
2. 查找 wiki/ 中所有 sources 包含 [[blog-rag-explained]] 的页面
3. 在这些页面中添加标注："⚠️ 来源已过时（deprecated）"
4. 如果某个概念页的唯一来源是这份 deprecated 资料，降低其 confidence
```

---

## 8. 质量保障

### 8.1 溯源机制

每条 Wiki 内容都必须能追溯到原始资料的精确位置：

- Frontmatter 的 `sources` 字段记录涉及的所有原始资料
- 正文中的脚注标记（`[^1]`、`[^2]`...）标注每个要点的来源
- "引用索引"区块定义每个脚注：`[[文件名]] L起始行-结束行 — 内容摘要`
- AI 和人都可以通过行号直接跳转到原始资料的对应段落，无需全文扫描

### 8.2 置信度标注

LLM 在生成摘要时自评 `confidence`（0-1）：

| 范围 | 含义 | 处理方式 |
|------|------|----------|
| 0.9 - 1.0 | 高度确信，有充分原文支持 | 正常显示 |
| 0.7 - 0.89 | 较为确信，但可能有细微偏差 | 正常显示 |
| 0.5 - 0.69 | 存在不确定性 | 标注 `⚠️ 需要人工确认` |
| < 0.5 | 低确信度，可能含推理或推测 | 标注 `⚠️ 需要人工确认`，建议用户查看原文 |

### 8.3 矛盾分级

| severity | 含义 | 处理方式 |
|----------|------|----------|
| `high` | 关键事实直接矛盾 | 必须人工裁决后才能在概念页中引用 |
| `medium` | 观点分歧或适用场景不同 | 提示人工关注，不阻塞引用 |
| `low` | 表述差异，实质相同 | LLM 自动标注，不提示 |

### 8.4 人工参与点

- **裁决矛盾**：所有 `severity: high` 的矛盾需要人在矛盾页中填写结论
- **确认低置信度内容**：所有 `confidence < 0.7` 的页面需要人审阅
- **修正错误**：人可直接在 Obsidian 中编辑任何 wiki 页面
- **补充理解**：人可添加 LLM 遗漏的上下文、领域知识、实践经验

---

## 9. 开源项目对比与借鉴

### 三个参考项目

| 项目 | Stars | 定位 | 核心思路 |
|------|-------|------|----------|
| **Semiont** | 44 | 语义知识层 | 事件溯源 + 图数据库 + AI Agent 三角色（标注/生成/链接） |
| **Mem0** | 52k | AI 记忆层 | 四操作更新模型（ADD/UPDATE/DELETE/NOOP）+ 图增强版 Mem0g |
| **Khoj** | 34k | AI 第二大脑 | RAG + pgvector + 原生 Obsidian/Emacs 客户端 |

### 各项目对本方案的贡献

| 本方案要素 | 借鉴自 | 具体借鉴内容 |
|-----------|--------|-------------|
| 底层 append-only | Semiont | 事件溯源思想、内容寻址（简化为 deprecated 标记） |
| 溯源机制 | Semiont | W3C Web Annotation 思想（简化为 sources 字段 + callout） |
| 四操作更新引擎 | Mem0 | ADD/UPDATE/DELETE/NOOP 模型 |
| 矛盾检测与标注 | Mem0g | Conflict Detector + Update Resolver 思想 |
| Obsidian 原生集成 | Khoj | 证明 Obsidian 作为知识管理前端是可行的 |
| 增量更新策略 | Mem0 + Khoj | 不做全量重读，只处理增量部分 |

### 本方案与三个项目的关键差异

| 维度 | Semiont | Mem0 | Khoj | **本方案** |
|------|---------|------|------|----------|
| 存储 | Neo4j + Qdrant + PostgreSQL | Qdrant (+ Neo4j for Mem0g) | PostgreSQL + pgvector | **文件系统（Markdown + JSON）** |
| 图谱 | 图数据库 | Mem0g 图增强 | 无 | **Obsidian 内置图谱（wikilinks）** |
| 部署复杂度 | 高（5+ 服务） | 中（向量库） | 中（PostgreSQL） | **零（纯 Obsidian）** |
| 规模上限 | 10万+ | 万级 | 万级 | **千级** |
| 人工参与 | 有 | 少 | 少 | **核心角色** |

---

## 10. 演进路线

### Phase 1：文件方案（本设计）

- 纯 Markdown + YAML frontmatter
- Obsidian 内置图谱提供可视化
- LLM 通过 Claude Code CLI 操作
- 手动触发更新（用户说"分析新资料"时执行）
- 适用规模：百级文档

### Phase 2：搜索增强

- 加入 Omnisearch 插件提供语义搜索
- 或引入轻量向量索引（如 `sqlite-vss`）
- 支持模糊查询和相似概念发现
- 适用规模：千级文档

### Phase 3：自动化

- 编写 Claude Code skill 自动监听 `sources/` 变更
- 新资料放入后自动触发更新流程
- 使用 Obsidian 的文件监听 + Claude Code 的 hook 机制
- 适用规模：千级文档

### Phase 4：图增强（可选）

- 如果概念关系复杂到 Obsidian 图谱不够用
- 引入 `graph.json` 或轻量图数据库
- 支持多跳推理和复杂关系查询
- 适用规模：万级文档

### 不建议的方向

- **不要过早引入向量数据库**：百级文档用 Obsidian 搜索足够
- **不要过早做知识合成**：先做好摘要和交叉引用，质量可控后再做推理
- **不要用 JSON 替代 Markdown**：Obsidian 生态围绕 Markdown 构建

---

## 11. 实施清单

### 搭建新仓库的步骤

- [ ] 创建 Obsidian 仓库
- [ ] 创建目录结构：`sources/`、`wiki/`、`wiki/contradictions/`、`templates/`
- [ ] 创建 Schema 层：`.claude/CLAUDE.md`（参考第 4.3 节）
- [ ] 创建三个模板文件（参考第 6 节）
- [ ] 放入第一批原始资料到 `sources/`
- [ ] 手动触发 LLM 分析第一批资料
- [ ] 在 Obsidian 中检查生成的 Wiki 页面
- [ ] 根据实际效果调整模板和 Schema 规则

### 迁移现有仓库的步骤

- [ ] 在现有仓库中创建 `sources/` 目录
- [ ] 将现有资料移入 `sources/`（保留原文件结构）
- [ ] 创建 `wiki/` 目录和子目录
- [ ] 创建 `templates/` 目录和模板文件
- [ ] 更新 `.claude/CLAUDE.md` 添加 Wiki 管理规则
- [ ] 对已有资料执行一次全量分析
- [ ] 审阅生成的 Wiki 页面，修正不准确的内容
- [ ] 建立日常维护流程（新资料 → 触发分析 → 审阅结果）

### 验证方法

搭建完成后，用以下方式验证系统是否正常工作：

1. **添加一篇新资料**：检查 LLM 是否正确生成了摘要页和概念页
2. **添加一篇与已有资料矛盾的资料**：检查 LLM 是否发现了矛盾并创建了矛盾页
3. **打开 Obsidian 图谱**：检查概念之间的链接关系是否合理
4. **点击 wikilinks 跳转**：检查所有链接是否指向存在的页面
5. **手动编辑一个 wiki 页面**：检查 LLM 下次运行时是否正确处理了人工修改
