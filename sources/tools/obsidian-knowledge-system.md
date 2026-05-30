---
name: obsidian-knowledge-system
description: Obsidian 知识库系统从零搭建指南
type: summary
updated: 2026-04-18
tags: obsidian, knowledge-base, skill, deployment
source: https://example.com
---

# Obsidian 知识库系统 — 从零实现指南

## 1. 系统概览

Obsidian 知识库系统由三个组件构成，形成依赖链：

```
/note-obsidian（斜杠命令）
    ↓ 调用
Skill('obsidian-remember')（知识仓库管理）
    ↓ 前置检查
Skill('obsidian-link')（仓库连接）
```

| 组件 | 类型 | 位置 | 职责 |
|------|------|------|------|
| obsidian-link | Skill | `/home/app/obsidian/skills/obsidian-link/SKILL.md` | 将 Obsidian 仓库通过 Docker 挂载 + symlink 桥接到 Claude Code 记忆系统 |
| obsidian-remember | Skill | `~/.claude/skills/obsidian-remember/SKILL.md` | 管理知识仓库的写入、更新、去重、合并和拆分 |
| note-obsidian | 斜杠命令 | `~/.claude/commands/note-obsidian.md` | 启动后台智能体执行复盘写入 |

**依赖关系：**
```yaml
obsidian-link:    depends: []
obsidian-remember: depends: [obsidian-link]
note-obsidian:    depends: [obsidian-remember]
```

**触发方式：**
- `obsidian-link`：用户说"连接 Obsidian"、"挂载知识库"、"链接仓库"
- `obsidian-remember`：用户说"记住"、"保存到笔记"、"记录到 Obsidian"、"写入 wiki"
- `/note-obsidian`：用户输入斜杠命令

## 2. 前置条件

### 2.1 Claude Code 环境

- Claude Code CLI 已安装并运行
- 项目目录下有 `.claude/` 目录结构
- 记忆目录存在：`<项目根>/.claude/projects/-home-app-project/memory/`

### 2.2 Obsidian Vault

- 宿主机上有 Obsidian vault 目录（含 `.obsidian/` 目录标志）
- Vault 内含 `MEMORY.md` 文件（核心记忆索引）
- 推荐目录结构：
  ```
  vault/
  ├── MEMORY.md
  ├── README.md
  ├── sources/
  ├── wiki/
  ├── skills/
  └── templates/
  ```

### 2.3 Docker 配置（容器环境）

如果 Claude Code 运行在 Docker 容器中，需要将 Obsidian vault 挂载到容器内。

`docker-compose.yml` 中需添加：

```yaml
volumes:
  - <宿主机 Obsidian 路径>:/home/app/obsidian
```

挂载点固定为容器内 `/home/app/obsidian`。

## 3. 三层知识库架构

知识库采用三层分工：

```
vault/
├── MEMORY.md              # 记忆索引（始终在上下文中，≤200行）
├── README.md              # 知识库导航（由人维护）
├── sources/               # 原始资料层（可读写，禁止删除）
├── wiki/                  # Wiki 层（LLM 可读写）
│   ├── {领域}.md          # 单文件领域（<600行）
│   └── {领域}-p{N}.md     # 拆分后的多部分领域
└── templates/             # 页面模板（可选）
```

**三层分工原则：**

| 层级 | 内容类型 | 权限 |
|------|----------|------|
| sources/ | 详细过程、完整教程、调试故事、代码原型 | 可读写，禁止删除 |
| wiki/ | 结论摘要、成功经验、失败教训、方法论原则 | 可读写 |
| MEMORY.md | 场景+摘要+指针，每条一行 | 仅追加/更新指针 |

**判断标准：** "怎么做的" → sources/；"做什么/不做什么" → wiki/

### MEMORY.md 结构

```
MEMORY.md
├── 管理规则      (固定, ~10 行)
├── 个人习惯      (max 10 entries)
├── 经验教训      (max 10 entries)
└── 知识库索引    (固定, 2 行)
```

条目格式：`- {场景}：{一句话摘要} → [[wiki/xxx.md|详情]](wiki/xxx.md)`

### Wiki 领域文档规范

每个 wiki 文档必须有 YAML frontmatter：

```yaml
---
name: 页面名称
description: 一句话描述覆盖的问题/场景范围
type: concept | summary | feedback | contradiction
updated: YYYY-MM-DD
tags: [标签1, 标签2]
---
```

