---
name: obsidian-remember
description: |
  管理 Obsidian 知识仓库的写入、更新、去重、合并和拆分。当用户要求"记住"、"保存到笔记"、"记录到 Obsidian"、
  "写入 wiki"、"更新知识库"，或提到任何需要持久化知识到 Obsidian 仓库的操作时，使用此技能。
  也适用于用户提到知识去重、笔记合并、文档拆分、上下文窗口优化等场景。
depends: [obsidian-link]
---

# Obsidian Remember — 知识仓库管理技能

你是一个 Obsidian 知识仓库的管理员。你的职责是确保知识被正确写入、维护，并且不会因为长期积累而退化（重复、冗余、膨胀）。

## 0. 前置检查：Obsidian 链接验证

在执行任何 obsidian-remember 操作之前，必须先验证项目记忆目录与 Obsidian 仓库之间的 symlink 是否完好。

```
开始 obsidian-remember 操作
       │
       ▼
  检查项目记忆目录是否存在
       │
       ├─ 不存在 → 执行 obsidian-link 技能建立链接，完成后继续
       │
       └─ 存在 → 检查关键 symlink 是否完好
                   │
                   ├─ MEMORY.md symlink 存在且目标可读 → 继续
                   ├─ wiki/ symlink 存在且目标可读     → 继续
                   ├─ sources/ symlink 存在且目标可读  → 继续
                   └─ 任一 symlink 断裂或缺失 → 执行 obsidian-link 修复，完成后继续
```

**验证命令：**
```bash
# 检查关键 symlink
MEMORY_DIR=$(dirname "$(find /home/app/.claude/projects -name "memory" -type d 2>/dev/null | head -1)")/memory
for item in MEMORY.md wiki sources; do
    target=$(readlink -f "$MEMORY_DIR/$item" 2>/dev/null)
    [ -e "$target" ] || echo "BROKEN: $item → $target"
done
```

如果 obsidian-link 无法修复（如 Docker 挂载缺失），停止当前操作并报告用户。

## 1. 仓库结构

每个 Obsidian 知识仓库遵循三层架构：

```
vault/
├── MEMORY.md              # 记忆索引 + 管理规则（始终在上下文中）
├── README.md              # 知识库导航索引（自动维护）
├── sources/               # 原始资料层（一事一档，可读写，禁止删除）
├── wiki/                  # Wiki 层（按标签集中分类，LLM 可读写）
│   └── {topic}.md         # 同类摘要集中在一个文件，靠 tags 分类
└── templates/             # 页面模板（可选）
```

**权限矩阵：**

| 操作 | sources/ | wiki/ | MEMORY.md | README.md | _index.md |
|------|----------|-------|-----------|-----------|-----------|
| 读取 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 创建 | ✅ | ✅ | 仅追加指针 | ❌ | ❌ |
| 编辑 | ✅ | ✅ | 仅更新指针 | ✅ | ✅ |
| 删除 | ❌ | 仅标记 deprecated | ❌ | ❌ | ❌ |

**文件放置规则：** feedback/project/reference 类详档放 wiki/，根目录只放 MEMORY.md、README.md、LLM-WIKI-ARCHITECTURE.md。

### wiki/ 与 sources/ 分工

**sources/（一事一档，可读写，禁止删除）**：完整过程、原始资料
- 每个话题一个独立文件，不需要分类
- 详细操作步骤（安装指南、配置流程）
- 代码原型和完整示例
- 调试故事（第一次试了什么、失败了、第二次怎么解决的）
- 工具的完整使用手册和 API 参考

**wiki/（按标签集中分类，LLM 读写）**：结论摘要
- 同类摘要集中在同一个 wiki 文件中，靠 YAML `tags` 分类
- 每条摘要必须标注来源索引：`← [[sources/xxx.md]] L起始-结束`
- 成功经验：验证过的好做法（一句话结论）
- 失败教训：禁止行为 + 原因（后果）+ 替代做法
- 方法论结论：提炼后的原则（不含推导过程）

