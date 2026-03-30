# .claude/rules/ 完整指南

> 官方文档：https://code.claude.com/docs/en/memory
> 实践参考：https://joseparreogarcia.substack.com/p/how-claude-code-rules-actually-work

---

## 是什么

`.claude/rules/` 是模块化规则目录。把 CLAUDE.md 中过长的内容拆成独立 Markdown 文件，按主题组织，可按文件路径条件加载。

**解决的核心问题：** 项目 CLAUDE.md 膨胀到 1000 行后，从指南变成负担。Rules 让你拆分并按需加载。

---

## 两种规则

### 无条件规则（Unconditional）

没有 `paths:` frontmatter 的文件，**启动时全部加载**。

```markdown
# .claude/rules/security.md
- 提交前检查无硬编码密钥
- 所有用户输入必须验证
```

### Scoped 规则（路径条件）

带 `paths:` frontmatter 的文件，**只在 Claude 处理匹配文件时加载**。

```markdown
---
paths:
  - "notebooks/**"
---
# Notebook 规则
- Notebook 是探索性的，不是生产代码
- 代码稳定后迁移到 src/ 并加测试
```

---

## 加载与触发机制

### 无条件规则

| 特性 | 行为 |
|------|------|
| 加载时机 | 会话启动时，与 `.claude/CLAUDE.md` 同优先级 |
| 触发条件 | 无，始终在上下文中 |
| Compaction 后 | 与 CLAUDE.md 一样，从磁盘重新加载 |

### Scoped 规则

| 特性 | 行为 |
|------|------|
| 加载时机 | Claude 读取匹配路径的文件时 |
| 触发条件 | 文件路径匹配 glob 模式 |
| 关键细节 | 触发于 **读取文件**（Read），不是每个工具调用 |

> 官方原话："Path-scoped rules trigger when Claude reads files matching the pattern, not on every tool use."

### 加载优先级

```
用户级 ~/.claude/rules/    → 先加载
项目级 .claude/rules/       → 后加载（优先级更高）
```

---

## Glob 模式参考

| 模式 | 匹配 |
|------|------|
| `**/*.ts` | 任何目录下的 TypeScript 文件 |
| `src/**/*` | src/ 下所有文件 |
| `*.md` | 项目根目录的 Markdown 文件 |
| `src/components/*.tsx` | 指定目录的 React 组件 |
| `src/**/*.{ts,tsx}` | src/ 下所有 TS/TSX 文件（花括号展开） |
| `tests/**/*.test.ts` | 测试文件 |

支持多模式组合：

```yaml
paths:
  - "src/models/churn/**"
  - "src/pipelines/**"
  - "tests/**/*.test.ts"
```

---

## 目录结构

```
your-project/
├── CLAUDE.md                  # 主项目指令（入口）
├── .claude/
│   ├── CLAUDE.md              # 等同于 ./CLAUDE.md
│   └── rules/
│       ├── python-style.md    # 无条件：始终加载
│       ├── testing.md         # 无条件：始终加载
│       ├── security.md        # 无条件：始终加载
│       ├── notebooks.md       # Scoped：notebooks/ 时加载
│       ├── production.md      # Scoped：生产代码时加载
│       └── ml/                # 支持子目录组织
│           ├── training.md
│           └── pipelines.md
```

所有 `.md` 文件递归发现，可以用子目录组织。

---

## 最佳使用方法

### 无条件规则：放什么

只放**不管在做什么都适用的规则**：

| 适合 | 不适合 |
|------|--------|
| 基础编码标准（格式、类型） | 特定目录的工作流 |
| 安全硬要求（密钥检查、输入验证） | ML 建模选择 |
| 测试覆盖率要求 | 实验性实践 |
| 项目假设（"批处理优先于实时"） | 逐步骤流程 |

**反模式：** 把所有规则都写成无条件的。这和写一个 1000 行的 CLAUDE.md 没区别，等于把问题从 CLAUDE.md 搬到了 rules/。

### Scoped 规则：放什么

放**只在特定文件/目录下才有意义的规则**：

| 场景 | paths | 内容 |
|------|-------|------|
| Notebook 开发 | `notebooks/**` | 探索性、原型、松约束 |
| 生产 ML 代码 | `src/models/**`, `src/pipelines/**` | 严格测试、可重现、无数据泄露 |
| API 端点 | `src/api/**/*.ts` | 输入验证、错误格式、OpenAPI 注释 |
| 前端组件 | `src/components/*.tsx` | 设计系统、无障碍要求 |
| 测试文件 | `tests/**/*.test.ts` | 测试约定、mock 规则 |

### CLAUDE.md + Rules 配合（核心策略）

Jose 的精准总结：

> **Scoped rules 防止局部错误（local mistakes）。**
> **CLAUDE.md 防止全局误解（global misunderstandings）。**

两者结合：

```
CLAUDE.md（期望层）
├── 告诉 Claude "这个仓库有几种不同的工作"
├── 列出场景和对应规则文件位置
├── 影响 Claude 的规划步骤
│
.claude/rules/（执行层）
├── 无条件规则 → 全局约束，始终在场
└── Scoped 规则 → 触及匹配文件时自动加载
```

在 CLAUDE.md 中写索引引导：

```markdown
## 如何在此仓库工作

### 实验性工作
- 查阅 .claude/rules/experiments.md
- 约束较松，快速迭代

### 生产代码修改
- src/models/ 和 src/pipelines/ 下有 scoped rules
- 可重现性、测试、评估是强制的
```

### 符号链接共享规则

```bash
# 共享整个规则目录
ln -s ~/shared-claude-rules .claude/rules/shared

# 共享单个文件
ln -s ~/company-standards/security.md .claude/rules/security.md
```

Claude 会解析符号链接并正常加载。循环链接会被检测和优雅处理。

### 用户级规则

个人规则放在 `~/.claude/rules/`，跨所有项目生效：

```
~/.claude/rules/
├── preferences.md    # 个人编码偏好
└── workflows.md      # 个人工作流
```

---

## 什么情况不该用 Rules

| 情况 | 原因 | 替代方案 |
|------|------|---------|
| 项目很小，规则不到 50 行 | 直接写 CLAUDE.md 更简单 | 不拆分 |
| 需要真正强制执行的约束 | Rules 是上下文不是强制 | 用 managed settings |
| 规则只对当前对话有用 | 不需要跨会话持久化 | 直接在对话中说 |
| 外部参考文档全文 | 浪费上下文空间 | 用 `@path` 导入或 MEMORY 索引 |

---

## 排错

| 问题 | 原因 | 解决 |
|------|------|------|
| Scoped 规则没触发 | Claude 没有读取匹配文件 | 在 CLAUDE.md 中加引导指向 |
| 规则被忽略 | 指令太模糊 | 写具体可验证的指令 |
| 多个规则冲突 | 不同文件对同一行为给出不同指导 | 合并或明确覆盖关系 |
| 不知道加载了什么 | 无法确认 | 运行 `/memory` 查看所有已加载文件 |

### 验证方法

永远不要假设 Claude 加载了什么。直接问：

> "告诉我你当前加载了哪些 CLAUDE.md 和 rules 文件"

或运行 `/memory` 查看完整列表。