超过 600 行必须拆分为 P1/P2，每部分有独立的 frontmatter 和 description。

## 4. obsidian-link 完整 SKILL.md

以下是 `obsidian-link` 技能的完整内容：

```markdown
---
name: obsidian-link
description: |
  将 Obsidian 仓库连接到当前项目的 Claude Code 记忆系统。
  当用户在新项目中想要连接 Obsidian 仓库、挂载知识库、设置记忆链接时使用此技能。
  适用于：新项目初始化、Docker 容器环境、需要跨项目共享 Obsidian 知识库的场景。
  触发短语：连接 Obsidian、挂载知识库、链接仓库、关联记忆、obsidian-link、设置 Obsidian 链接。
depends: []
---

# Obsidian Link 技能

将 Obsidian 仓库通过 Docker 挂载 + symlink 桥接到项目的 Claude Code 记忆系统，实现：
- **Docker 挂载**：自动修改 `docker-compose.yml` 将 Obsidian 仓库挂入容器
- **Symlink 桥接**：在项目记忆目录创建链接，指向 Obsidian 仓库内容
- **自动发现**：扫描仓库结构，找到含 MEMORY.md 的 vault

## 整体流程

```
用户提供宿主机 Obsidian 路径
        │
        ▼
  ┌─ 检测运行环境 ─┐
  │                 │
  Docker 容器    非 Docker
  │                 │
  ▼                 ▼
检查是否已挂载    直接用绝对路径
  │                 │
  未挂载 → 修改 docker-compose.yml
  │              提示用户重启容器
  ▼                 │
  ┌─── 发现 vault ───┐
  │  扫描 .obsidian/  │
  │  定位 MEMORY.md   │
  └────────┬─────────┘
           │
           ▼
  ┌─── 创建 symlink ──┐
  │  MEMORY.md        │
  │  wiki/            │
  │  sources/         │
  │  skills/          │
  │  templates/       │
  │  README.md        │
  └────────┬──────────┘
           │
           ▼
      验证链接可用
```

## 第一步：获取宿主机 Obsidian 路径

向用户询问 Obsidian 仓库在宿主机上的绝对路径。

如果用户没有明确给出，可以检查以下常见位置：
- `~/Obsidian`
- `~/Documents/Obsidian`
- `~/obsidian-vault`

确认路径存在且包含 `.obsidian/` 目录或 `MEMORY.md` 文件。

## 第二步：环境检测与挂载配置

### 检测是否在 Docker 容器中

```bash
# 检测方法 1：/.dockerenv 文件
[ -f /.dockerenv ] && echo "IN_DOCKER=true" || echo "IN_DOCKER=false"

# 检测方法 2：cgroup 信息
cat /proc/1/cgroup 2>/dev/null | grep -q docker && echo "IN_DOCKER=true" || echo "IN_DOCKER=false"
```

### Docker 容器环境

1. **检查是否已挂载**：查看 `docker-compose.yml` 中是否已有 Obsidian 挂载
2. **未挂载时修改配置**：

在 `docker-compose.yml` 的 `volumes` 部分添加 Obsidian 挂载：

```yaml
volumes:
  # ... 已有挂载 ...
  # Obsidian vault (knowledge base)
  - <宿主机路径>:/home/app/obsidian
```

挂载点固定为 `/home/app/obsidian`（容器内路径）。

**重要**：修改 docker-compose.yml 后，需要用户重启容器才能生效。告诉用户：

> 已更新 docker-compose.yml，需要重启容器使挂载生效：
> ```bash
> docker compose down && docker compose up -d
> ```
> 重启后再次运行此技能继续配置。

**如果用户无法立即重启**：先完成后续步骤的准备工作，等重启后再执行 symlink 创建。

3. **已挂载时继续**：跳过此步骤

### 非 Docker 环境

直接使用用户提供的绝对路径作为 Obsidian 仓库路径，无需挂载。

## 第三步：发现 Vault

容器内 Obsidian 仓库挂载在 `/home/app/obsidian`（Docker）或用户提供的路径（非 Docker）。

扫描仓库结构，找到包含 `MEMORY.md` 的 vault 目录：

```bash
# 优先查找含 MEMORY.md 的目录
find /home/app/obsidian -maxdepth 2 -name "MEMORY.md" -type f 2>/dev/null