**判断标准：** 如果内容是"怎么做的"→ sources/（新建文件）；如果内容是"做什么/不做什么"→ wiki/（找同类文件追加或新建）

**所有记录内容的三层分工（强制）：**
1. 详细过程或完整内容 → sources/（安装步骤、代码原型、调试故事、完整教程、API参考）
2. 结论摘要 → wiki/（成功经验、失败教训、禁令+原因+替代、方法论原则）
3. 导航索引 → MEMORY.md（场景+摘要+指针）

**写入时的判断流程（强制，每条记录必须经过）：**

首先判断内容中是否包含**过程信号词**。过程信号词包括：
- 时间线叙述："之前/后来/先/然后/最终"、"搞了一整天/花了 X 小时"
- 对比经历："用了 A 失败了，换成 B 成功了"、"试了 X 次"
- 具体场景："写了 XX 脚本"、"在 XX 项目中遇到了"

```
新内容 X
  ├─ 同时包含过程信号和结论？→ 必须分离：过程 → sources/（新建文件），结论 → wiki/（找同类追加或新建）
  │   （这是最常见的情况，用户通常会描述经历 + 总结教训）
  ├─ 只有过程（无结论）？    → 新建 sources/xxx.md
  ├─ 只有结论（无过程）？    → 写入 wiki/（找同类追加或新建）
  ├─ 跨会话适用？            → MEMORY.md 加指针
  └─ 不确定？                → 保守处理：过程放 sources/（新建），结论放 wiki/
```

**禁止：** 将详细过程直接写入 wiki/ 或 MEMORY.md。违反三层分工的写入操作是错误的。
**常见错误：** 用户说"我之前用 X 失败了，后来换了 Y"时，不要把整段当作"结论"——前半段是调试故事，属于 sources/；只有"优先用 Y"这个结论属于 wiki/。

### MEMORY.md 内部结构

MEMORY.md 极简化，每个条目只有一行：**摘要 + 索引**。

```
MEMORY.md
├── 管理规则      (固定, ~10 行)
├── 个人习惯      (max 10 entries — 工作习惯+技术偏好合并)
├── 经验教训      (max 10 entries — 按场景/问题组织，成功和失败写在一起)
└── 知识库索引    (固定, 2 行 — 找不到的都在这里找)
```

**条目格式统一为：**
```
- {场景}：{一句话摘要} → [[wiki/xxx.md|详情]]([wiki/xxx.md](wiki/xxx.md))
```

无 wiki 详档的条目省略索引部分，只有摘要。

**MEMORY.md 入门门槛：** 不是每条知识都需要 MEMORY.md 入口。只有满足以下条件之一的条目才能获得：

1. **跨会话适用** — 在大多数会话中都适用（用户偏好、习惯、方法论）
2. **跨领域影响** — 影响多个工作领域的教训（如工具选择失误）
3. **高频引用** — 被频繁引用的约束或方法

不符合条件的知识不写入 MEMORY.md，通过 README.md 和领域文档浏览发现。

### Wiki 标签分类结构

Wiki 文件靠 YAML `tags` 集中分类。同类摘要写入同一个文件，通过 tags 归属同一类别。

**新知识归入 wiki 的规则：**
1. 扫描 wiki/ 所有文件的 `tags`，找到与内容匹配的文件
2. tags 有重叠 → 追加摘要到该文件，标注来源索引
3. 无 tags 重叠 → 新建 wiki 文件，打上合适的 tags
4. wiki 文件超 600 行 → 拆分为 P1/P2

**来源索引格式（强制）：** wiki 中的每条摘要必须标注来源：
```
- 结论内容 ← [[sources/xxx.md]] L起始-结束
```
这样 wiki 文件既是分类摘要，又能通过索引直接跳回原始资料。

## 2. 写入流程

### 2.1 确定目标文件

在写入前，先执行以下检查：

