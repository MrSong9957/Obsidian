# AI IDE 产品需求文档（PRD）

## 1. 产品定位

AI IDE 是一个面向 AI 时代的软件开发环境。

核心目标不是简单地给传统 IDE 增加 AI 聊天或代码补全，而是重新设计**人、AI、代码、文档和插件系统之间的协作方式**：

> 人负责表达意图、理解关键逻辑和做决策；AI 负责生成、维护和同步大量机械性代码与文档；IDE 负责保证整个过程始终可理解、可追踪、可修改。

以 VS Code 的界面布局、快捷键体系和核心交互习惯作为兼容目标，以 Rust 原生编辑器架构作为实现基础；优先复用成熟开源 Rust 编辑器组件，而非直接将 VS Code TypeScript 源码机械迁移为 Rust。

> Lapce 做第一候选，Code-OSS + Rust Hybrid 做对照方案。

同时提供组件级动态插件架构，使 IDE 能够在运行时安全地加载、卸载、替换和重新配置功能，而不依赖重启整个应用。



---

## 2. 核心问题

### 2.1 Vibe Coding 正在制造新的代码黑盒

现有 AI Coding 工具可以快速生成大量代码，但通常存在一个根本问题：

> 代码增长速度远高于开发者理解代码的速度。

最终可能出现：

* AI 写出了能运行的代码；
* 创造这个系统的人却不知道代码内部如何工作；
* 函数之间的数据流和依赖关系越来越难理解；
* 修改一个功能时无法判断影响范围；
* 文档落后于真实代码；
* AI 下一次修改时也缺少准确上下文。

AI IDE 要解决的不是单纯的“生成更多代码”，而是：

> **在保持 AI 高生产力的同时，让人始终能够理解和控制系统。**

---

## 3. 产品原则

### 3.1 保持传统 IDE 的低学习成本

优先继承 VS Code 已经成熟的：

* 编辑方式；
* 文件树；
* 标签页；
* 快捷键；
* 搜索；
* 调试；
* Git 工作流；
* Extension 使用习惯。

AI 能力应自然融入这些操作，而不是要求用户学习完全不同的开发范式。

### 3.2 AI 优先，但人保持控制

AI 可以负责：

* 命名建议；
* 代码生成；
* 重复实现；
* 上下文搜集；
* 文档同步；
* 依赖分析；
* 代码解释。

但关键设计、逻辑和修改仍必须能够被人直接看到和编辑。

### 3.3 Source of Truth 始终是代码

文档、关系图和 AI 上下文都必须从真实代码状态派生。

不能出现：

```text
代码
  ↓
人工维护文档
  ↓
文档逐渐失真
```

而应该是：

```text
代码
  ↓
IDE 分析
  ↓
结构化语义信息
  ↓
文档 / AI 上下文 / 可视化
```

代码变化后，下游信息自动更新。

---

### 3.4 一切从简

只生成真正有价值的信息。

避免：

* 大量模板化文档；
* 重复描述代码本身；
* 为了“完整”产生没人阅读的文档；
* 为 AI 单独维护另一套复杂知识库。

文档应同时适合：

* 人阅读；
* AI 检索；
* AI 理解；
* 自动更新。

---

# 4. 核心功能

## 4.1 智能补全

智能补全保留 VS Code、Cursor 等 IDE 已经形成的基本交互：

```text
用户输入代码
    ↓
AI 预测后续内容
    ↓
IDE 显示 Ghost Text
    ↓
TAB 接受
```

但 AI IDE 的补全不能只根据当前几百行文本预测代码，而应理解：

* 当前函数；
* 当前文件；
* 类型信息；
* 调用关系；
* 上游函数；
* 下游函数；
* 项目结构；
* 相关文档；
* 用户当前意图。

目标是从：

> 文本补全

升级为：

> **基于程序结构和数据流的语义补全。**

---

## 4.2 注释生成代码（兼容入口）

用户可以先使用自然语言描述逻辑，再由 AI 实现代码。

> **产品定位：兼容入口。** 推荐通过「结构化代码生成器」（@sec:4.4）描述功能，由 IDE / AI 同时生成结构化注释和代码。本节保留为已有习惯用户提供支持。

例如：