# 同时检查 .obsidian/ 目录（标准 Obsidian vault 标志）
find /home/app/obsidian -maxdepth 2 -name ".obsidian" -type d 2>/dev/null
```

如果找到多个 vault，让用户选择。如果未找到任何 vault，在仓库根目录创建基本结构。

确定 vault 路径后，记录它用于后续 symlink 创建。下文称此路径为 `$VAULT_PATH`。

## 第四步：创建 Symlink

项目记忆目录固定为：
```
<项目根>/.claude/projects/-home-app-project/memory/
```

实际路径可通过以下方式获取：
```bash
# 获取当前项目记忆目录
MEMORY_DIR=$(dirname "$(find /home/app/.claude/projects -name "memory" -type d 2>/dev/null | head -1)")/memory
```

需要创建的 symlink：

| Symlink | 目标 | 说明 |
|---------|------|------|
| `MEMORY.md` | `$VAULT_PATH/MEMORY.md` | 核心记忆索引 |
| `README.md` | `$VAULT_PATH/README.md` | 知识库导航 |
| `wiki/` | `$VAULT_PATH/wiki/` | Wiki 知识文档 |
| `sources/` | `$VAULT_PATH/sources/` | 原始资料 |
| `skills/` | `$VAULT_PATH/skills/` | 技能目录 |
| `templates/` | `$VAULT_PATH/templates/` | 模板 |

**只链接 vault 中实际存在的目录和文件**，不创建不存在的链接。

创建命令（使用绝对路径）：
```bash
cd "$MEMORY_DIR"
ln -sf "$VAULT_PATH/MEMORY.md" MEMORY.md
ln -sf "$VAULT_PATH/README.md" README.md
ln -sf "$VAULT_PATH/wiki" wiki
# ... 只创建实际存在的 ...
```

如果 `$MEMORY_DIR` 中已有同名文件（非 symlink），先备份再创建：
```bash
[ -f MEMORY.md ] && [ ! -L MEMORY.md ] && mv MEMORY.md MEMORY.md.bak
ln -sf "$VAULT_PATH/MEMORY.md" MEMORY.md
```

## 第五步：验证

逐项验证每个 symlink 是否可用：

```bash
cd "$MEMORY_DIR"
echo "=== 验证 symlink ==="
for link in $(find . -maxdepth 1 -type l); do
    target=$(readlink -f "$link")
    if [ -e "$target" ]; then
        echo "  ✓ $link → $target"
    else
        echo "  ✗ $link → $target (BROKEN)"
    fi
done

echo ""
echo "=== 验证内容可读 ==="
cat MEMORY.md | head -3
ls wiki/ | head -5
```

如果有断裂的链接，报告给用户并尝试修复。

## 输出

完成后向用户展示：

1. **链接结构图**：显示所有创建的 symlink 及其目标
2. **访问验证**：确认每个链接都可读取
3. **下一步建议**：
   - MEMORY.md 偏好会在下次会话自动加载
   - wiki/sources 内容可通过 Claude Code 的 Read 工具按需读取
   - 在 Obsidian 侧编辑的内容会实时同步到项目

## 边界情况

- **vault 不含 MEMORY.md**：提示用户是否从模板创建，或仅链接已有内容
- **项目记忆目录已有内容**：备份旧文件到 `.bak`，然后创建 symlink
- **非标准项目路径**：通过 `find` 查找实际记忆目录，不硬编码路径
- **Windows WSL 环境**：宿主机路径格式可能需要转换（`C:\Users\...` → `/mnt/c/Users/...`）
- **符号链接权限**：如果创建失败，检查文件系统是否支持 symlink（某些 Windows 环境需要管理员权限）
```

## 5. obsidian-remember 完整 SKILL.md

以下是 `obsidian-remember` 技能的完整内容。文件位于 `~/.claude/skills/obsidian-remember/SKILL.md`。

