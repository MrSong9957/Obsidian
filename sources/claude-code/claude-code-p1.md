---
name: Claude Code 配置（完整参考）
description: settings.json/CLAUDE.md/rules/记忆系统/技能位置的完整参考文档，含代码示例、安装步骤和排错指南
type: concept
updated: 2026-04-17
tags: claude-code, settings, claudemd, memory, rules
source: https://example.com
---
# Claude Code 配置（完整参考）

> 本文档为 wiki/claude-code-p1.md 的详细版本，包含完整代码示例、安装步骤和排错指南。

---

## CLAUDE.md 完整指南

> 官方文档：https://code.claude.com/docs/en/memory
> 实践参考：https://joseparreogarcia.substack.com/p/how-claude-code-rules-actually-work

### 是什么

CLAUDE.md 是你写给 Claude 的持久化指令文件。Claude 在每次会话开始时读取，作为行为指南。

**核心认知：CLAUDE.md 是上下文（context），不是强制执行（enforcement）。** Claude 会尽力遵守，但不保证 100% 合规，特别是指令模糊或互相矛盾时。

### 三级作用域

| 作用域 | 位置 | 优先级 | 用途 |
|--------|------|--------|------|
| 组织级 | Windows: `C:\Program Files\ClaudeCode\CLAUDE.md` | 最高 | 公司级编码标准、安全政策 |
| 用户级 | `~/.claude/CLAUDE.md` | 中 | 个人偏好，跨项目生效 |
| 项目级 | `./CLAUDE.md` 或 `./.claude/CLAUDE.md` | 低 | 项目架构、构建命令、团队约定 |

**优先级规则：更具体的位置覆盖更宽泛的。** 项目级 > 用户级 > 组织级。

### 加载机制

#### 何时加载

**每次会话开始时完整加载。** 与 MEMORY.md 的 200 行限制不同，CLAUDE.md 没有截断限制（全文加载）。

#### 加载顺序

```
1. 从工作目录向上遍历目录树，加载每层的 CLAUDE.md
2. 子目录的 CLAUDE.md 在 Claude 读取该子目录文件时按需加载
3. 用户级 ~/.claude/CLAUDE.md 始终加载
4. 组织级 CLAUDE.md 始终加载，且无法被排除
```

#### Compaction 后的行为（关键）

**CLAUDE.md 完整存活于 compaction。** 执行 `/compact` 后，Claude 会从磁盘重新读取 CLAUDE.md 并重新注入上下文。

> 官方原话："CLAUDE.md fully survives compaction. After /compact, Claude re-reads your CLAUDE.md from disk and re-injects it fresh."

这意味着：如果你发现 compaction 后某条指令消失了，说明那条指令只存在于对话中，没有写入 CLAUDE.md。

### 导入机制

用 `@path` 语法在 CLAUDE.md 中引用其他文件：

```markdown
# 项目说明
见 @README.md 了解项目概述
见 @package.json 了解可用命令

# 个人偏好（不提交到 git）
@~/.claude/my-project-instructions.md
```

- 支持相对路径和绝对路径
- 相对路径基于包含导入的文件所在目录
- 最多递归 5 层导入
- AGENTS.md 兼容：`@AGENTS.md` 然后追加 Claude 专属指令

### 最佳使用方法

#### 写什么

CLAUDE.md 的职责是**防止全局误解**（prevent global misunderstandings）：

| 适合写 | 不适合写 |
|--------|---------|
| 编码规范（缩进、类型要求） | 逐步骤工作流 |
| 构建和测试命令 | 只在特定目录才适用的规则 |
| 架构决策和项目结构 | 实验性或经常变化的实践 |
| 命名约定 | 外部文档的完整内容 |
| 常用工作流（简短描述） | 超过 200 行的详细文档 |

#### 写作原则

**1. 精简：每个文件不超过 200 行**

长文件消耗上下文空间，降低遵守率。超长内容用 `@path` 导入或拆分到 `.claude/rules/`。

