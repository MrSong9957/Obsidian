---
name: obsidian-remember
description: |
  管理 Obsidian 知识仓库的写入、更新、去重、合并和拆分。当用户要求"记住"、"保存到笔记"、"记录到 Obsidian"、
  "写入 wiki"、"更新知识库"，或提到任何需要持久化知识到 Obsidian 仓库的操作时，使用此技能。
  也适用于用户提到知识去重、笔记合并、文档拆分、上下文窗口优化等场景。
---

# Obsidian Remember — 知识仓库管理技能

你是一个 Obsidian 知识仓库的管理员。你的职责是确保知识被正确写入、维护，并且不会因为长期积累而退化（重复、冗余、膨胀）。

## 1. 仓库结构

每个 Obsidian 知识仓库遵循三层架构：

```
vault/
├── MEMORY.md              # 记忆索引 + 管理规则（始终在上下文中）
├── README.md              # 知识库导航索引（由人维护）
├── sources/               # 原始资料层（只读）
├── wiki/                  # Wiki 层（LLM 可读写）
│   ├── *.md               # 各种类型的知识页面
│   └── contradictions/    # 矛盾标注（可选）
└── templates/             # 页面模板（可选）
```

**权限矩阵：**

| 操作 | sources/ | wiki/ | MEMORY.md | README.md |
|------|----------|-------|-----------|-----------|
| 读取 | ✅ | ✅ | ✅ | ✅ |
| 创建 | ❌ | ✅ | 仅追加指针 | ❌ |
| 编辑 | ❌ | ✅ | 仅更新指针 | ❌ |
| 删除 | ❌ | 仅标记 deprecated | ❌ | ❌ |

## 2. 写入流程

### 2.1 确定目标文件

在写入前，先执行以下检查：

```
用户要求记录内容 X
       │
       ▼
  1. 扫描 wiki/ 所有 .md 文件
  2. 读取每个文件的 frontmatter（name, description, tags）
  3. 判断 X 是否属于已有页面：
     a. 完全匹配 → 追加到已有页面
     b. 高度相关 → 合并到已有页面（见 §4）
     c. 全新主题 → 创建新页面
```

### 2.2 Frontmatter 规范

每个 wiki 页面必须有 YAML frontmatter：

```yaml
---
name: 页面名称（简短，用于索引查找）
description: 一句话描述 — 用于判断相关性和搜索
type: concept | summary | feedback | contradiction
updated: YYYY-MM-DD
tags: [标签1, 标签2]
---
```

可选字段（按需）：
- `sources`: 来源资料列表（如 `"[[sources/xxx]]"`）
- `confidence`: 置信度 0-1
- `status`: active / deprecated

**关键规则：** `name` 和 `description` 是去重和搜索的主要依据，必须准确反映页面内容。

### 2.3 内容格式

- 使用 Obsidian 原生语法：`[[wikilinks]]`、frontmatter、callout（`> [!quote]`）、tags、脚注
- 不使用 HTML 标签或自定义语法
- 注释和说明文字用中文
- 文件名使用 kebab-case（如 `prompt-toolkit.md`），不在文件名中使用中文或下划线
- 每个页面必须有 H1 标题（`# 标题`），标题放在 frontmatter 之后的第一行

### 2.4 MEMORY.md 索引更新

当 wiki/ 中新增或删除文件时，同步更新 `MEMORY.md`：
- 新增文件：在对应分类下添加一行指针（不超过 150 字符）
- 格式：`- [显示名](wiki/filename.md) — 一句话说明`
- 如果该页面对应用户偏好，行末加 `→ [详情](wiki/xxx.md)`

**MEMORY.md 不超过 200 行。** 接近上限时，合并相似条目或移除低价值指针。

## 3. 去重检测

### 3.1 判定标准

写入前，用以下维度判断是否与已有内容重复：

| 维度 | 权重 | 判定方法 |
|------|------|----------|
| `name` 相似度 | 高 | 名称完全相同或为子集 |
| `tags` 重叠度 | 中 | 3 个以上相同标签 |
| `description` 语义 | 高 | 描述的是同一概念/事件/经验 |
| 内容核心命题 | 高 | 正文的核心观点是否已被覆盖 |

### 3.2 去重决策

```
新内容 vs 已有页面:
  ├─ name 相同 + 内容重叠 > 80% → 跳过，不写入（通知用户已有记录）
  ├─ name 相同 + 内容互补   → 追加到已有页面（§4.1 追加模式）
  ├─ name 不同但高度相关    → 合并或交叉链接（§4.2 合并模式）
  └─ 无相关性              → 创建新页面（§2）
```

### 3.3 增量积累型文档

某些文档天然适合增量积累（如调试经验、学习记录）。这些文档使用日期分节格式：

```markdown
## YYYY-MM-DD: 事件标题

### 问题 / 背景
...

### 根因 / 分析
...

### 修复 / 结论
...

### 教训 / 要点
...
```

新的同类记录追加到文件末尾，不修改已有内容。但仍需监控总长度（见 §5 拆分规则）。

## 4. 合并策略

### 4.1 追加模式（同主题扩展）

当新内容是对已有页面的补充时：

1. 找到目标页面的合适插入位置（按主题分区）
2. 插入新内容，保持页面结构一致
3. 更新 frontmatter 的 `updated` 日期
4. 检查是否需要更新 `name` 或 `description`

### 4.2 合并模式（两个相关页面合为一个）

当两个页面主题高度相关但独立存在时：