```
用户要求记录内容 X
       │
       ▼
  1. 过程/原始资料 → 新建 sources/xxx.md（一事一档）
  2. 结论/摘要 → 扫描 wiki/ 所有文件的 tags：
     a. tags 有重叠 → 追加摘要到该文件（§4.1）
     b. 无 tags 重叠 → 新建 wiki 文件，打上分类 tags
  3. 每条 wiki 摘要必须标注来源索引：← [[sources/xxx.md]] L起始-结束
```

### 2.2 Frontmatter 规范

每个 wiki 页面必须有 YAML frontmatter：

```yaml
---
name: 页面名称（简短，用于索引查找）
description: 一句话描述 — 覆盖哪些问题/场景
type: concept | summary | feedback | contradiction
updated: YYYY-MM-DD
tags: [标签1, 标签2]
sources: [[sources/xxx.md]]
---
```

**关键规则：** `tags` 是分类的核心字段，决定同类摘要是否归入此文件。`sources` 标注此 wiki 文件的来源资料。

### 2.3 内容格式

- 使用 Obsidian 原生语法：`[[wikilinks]]`、frontmatter、callout（`> [!quote]`）、tags、脚注
- 不使用 HTML 标签或自定义语法
- 注释和说明文字用中文
- 文件名使用 kebab-case（如 `dev-practice.md`），不在文件名中使用中文或下划线
- 双链统一带 `.md` 扩展名：`[[wiki/xxx.md|显示名]]`，不带扩展名的格式禁用
- 每个页面必须有 H1 标题（`# 标题`），标题放在 frontmatter 之后的第一行

### 2.4 MEMORY.md 索引更新

当 wiki 文件有新增内容时，先判断是否需要 MEMORY.md 指针，再更新：

```
wiki 文件 P 有新增内容
       │
       ▼
  新增内容是否满足 MEMORY.md 入门条件？（见 §1）
       │
       ├─ 否 → 仅更新 wiki 文件，不更新 MEMORY.md
       │
       └─ 是 → 检查 MEMORY.md 是否已有该场景的条目
               │
               ├─ 已有 → 更新摘要，不新增行
               └─ 没有 → 确定分类（个人习惯/经验教训）
                         检查分类是否已满
                         添加一行：摘要 + 索引
```

**指针格式：** `- {场景}：{摘要} → [[wiki/xxx.md|详情]]([wiki/xxx.md](wiki/xxx.md))`
**单行不超过 150 字符。**

**MEMORY.md 不得超过 200 行。**

### 2.5 索引维护

当 wiki/ 或 sources/ 有文件变更时，同步更新对应的索引文件：

**README.md 更新规则：**
```
wiki/ 有文件变更
       │
       ▼
  1. 新增文件 → 检查 README.md 对应章节是否需要添加条目
     ├─ 文件属于已有章节（Claude Code/开发实践/架构与学习/工具指南）→ 添加到对应章节
     └─ 文件属于新领域 → 创建新章节
  2. 删除文件 → 从 README.md 移除对应条目
  3. 更新 README.md 的 updated 字段（如有）
```

**wiki/_index.md 更新规则：**
```
wiki/ 有文件变更
       │
       ▼
  1. 新增文件 → 添加到 _index.md 表格，包含页面名、说明、标签
  2. 删除文件 → 从 _index.md 移除对应行
  3. 更新 _index.md 的 updated 字段为当前日期
```

**sources/_index.md 更新规则：**
```
sources/ 有文件变更
       │
       ▼
  1. 新增文件 → 添加到对应子目录的表格
  2. 删除文件 → 从 _index.md 移除对应行
  3. 更新 _index.md 的 updated 字段为当前日期
```

**索引完整性检查（写入操作后自动执行）：**
- README.md 中的 wikilinks 指向存在的文件
- wiki/_index.md 包含所有 wiki/ 文件（排除 _index.md 自身）
- sources/_index.md 包含所有 sources/ 文件（排除 _index.md 自身和子目录集合）

## 3. 去重检测

### 3.1 判定标准

写入前，用以下维度判断是否与已有内容重复：

