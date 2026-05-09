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

Python 3.13+ asyncio | anthropic SDK | prompt-toolkit + Rich | pydantic 2 | diff-match-patch | sqlite3 | click | pytest
不用：agent 框架、Textual、手写 ANSI

---

### 架构

单文件原型 `CLI/Python/agent_loop.py`，数据流：
`main() → REPL → run_agent_loop() → client.messages.create() → 解析 stop_reason → 工具调用循环`

---

### 文件约定

- **设计文档**：`docs/superpowers/specs/YYYY-MM-DD--<name>.md`
- **任务计划**：`docs/superpowers/plans/YYYY-MM-DD-<name>.md`
- **个人技能**：`~/.claude/skills/<name>/SKILL.md`
- **Worktree 目录**：`.worktrees/`（项目本地隐藏，必须 git check-ignore 验证）

---

### 项目

**mi-code** — Python AI 终端编程助手，对标 Claude Code 的全功能开源实现。
- 不用 agent 框架，从底层构建全部能力
- API 兼容层：Anthropic SDK 0.94.0 → Zhipu GLM
