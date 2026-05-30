---
name: claude-md
description: 全局（karpathy 行为准则）+ 项目级（superpowers 工作流）双层 CLAUDE.md 创建方法
type: reference
updated: 2026-05-10
tags: claude-md, template, karpathy, superpowers, kv-cache, two-tier
source: https://example.com
---

# CLAUDE.md 双层架构模板

> 来源：
> - 全局层：[andrej-karpathy-skills](https://github.com/forrestchang/andrej-karpathy-skills) — Karpathy 四条行为准则
> - 项目层：[superpowers](https://github.com/obra/superpowers) — Superpowers 开发工作流方法论

---

# 第一层：全局 CLAUDE.md

> 文件位置：`~/.claude/CLAUDE.md`（用户级，跨项目生效）

全局层放**行为准则 + 个人偏好**，不放项目特定配置。

```markdown
# CLAUDE.md

---

## andrej-karpathy-skills

行为准则，减少常见 LLM 编码错误。与项目特定指令合并使用。

**权衡：** 这些准则偏向谨慎而非速度。对于琐碎任务，请自行判断。

### 1. 思考先行

**不要假设。不要隐藏困惑。呈现权衡。**

实现前：
- 明确陈述你的假设。如果不确定，询问。
- 如果存在多种解释，展示它们——不要默默选择。
- 如果存在更简单的方法，说出来。必要时提出异议。
- 如果某事不清楚，停下来。指出困惑之处。询问。

### 2. 极简主义

**解决问题的最少代码。不做推测性实现。**

- 不添加超出要求的功能。
- 不为一次性代码做抽象。
- 不添加未被要求的"灵活性"或"可配置性"。
- 不处理不可能场景的错误。
- 如果你写了 200 行而 50 行就能解决，重写它。

问自己："资深工程师会说这过于复杂吗？"如果是，简化。

### 3. 精准修改

**只触碰必须修改的。只清理自己造成的混乱。**

编辑现有代码时：
- 不要"改进"相邻的代码、注释或格式。
- 不要重构没坏的东西。
- 匹配现有风格，即使你自己会做得不同。
- 如果发现无关的死代码，提及它——不要删除它。

当你的更改产生孤儿代码时：
- 删除你的更改导致未使用的导入/变量/函数。
- 除非被要求，否则不要删除预先存在的死代码。

检验标准：每一行更改都应直接追溯到用户的请求。

### 4. 目标驱动

**定义成功标准。循环直到验证通过。**

将任务转化为可验证的目标：
- "添加验证" → "为无效输入编写测试，然后让它们通过"
- "修复 bug" → "编写重现它的测试，然后让它通过"
- "重构 X" → "确保测试在重构前后都通过"

对于多步骤任务，简要说明计划：
1. [步骤] → 验证：[检查]
2. [步骤] → 验证：[检查]
3. [步骤] → 验证：[检查]

强有力的成功标准让你能独立循环。弱标准（"让它工作"）需要不断澄清。

---

**这些准则有效的标志：** 差异中不必要的更改更少，因过度复杂而重写的次数更少，澄清问题出现在实现之前而非错误之后。

---

## 个人偏好

- 语言：中文
- 工作流：自主闭环，多次失败才请求介入
- 子代理：可并行或需隔离时用；简单操作直接做
- 代码讲解：底层逻辑优先，不跳过用户的问题
- 开发方法：默认 TDD
```

---

# 第二层：项目级 CLAUDE.md

> 文件位置：`./CLAUDE.md`（项目根目录，项目特有）

项目层放**开发工作流 + 项目配置**。每个项目独立定制。

## 核心工作流（Superpowers 七阶段）

```markdown
# CLAUDE.md

---

## Superpowers

### 开发工作流

遵循 Superpowers 七阶段工作流：

#### 1. 头脑风暴（Brainstorming）`brainstorming`
写代码之前，先通过提问细化需求，探索替代方案，分段展示设计供验证。

#### 2. Git Worktree `using-git-worktrees`
设计批准后，创建隔离工作空间（新分支），验证测试基线干净。

#### 3. 编写计划（Writing Plans）`writing-plans`
将工作拆分为 2-5 分钟的小任务。每个任务包含：精确文件路径、完整代码、验证步骤。

#### 4. 子代理驱动开发（Subagent-Driven Development）`subagent-driven-development` / `dispatching-parallel-agents` / `executing-plans`
每个任务分派独立子代理执行，双阶段审查（规格合规 + 代码质量）。

#### 5. 测试驱动开发（TDD）`test-driven-development`
严格 RED-GREEN-REFACTOR：先写失败测试 → 看它失败 → 写最小代码 → 看它通过 → 提交。
测试之前写的代码应删除重写。

#### 6. 代码审查（Code Review）`requesting-code-review` / `receiving-code-review`
任务间进行审查，按严重性报告问题。CRITICAL 级别阻止推进。

#### 7. 完成分支（Finishing Branch）`finishing-a-development-branch` / `verification-before-completion`
验证测试通过，呈现选项（merge/PR/保留/丢弃），清理 worktree。

---

### 技能库

可用技能通过 `Skill` 工具调用，按阶段归类：

#### 规划阶段
| 技能 | 触发时机 |
|------|---------|
| brainstorming | 任何创造性工作（新功能、新组件、行为修改）**之前**必须使用 |
| writing-plans | 有需求/规格需要拆分为多步任务时，写代码之前 |
| using-git-worktrees | 需要隔离工作空间、创建 feature 分支时 |

#### 执行阶段
| 技能 | 触发时机 |
|------|---------|
| subagent-driven-development | 当前会话中执行已拆分的独立任务 |
| dispatching-parallel-agents | 2+ 个无依赖的独立任务需要并行时 |
| executing-plans | 在独立会话中执行已有计划（含审查检查点） |
| test-driven-development | 实现功能或修复 bug 前，先写测试 |

#### 质量保障
| 技能 | 触发时机 |
|------|---------|
| verification-before-completion | 声称工作完成/修复/通过 **之前**，必须先运行验证 |
| requesting-code-review | 完成任务、实现大功能、合并前 |
| receiving-code-review | 收到代码审查反馈时，先验证再实施 |

#### 收尾阶段
| 技能 | 触发时机 |
|------|---------|
| finishing-a-development-branch | 实现完成、测试通过后，决定 merge/PR/保留/丢弃 |

#### 其他技能
| 技能 | 触发时机 |
|------|---------|
| using-superpowers | 每次对话开始时，建立技能发现和调用规则 |
| writing-skills | 创建/编辑技能，或验证技能可用性 |
| systematic-debugging | 跨阶段：遇到 bug、测试失败、意外行为时，**先观察再修复** |

---

### 设计原则

- 系统化优于临时应对：流程优于猜测
- 复杂度缩减：简洁是首要目标

技能分为两类：
- **刚性技能**（TDD、调试）：严格执行，不走捷径
- **弹性技能**（模式）：按上下文灵活应用

技能优先级：流程技能优先（brainstorming、debugging）→ 实施技能其次（frontend-design 等）

技能调用：1% 规则，即使只有 1% 可能适用也必须先调用技能；调用顺序：技能检查在澄清问题之前；指令优先级：用户指令 > superpowers 技能 > 默认行为

### 铁律

1. **TDD 铁律**：没有失败测试，不写生产代码
2. **调试铁律**：没有根因调查，不实施修复
3. **验证铁律**：没有运行证据，不声称完成

### 子代理派发规则

派发子代理时，将对应技能要求注入 prompt：
- 实现任务 → `先调用 Skill 工具加载 test-driven-development 技能`
- 代码质量审查 → `先调用 Skill 工具加载 requesting-code-review 技能`
- 完成分支 → `先调用 Skill 工具加载 finishing-a-development-branch 技能`

---

## 项目信息

### 技术栈

[项目技术栈描述]

---

### 架构

[项目架构描述]

---

### 文件约定

- **设计文档**：`docs/superpowers/specs/YYYY-MM-DD--<name>.md`
- **任务计划**：`docs/superpowers/plans/YYYY-MM-DD-<name>.md`
- **个人技能**：`~/.claude/skills/<name>/SKILL.md`
- **Worktree 目录**：`.worktrees/`（项目本地隐藏，必须 git check-ignore 验证）

---

### 项目

[项目描述]
```

---

# 配套文件结构

```
~/.claude/
├── CLAUDE.md                    # 全局指令（karpathy 行为准则）
├── rules/common/                # 详细规则文件
│   ├── development-workflow.md  # 开发流程
│   ├── code-review.md           # 代码审查
│   ├── security.md              # 安全检查
│   ├── coding-style.md          # 编码风格
│   ├── testing.md               # 测试要求
│   ├── git-workflow.md          # Git 工作流
│   ├── agents.md                # Agent 编排
│   ├── patterns.md              # 设计模式
│   ├── performance.md           # 性能优化
│   └── hooks.md                 # Hooks 系统
├── agents/                      # Agent 定义文件
└── skills/                      # 技能定义

<project>/
├── CLAUDE.md                    # 项目指令（superpowers 工作流 + 项目配置）
└── .claude/
    └── settings.json            # 项目级 Claude Code 设置
```

---

# 关键设计决策

| 决策 | 原因 |
|------|------|
| 全局层用中文 | 母语更精确、更易维护，减少翻译歧义 |
| 项目层用 superpowers 工作流 | 七阶段方法论替代零散指令，确保从设计到交付的完整流程 |
| 个人偏好只放 5 条核心偏好，不放项目特有配置 | 核心偏好跨项目稳定，项目特有配置放在项目级 CLAUDE.md |
| 项目级 CLAUDE.md 两级分区（Superpowers + 项目信息） | 符合 KV 缓存规则：稳定前缀高命中率，动态后缀不影响前缀缓存 |
| 铁律替代设计原则中的重复项 | TDD/调试/验证各一条铁律，比"测试先行""证据优于声明"更精确、更不易遗漏 |
| 技能调用 1% 规则 | 即使只有 1% 可能适用也必须先调用技能，防止遗漏关键流程 |
| 详细规则外置到 rules/ | CLAUDE.md 保持精简（< 200 行），rules 按需加载 |
| ECC 补强作为可选模块独立存放 | 不是所有项目都使用 ECC 插件，保持模板精简，按需添加 |

---

# ECC 补强模块（可选）

> 安装 ECC 插件后，按需将以下内容添加到项目级 CLAUDE.md 的 `技能库` 节之后、`设计原则` 节之前。

```markdown
### ECC 补强（Everything Claude Code 插件）

ECC 提供基础设施层，补充 Superpowers 不覆盖的领域。两者分工：

**Superpowers 负责（流程层）**：TDD、规划、代码审查、Git 工作流、调试、验证
**ECC 负责（基础设施层）**：编码规范、上下文管理、语言模式、安全、持续学习

重叠时优先用 Superpowers：TDD 流程、任务规划、代码审查、分支管理。

#### Strategic Compacting

在逻辑节点手动 `/compact`，不让自动压缩中途触发：
- **压缩时机**：探索完成→执行前 / 里程碑完成→下一个前
- **禁止压缩**：任务执行中途 / 用户反馈未消化

#### Continuous Learning

发现非平凡模式时主动 `/learn`，不必等会话结束。适合提取：错误解决方案、调试技巧、变通方案、项目特定模式。

#### Session Memory

撞上下文限制前主动 `/save-session`。状态文件包含：有效方法（附证据）/ 失败尝试 / 待办。跨天继续时 `/resume-session`。

#### Token Economics

子代理模型选择：重复性任务→Haiku / 90% 编码→Sonnet（默认）/ 架构决策或安全关键→Opus。

#### 子代理上下文传递

传目标不只传查询——告诉子代理"为什么需要这个信息"。评估返回值充分性，不充分则追问（最多 3 轮）。

#### Context Budget

上下文使用超 70% 时主动 `/context-budget` 审计。经验阈值：MCP < 10 个，活跃工具 < 80 个。

#### 常用命令

| 命令 | 用途 |
|------|------|
| /build-fix | 修复构建错误 |
| /e2e | 生成和运行 E2E 测试 |
| /test-coverage | 检查测试覆盖率 |
| /refactor-clean | 清理死代码 |
| /context-budget | 上下文窗口审计 |
| /strategic-compact | 结构化压缩 |
| /save-session /resume-session | 跨会话状态管理 |
| /learn | 提取可复用模式 |
```

---

# 官方四层体系

> 来源：[Claude Code 官方文档 — 记忆系统](https://code.claude.com/docs/zh-CN/memory)（2026-05-30 查阅）

上面的「双层架构模板」是实用模板，这里补充官方文档的完整层级定义。两者不冲突——模板中的「全局 + 项目」对应官方的「用户 + 项目」。

## 四层 CLAUDE.md

| 层级 | 位置 | 谁管理 | 共享范围 | 加载时机 |
|------|------|--------|---------|---------|
| 托管策略 | Windows: `C:\Program Files\ClaudeCode\CLAUDE.md`<br>macOS: `/Library/Application Support/ClaudeCode/CLAUDE.md`<br>Linux/WSL: `/etc/claude-code/CLAUDE.md` | IT/DevOps | 组织内所有用户 | 最先加载，不可排除 |
| 用户指令 | `~/.claude/CLAUDE.md` | 你自己 | 仅你（所有项目） | 第二加载 |
| 项目指令 | `./CLAUDE.md` 或 `./.claude/CLAUDE.md`（二选一，等效） | 团队 | 通过 git 共享 | 第三加载 |
| 本地指令 | `./CLAUDE.local.md` | 你自己 | 仅你（当前项目），加入 `.gitignore` | 第四加载，最后读取的内容 |

**关键规则：**
- 所有层级的内容**拼接**到上下文中，不是相互覆盖
- 同一目录中，`CLAUDE.local.md` 在 `CLAUDE.md` 之后附加（个人笔记最后读取）
- 从文件系统根目录向下排序到工作目录——越接近工作目录的指令越晚出现，优先级语义上更高

## 加载顺序详解

Claude Code 从当前工作目录**向上遍历**目录树，在每个目录检查 `CLAUDE.md` 和 `CLAUDE.local.md`。同时，工作目录下的子目录中的 CLAUDE.md **按需加载**（Claude 读取该子目录文件时才注入）。

示例：在 `project/src/app/` 中启动 Claude Code：

```
启动时加载（从上到下拼接到上下文）：
  project/CLAUDE.md          ← 最先
  project/CLAUDE.local.md
  project/src/CLAUDE.md
  project/src/CLAUDE.local.md
  project/src/app/CLAUDE.md
  project/src/app/CLAUDE.local.md  ← 最后（最高语义优先级）

按需加载（Claude 读取子目录文件时才注入）：
  project/src/app/utils/CLAUDE.md
  project/src/app/components/CLAUDE.md
  ...
```

**压缩后行为：** 项目根目录 CLAUDE.md 在 `/compact` 后自动重新注入；子目录中的嵌套 CLAUDE.md 不会自动重新注入，需等 Claude 下次读取该子目录文件。

## `.claude/rules/` 规则系统

规则是指令的模块化拆分，按主题组织到 `.claude/rules/` 目录中。与 CLAUDE.md 的区别：规则可以限定到特定文件路径（按需加载），减少上下文噪音。

```
<project>/
├── CLAUDE.md                 # 主项目指令
└── .claude/
    ├── CLAUDE.md             # 等效的项目指令位置
    └── rules/
        ├── code-style.md     # 无 paths → 启动时无条件加载
        ├── testing.md        # 无 paths → 启动时无条件加载
        └── api-design.md     # 有 paths → 按需加载
```

### 无条件规则 vs 路径范围规则

| 类型 | frontmatter | 加载时机 | 用途 |
|------|-------------|---------|------|
| 无条件规则 | 无 `paths` 字段 | 启动时加载，与 `.claude/CLAUDE.md` 同优先级 | 全局适用的编码标准 |
| 路径范围规则 | 有 `paths` 字段（glob 模式） | Claude 处理匹配文件时才加载 | 特定目录/文件类型的规则 |

路径范围规则示例：

```markdown
---
paths:
  - "src/api/**/*.ts"
---

# API 开发规则
- 所有端点必须输入验证
- 使用标准错误响应格式
```

### 用户级规则

`~/.claude/rules/` 中的个人规则适用于所有项目，在项目规则**之前**加载（项目规则优先级更高）。

## 自动记忆系统

自动记忆让 Claude 跨会话积累知识，无需手动编写。Claude 自行决定什么值得记住（构建命令、调试见解、架构笔记等）。

### 官方机制

| 项目 | 说明 |
|------|------|
| 存储位置 | `~/.claude/projects/<project-hash>/memory/` |
| 入口文件 | `MEMORY.md`（索引，启动时加载前 200 行或 25KB） |
| 主题文件 | `debugging.md`、`patterns.md` 等（按需读取，启动时不加载） |
| 共享范围 | 同一 git 仓库的所有 worktrees 和子目录共享 |
| 跨机器 | 不自动同步，机器本地 |
| 版本要求 | Claude Code v2.1.59+ |
| 禁用方式 | 设置 `autoMemoryEnabled: false` 或环境变量 `CLAUDE_CODE_DISABLE_AUTO_MEMORY=1` |

### MEMORY.md 容量限制

| 限制 | 说明 |
|------|------|
| 行数 | 前 200 行 |
| 大小 | 前 25KB |
| 取先到者 | 行数或大小任一达到即截断 |
| 超出部分 | 会话启动时不加载 |
| CLAUDE.md 对比 | 无论长度都完整加载（但较短文件遵守度更高） |

### 自定义存储位置

在 `~/.claude/settings.json` 中设置：

```json
{
  "autoMemoryDirectory": "~/my-custom-memory-dir"
}
```

必须是绝对路径或以 `~/` 开头。仅从策略和用户设置接受，不从项目或本地设置接受（防止克隆仓库重定向记忆到敏感位置）。

### 个人实践：Symlink 桥接 Obsidian 仓库

通过两个自定义技能（`obsidian-link` + `obsidian-remember`）将项目记忆目录 symlink 到 Obsidian 仓库，实现：
1. **复用积累**：项目中直接读取已整理的知识文档（wiki/sources/templates）
2. **同步备份**：通过 Obsidian 仓库的 git 仓库同步到 GitHub
3. **知识管理**：Claude 自动积累的记忆和手动整理的知识在同一个仓库中

#### 桥接结构

```
~/.claude/projects/<project-hash>/memory/
├── MEMORY.md       → symlink → $vault/MEMORY.md
├── README.md       → symlink → $vault/README.md
├── wiki/           → symlink → $vault/wiki/
├── sources/        → symlink → $vault/sources/
├── skills/         → symlink → $vault/skills/
└── templates/      → symlink → $vault/templates/
```

Claude 的自动记忆直接写入 Obsidian 仓库对应位置；Obsidian 侧的编辑实时同步到项目记忆。

#### obsidian-link 技能

> 技能路径：`~/.claude/skills/obsidian-link/SKILL.md`

自动完成从项目记忆目录到 Obsidian 仓库的 symlink 桥接，支持三种环境：

| 环境 | 文件链接方式 | 目录链接方式 | 权限要求 |
|------|------------|------------|---------|
| Docker 容器 | Symlink | Symlink | 无（先挂载 volume） |
| 非 Docker（Linux/macOS） | Symlink | Symlink | 无 |
| Windows | Symlink | Junction | 管理员或开发者模式 |

流程：获取 Obsidian 路径 → 检测环境 → Docker 则挂载 volume → 发现 vault（扫描 `.obsidian/` 或 `MEMORY.md`）→ 创建 symlink → 验证链接可用。

**Windows 关键细节：**
- 目录用 Junction（不需要管理员权限），文件用 Symlink（需要管理员或开发者模式）
- 跨盘符（如 C: 和 E:）不可用硬链接
- 所有条目必须平铺在 memory 根目录，不能嵌套——Claude Code 不扫描子目录

#### obsidian-remember 技能

> 技能路径：`~/.claude/skills/obsidian-remember/SKILL.md`

管理 Obsidian 知识仓库的写入、更新、去重、合并和拆分。依赖 `obsidian-link`（写入前先验证 symlink 完好）。

**三层架构：**

| 层 | 路径 | 内容 | 权限 |
|----|------|------|------|
| 导航索引 | `MEMORY.md` | 每条一行摘要 + 指针，< 200 行 | 仅追加指针 |
| 结论摘要 | `wiki/` | 成功经验、失败教训、方法论原则 | 可读写 |
| 原始资料 | `sources/` | 完整过程、调试故事、代码原型 | 可读写，禁止删除 |

**MEMORY.md 入门门槛**（不是所有知识都需要索引）：
1. 跨会话适用（用户偏好、习惯、方法论）
2. 跨领域影响（影响多个工作领域的教训）
3. 高频引用（被频繁引用的约束或方法）

**写入判断流程：**
```
新内容 X
  ├─ 同时包含过程和结论？→ 过程 → sources/，结论 → wiki/
  ├─ 只有过程？          → sources/{领域}.md
  ├─ 只有结论？          → wiki/{领域}.md
  └─ 跨会话适用？        → MEMORY.md 加指针
```

**领域文档管理：**
- 按问题领域组织（如 `dev-practice.md`、`agent-arch-p1.md`）
- 超 600 行必须拆分为 P1/P2
- 90 天未更新且无引用 → 标记为待删除

#### 当前状态

Obsidian 仓库（`E:/Files/Obsidian`）作为直接工作目录，memory 目录中是普通文件而非 symlink——无需桥接。`obsidian-link` 技能用于新项目或 Docker 环境中连接 Obsidian 知识库。