| 维度 | 权重 | 判定方法 |
|------|------|----------|
| 问题场景 | 高 | 是否是同一问题/场景的经验 |
| `tags` 重叠度 | 中 | 3 个以上相同标签 |
| 内容核心命题 | 高 | 核心观点是否已被覆盖 |

### 3.2 去重决策

```
新内容 vs wiki/ 文件已有内容:
  ├─ 同一问题 + 内容重叠 > 80% → 跳过（通知用户已有记录）
  ├─ 同一问题 + 内容互补   → 追加到已有节，标注来源索引
  ├─ 相关问题              → 新建子节
  └─ 无相关性              → 扫描其他 wiki 文件的 tags，或新建文件
```

### 3.3 增量积累

某些主题天然适合增量积累（如调试经验）。新的同类记录追加到 wiki 文件的对应节末尾，不修改已有内容。

### 3.4 MEMORY.md 条目去重

写入新 MEMORY.md 条目前，检查现有条目：

```
新条目 N 拟加入 MEMORY.md
       │
       ▼
  扫描当前分类下所有条目：
       │
       ├─ 已有条目 M 与 N 语义相同 → 更新 M 的摘要，不新增行
       ├─ 已有条目 M 是 N 的子集   → 用 N 替换 M
       ├─ 已有条目 M 与 N 高度相关 → 合并为一行
       └─ 无相关性               → 新增条目 N
```

## 4. 合并策略

### 4.1 追加模式（同类扩展）

当新内容的 tags 与已有 wiki 文件重叠时：

1. 在 wiki 文件中找到合适的节或位置
2. 追加摘要，标注来源索引：`← [[sources/xxx.md]] L起始-结束`
3. 更新 frontmatter 的 `updated` 日期和 `sources` 字段
4. 如有必要，更新 `tags` 以反映新增的分类范围

### 4.2 新文件创建

当新知识的 tags 与所有已有 wiki 文件无重叠时：

1. 创建新 wiki 文件（如 `new-topic.md`）
2. 编写完整的 frontmatter，tags 写清分类，sources 指向来源资料
3. 写入内容，每条摘要标注来源索引
4. 在 MEMORY.md 添加指针（如满足入门条件，见 §1）

## 5. 拆分策略

### 5.1 触发条件

当领域文档超过 **600 行**时，必须考虑拆分。

### 5.2 拆分方法

按子主题拆分为 P1/P2，每部分有独立的 frontmatter 和 description：

```
原始: agent-arch.md (1200 行)
  ├── Agent 循环与架构 (600 行)
  └── Harness 工程与人话编程 (600 行)

拆分为:
  agent-arch-p1.md → description: "Agent循环/多智能体/系统提示词/教学脚本"
  agent-arch-p2.md → description: "Harness工程方法论/人话编程规范"
```

### 5.3 拆分后必做

1. 每个新文件都有完整的 frontmatter，description 聚焦子主题
2. P1 末尾添加 `> 续篇：[[wiki/xxx-p2.md|xxx Part 2]]`
3. P2 开头添加 `> 前篇：[[wiki/xxx-p1.md|xxx Part 1]]`
4. 更新 MEMORY.md 中对应的指针
5. 检查所有 wikilinks 指向正确

## 6. 维护操作

### 6.1 检查仓库健康度

当用户要求检查或维护仓库时，执行以下检查：

**第一步：运行自动化检测脚本**

```bash
python {SKILL_DIR}/scripts/vault_checker.py
```

脚本输出含义：
- `All files have complete metadata.` — 所有文件 frontmatter 齐全
- `Found N file(s) with missing metadata:` — 列出每个文件缺失的字段（name/description/type/updated/tags/source）

如需自动修复缺失字段（插入默认值）：

```bash
python {SKILL_DIR}/scripts/vault_checker.py --fix
```

**第二步：手动检查（脚本不覆盖的项目）**

