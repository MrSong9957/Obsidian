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

### 全部技能简述

| 技能 | 触发场景 | 简述 |
|------|----------|------|
| `using-superpowers` | 每次对话开始 | 建立"先调用技能"规则，任何响应（含澄清提问）前先加载相关技能 |
| `brainstorming` | 任何创造性工作前 | 写代码前通过提问细化需求、探索替代方案、分段展示设计供验证 |
| `using-git-worktrees` | 需要隔离工作空间 | 确保隔离工作空间存在（原生工具或 git worktree 兜底） |
| `writing-plans` | 有规格/需求、动代码前 | 把多步任务拆成可执行的小任务计划 |
| `writing-skills` | 创建/编辑/验证技能时 | 用 TDD 思路写技能文档（压力场景→基线→写技能→验证→重构） |
| `executing-plans` | 跨会话执行计划时 | 在独立会话执行写好的实现计划，带审查检查点 |
| `subagent-driven-development` | 当前会话执行独立任务 | 把实现计划中的独立任务分派子代理执行 |
| `dispatching-parallel-agents` | 有 2+ 个无依赖独立任务 | 并行分派互不依赖的独立任务 |
| `test-driven-development` | 实现任何功能或修复前 | 严格 RED-GREEN-REFACTOR，先写失败测试再写最小实现 |
| `systematic-debugging` | 遇到 bug/测试失败/异常行为 | 提出修复前必须先找根因，禁止症状式打补丁 |
| `requesting-code-review` | 完成任务/重大功能/合并前 | 审查工作是否满足需求 |
| `receiving-code-review` | 收到代码审查反馈时 | 实施建议前需技术严谨核实，拒绝表演式同意或盲目照做 |
| `verification-before-completion` | 即将声称完成/修复/通过前 | 必须运行验证命令并确认输出，先有证据再下结论 |
| `finishing-a-development-branch` | 实现完成且测试全通过 | 呈现结构化选项（merge/PR/保留/丢弃），清理 worktree |


---

### 设计原则

- 系统化优于临时应对：流程优于猜测
- 复杂度缩减：简洁是首要目标

### 铁律

1. **TDD 铁律**：没有失败测试，不写生产代码
2. **调试铁律**：没有根因调查，不实施修复
3. **验证铁律**：没有运行证据，不声称完成

### 子代理派发规则

派发子代理时，父代理必须先调用 Skill 工具加载技能完整内容，再用 `---SKILL_NAME START/END---` 标记注入子代理 prompt（子代理无法调用 Skill 工具）：
- 实现任务 → 加载 test-driven-development
- 代码质量审查 → 加载 requesting-code-review
- 完成分支 → 加载 finishing-a-development-branch

---

## 项目信息

### 技术栈

[项目技术栈描述]

---

### 架构

[项目架构描述]

---

### 项目

[项目描述]
```

---