**2. 具体：可验证的指令**

```markdown
# 差 — 模糊
格式化代码

# 好 — 具体可验证
使用 2 空格缩进
```

```markdown
# 差
测试你的更改

# 好
提交前运行 npm test
```

**3. 结构化：用 Markdown 标题和列表**

Claude 扫描结构的方式和人类一样：有组织的分节比密集段落更容易遵守。

**4. 无矛盾：定期审查**

如果两个文件对同一行为给出不同指导，Claude 可能随机选一个。用 `/memory` 命令查看当前加载了哪些文件，检查冲突。

**5. 注释给人类看**

HTML 注释 `<!-- 给人看的备注 -->` 会被 Claude 过滤，不消耗上下文：

```markdown
<!-- 下次重构时记得更新这个部分 -->
- API 端点必须包含输入验证
```

#### CLAUDE.md 作为索引

一个高阶用法：在项目 CLAUDE.md 中写"路线图"，引导 Claude 在不同场景下查阅不同规则：

```markdown
## 如何在此仓库工作

### 实验性工作
- 查阅 `.claude/rules/experiments.md`
- 约束较松，快速迭代

### 生产代码修改
- `src/models/` 和 `src/pipelines/` 下有严格的 scoped rules
- 可重现性、测试、评估是强制的
```

这不是强制执行，但会**影响 Claude 的规划步骤**，让它更可能做"正确的事"。

### 大型项目技巧

#### monorepo 排除不相关文件

```json
// .claude/settings.local.json
{
  "claudeMdExcludes": [
    "**/other-team/CLAUDE.md",
    "/home/user/monorepo/legacy/.claude/rules/**"
  ]
}
```

#### 附加目录加载 CLAUDE.md

默认 `--add-dir` 的目录不加载 CLAUDE.md，加环境变量启用：

```bash
CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1 claude --add-dir ../shared-config
```

#### 子目录 CLAUDE.md

子目录中的 CLAUDE.md 不会在启动时加载，只有当 Claude 读取该子目录的文件时才加载。适合模块级指令：

```
project/
├── CLAUDE.md              # 项目级（启动时加载）
├── src/
│   ├── billing/
│   │   └── CLAUDE.md      # 只在编辑 billing/ 时加载
│   └── auth/
│       └── CLAUDE.md      # 只在编辑 auth/ 时加载
```

### 排错

| 问题 | 检查方法 |
|------|---------|
| Claude 没遵守指令 | 运行 `/memory` 确认文件已加载 |
| 指令 compaction 后消失 | 该指令只存在于对话中，未写入 CLAUDE.md |
| 两个指令互相冲突 | 检查全局 + 项目 + 子目录的所有 CLAUDE.md |
| 指令太长被忽略 | 缩短到 200 行以内，拆分到 rules/ |

---

## .claude/rules/ 完整指南

> 官方文档：https://code.claude.com/docs/en/memory
> 实践参考：https://joseparreogarcia.substack.com/p/how-claude-code-rules-actually-work

### 是什么

`.claude/rules/` 是模块化规则目录。把 CLAUDE.md 中过长的内容拆成独立 Markdown 文件，按主题组织，可按文件路径条件加载。

**解决的核心问题：** 项目 CLAUDE.md 膨胀到 1000 行后，从指南变成负担。Rules 让你拆分并按需加载。

### 两种规则

#### 无条件规则（Unconditional）

没有 `paths:` frontmatter 的文件，**启动时全部加载**。

```markdown
# .claude/rules/security.md
- 提交前检查无硬编码密钥
- 所有用户输入必须验证
```

#### Scoped 规则（路径条件）

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

### 加载与触发机制

#### 无条件规则

| 特性 | 行为 |
|------|------|
| 加载时机 | 会话启动时，与 `.claude/CLAUDE.md` 同优先级 |
| 触发条件 | 无，始终在上下文中 |
| Compaction 后 | 与 CLAUDE.md 一样，从磁盘重新加载 |