```
1. 检查每个文档：
   ├─ description 质量（是否清晰描述覆盖范围）
   ├─ 文档行数（> 600 行标记为待拆分）
   ├─ 文件命名规范（kebab-case）
   ├─ wikilinks 有效性
   └─ 文档老化（updated 超过 90 天 → 标记为待删除，见 §8）
2. 检查 MEMORY.md：
   ├─ 行数（< 200 行）
   ├─ 每个指针指向的文件是否存在
   └─ 是否有领域文档未被索引
3. 检查索引完整性：
   ├─ README.md 中的 wikilinks 指向存在的文件
   ├─ wiki/_index.md 包含所有 wiki/ 文件
   ├─ sources/_index.md 包含所有 sources/ 文件
   └─ 索引文件的 updated 字段是否为最新
4. 汇总报告，按优先级分级：
   ├─ P0（必须修复）：缺失 frontmatter、超大文件
   ├─ P1（建议修复）：description 不清晰、断链、老化文档、索引缺失
   └─ P2（可选优化）：接近阈值的文件
```

### 6.2 精简优化

当领域文档接近但未达到拆分阈值时，可先做精简：

1. 移除重复表述
2. 将冗长描述压缩为要点
3. 合并可合并的小节

## 7. 边界情况处理

### 7.1 仓库不可访问

```
vault_path 不存在或不可读
       │
       ├─ vault_path 目录不存在 → 报错：仓库路径无效，请检查配置
       ├─ wiki/ 目录缺失       → 自动创建
       ├─ sources/ 目录缺失    → 自动创建
       └─ MEMORY.md 缺失       → 报错：核心文件缺失，无法继续
```

### 7.2 Frontmatter 异常

- **缺失 frontmatter**：写入前必须添加完整的 frontmatter（见 §2.2），不跳过
- **updated 字段格式错误**：视为 `1970-01-01`（触发老化检查）
- **updated 字段缺失**：视为当前日期，立即补全

### 7.3 MEMORY.md 容量溢出

当 MEMORY.md 更新后超过 200 行时：

```
MEMORY.md 超过 200 行
       │
       ▼
  1. 扫描所有条目，找到引用的 wiki 文档已 deprecated 的条目 → 删除
  2. 找到内容最陈旧的条目（无 wiki 指针的优先）→ 删除
  3. 找到摘要最长的条目 → 压缩摘要
  4. 仍然超限 → 报告用户，请求手动清理
```

### 7.4 拆分时的交叉引用

文档拆分时，检查所有 wiki/ 文档中的 `[[wikilinks]]`：
- 指向被拆分文档的链接 → 更新为指向正确的 P1/P2
- 指向被拆分文档中特定标题的链接（`[[doc#heading]]`）→ 根据标题所在位置更新

### 7.5 删除请求

当用户请求删除 sources/ 中的内容时：
- **拒绝直接删除**，告知用户 sources/ 禁止删除
- **替代方案**：如内容确已过时，在 wiki/ 中标记对应知识为 deprecated，并在 sources/ 文档开头添加 `> [!warning] 本文档内容已过时` callout
- 如果用户坚持删除，需要用户明确确认后才能执行

### 7.6 Symlink 断裂

```
检测到 symlink 断裂
       │
       ▼
  1. 检查目标路径是否存在
     ├─ 目标路径存在但路径变了 → 重建 symlink
     └─ 目标路径不存在 → 运行 obsidian-link 重建所有链接
  2. 如果无法修复 → 报告用户，建议手动运行 obsidian-link 技能
```

## 8. Wiki 文档老化管理

### 8.1 老化规则

每个 wiki 文档的 frontmatter `updated` 字段用于追踪最后更新时间。

**老化阈值：** 90 天（3 个月）

当 wiki 文档的 `updated` 距今超过 90 天时，执行以下流程：