```rust
// 读取用户配置。
// 如果文件不存在则创建默认配置。
// 配置文件存在但缺少字段时自动补齐。
// 修改原文件前创建备份。
```

用户选中注释并执行快捷键后，IDE 解析注释中的开发意图，汇入统一代码生成流程（@sec:4.5）：

```text
源码中人工编写注释
    ↓
选中注释 → 手动触发「注释生成代码」
    ↓
IDE 解析注释中的开发意图
    ↓
汇入统一代码生成流程（@sec:4.5）
    ↓
AI 生成结构化注释 + 代码
    ↓
用户审查 / 修改 / 接受
    ↓
写入代码
    ↓
同步更新相关文档（@sec:5.4）
```

生成的代码中保留适合长期留存的结构化注释，用于：

* 表达设计意图；
* 帮助人类理解代码；
* 为 AI 后续修改提供上下文。

---

## 4.3 上下游函数关联

生成或修改函数时，用户可以显式告诉 AI：

> 这个函数处于哪条数据链中。

IDE 提供函数搜索和下拉选择能力。

例如：

```text
上游函数
UserInput::parse()

当前函数
validate_request()

下游函数
AgentRuntime::execute()
```

生成代码时上下游实现、输入输出和类型信息一并加入 AI 上下文。

目的不是简单增加 Token，而是让 AI 理解：

```text
数据从哪里来
    ↓
当前函数需要完成什么转换
    ↓
结果最终流向哪里
```

这样可以减少单个函数局部正确、整个调用链却错误的问题。

---

## 4.4 结构化代码生成器（推荐主入口）

> **产品定位：意图驱动代码生成的推荐统一入口。** 用户通过结构化代码生成器描述功能，由 IDE / AI 同时生成结构化注释和代码，代码确认写入后自动同步更新相关文档。

用户不需要直接编写完整 Prompt。

例如创建函数时打开：

```text
创建函数

函数名
[ 自动生成 ▼ ]

参数 *
[ user_id ] [ UserId ]

[ + 添加参数 ]

返回值 *
[ Result<User> ]

功能描述 *
[ 根据用户 ID 查询用户，如果不存在返回 NotFound ]

上游函数
[ 搜索函数... ]

下游函数
[ 搜索函数... ]

其他约束
[ 可选 ]
```

其中：

### 必填

至少包括：

* 输入；
* 返回值；
* 功能描述。

### 可选

包括：

* 函数名；
* 上游函数；
* 下游函数；
* 额外约束。

函数名允许直接选择：

```text
AI 自动命名
```

IDE 根据：

* 功能描述；
* 当前模块；
* 项目命名风格；
* 已有 symbol；

自动产生候选名称，并避免明显重名。

### 推荐流程

```text
结构化代码生成器
    ↓
用户描述功能、输入输出、上下游关系和必要约束
    ↓
IDE 整理结构化开发意图
    ↓
AI 生成结构化注释 + 代码
    ↓
用户审查 / 修改 / 接受
    ↓
写入代码
    ↓
更新 Symbol、调用关系等代码索引
    ↓
同步更新相关文档（@sec:5.4）
```

生成的结构化注释保留在源码中，用于表达设计意图、帮助人类理解以及为 AI 后续修改提供上下文。

> **Source of Truth 始终是代码。** 结构化生成器中的功能描述是生成阶段的用户意图输入，不是独立的长期 Source of Truth。

---

## 4.5 生成流程

AI 代码生成分为两类，工作流和副作用不同。

### 普通智能补全

保持轻量交互，不要求每次 TAB 补全都生成结构化注释或触发文档同步。

```text
用户输入代码
    ↓
IDE 获取语义上下文（Symbol / 类型 / 调用关系）
    ↓
AI 预测补全内容
    ↓
Ghost Text 展示
    ↓
TAB 接受
```

不需要结构化生成器表单，不强制生成结构化注释，不因单次补全触发完整文档同步。

### 主动功能生成

用户主动描述一个完整功能并生成代码时，使用统一流程：