```markdown
---
name: obsidian-remember
description: |
  管理 Obsidian 知识仓库的写入、更新、去重、合并和拆分。当用户要求"记住"、"保存到笔记"、"记录到 Obsidian"、
  "写入 wiki"、"更新知识库"，或提到任何需要持久化知识到 Obsidian 仓库的操作时，使用此技能。
  也适用于用户提到知识去重、笔记合并、文档拆分、上下文窗口优化等场景。
depends: [obsidian-link]
---

# Obsidian Remember — 知识仓库管理技能

你是一个 Obsidian 知识仓库的管理员。你的职责是确保知识被正确写入、维护，
并且不会因为长期积累而退化（重复、冗余、膨胀）。

## 0. 前置检查：Obsidian 链接验证

在执行任何 obsidian-remember 操作之前，必须先验证项目记忆目录与 Obsidian 仓库
之间的 symlink 是否完好。

验证命令：
```bash
MEMORY_DIR=$(dirname "$(find /home/app/.claude/projects -name "memory" -type d 2>/dev/null | head -1)")/memory
for item in MEMORY.md wiki sources; do
    target=$(readlink -f "$MEMORY_DIR/$item" 2>/dev/null)
    [ -e "$target" ] || echo "BROKEN: $item → $target"
done
```

如果任一 symlink 断裂或缺失，先执行 obsidian-link 技能修复。

## 1. 仓库结构

三层架构：sources/（原始资料）→ wiki/（结论摘要）→ MEMORY.md（导航索引）

权限矩阵：
- sources/: 可读写，禁止删除
- wiki/: 可读写，仅标记 deprecated
- MEMORY.md: 仅追加/更新指针
- README.md: 只读

## 2. 写入流程

1. 判断内容属于哪个问题领域
2. 扫描对应领域文档，检查去重
3. 执行写入（追加/新建节/新建领域文档）
4. 检查文档行数（>600 行触发拆分）
5. 判断是否需要 MEMORY.md 条目
6. 更新 MEMORY.md

## 3. 去重检测

按问题场景、tags 重叠度、核心命题三个维度判定。
同一问题 + 内容重叠 > 80% → 跳过；同一问题 + 内容互补 → 追加。

## 4. 合并策略

- 追加模式：在已有领域文档中找到合适位置追加
- 新领域创建：创建新领域文档 + MEMORY.md 指针

## 5. 拆分策略

文档超过 600 行时按子主题拆分为 P1/P2，每部分有独立 frontmatter。

## 6. 维护操作

健康检查扫描所有文档的 frontmatter、行数、wikilinks、老化状态。

## 7. 边界情况

- 仓库不可访问、frontmatter 异常、MEMORY.md 容量溢出、symlink 断裂等处理。

## 8. 老化管理

wiki 文档 updated 超过 90 天标记为老化，deprecated 的可直接删除，非 deprecated 需用户确认。

## 9. 配置

```yaml
vault_path: /home/app/obsidian
memory_file: MEMORY.md
wiki_dir: wiki
sources_dir: sources
split_threshold_lines: 600
memory_max_lines: 200
dedup_similarity_threshold: 0.8
memory_category_max:
  个人习惯: 10
  经验教训: 10
wiki_aging_days: 90
```
```

> **注**：上方为精简版。完整内容见实际文件 `~/.claude/skills/obsidian-remember/SKILL.md`，
> 包含详细的流程图、判定标准、操作清单模板和经验教训记录规范（约 540 行）。

## 6. /note-obsidian 完整命令

以下是 `note-obsidian` 斜杠命令的完整内容：

```markdown
---
name: note-obsidian
description: 启动后台智能体执行复盘写入。
depends: [obsidian-remember]
---

启动后台智能体执行复盘写入。

使用 Agent 工具（description: "后台复盘写入", run_in_background: true），prompt 内容：

"你是复盘写入 agent。请执行以下任务：

1) 回顾本轮对话内容，提炼有价值的知识：
   - 用户提到的新经验、新发现
   - 成功的做法、失败的教训
   - 用户明确要求记住的内容

2) 调用 Skill('obsidian-remember') 加载知识仓库管理技能，按其中的写入流程执行。

3) 对每条提炼出的知识，按 obsidian-remember 技能的规范写入 wiki/ 并更新 MEMORY.md。

4) 如无新知识值得记录，静默结束。
   如有写入，报告摘要。"

完成后回复"后台 note-obsidian agent 已启动"。
```

## 7. 注册与部署

### 7.1 文件创建

从零部署需要创建以下文件：

```
~/.claude/
├── skills/
│   └── obsidian-remember/
│       └── SKILL.md                    # 知识仓库管理技能
├── commands/
│   └── note-obsidian.md                # 复盘写入斜杠命令
└── projects/-home-app-project/memory/  # 记忆目录（自动存在）

/home/app/obsidian/
└── skills/
    └── obsidian-link/
        └── SKILL.md                    # 仓库连接技能
```