```
wiki 文档 D 的 updated 超过 90 天
       │
       ▼
  1. 检查 D 是否已被标记为 deprecated
     ├─ 是 → 直接删除文件
     └─ 否 → 继续
  2. 检查 D 是否被其他文档通过 [[wikilinks]] 引用
     ├─ 被引用 → 保留，但报告用户建议审查
     └─ 无引用 → 删除文件（需用户确认）
  3. 检查 MEMORY.md 是否有指向 D 的指针
     ├─ 有 → 删除前移除 MEMORY.md 指针
     └─ 无 → 继续删除
  4. 删除后，检查是否需要更新其他文档的引用
```

**禁止自动删除非 deprecated 的文档。** 非 deprecated 文档即使无引用也必须用户确认后才能删除。

### 8.2 老化检查集成

文档老化检查集成到健康检查（§6.1）中。健康检查报告中，老化文档单列：

```
P1（建议修复）：
  - 老化文档（deprecated，可直接删除）：
    - wiki/xxx.md（最后更新：2026-01-15）
  - 老化文档（非 deprecated，无引用，建议删除）：
    - wiki/yyy.md（最后更新：2026-01-10，需确认）
  - 老化文档（被引用，建议审查）：
    - wiki/zzz.md（最后更新：2026-01-05，被 2 个文档引用）
```

## 9. 配置

默认配置（可在使用时覆盖）：

```yaml
# 仓库路径
vault_path: /home/app/obsidian

# 关键文件路径
memory_file: MEMORY.md
wiki_dir: wiki
sources_dir: sources

# 阈值
split_threshold_lines: 600
memory_max_lines: 200
dedup_similarity_threshold: 0.8

# MEMORY.md 分类容量
memory_category_max:
  个人习惯: 10
  经验教训: 10

# MEMORY.md 条目老化
memory_entry_stale_days: 30

# Wiki 文档老化
wiki_aging_days: 90
wiki_aging_auto_delete_deprecated: true
wiki_aging_require_confirm_active: true
```

## 10. 操作清单模板

每次执行写入操作时，按此清单执行：

```
□ 0. 验证 Obsidian 链接（见 §0）— 如未链接，先执行 obsidian-link
□ 1. 确认 vault_path
□ 2. 过程/原始资料 → 新建 sources/ 文件
□ 3. 结论/摘要 → 扫描 wiki/ 文件 tags 找同类，检查去重
□ 4. 执行 wiki 写入（追加/新建），每条摘要标注来源索引
□ 5. 检查文档行数，必要时拆分
□ 6. 判断是否需要 MEMORY.md 条目（见 §1 入门门槛）
□ 7. 如需添加：检查分类容量、执行条目去重（§3.4）
□ 8. 更新 MEMORY.md
□ 9. 验证 wikilinks 有效性
□ 10. 更新索引文件（README.md、wiki/_index.md、sources/_index.md，见 §2.5）
```

## 11. 经验教训记录规范

### 11.1 什么值得记录

- **成功经验**：验证过的好做法、有效的方法论
- **失败教训**：反复出现的错误、代价高昂的失误、用户明确纠正的行为

成功和失败写在**同一个 wiki 分类文件**里（按 tags 聚集），MEMORY.md 只记摘要 + 索引。

### 11.2 失败教训三要素

失败教训在 wiki 文档中必须包含三个要素：

```
### {问题场景}

**禁止：** <具体行为>
**原因：** <发生了什么，导致了什么后果>
**替代：** <应该怎么做>
```

三要素的目的：
- **原因**让 AI 理解为什么存在这条规则，在边界情况下能自主判断
- **替代**告诉 AI 应该怎么做，避免因规则变得畏手畏脚
- 如果原因不再成立（如工具已升级），规则应被移除

### 11.3 MEMORY.md 中的经验教训条目

经验教训按**场景/问题**组织，不按成功/失败分类。每条只有摘要 + 索引：

```
- 网络：JS 渲染页面 web reader 遗漏内容，用 curl + grep 替代 → [[wiki/zhipu-mcp.md|详情]](...)
- 删除操作：rm -rf 前必须先确认目录内容 → [[wiki/dev-practice.md|详情]](...)
```

具体的成功经验、失败教训、禁止行为、替代方案都写在 wiki 文档里。