```text
结构化代码生成器（推荐，@sec:4.4）
    或
人工源码注释（兼容，@sec:4.2）
    ↓
解析用户意图
    ↓
分析 Symbol / 类型 / 调用关系 / 上下游
    ↓
收集必要项目上下文
    ↓
AI 生成结构化注释 + 候选代码
    ↓
展示 Diff / 用户审查
    ↓
写入代码
    ↓
更新代码索引（Symbol、调用关系）
    ↓
同步更新相关文档（@sec:5.4）
```

两类流程共享底层能力（Symbol 分析、类型信息、上下文检索、AI 服务），但用户级工作流和最终副作用不同。

---

## 4.6 支持 Vibe Coding

* 人与 AI IDE 的协作是主要开发模式，但系统同时也支持 Vibe Coding 模式
* 在 Vibe Coding 模式下，用户可以更自由地通过自然语言或模糊意图驱动 AI 生成代码
* IDE 会降低结构化约束，允许更快速的探索式开发
* 同时仍保留基础的代码可追踪性与回溯能力，避免完全失控的黑盒生成

---

# 5. 智能文档

## 5.1 基本原则

开发者正常使用 IDE 写代码即可。

不要求额外承担大量文档维护工作。

IDE 自动从代码变化中维护必要的团队知识。

目标：

> **代码改变，文档跟着改变。**

通过结构化代码生成器（@sec:4.4）或注释生成代码（@sec:4.2）创建的代码，在确认写入后自动触发文档同步更新，不要求用户再次手工维护文档。

核心约束：

* 代码是唯一 Source of Truth，文档从代码状态派生；
* 人与 AI 使用同一份结构化 Markdown，不维护两套知识体系；
* 能从代码直接得到的信息用表格或索引表达，不写成长篇解释；
* 只维护真正影响理解、协作和 AI 上下文的信息；
* 文档更新不得成为正常编码流程的阻塞点。

---

## 5.2 最小文档体系

默认不创建大量文档。

第一阶段只维护三种文档，覆盖 PRD 全部文档要求：

```text
README.md

docs/
├─ architecture.md
└─ modules/
   ├─ <module-a>.md
   ├─ <module-b>.md
   └─ ...
```

不额外创建：

* 每函数一个 Markdown；
* 每类型一个 Markdown；
* 独立 `functions.md` / `types.md` / `call-graph.md`；
* AI 专用文档或知识库。

函数索引、类型索引、调用关系统一收敛到对应模块文档。

---

## 5.3 统一格式

### README.md

项目入口和导航，不堆架构细节。

```markdown
# AI IDE

一句话说明项目是什么。

## Run

最小启动方式。

## Main Entry

- `src/main.rs`

## Architecture

- [Architecture](docs/architecture.md)

## Modules

- [Runtime](docs/modules/runtime.md)
- [Provider](docs/modules/provider.md)
- [Tools](docs/modules/tools.md)
```

### docs/architecture.md

只描述跨模块、全局级信息。

```markdown
# Architecture

## Responsibility

系统整体负责什么。

## Entry

- `main`
- `App::run`

## Modules

| Module | Responsibility | Entry | Document |
|---|---|---|---|
| Runtime | Agent 生命周期与调度 | `AgentRuntime::spawn` | `modules/runtime.md` |
| Provider | 模型调用 | `Provider::request` | `modules/provider.md` |
| Tools | 工具执行 | `ToolExecutor::execute` | `modules/tools.md` |

## Dependencies

Runtime → Provider
Runtime → Tools
Tools → Filesystem

## Data Flow

User → UI → Agent Runtime → Provider → Tool → Runtime Event → UI

## Design Constraints

只记录真正影响多个模块的长期架构约束。
```

`Modules` 表即文档系统目录，不额外维护 `index.md`。

### docs/modules/\<module\>.md

每个具有独立职责的重要模块维护一份文档。

```markdown
# <Module>

## Responsibility

模块负责什么。

## Entry

- `<主要入口>`

## Core Types

| Symbol | Type | Location | Summary |
|---|---|---|---|
| `AgentRuntime` | struct | `src/runtime.rs` | Runtime 主控制对象 |
| `RuntimeHandle` | struct | `src/runtime.rs` | 外部控制句柄 |

## Depends On

- `<dependency>`

## Symbols

| Symbol | Kind | Location | Summary |
|---|---|---|---|
| `AgentRuntime::spawn` | function | `src/runtime.rs` | 创建并启动 Runtime |
| `RuntimeHandle::cancel` | function | `src/runtime.rs` | 取消当前运行 |

## Relationships

| From | Relation | To |
|---|---|---|
| `AgentRuntime::spawn` | calls | `Provider` |
| `AgentRuntime` | uses | `ToolExecutor` |
| `RuntimeHandle::cancel` | writes | `RuntimeState` |

## Data Flow

Input → Module → Dependency → Output
```