### 7.2 obsidian-link 部署

`obsidian-link` 部署在 Obsidian vault 内部的 `skills/` 目录中：

```bash
# 确保 vault 的 skills 目录存在
mkdir -p /home/app/obsidian/skills/obsidian-link

# 将 SKILL.md 放入
# （内容见第 4 节）
```

**为什么放在 vault 内？** obsidian-link 操作的是 vault 与 Claude Code 的连接，
属于 vault 级别的配置，跟随 vault 走。

### 7.3 obsidian-remember 部署

`obsidian-remember` 部署在 Claude Code 的全局 skills 目录中：

```bash
# 确保 skills 目录存在
mkdir -p ~/.claude/skills/obsidian-remember

# 将 SKILL.md 放入
# （内容见第 5 节）
```

**为什么全局？** 知识仓库管理是跨项目的通用能力，所有项目都可复用。

### 7.4 note-obsidian 部署

斜杠命令放在 Claude Code 的 commands 目录中：

```bash
# 确保 commands 目录存在
mkdir -p ~/.claude/commands

# 将 note-obsidian.md 放入
# （内容见第 6 节）
```

### 7.5 Symlink 桥接

运行 `obsidian-link` 技能或在记忆目录手动创建 symlink：

```bash
MEMORY_DIR=~/.claude/projects/-home-app-project/memory
VAULT_PATH=/home/app/obsidian

cd "$MEMORY_DIR"
ln -sf "$VAULT_PATH/MEMORY.md" MEMORY.md
ln -sf "$VAULT_PATH/README.md" README.md
ln -sf "$VAULT_PATH/wiki" wiki
ln -sf "$VAULT_PATH/sources" sources
```

## 8. 验证

### 8.1 端到端验证步骤

**Step 1：验证 obsidian-link**

```bash
# 检查技能文件存在
cat /home/app/obsidian/skills/obsidian-link/SKILL.md | head -5

# 检查 symlink 完好
MEMORY_DIR=$(dirname "$(find /home/app/.claude/projects -name "memory" -type d 2>/dev/null | head -1)")/memory
for item in MEMORY.md wiki sources; do
    target=$(readlink -f "$MEMORY_DIR/$item" 2>/dev/null)
    [ -e "$target" ] && echo "OK: $item" || echo "BROKEN: $item"
done
```

**Step 2：验证 obsidian-remember**

```bash
# 检查技能文件存在
cat ~/.claude/skills/obsidian-remember/SKILL.md | head -5

# 检查依赖声明
grep "depends:" ~/.claude/skills/obsidian-remember/SKILL.md
# 应输出: depends: [obsidian-link]
```

**Step 3：验证 note-obsidian**

```bash
# 检查命令文件存在
cat ~/.claude/commands/note-obsidian.md | head -5

# 检查依赖声明
grep "depends:" ~/.claude/commands/note-obsidian.md
# 应输出: depends: [obsidian-remember]
```

**Step 4：端到端功能测试**

在 Claude Code 中：

1. 输入 `/note-obsidian`，确认后台 agent 启动
2. Agent 应调用 `Skill('obsidian-remember')`
3. obsidian-remember 应先执行前置检查（验证 symlink）
4. 如果 symlink 完好，继续执行复盘写入
5. 如果 symlink 断裂，先调用 `obsidian-link` 修复

**Step 5：验证写入结果**

```bash
# 检查 wiki 目录有新内容
ls -la /home/app/obsidian/wiki/

# 检查 MEMORY.md 更新
head -30 /home/app/obsidian/MEMORY.md

# 检查 MEMORY.md 行数不超过 200
wc -l /home/app/obsidian/MEMORY.md
```

### 8.2 依赖元数据验证

确认三个组件的 frontmatter 都包含 `depends` 字段：

```bash
echo "=== obsidian-link ==="
grep "depends:" /home/app/obsidian/skills/obsidian-link/SKILL.md

echo "=== obsidian-remember ==="
grep "depends:" ~/.claude/skills/obsidian-remember/SKILL.md

echo "=== note-obsidian ==="
grep "depends:" ~/.claude/commands/note-obsidian.md
```

预期输出：
```
=== obsidian-link ===
depends: []
=== obsidian-remember ===
depends: [obsidian-link]
=== note-obsidian ===
depends: [obsidian-remember]
```