#### Scoped 规则

| 特性 | 行为 |
|------|------|
| 加载时机 | Claude 读取匹配路径的文件时 |
| 触发条件 | 文件路径匹配 glob 模式 |
| 关键细节 | 触发于 **读取文件**（Read），不是每个工具调用 |

> 官方原话："Path-scoped rules trigger when Claude reads files matching the pattern, not on every tool use."

#### 加载优先级

```
用户级 ~/.claude/rules/    → 先加载
项目级 .claude/rules/       → 后加载（优先级更高）
```

### Glob 模式参考

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

### 目录结构

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

### 最佳使用方法

#### 无条件规则：放什么

只放**不管在做什么都适用的规则**：

| 适合 | 不适合 |
|------|--------|
| 基础编码标准（格式、类型） | 特定目录的工作流 |
| 安全硬要求（密钥检查、输入验证） | ML 建模选择 |
| 测试覆盖率要求 | 实验性实践 |
| 项目假设（"批处理优先于实时"） | 逐步骤流程 |

**反模式：** 把所有规则都写成无条件的。这和写一个 1000 行的 CLAUDE.md 没区别，等于把问题从 CLAUDE.md 搬到了 rules/。

#### Scoped 规则：放什么

放**只在特定文件/目录下才有意义的规则**：

| 场景 | paths | 内容 |
|------|-------|------|
| Notebook 开发 | `notebooks/**` | 探索性、原型、松约束 |
| 生产 ML 代码 | `src/models/**`, `src/pipelines/**` | 严格测试、可重现、无数据泄露 |
| API 端点 | `src/api/**/*.ts` | 输入验证、错误格式、OpenAPI 注释 |
| 前端组件 | `src/components/*.tsx` | 设计系统、无障碍要求 |
| 测试文件 | `tests/**/*.test.ts` | 测试约定、mock 规则 |

#### CLAUDE.md + Rules 配合（核心策略）

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

#### 符号链接共享规则

```bash
# 共享整个规则目录
ln -s ~/shared-claude-rules .claude/rules/shared

# 共享单个文件
ln -s ~/company-standards/security.md .claude/rules/security.md
```

Claude 会解析符号链接并正常加载。循环链接会被检测和优雅处理。

#### 用户级规则

个人规则放在 `~/.claude/rules/`，跨所有项目生效：

```
~/.claude/rules/
├── preferences.md    # 个人编码偏好
└── workflows.md      # 个人工作流
```

### 什么情况不该用 Rules

| 情况 | 原因 | 替代方案 |
|------|------|---------|
| 项目很小，规则不到 50 行 | 直接写 CLAUDE.md 更简单 | 不拆分 |
| 需要真正强制执行的约束 | Rules 是上下文不是强制 | 用 managed settings |
| 规则只对当前对话有用 | 不需要跨会话持久化 | 直接在对话中说 |
| 外部参考文档全文 | 浪费上下文空间 | 用 `@path` 导入或 MEMORY 索引 |

### 排错

| 问题 | 原因 | 解决 |
|------|------|------|
| Scoped 规则没触发 | Claude 没有读取匹配文件 | 在 CLAUDE.md 中加引导指向 |
| 规则被忽略 | 指令太模糊 | 写具体可验证的指令 |
| 多个规则冲突 | 不同文件对同一行为给出不同指导 | 合并或明确覆盖关系 |
| 不知道加载了什么 | 无法确认 | 运行 `/memory` 查看所有已加载文件 |

#### 验证方法

永远不要假设 Claude 加载了什么。直接问：

> "告诉我你当前加载了哪些 CLAUDE.md 和 rules 文件"

或运行 `/memory` 查看完整列表。

---

## Claude Code Auto Memory 完整指南

> 基于官方文档整理：https://code.claude.com/docs/en/memory

### 核心定位

Auto Memory 是 Claude Code 的两大持久化系统之一：