1. 选择保留哪个文件名（选描述性更强的）
2. 将被合并页面的内容融入保留页面
3. 被合并页面改为只保留 frontmatter + 重定向说明：
   ```markdown
   > 本页内容已合并至 [[保留页面名]]。
   ```
4. 被合并页面的 frontmatter 添加 `status: deprecated`
5. 更新 MEMORY.md 中的指针

### 4.3 交叉链接模式（独立但相关）

当两个页面相关但不适合合并时：

1. 在两个页面中互相添加 `[[wikilink]]`
2. 在相关概念区块说明关系
3. 不合并文件

## 5. 拆分策略

### 5.1 触发条件

当 wiki 页面超过 **500 行**时，必须考虑拆分。

### 5.2 拆分判断

```
页面 > 500 行
       │
       ▼
  1. 分析页面结构：有哪些独立的主题分区？
  2. 判断分区之间的关联性：
     ├─ 强关联（互相引用频繁）→ 只做内部优化（精简冗余），不拆分
     └─ 弱关联（可独立理解）  → 执行拆分
```

### 5.3 拆分方法

**方法 A：按主题分区拆分**

适用于：一个大而全的页面包含多个独立主题

```
原始: wiki/big-topic.md (600 行)
  ├── 主题 A (200 行)
  ├── 主题 B (150 行)
  └── 主题 C (250 行)

拆分为:
  wiki/topic-a.md (主题 A，200 行)
  wiki/topic-b.md (主题 B，150 行)
  wiki/topic-c.md (主题 C，250 行)
  wiki/big-topic.md → 保留为索引页，包含 [[wikilinks]] 指向拆分后的页面
```

索引页格式：
```markdown
---
name: 原始名称（索引）
description: 主题索引 — 拆分后的子页面导航
type: concept
status: active
updated: YYYY-MM-DD
tags: [原有标签]
---

# 原始标题

> 本页已拆分为以下子页面：

- [[topic-a|主题 A]] — 简要说明
- [[topic-b|主题 B]] — 简要说明
- [[topic-c|主题 C]] — 简要说明
```

**方法 B：按时间线拆分**

适用于：增量积累型文档（如调试经验、学习记录）

```
原始: wiki/debugging-lessons.md (600 行)
  ├── 2026-03 的记录 (200 行)
  ├── 2026-04 的记录 (250 行)
  └── 2026-05 的记录 (150 行)

拆分为:
  wiki/debugging-lessons.md → 保留最新 3 个月 + 索引链接
  wiki/archived/debugging-2026-q1.md → 早期记录归档
```

### 5.4 拆分后必做

1. 每个新文件都有完整的 frontmatter
2. 更新 MEMORY.md 中的指针
3. 原文件保留为索引或保留最新内容
4. 检查所有 `[[wikilinks]]` 指向是否正确

## 6. 维护操作

### 6.1 检查仓库健康度

当用户要求检查或维护仓库时，执行以下检查：

```
1. 扫描 wiki/ 所有 .md 文件
2. 检查每个文件：
   ├─ frontmatter 完整性（name, description, type, updated, tags）
   ├─ H1 标题存在性（frontmatter 后的第一行必须是 # 标题）
   ├─ 文件行数（> 500 行标记为待拆分，> 400 行标记为需关注）
   ├─ 文件命名规范（应为 kebab-case，无中文、无下划线）
   ├─ wikilinks 有效性（指向的目标文件或目录是否存在）
   └─ updated 时效性（超过 90 天标记为可能过时）
3. 检查 MEMORY.md：
   ├─ 行数（< 200 行）
   ├─ 每个指针指向的文件是否存在
   └─ 是否有 wiki/ 文件未被索引
4. 检查 wiki/ 整体：
   ├─ 是否有主题高度重叠的页面（合并候选）
   └─ 是否有命名不一致的文件（如 snake_case 混用）
5. 汇总报告，按优先级分级：
   ├─ P0（必须修复）：缺失 frontmatter、超大文件
   ├─ P1（建议修复）：断链、命名不规范、缺失 H1
   └─ P2（可选优化）：接近阈值的文件、缺失可选字段
```

### 6.2 精简优化

当文档接近但未达到拆分阈值时，可先做精简：

1. 移除重复表述
2. 将冗长描述压缩为要点
3. 合并可合并的小节
4. 将详细的引用索引移到折叠区（Obsidian callout）

## 7. 配置

默认配置（可在使用时覆盖）：

```yaml
# 仓库路径（默认为当前项目的 Obsidian 挂载点）
vault_path: /home/app/obsidian/Claude Code Memory

# 关键文件路径
memory_file: MEMORY.md
wiki_dir: wiki
sources_dir: sources

# 阈值
split_threshold_lines: 500        # 超过此行数触发拆分评估
memory_max_lines: 200             # MEMORY.md 最大行数
dedup_similarity_threshold: 0.8   # 内容重叠度超过此值视为重复

# 页面类型
page_types: [concept, summary, feedback, contradiction]
```

使用不同的仓库时，通过 `vault_path` 指定路径即可。所有操作都基于相对于 `vault_path` 的路径。

## 8. 操作清单模板

每次执行写入操作时，按此清单执行：

```
□ 1. 确认 vault_path（默认或用户指定）
□ 2. 扫描现有 wiki/ 文件，检查去重
□ 3. 决定写入策略：新建 / 追加 / 合并 / 跳过
□ 4. 执行写入
□ 5. 检查目标文件行数，必要时建议拆分
□ 6. 更新 MEMORY.md 索引
□ 7. 验证 wikilinks 有效性
```
