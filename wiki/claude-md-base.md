---
name: CLAUDE.md 双层架构模板
description: 全局（karpathy 行为准则）+ 项目级（superpowers 工作流）双层 CLAUDE.md 创建方法
type: reference
updated: 2026-05-09
tags: [claude-md, template, karpathy, superpowers, kv-cache, two-tier]
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

#### 1. 头脑风暴（Brainstorming）
写代码之前，先通过提问细化需求，探索替代方案，分段展示设计供验证。

#### 2. Git Worktree
设计批准后，创建隔离工作空间（新分支），验证测试基线干净。

#### 3. 编写计划（Writing Plans）
将工作拆分为 2-5 分钟的小任务。每个任务包含：精确文件路径、完整代码、验证步骤。

#### 4. 子代理驱动开发（Subagent-Driven Development）
每个任务分派独立子代理执行，双阶段审查（规格合规 + 代码质量）。

#### 5. 测试驱动开发（TDD）
严格 RED-GREEN-REFACTOR：先写失败测试 → 看它失败 → 写最小代码 → 看它通过 → 提交。
测试之前写的代码应删除重写。

#### 6. 代码审查（Code Review）
任务间进行审查，按严重性报告问题。CRITICAL 级别阻止推进。

#### 7. 完成分支（Finishing Branch）
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
| systematic-debugging | 遇到 bug、测试失败、意外行为时，**先观察再修复** |

#### 收尾阶段
| 技能 | 触发时机 |
|------|---------|
| finishing-a-development-branch | 实现完成、测试通过后，决定 merge/PR/保留/丢弃 |

#### 其他技能
| 技能 | 触发时机 |
|------|---------|
| writing-skills | 创建/编辑技能，或验证技能可用性 |

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