---
name: Agent 架构（下）- 详细资料
description: Harness工程方法论/人话编程规范的完整详细内容
type: concept
updated: 2026-04-17
tags: agent, harness, natural-language, methodology
source: https://example.com
---
# Agent 架构（下）- 详细资料

> 入口文档：[[wiki/agent-arch.md|Agent 架构核心]] | 索引：[[sources/agent/agent-arch-references.md|Agent 架构详细参考]]

---

## Harness Engineering: AI Agent-First 软件开发 - 详细内容

> 原文: [Harness engineering: leveraging Codex in an agent-first world](https://openai.com/index/harness-engineering/)
> 作者: OpenAI 团队
> 时间: 2025年8月 - 2026年1月

### 一、原文核心内容（中文摘要）

#### 实验背景

OpenAI 团队进行了一项为期五个月的实验：**用 0 行人工编写代码构建和发布一个软件产品**。

**实验结果：**
- 产品有内部日常用户和外部 alpha 测试者
- 正常上线、部署、出错、修复
- 每一行代码（应用逻辑、测试、CI 配置、文档、可观测性、内部工具）都由 Codex 编写
- **构建时间约为手写的 1/10**
- 3 名工程师驱动 Codex，约 1,500 个 PR 被合并
- 平均每位工程师每天 3.5 个 PR

**核心理念：**
> **人类驾驶，Agent 执行。** (Humans steer. Agents execute.)

#### 工作方式的变化

**不再写代码，而是：**
1. **设计环境** — 为 Agent 提供必要的工具、抽象和内部结构
2. **指定意图** — 用 prompt 描述任务
3. **构建反馈循环** — 让 Agent 能可靠地工作

**工作流程：**
- 工程师通过 prompt 与系统交互
- Agent 运行后打开 PR
- Agent 自我审查、响应反馈、迭代
- 人类可能审查 PR，但不强制要求
- 逐步将审查工作推向 agent-to-agent

**长时间运行的 Agent：**
- 单个 Codex 运行可以处理一个任务长达 6 小时（通常在人类睡觉时进行）

#### 上下文管理的关键教训

> **给 Codex 一张地图，而不是一本 1000 页的说明书。**

**问题：**
- 巨大的指令文件会挤占任务、代码和相关文档的空间
- 太多指导变成"无指导"——当一切都是"重要"的，就没有什么是重要的
- 单一大文件难以验证、会腐烂

**解决方案：**
- `AGENTS.md` 约 100 行，作为**目录**而非百科全书
- 仓库知识库存储在结构化的 `docs/` 目录
- 设计文档、架构文档、质量文档都有索引
- 计划作为一等工件
- 使用 linter 和 CI 验证文档是最新的

#### 架构约束

**边界严格，实现自由：**
- 每个业务域划分为固定层级：Types → Config → Repo → Service → Runtime → UI
- 依赖方向严格验证
- 跨域关注点（auth、connectors、telemetry）通过单一接口进入
- 使用自定义 linter 强制执行

**这种架构通常在数百名工程师时才需要，但在 coding agent 环境下，它是早期前提条件。**

#### 人类品味的编码

**黄金原则（Golden Principles）：**
- 优先使用共享工具包而非手写辅助函数
- 不"YOLO 风格"探测数据——验证边界或依赖类型化 SDK
- 定期运行后台 Codex 任务扫描偏差、更新质量等级、打开重构 PR

**像垃圾回收一样处理技术债务：**
- 持续小额偿还，而不是让它复利增长
- 人类品味被捕获一次，然后持续强制执行

#### Agent 的完整能力

**单个 prompt 可以驱动 Codex 完成：**
1. 验证代码库当前状态
2. 复现报告的 bug
3. 录制演示失败的视频
4. 实现修复
5. 通过驱动应用验证修复
6. 录制第二段演示解决的视频
7. 打开 PR
8. 响应 agent 和人类反馈
9. 检测并修复构建失败
10. 仅在需要判断时升级到人类
11. 合并变更

---

### 二、结构化解读

#### 1. 核心概念：什么是 Harness Engineering？

**Harness Engineering（驾驭工程）** 是一门新学科：
> 设计环境、约束和反馈循环，使 AI coding agent 能够可靠地完成软件开发任务。

**与传统软件工程的区别：**

| 传统软件工程 | Harness Engineering |
|------------|-------------------|
| 编写代码 | 设计环境和约束 |
| 直接实现 | 指定意图 |
| 人工审查 | Agent 自我审查 |
| 技术债务积累 | 持续"垃圾回收" |
| 文档是人看的 | 文档是 Agent 看的 |

#### 2. 三大核心能力

**A. 环境设计（Environment Design）**
- 让 Agent 能启动独立的应用实例（每个 git worktree 一个）
- 暴露日志、指标、追踪给 Agent
- 通过 Chrome DevTools Protocol 让 Agent 能操作 UI

##### 2.1 环境设计详解

###### A. 应用实例隔离

**问题：** 多个 Agent 并行工作，会互相干扰

| 冲突类型 | 示例 |
|---------|------|
| 文件冲突 | 两个 Agent 同时修改同一文件 |
| 端口冲突 | 两个实例都监听 3000 端口 |
| 数据冲突 | 共享数据库，测试数据互相污染 |
| 日志混乱 | 无法区分哪个 Agent 的日志 |

**解决方案：** 每个 Agent 任务一个独立沙盒

| 层面 | 隔离方式 |
|-----|---------|
| 代码 | git worktree（独立目录） |
| 端口 | 动态分配（3001, 3002...） |
| 数据库 | 每个 worktree 独立 db 文件 |
| 日志 | 每个 worktree 独立日志目录 |

###### B. 可观测性暴露

**问题：** 原始日志太乱，Agent 难以高效分析

**解决方案：** 结构化查询能力

| 查询语言 | 能问的问题 |
|---------|-----------|
| LogQL | "过去 5 分钟有多少 ERROR 日志？" |
| PromQL | "API 响应时间 P99 超过 500ms 了吗？" |

```sql
-- LogQL: 查错误数量
count_over_time({level="error"}[5m])

-- PromQL: 查响应时间
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))
```

**Agent 用例：**
- 应用启动成功了吗？ → 查 `server_started` 指标
- 有数据库连接错误吗？ → 查 `{component="db", level="error"}`
- 响应时间达标吗？ → 查 P99 latency

###### C. UI 驱动能力

**问题：** Agent 如何验证前端界面改动？

**解决方案：** Chrome DevTools Protocol (CDP)

| 能力 | 具体操作 | Agent 用途 |
|-----|---------|-----------|
| DOM 操作 | 获取页面结构、查找元素 | "确认按钮存在且文字正确" |
| 模拟用户 | 点击、输入、滚动 | "点击提交按钮，填表单" |
| 截图 | 捕获页面画面 | "录一个 bug 复现视频" |
| 网络监控 | 拦截请求、查看响应 | "确认 API 调用成功" |

##### 2.2 完整工作流示例

Agent 收到任务："修复登录按钮点击无反应的 bug"

```
1. 启动应用（实例隔离）
2. 打开浏览器，导航到登录页（CDP）
3. 输入用户名、密码（CDP 模拟输入）
4. 点击登录按钮（CDP 模拟点击）
5. 检查是否跳转到首页（CDP 查 DOM）
6. 查询日志确认无错误（LogQL）
7. 截图保存证据（CDP 截图）
```

**三个能力的协作：**

| 能力 | 在此流程中的作用 |
|-----|-----------------|
| 实例隔离 | 独立环境，不影响其他任务 |
| 可观测性暴露 | 验证后端逻辑正确 |
| UI 驱动能力 | 验证前端交互正确 |

#### 2.3 指定意图详解

##### A. 意图指定的本质

**核心转变：从"写代码"到"描述目标"**

| 传统开发 | Agent-First 开发 |
|---------|-----------------|
| 实现细节 | 目标描述 |
| 逐步指令 | 意图 + 边界 |
| 人工编码 | Agent 执行 |

**工程师的新技能：**
- 将模糊需求转化为清晰 prompt
- 定义验收标准而非实现步骤
- 编码约束而非逐一指导

##### B. 六要素 Prompt 结构

| 要素 | 作用 | 示例 |
|-----|------|------|
| Role/Tone | 角色和语气 | "你是一个前端工程师" |
| Task Definition | 任务目标 | "实现暗色主题切换" |
| Rules | 边界约束 | Do/Don't 列表 |
| Context | 相关知识 | 输入数据、定义 |
| Output | 输出格式 | Markdown/JSON |
| Reminder | 关键重复 | 最重要的 2 个约束 |

**Prompt 模板：**

```markdown
# Role / Tone
你是一个 [角色]，专精于 [领域]。

# Task Definition
你的目标：[一句话描述预期结果]
成功意味着：[2-4 条"好"的标准]

# Rules & Guardrails
必须做：[必需行为]
禁止做：[禁止行为]

# Data / Context
输入：[相关数据、约束、示例]

# Output Structure
返回格式：[Markdown / 表格 / JSON]

# Key Reminder
[重复两个最重要的约束]
```

##### C. AGENTS.md 作为意图载体

> **给 Agent 一张地图，而不是 1000 页的说明书。**

**角色定位：**
- 目录（~100 行），不是百科全书
- 指向深度文档的指针
- 黄金原则的编码

**结构示例：**

```
AGENTS.md
├── 项目概述（1-2 句）
├── 架构约束（依赖方向、层级）
├── 黄金原则（品味不变量）
└── 文档索引（指向 docs/）
```

##### D. 好的 vs 不够明确的 Prompt

**好的 prompt：**

```
实现用户登录功能：
- 使用现有的 AuthService
- 支持邮箱 + 密码登录
- 密码错误时显示友好提示
- 登录成功后跳转到 dashboard
- 确保所有新代码都有测试
```

**不够明确的 prompt：**

```
写一个登录页面
```

**对比：**

| 好的 prompt | 不够明确的 prompt |
|------------|-----------------|
| 明确使用什么（AuthService） | 未指定实现方式 |
| 明确边界条件（错误提示） | 边界模糊 |
| 明确验收标准（测试、跳转） | 无验收标准 |

##### E. 意图指定的三个层次

```
┌─────────────────────────────────────────┐
│ Level 3: 业务意图                        │
│ "让新用户能在 30 秒内完成注册"            │
├─────────────────────────────────────────┤
│ Level 2: 功能意图                        │
│ "实现邮箱验证 + 密码强度检查的注册表单"   │
├─────────────────────────────────────────┤
│ Level 1: 实现意图                        │
│ "创建 RegisterForm 组件，包含 email、     │
│  password 字段和验证逻辑"                │
└─────────────────────────────────────────┘
```

**OpenAI 的实践：** 工程师主要在 Level 2-3 工作，Agent 负责 Level 1 的具体实现。

**B. 约束编码（Constraint Encoding）**
- 架构层级严格限制
- 依赖方向强制执行
- 命名规范、文件大小限制等"品味不变量"

#### 2.4 反馈循环详解

##### A. 为什么需要反馈循环？

**类比：新入职的工程师**

想象你雇佣了一个非常聪明但刚入职的工程师。他写完代码后，你什么都不给他看——不看测试结果、不看 CI 报告、不看代码审查意见。

**他能把工作做好吗？**

Agent 也是如此。没有反馈，Agent 就像"盲人摸象"——不知道自己做对没有。

##### B. 反馈信号金字塔

```
┌─────────────────────────────────────────────────────┐
│ 层次5: 生产数据、用户行为  ← 最真实，但最慢          │
│ 层次4: E2E 测试、视觉验证  ← 模拟真实用户            │
│ 层次3: 集成测试、API 测试  ← 验证组件交互            │
│ 层次2: 单元测试            ← 验证逻辑正确性          │
│ 层次1: 编译、类型、lint    ← 最基础，最快            │
└─────────────────────────────────────────────────────┘
```

**Agent 能自己获取吗？**

| 层次 | Agent 能自己获取吗？ | 你需要做什么？ |
|-----|---------------------|---------------|
| 编译/类型/lint | 能 | 无（写完代码就自动发生） |
| 单元测试 | 能 | **写测试代码** |
| 集成测试 | 能 | **写测试代码 + 配置环境** |
| E2E 测试 | 部分 | **写测试代码 + UI 驱动能力** |
| 生产数据 | 不能 | **部署后才能获取** |

**核心洞察：** 层次越往上，信号越有价值，但 Agent 越难自己获取。

##### C. 反馈循环的核心结构

```
人提供"期望"  ──→  Agent 迭代"实现"  ──→  系统验证"差距"
     ↑                                          │
     └────────── 如果有差距，告诉 Agent ─────────┘
```

**人的角色不是"监督 Agent"，而是"定义期望"。**

##### D. 黄金测试：人提供"正确答案"

**问题场景：**

Agent 写了错误的代码，然后自己写了错误的测试，测试通过了：

```javascript
function add(a, b) {
  return a - b;  // 错误！应该是 a + b
}

test("add works", () => {
  expect(add(1, 1)).toBe(0);  // 测试也错了！
});
// 测试通过，Agent 报告"功能完成"
```

**解决方案：人提供"期望"**

```javascript
// 人定义：输入 1 和 1，结果应该是 2
test("add works", () => {
  expect(add(1, 1)).toBe(2);  // 人提供正确答案
});
```

**这就是测试的本质：把"期望"变成"可验证的断言"。**

##### E. 实践示例：清空对话历史

**场景：** 为 ai-terminal 添加"清空对话历史"功能

**第一步：定义期望（人）**

验收标准：
- 数据层：数据库的消息记录被删除
- 界面层：重新渲染后，消息列表为空

**第二步：转化为断言**

```javascript
// 数据层断言
const messages = db.getMessages(sessionId);
expect(messages.length).toBe(0);

// 界面层需要 E2E 测试或视觉验证
```

**关键：** 人定义"什么是对的"，Agent 实现"怎么做到"。

##### F. 三种反馈机制

| 机制 | 描述 | 成本 |
|-----|------|-----|
| Agent 自我审查 | Agent 运行测试、检查 lint、读取错误日志 | 低（自动化） |
| Agent 间审查 | 一个 Agent 审查另一个 Agent 的代码 | 中（并行任务） |
| 后台清理任务 | 定期扫描代码库，发现偏差自动提交 PR | 低（定期运行） |

**Agent 间审查示例：**

```
┌──────────────┐     PR      ┌──────────────┐
│  Agent A     │ ─────────→  │  Agent B     │
│  实现功能    │             │  审查代码    │
└──────────────┘             └──────────────┘
      ↑                             │
      └─────────── 反馈 ────────────┘
```

Agent B 检查：
- 代码是否符合架构约束？
- 测试覆盖率是否达标？
- 是否违反黄金原则？

发现问题后，Agent B 在 PR 上留言，Agent A 响应反馈并修改。

### 3. 关键教训总结

| 教训 | 实践 |
|-----|------|
| 上下文是稀缺资源 | AGENTS.md 作为目录，不是百科全书 |
| 约束比指导更重要 | 边界严格，实现自由 |
| 可观测性是 Agent 的眼睛 | 让日志、指标、UI 对 Agent 可见 |
| 技术债务像高利贷 | 持续小额偿还 |
| 人类品味编码化 | 捕获一次，持续执行 |
| 纠正成本低，等待成本高 | PR 短生命周期，测试失败就重跑 |

### 4. 对工程师的启示

**技能转变：**
- **从：** 写代码、调试、测试、审查
- **到：** 设计约束、编写 prompt、构建工具、编码品味

**核心能力：**
1. **系统思维** — 理解 Agent 如何与环境交互
2. **抽象设计** — 创建 Agent 能理解和使用的抽象
3. **约束工程** — 将人类判断编码为可执行的规则
4. **文档作为代码** — 文档是 Agent 的上下文来源

**心态转变：**
- 不追求代码完美，追求**Agent 可理解性**
- 不担心模式不一致，建立**持续纠正机制**
- 不害怕 Agent 复制坏模式，建立**黄金原则**

---

### 三、实践清单

#### 环境设计
- [ ] 让应用可按 git worktree 启动
- [ ] 暴露日志给 Agent（LogQL/PromQL）
- [ ] 集成 Chrome DevTools Protocol（如果适用）

#### 约束设计
- [ ] 定义架构层级和依赖方向
- [ ] 编写自定义 linter 强制执行
- [ ] 确定可接受的抽象和模式

#### 文档设计
- [ ] 保持 AGENTS.md 简短（~100 行）
- [ ] 建立结构化的 docs/ 目录
- [ ] 用 CI 验证文档新鲜度

#### 反馈循环
- [ ] 建立 Agent 自我审查流程
- [ ] 设置后台清理任务
- [ ] 编码黄金原则

---

### 四、延伸阅读

- [OpenAI 官方博客原文](https://openai.com/index/harness-engineering/)
- [InfoQ: OpenAI Introduces Harness Engineering](https://www.infoq.com/news/2026/02/openai-harness-engineering-codex/)
- [NXCode: Harness Engineering Complete Guide](https://www.nxcode.io/resources/news/harness-engineering-complete-guide-ai-agent-codex-2026)

---

## 人话编程规范 - 详细内容

> 创建日期：2026-03-31

---

### 1. 概述

#### 定义

**人话编程**是一种用自然语言按照编程语言的结构（缩进、函数块、类块）来准确描述实现逻辑的编程方法。文件格式为 Markdown，内容结构直接对标代码结构，AI 负责在人与编程语言之间双向翻译。

#### 核心理念

- **人写人话，AI 写代码** — 人类用自然语言描述逻辑，AI 翻译成任意编程语言
- **结构即代码结构** — 缩进、块、函数边界与目标编程语言一一对应
- **双向无损** — 代码 → 人话 → 代码，结构信息不丢失

#### 与其他方法的区别

| 方法 | 特点 |
|------|------|
| 传统编程 | 用编程语言写文件，人学机器的语言 |
| 伪代码 | 不够严格，无法精确双向翻译 |
| Vibe Coding | 口头描述让 AI 自由发挥，不保留结构控制 |
| **人话编程** | 用自然语言 + 编程结构描述，AI 精确翻译，人保持控制 |

#### 适用场景

- 编程新手入门（用中文理解编程概念）
- 开发者快速原型（用人话描述逻辑，AI 生成代码）
- 代码文档化（把代码翻译成人话，便于理解和审查）
- 跨语言移植（同一个人话描述翻译成不同语言）

#### 适用语言范围

适用于**通用编程语言**（过程式、面向对象、函数式）：
Python、JavaScript、TypeScript、Go、Rust、C++、Java、C#、Ruby、PHP 等

**不适用于**领域特定语言（DSL）：
SQL（声明式查询）、HTML/CSS（标记语言）、正则表达式（模式匹配）、Shell（管道式）

原因：人话编程基于过程式结构（函数、条件、循环、异常），DSL 缺乏这些结构。

---

### 2. 基本规则

共 7 条，是整个人话编程的基石：

1. **缩进即结构** — 用 1 个 TAB 表示一层代码块。缩进层级与目标语言完全对应
2. **冒号开块** — 行尾冒号 `：` 表示接下来是一个缩进块（函数体、条件体、循环体等）
3. **自然语言表达** — 在结构框架内，用中文自然语言自由描述逻辑
4. **类型标注可选** — 可以用括号标注类型如 `n（整数）`，也可以省略
5. **字面量直写** — 数字、字符串、正则等直接写原文，不加引号（除非必要）
6. **原始名标注** — 反向翻译时，在函数名、变量名后用括号标注原始名称，如 `计算阶乘（factorial）：`、`用户列表（users） = []`
7. **注释保留** — 源代码中的注释用 `注：` 保留，并附上原始注释文本：`注：# Check if n is base case`

---

### 3. 基础语法元素

#### 3.1 变量

```
名字 = "张三"
年龄 = 25
是否成年 = 年龄 >= 18
```

| Python | JavaScript | Go |
|--------|------------|----|
| `name = "张三"` | `let name = "张三"` | `name := "张三"` |
| `age = 25` | `let age = 25` | `age := 25` |
| `is_adult = age >= 18` | `let isAdult = age >= 18` | `isAdult := age >= 18` |

#### 3.2 函数定义

```
计算阶乘：
	输入：n（整数）
	如果 n <= 1：
		返回 1
	否则：
		返回 n * 计算阶乘(n - 1)
```

| Python | JavaScript | Go |
|--------|------------|----|
| `def factorial(n):` | `function factorial(n) {` | `func factorial(n int) int {` |
| `    if n <= 1:` | `    if (n <= 1) {` | `    if n <= 1 {` |
| `        return 1` | `        return 1;` | `        return 1` |
| `    else:` | `    } else {` | `    } else {` |
| `        return n * factorial(n - 1)` | `        return n * factorial(n - 1);` | `        return n * factorial(n - 1)` |
| | `    }` | `    }` |
| | `}` | `}` |

#### 3.3 函数表达方式

不同语言有多种定义函数的方式，反向翻译时通过行尾标记区分：

| 人话写法 | JavaScript | 说明 |
|----------|------------|------|
| `函数名：` | `function fn() {}` | 普通函数声明 |
| `函数名（箭头）：` | `const fn = () => {}` | 箭头函数 |
| `函数名（方法）：` | `fn() {}`（在 class 内） | 类方法 |
| `函数名（异步）：` | `async function fn() {}` | 异步函数 |
| `函数名（异步箭头）：` | `const fn = async () => {}` | 异步箭头函数 |

示例：

```
读取文件内容（箭头）：
	输入：file（文件路径）
	读取文件，编码为 utf-8
	返回内容

异步获取数据（异步箭头）：
	输入：url（字符串）
	发送 GET 请求到 url
	解析响应 JSON
	返回数据
```

翻译为 JavaScript：

```javascript
const readFileContent = (file) => {
  const content = readFileSync(file, 'utf-8');
  return content;
};

const fetchData = async (url) => {
  const response = await fetch(url);
  const data = await response.json();
  return data;
};
```

不标注时默认为普通函数声明。反向翻译时，如果原始代码是箭头函数，必须标注 `（箭头）`。

#### 3.4 条件判断

```
如果 成绩 >= 90：
	等级 = "优秀"
否则如果 成绩 >= 60：
	等级 = "及格"
否则：
	等级 = "不及格"
```

对应关系：
- `如果` → `if`
- `否则如果` → `elif` / `else if`
- `否则` → `else`

#### 3.5 循环

**遍历循环：**

```
遍历 1 到 10 的每个数字 i：
	打印 i
```

**条件循环：**

```
当 还有数据 时重复：
	读取下一行
	处理该行数据
```

对应关系：
- `遍历...每个...` → `for ... in ...`
- `当...时重复` → `while ...`
- `跳出` → `break`
- `跳过` → `continue`

#### 3.6 其他基本元素

返回值、注释、导入依赖：

```
返回 结果                          → return result
注：这是一个注释                    → # 这是一个注释 / // 这是一个注释
引入 math, os                      → import math, os
引入 requests 作为 req             → import requests as req
```

---

### 4. 高级语法元素

#### 4.1 类与对象

```
类 用户管理器：
	属性：
		用户列表 = 空列表
		数据库连接

	初始化（数据库地址）：
		连接到数据库地址
		将连接保存到 self.数据库连接

	添加用户（名字，年龄）：
		如果 名字 为空：
			抛出 "用户名不能为空"
		创建新用户 = {名字: 名字, 年龄: 年龄, id: 生成ID()}
		添加到 用户列表
		返回 新用户的 id

	查找用户（用户id）：
		遍历 用户列表 的每个 用户：
			如果 用户.id == 用户id：
				返回 用户
		返回 空
```

#### 4.2 异常处理

```
尝试：
	打开文件 "data.txt"
	读取内容
捕获 文件不存在错误：
	打印 "文件不存在，使用默认配置"
	内容 = 默认配置
捕获 其他错误 为 e：
	打印 "发生错误：" + e
	内容 = 空
最终：
	关闭文件（如果已打开）
```

对应关系：
- `尝试` → `try`
- `捕获 ...错误` → `catch` / `except`
- `捕获 ... 为 e` → `catch(e)` / `except ... as e`
- `最终` → `finally`

#### 非异常错误处理

Go、Rust、C 等语言不使用异常，而是通过返回值传递错误。人话中用自然语言描述错误检查逻辑，AI 根据目标语言选择对应模式：

**Go 风格（多返回值）：**

```
读取文件内容：
	内容, 错误 = 读取文件(文件路径)
	如果 错误 不为空：
		返回空, 错误
	返回 内容, 空
```

→ Go: `content, err := os.ReadFile(path); if err != nil { return "", err }; return string(content), nil`

**Rust 风格（Result 类型）：**

```
读取文件内容：
	结果 = 读取文件(文件路径)
	如果 结果 为错误：
		返回错误
	内容 = 解包结果
	返回 内容
```

→ Rust: `let content = fs::read_to_string(path)?;`

**原则**：人话不绑定特定错误处理模型。使用 `尝试/捕获` 描述异常模式，用 `错误不为空`/`为错误` 描述返回值模式。AI 识别语义后适配目标语言。

#### 4.3 导入依赖

```
引入 math, os
引入 requests 作为 req
```

→ Python: `import math, os` / `import requests as req`
→ JS: `import * as math from 'math'` / `import req from 'requests'`
→ Go: `import ("math"; "os")`

#### 4.4 并发（简述）

```
并发执行：
	任务1 = 启动任务 获取用户信息
	任务2 = 启动任务 获取订单信息
	用户信息 = 等待 任务1
	订单信息 = 等待 任务2
	返回 合并(用户信息, 订单信息)
```

---

### 5. 完整示例

#### 示例 1：斐波那契数列

```
计算第n个斐波那契数：
	输入：n（非负整数）
	如果 n <= 1：
		返回 n
	前一个 = 0
	当前 = 1
	遍历 2 到 n 的每个 i：
		下一个 = 前一个 + 当前
		前一个 = 当前
		当前 = 下一个
	返回 当前
```

翻译成 Python：

```python
def fibonacci(n):
    if n <= 1:
        return n
    prev = 0
    curr = 1
    for i in range(2, n + 1):
        next_val = prev + curr
        prev = curr
        curr = next_val
    return curr
```

#### 示例 2：API 数据处理（生产级）

```
引入 requests

获取天气（get_weather）：
	输入：城市名（city_name，字符串）
	前置校验：
		如果 城市名 为空或仅含空白：
			抛出 ValueError "城市名不能为空"
	尝试：
		注：调用天气 API // Call weather API
		发送 GET 请求到 https://api.weather.com/v1
			超时：10 秒
			参数：{city: 城市名, unit: "metric", lang: "zh_cn"}
			请求头：{Accept: "application/json"}
		如果 响应.状态码 == 404：
			返回 null（城市不存在，非错误）
		如果 响应.状态码 == 429：
			等待 5 秒后重试，最多 2 次
			仍失败则抛出 "天气 API 限流，请稍后重试"
		如果 响应.状态码 != 200：
			抛出 "天气 API 请求失败：" + 响应.状态码
		数据 = 解析JSON(响应.文本)
			如果解析失败或缺少 main 字段：
				抛出 "天气 API 返回格式异常"
		返回 {
			温度: 数据.main.temp（浮点数，摄氏度）,
			湿度: 数据.main.humidity（整数，百分比）,
			描述: 数据.weather[0].description（字符串）,
			更新时间: 数据.dt（UNIX 时间戳）
		}
	捕获 requests.ConnectionError：
		记录警告日志 "网络连接失败，城市：" + 城市名
		返回 null
	捕获 requests.Timeout：
		记录警告日志 "天气 API 超时，城市：" + 城市名
		返回 null
```

翻译成 Python：

```python
import requests
import logging

logger = logging.getLogger(__name__)

def get_weather(city_name, _retries=0):
    if not city_name or not city_name.strip():
        raise ValueError("城市名不能为空")

    try:
        # Call weather API
        response = requests.get(
            "https://api.weather.com/v1",
            params={"city": city_name, "unit": "metric", "lang": "zh_cn"},
            headers={"Accept": "application/json"},
            timeout=10
        )

        if response.status_code == 404:
            return None
        if response.status_code == 429:
            if _retries < 2:
                import time
                time.sleep(5)
                return get_weather(city_name, _retries=_retries + 1)
            raise Exception("天气 API 限流，请稍后重试")
        if response.status_code != 200:
            raise Exception(f"天气 API 请求失败：{response.status_code}")

        data = response.json()
        if "main" not in data:
            raise Exception("天气 API 返回格式异常")

        return {
            "temp": float(data["main"]["temp"]),
            "humidity": int(data["main"]["humidity"]),
            "description": str(data["weather"][0]["description"]),
            "updated_at": int(data["dt"])
        }
    except requests.ConnectionError:
        logger.warning(f"网络连接失败，城市：{city_name}")
        return None
    except requests.Timeout:
        logger.warning(f"天气 API 超时，城市：{city_name}")
        return None
```

#### 示例 3：反向翻译 — Python 代码转人话

原始代码：

```python
class DataProcessor:
    def __init__(self, source_path):
        self.data = []
        self.source_path = source_path

    def load(self):
        with open(self.source_path, 'r') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#'):
                    self.data.append(line.split(','))

    def filter_by(self, column, value):
        return [row for row in self.data if row[column] == value]

    def count(self):
        return len(self.data)
```

反向翻译结果：

```
类 数据处理器（DataProcessor）：
	属性：
		数据（data） = 空列表
		源文件路径（source_path）

	初始化（source_path）：
		self.源文件路径（source_path） = source_path

	加载（load）：
		注：从文件读取并解析 CSV 数据 // Read and parse CSV data from source file
		打开 self.源文件路径 用于读取，作为 f：
			遍历 f 的每一行：
				去除行首尾空白
				如果 行不为空 且 不以 # 开头：
					按逗号分割行，添加到 self.data

	按列筛选（filter_by）：
		输入：column（列号），value（值）
		返回 self.data 中所有该列等于指定值的行

	计数（count）：
		返回 self.data 的长度
```

---

### 6. 反向翻译规则

从编程语言翻译为人话时，遵循以下规则：

#### 结构保留

| 代码结构 | 人话对应 |
|----------|----------|
| `def` / `function` / `func` | `函数名：` 或直接 `名称：` |
| `class` | `类 名称：` |
| `if` / `elif` / `else` | `如果` / `否则如果` / `否则` |
| `for ... in ...` | `遍历 ... 的每个 ...` |
| `while ...` | `当 ... 时重复` |
| `try` / `catch` / `finally` | `尝试` / `捕获` / `最终` |
| `return` | `返回` |
| `import` | `引入` |
| `#` / `//` 注释 | `注：` |
| 缩进层级 | 保持相同层级（TAB） |

#### 翻译原则

1. **函数名直译 + 原始名标注**：`get_weather` → `获取天气（get_weather）：`，`calculate_total` → `计算总数（calculate_total）：`
2. **变量名直译 + 原始名标注**：`user_name` → `用户名（user_name）`，`is_valid` → `是否有效（is_valid）`
3. **保留代码结构**：该缩进的地方必须缩进，该有块的地方必须用冒号开块
4. **字面量原样保留**：字符串、数字、URL、正则表达式等不做翻译
5. **运算符可选翻译**：`==` 可以写成 `等于`，也可以保留原文；`>=` 可以写成 `大于等于`，也可以保留
6. **注释双向保留**：源码注释翻译为 `注：中文翻译 // 原始注释`，确保还原时能找回原文

#### 不翻译的部分

- 字符串字面量中的内容
- URL 和文件路径
- 正则表达式
- API 端点和参数名
- 数据库字段名

#### 原始名标注规则

反向翻译时，所有标识符（函数名、变量名、参数名、类名）必须在首次出现时标注原始名称。格式：`中文名（original_name）`。完整示例见第 5 节示例 3。

---

### 7. 写作指南

#### 句式约定

| 场景 | 写法 | 示例 |
|------|------|------|
| 描述动作 | 祈使句 | `打开文件`、`读取内容` |
| 描述条件 | `如果...则...` | `如果 年龄 >= 18：` |
| 描述循环 | `遍历` / `当...时重复` | `遍历 用户列表 的每个 用户：` |
| 描述赋值 | `=` 或 `设为` | `总数 = 0` 或 `将总数设为 0` |
| 描述返回 | `返回` | `返回 计算结果` |
| 描述异常 | `抛出` | `抛出 "文件不存在"` |

#### 避免歧义的技巧

1. **明确作用对象**：不说 `删除它`，说 `删除临时文件`
2. **明确执行顺序**：用 `先...再...` 或有序步骤
3. **明确数据类型**：在首次出现时用括号标注类型，如 `结果（布尔值）`
4. **明确范围**：不说 `所有`，说 `列表中的每个元素`
5. **复杂逻辑拆步骤**：一个缩进块内不要塞太多逻辑，拆成有序步骤

#### 命名建议

- 函数名用动词开头：`获取用户`、`计算总数`、`验证输入`
- 变量名用名词：`用户列表`、`当前页码`、`最大重试次数`
- 布尔变量用 `是否` 开头：`是否登录`、`是否有效`
- 常量全大写风格可保留：`MAX_RETRY` 或翻译为 `最大重试次数`

#### 生产级写作清单

教程级人话只描述核心逻辑，生产级人话需要额外覆盖以下维度。除明确标注为「教程示例」的内容外，所有面向实际使用的人话文档都应按此清单检查：

**每个函数应覆盖：**

- [ ] **前置校验** — 输入为 null/空/越界时的行为
- [ ] **超时** — 网络请求、文件操作的超时时间
- [ ] **重试** — 哪些错误重试、最多几次、退避策略（固定/指数）
- [ ] **错误分类** — 哪些错误可恢复（重试）、哪些不可恢复（直接抛出）、哪些降级（返回默认值）
- [ ] **边界值** — 空输入、极大输入、并发访问时的行为
- [ ] **资源释放** — 文件句柄、网络连接、锁的释放时机

**人话中的写法示例：**

```
前置校验：
	如果 输入 为 null 或 undefined，抛出 TypeError "text is required"
	如果 输入长度 为 0，直接返回 0，不发请求

超时与重试：
	请求超时 30 秒
	如果 429 限流：
		从响应头读取 Retry-After，若无则默认 5 秒
		重试最多 3 次
		3 次仍失败，抛出异常
	如果 >= 500 服务端错误：
		指数退避重试最多 2 次（1秒、2秒）
		仍失败，抛出异常
	如果 401 认证失败：
		直接抛出，不重试

降级处理：
	如果请求失败，打印警告到 stderr，返回 null
	不抛异常，不中断调用方
```