其中：

* `Core Types` 满足核心数据结构要求；
* `Symbols` 同时承担函数 / 类型索引；
* `Relationships` 记录 `calls / called_by / uses / implements / reads / writes`；
* `Data Flow` 只描述重要数据链，不逐行复述代码。

---

## 5.4 更新机制

文档更新分为两类，避免每次微小代码变化都调用 AI 重写整个文档。

### 实时结构信息

代码变化后根据 AST / Symbol Index / Dependency Graph 确定性更新，不需要 AI 重新理解模块语义：

* Entry
* Core Types
* Depends On
* Symbols
* Relationships

### 后台语义信息

代码发生有意义的语义变化时，由 AI 异步更新：

* Responsibility
* Summary
* Data Flow
* architecture.md 中的架构说明和跨模块关系

支持闲时运行或按计划定期运行（如每天、每周、Git commit 后、PR 创建前）。

整体流程：

```text
代码变化
    ↓
更新 AST / Symbol / Dependency Graph
    ↓
实时更新结构化文档字段
    ↓
判断语义是否变化
    ↓
必要时 AI 更新 Responsibility / Summary / Data Flow / Architecture
```

---

# 6. 代码理解能力

AI IDE 不应只维护文本索引，而应维护项目的结构化代码模型。

核心关系：

```text
Project
 ├─ Module
 │   ├─ Type
 │   ├─ Function
 │   └─ Variable
 │
 └─ Relationship
     ├─ calls
     ├─ called_by
     ├─ uses
     ├─ implements
     ├─ reads
     └─ writes
```

由此形成：

```text
代码
↓
AST / Symbol
↓
语义索引
↓
Dependency Graph
↓
AI Context
```

这套结构同时服务于：

* 智能补全；
* 上下游函数选择；
* 重构；
* AI 生成代码；
* 文档生成；
* 影响分析；
* 代码导航。

---

# 7. 动态插件系统

## 7.1 目标

AI IDE 的插件系统不能简单复制传统 VS Code Extension Host。

目标是支持：

```text
安装
加载
激活
停用
卸载
替换
重新配置
```

并尽量无需重启 IDE。

最终形成：

> **组件级热插拔架构。**

---

## 7.2 为什么不能只依赖传统插件模型

传统插件系统通常存在两个问题。

### 副作用无法可靠撤销

插件激活后可能：

* 注册事件；
* 注册命令；
* 创建任务；
* 修改状态；
* 打开连接；
* 创建缓存。

如果这些操作没有统一生命周期管理，卸载插件时就无法保证系统恢复原状。

### 依赖关系缺少运行时管理

插件可能依赖：

```text
插件 A
↓
插件 B
↓
插件 C
```

当 B：

* 消失；
* 升级；
* 替换；
* 配置变化；

A 必须能够立即得到正确响应。

不能把这些逻辑全部交给插件自己实现。

---

# 8. 时空可组合组件模型

插件系统采用两个独立的组合维度。

## 8.1 时间可组合性

组件对系统产生的副作用必须能够撤销。

例如：

```text
Plugin A
↓
register_command("foo")
↓
Runtime 记录 effect
```

卸载时：

```text
Plugin A unload
↓
Runtime 查找 effects
↓
unregister_command("foo")
↓
环境恢复
```

组件创建每一个需要撤销的副作用时，同时向 Runtime 注册其逆操作。

即：

```text
Effect
+
Undo
```

由 Runtime 统一追踪生命周期。

---

## 8.2 空间可组合性

插件显式声明自己依赖什么能力。

例如：

```text
GitPanel

requires:
  - git.repository
  - ui.sidebar
```

Runtime 负责解析当前环境：

```text
依赖满足
→ activate

依赖消失
→ deactivate

依赖重新出现
→ reactivate
```

插件自己不需要不断轮询整个系统。