| | CLAUDE.md | Auto Memory |
|---|---|---|
| **谁写的** | 你（人工） | Claude（自动） |
| **内容** | 指令和规则 | 学习和经验 |
| **作用域** | 全局 / 项目 / 组织 | 每个 git 仓库 |
| **加载方式** | 每次会话完整加载 | 每次会话加载前 200 行或 25KB |
| **用途** | 编码规范、工作流、架构 | 构建命令、调试经验、你发现的偏好 |

**简单理解：CLAUDE.md 是你给 Claude 的规章制度，MEMORY.md 是 Claude 给自己记的笔记本。**

### 存储位置

```
~/.claude/projects/<project>/memory/
├── MEMORY.md            # 索引入口，每次会话加载
├── debugging.md         # 主题文件，按需加载
├── api-conventions.md   # 主题文件，按需加载
└── ...                  # Claude 创建的其他主题文件
```

- `<project>` 路径由 git 仓库根目录推导
- 同一仓库的所有 worktree 和子目录共享一个 memory 目录
- 非 git 项目使用项目根目录的绝对路径
- Auto Memory 是**机器本地**的，不会跨设备同步

#### 自定义存储位置

在 user 或 local settings 中设置：

```json
{
  "autoMemoryDirectory": "~/my-custom-memory-dir"
}
```

> 注意：此设置**不能**在项目 settings（`.claude/settings.json`）中配置，防止共享项目重定向写入路径。

### 加载规则

#### MEMORY.md（索引文件）

| 规则 | 说明 |
|------|------|
| **加载时机** | 每次会话开始时 |
| **加载限制** | 前 **200 行** 或 **25KB**（以先达到者为准） |
| **超限处理** | 超出部分**不会**被加载 |
| **加载频率** | 仅会话开始一次，不会每轮重新加载 |
| **Compaction 后** | 不会重新从磁盘读取（与 CLAUDE.md 不同） |

#### 主题文件（debugging.md 等）

| 规则 | 说明 |
|------|------|
| **加载时机** | **不**在启动时加载 |
| **读取方式** | Claude 使用 Read 工具按需读取 |
| **触发条件** | Claude 判断对话需要时主动读取 |

### Compaction 行为对比

这是两个系统最关键的区别：

| 场景 | CLAUDE.md | MEMORY.md |
|------|-----------|-----------|
| **会话开始** | 完整加载 | 前 200 行/25KB |
| **对话过长后** | 上下文压缩，但**会从磁盘重新读取并注入** | 上下文压缩，**不会重新注入** |
| **`/compact` 后** | 从磁盘重新加载，完整恢复 | 已压缩的内容丢失 |

**结论：CLAUDE.md 的持久性强于 MEMORY.md。** 如果你有一条规则必须在长对话中始终生效，放在 CLAUDE.md 里更可靠。

### 自动保存机制

#### 什么时候 Claude 会保存

Claude **不是**每次会话都保存。它根据"这条信息在未来的对话中是否有用"自主判断。

典型触发场景：
- 你说"记住这个"或"以后都这样做"
- 你纠正了 Claude 的做法（"别这样做，要那样做"）
- Claude 了解到你的角色或偏好

#### 保存格式

纯 Markdown 文件，你可以随时编辑或删除。

#### 管理方式

- 会话中运行 `/memory` 浏览和管理
- 直接编辑 `~/.claude/projects/<project>/memory/` 下的文件
- MEMORY.md 充当目录索引，Claude 用它追踪"什么存在哪里"

### 开关控制

Auto Memory **默认开启**（需要 Claude Code v2.1.59+）。

三种关闭方式：

```json
// 方式 1：项目 settings
{ "autoMemoryEnabled": false }

// 方式 2：环境变量
CLAUDE_CODE_DISABLE_AUTO_MEMORY=1

// 方式 3：会话中运行 /memory，切换开关
```

### MEMORY.md 索引设计要点

#### 限制

索引超过 200 行或 25KB 后，超出部分**永远不会被加载**。这意味着 Claude 在新会话中根本不知道那些条目存在。