---

## 8.3 Component

Effect 和 Dependency 最终统一为 Component。

概念模型：

```text
Component
├─ provides
├─ requires
├─ effects
├─ state
└─ lifecycle
```

生命周期：

```text
Created
   ↓
Resolved
   ↓
Activated
   ↓
Running
   ↓
Deactivated
   ↓
Removed
```

Runtime 对每一个 Component 管理：

```text
依赖
+
副作用
+
生命周期
```

---

# 9. 配置协调

组件配置变化后，不应该简单要求：

```text
Restart IDE
```

而是计算：

```text
当前状态
+
目标配置
↓
差异
↓
最小变更集合
```

例如：

```text
当前：
A + B + C

目标：
A + B2 + D
```

系统执行：

```text
Deactivate C
Remove C

Deactivate B
Replace B → B2
Activate B2

Load D
Activate D
```

而 A 保持运行。

这就是 **Configuration Reconciliation**。

---

# 10. 热模块替换

组件更新时：

```text
旧 Component
↓
Deactivate
↓
撤销 Effects
↓
加载新 Component
↓
重新解析 Dependencies
↓
Activate
```

理想情况下：

```text
IDE 主进程
不重启

其他组件
不重启

用户编辑状态
不丢失
```

只替换发生变化的组件。

---

# 11. 插件能力模型

插件不直接任意修改 IDE 全局状态。

系统提供 Capability。

例如：

```text
editor.document
editor.completion
editor.command
ui.sidebar
ui.statusbar
workspace.filesystem
git.repository
ai.model
ai.context
docs.index
```

组件声明：

```text
requires
provides
```

例如：

```text
AI Completion Plugin

requires:
- editor.document
- ai.model
- code.symbol_index

provides:
- editor.completion
```

这样插件之间可以围绕明确能力组合，而不是通过任意全局对象互相调用。

---

# 12. AI 能力本身也组件化

AI IDE 的核心 AI 功能不应成为不可替换的巨型内核。

例如：

```text
AI System
├─ Completion Component
├─ Context Component
├─ Symbol Retrieval Component
├─ Code Generation Component
├─ Documentation Component
├─ Model Provider Component
└─ Agent Component
```

因此未来可以动态替换：

```text
Claude
→ DeepSeek

Context Retriever v1
→ Context Retriever v2

普通 Completion
→ 新型 Agent Completion
```

而不需要重构整个 IDE。

这也是插件架构与 AI IDE 长期演化能力之间最重要的连接点。

---

# 13. 总体架构

```text
┌───────────────────────────────────────┐
│                AI IDE                 │
├───────────────────────────────────────┤
│              UI / Editor              │
│                                       │
│ File Tree │ Editor │ Search │ Git     │
│ AI UI     │ Docs   │ Debug  │ Plugin  │
├───────────────────────────────────────┤
│             AI Services               │
│                                       │
│ Completion                            │
│ Code Generation                       │
│ Context Engine                        │
│ Agent                                 │
│ Documentation                         │
├───────────────────────────────────────┤
│          Code Intelligence            │
│                                       │
│ Parser / AST                          │
│ Symbol Index                          │
│ Type Information                      │
│ Dependency Graph                      │
├───────────────────────────────────────┤
│          Component Runtime            │
│                                       │
│ Lifecycle                             │
│ Dependency Resolution                 │
│ Effect Tracking                       │
│ Configuration Reconciliation          │
│ Hot Module Replacement                │
├───────────────────────────────────────┤
│            Core Services              │
│                                       │
│ Filesystem │ Git │ Tasks │ Settings   │
│ Network    │ Terminal │ Storage       │
└───────────────────────────────────────┘
```

---

# 14. 关键新增能力：函数 ↔ 文档一键跳转

## 14.1 功能定义

AI IDE 支持在**函数与对应文档之间双向一键跳转**，用于快速在“代码实现”和“语义说明”之间切换。

该能力是在现有 VS Code 跳转体系（Go to Definition / Go to References / Go to Implementation 等）之上的增强扩展，同时保持对主流基础跳转能力的完全兼容。

---

## 14.2 交互方式

在函数定义或调用处提供入口：

* 函数右上角按钮：`📄 文档`
* 或快捷键：`