#### 最佳实践

1. **索引保持精简** — 每条一行，控制在 200 行以内
2. **详细信息放主题文件** — Claude 会按需用 Read 工具读取
3. **索引格式**：`- [标题](文件名) — 一句话摘要`
4. **定期清理** — 过时的记忆直接删除

#### 不该存什么

以下内容不需要存入 Memory（因为可以通过其他方式获取）：

| 不要存 | 原因 |
|--------|------|
| 代码模式/架构 | 直接读代码就知道 |
| Git 历史 | `git log` 是权威来源 |
| 调试方案 | 修复已经在代码和 commit 里了 |
| 已在 CLAUDE.md 中的规则 | 重复浪费上下文空间 |
| 通用编程知识 | Claude 本身就知道 |

#### 适合存什么

| 适合存 | 原因 |
|--------|------|
| 你纠正过 Claude 的事 | 防止下次犯同样的错 |
| 你的沟通偏好 | "中文"、"原理优先"、"不用 emoji" |
| 项目中不明显的关键上下文 | "这个 API key 下周过期"、"移动团队在用这个分支" |
| 外部资源位置 | "pipeline bug 在 Linear INGEST 项目" |

### 持久化优先级总结

从"最可靠"到"最不可靠"排序：

```
1. Hooks（UserPromptSubmit）     → 每轮都执行，最可靠
2. CLAUDE.md                     → compaction 后会重新加载
3. .claude/rules/（无条件）       → 同 CLAUDE.md
4. .claude/rules/（paths: 限定） → 按需加载，只处理匹配文件时触发
5. MEMORY.md（前 200 行）        → 仅会话开始加载，不重载
6. MEMORY.md 主题文件            → Claude 按需读取
7. 对话中的口头约定               → compaction 后可能丢失
```

### 实用命令

| 命令 | 作用 |
|------|------|
| `/memory` | 查看所有已加载的 CLAUDE.md、rules 和 auto memory |
| "记住这个" | 让 Claude 保存到 auto memory |
| "加到 CLAUDE.md" | 让 Claude 写入 CLAUDE.md 而非 auto memory |

### 官方文档

- 完整文档：https://code.claude.com/docs/en/memory
- Hooks 文档：https://code.claude.com/docs/en/hooks

---

## Claude Code 记忆系统：Active Recall 机制

### 四阶段筛选

每轮对话依次执行：**扫描 → 摘要 → 选择 → 注入**

1. **扫描阶段**：最多取 200 个记忆文件，仅读前 30 行（覆盖 frontmatter 元数据，成本极低）
2. **摘要阶段**：将元数据整合成记忆清单
3. **选择阶段**：用 Sonnet 模型选最多 **5 条**相关记忆，宁缺毋滥；同时过滤已注入内容、校验文件真实性
4. **注入阶段**：读取选中文件内容，单文件超 4000 字节会截断并附提示

### 容量管控规则

| 维度 | 上限 | 说明 |
|------|------|------|
| 单轮注入 | 5 条共 20,000 字节 | 防单轮记忆占比过高 |
| 会话累计 | 60,000 字节 | 防上下文被记忆撑爆 |
| 压缩后 | 计数器重置 | 可重新注入已用过的记忆 |

### 去重机制

同一会话中已注入的记忆文件会提前从候选集中过滤，避免浪费模型选择名额。

### 过期提示

注入超过 **2 天**的记忆时，自动附加提醒：记忆为时间快照，可能过时，决策前需验证当前状态。
设计动机：用户反馈旧记忆过时导致错误决策。

### 架构哲学

**「元数据粗筛 + 便宜模型精选」**两阶段架构，将记忆调用成本降低一个数量级，无需向量数据库或复杂相似度计算。

### 实践启示

该机制为 Agent 记忆系统的解锁环节提供了可直接复用的落地方案：
- 大量记忆时用 frontmatter 做低成本初筛
- 便宜模型做精选，贵的模型做决策
- 硬性上限防止单一维度膨胀
