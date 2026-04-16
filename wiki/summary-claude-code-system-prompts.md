---
name: Claude Code 系统提示词摘要
description: Claude Code npm 包的 250 个系统提示词提取——了解 Claude Code 内部提示词结构时查阅
type: summary
source: "[[sources/claude-code-system-prompts/README]]"
confidence: 0.95
updated: 2026-04-06
tags: [Claude-Code, 系统提示词, 提示工程, harness]
---

# 摘要：Claude Code System Prompts

## 一句话总结

Claude Code 并非单一系统提示词，而是由 250 个可组合的提示词片段构成，分为 Agent、Data、System、Tool、Skill、Reminder 六大类，按运行时条件动态组装。[^1]

## 六大分类及职责

### 1. System Prompts（约 101 个）

主系统提示词的各个片段，定义 Claude Code 的核心行为准则。

- **行为约束**：不做不必要的添加、不做过早抽象、不给时间估计、安全第一（防注入/XSS）[^2]
- **任务执行**：读代码再改、编辑优于创建、被阻塞时换思路、大胆接受复杂任务 [^3]
- **子代理管理**：Fork 使用指南、子代理委派示例、子代理 prompt 编写指南 [^4]
- **记忆系统**：Agent 记忆指令、记忆文件选择、会话记忆更新、记忆描述 [^5]
- **模式控制**：Plan 模式（5 阶段）、Auto 模式、Minimal 模式、Learning 模式 [^6]

### 2. Tool Descriptions（约 75 个）

内置工具的声明和使用说明，是模型理解"我能做什么"的关键。

- **核心工具**：Bash、Read、Write、Edit、Glob、Grep [^7]
- **高级工具**：Agent（子代理）、AskUserQuestion、EnterPlanMode、TodoWrite [^8]
- **团队工具**：TeammateTool、SendMessage、TeamDelete [^9]
- **Bash 子规则**：sandbox 策略（约 20 个片段）、git 操作规范、sleep 使用规则 [^10]

### 3. Agent Prompts（约 34 个）

子代理和 AI 驱动的工具函数，各自独立的系统提示词。

- **子代理**：Explore（只读搜索）、Plan（规划）、General-purpose（通用） [^11]
- **创建助手**：Agent 创建架构师、CLAUDE.md 创建、Statusline 配置 [^12]
- **Slash 命令**：/batch、/pr-comments、/review-pr、/schedule、/security-review [^13]
- **AI 工具函数**：会话摘要、标题生成、安全监控、验证专家、WebFetch 摘要 [^14]

### 4. Data（约 27 个）

嵌入的参考文档模板，按需注入提供 API 知识。

- **SDK 参考**：Python/TypeScript Agent SDK、Claude API（6 种语言 + cURL） [^15]
- **概念文档**：Tool Use 概念、Streaming 参考、Prompt Caching 设计 [^16]

### 5. Skills（约 13 个）

内置技能的完整 prompt，用于专业化任务。

- **核心技能**：/init（CLAUDE.md 创建）、/loop、/stuck、/simplify [^17]
- **API 构建技能**：Claude API 构建指南 + 语言参考模板 [^18]
- **验证技能**：Verify skill + CLI/Server 验证示例 [^19]
- **配置技能**：update-config 7 步验证流 [^20]

### 6. System Reminders（约 40 个）

运行时动态注入的通知片段，报告环境状态变化。

- **文件事件**：文件被外部修改、文件为空、文件被截断 [^21]
- **模式切换**：Plan 模式激活/退出、Session 延续、技能调用 [^22]
- **团队协调**：Team Coordination、Team Shutdown [^23]
- **工具提醒**：Task tools reminder、Token usage、USD budget [^24]

## 提示词格式

每个提示词文件采用统一的 YAML frontmatter + Markdown body 格式：[^25]

```
<!--
name: '提示词名称'
description: 一行描述
ccVersion: 2.1.84
variables:                    ← 模板变量（运行时替换）
  - TOOL_NAME
agentMetadata:                ← 子代理元数据（仅 agent 类型有）
  agentType: 'Explore'
  model: 'haiku'
  disallowedTools: [...]      ← 禁用的工具列表
-->
（Markdown 格式的提示词正文，含 ${VARIABLE} 模板变量）
```

关键设计特点：
- **条件组合**：不是单一长字符串，而是运行时按环境/配置动态拼接 [^26]
- **模板变量**：`${GLOB_TOOL_NAME}` 等占位符在运行时替换为实际工具名 [^27]
- **版本追踪**：每个片段标注 `ccVersion`，可通过 CHANGELOG 追踪变更 [^28]

## 与现有 wiki 的关系

- [[Everything-Claude-Code.md]] — ECC 技能集覆盖了 Skill 分类（13 个）的大部分，但 System/Tool/Agent/Data 部分未覆盖
- [[harness-engineering.md]] — 250 个提示词就是"代码是 harness"理念的生产级实现
- [[multi-agent-framework.md]] — Agent Prompts 中的 Explore/Plan/Verify 等子代理是多智能体协作的实际案例
- [[summary-learn-claude-code.md]] — learn-claude-code 的教学版 Agent Loop 对应 System Prompts 中的 conversation summarization 和 context compaction

## 引用索引

[^1]: [[sources/claude-code-system-prompts/README]] `L30-57` — 多系统提示词的设计理由和构成
[^2]: [[sources/claude-code-system-prompts/README]] `L169-181` — 行为约束类 system prompt
[^3]: [[sources/claude-code-system-prompts/README]] `L169-181` — 任务执行类 system prompt
[^4]: [[sources/claude-code-system-prompts/README]] `L182-184` — Fork 和子代理管理
[^5]: [[sources/claude-code-system-prompts/README]] `L160-164` — 记忆系统相关 prompt
[^6]: [[sources/claude-code-system-prompts/README]] `L192-194` — 模式控制相关 prompt
[^7]: [[sources/claude-code-system-prompts/README]] `L265-289` — 核心工具描述
[^8]: [[sources/claude-code-system-prompts/README]] `L271-287` — 高级工具描述
[^9]: [[sources/claude-code-system-prompts/README]] `L280-286` — 团队工具描述
[^10]: [[sources/claude-code-system-prompts/README]] `L269-339` — Bash sandbox 子规则
[^11]: [[sources/claude-code-system-prompts/README]] `L78-79` — Explore 和 Plan 子代理
[^12]: [[sources/claude-code-system-prompts/README]] `L83-86` — 创建助手类
[^13]: [[sources/claude-code-system-prompts/README]] `L89-94` — Slash 命令类
[^14]: [[sources/claude-code-system-prompts/README]] `L97-119` — AI 工具函数类
[^15]: [[sources/claude-code-system-prompts/README]] `L127-136` — SDK 参考类
[^16]: [[sources/claude-code-system-prompts/README]] `L147-153` — 概念文档类
[^17]: [[sources/claude-code-system-prompts/README]] `L350-353` — 核心技能
[^18]: [[sources/claude-code-system-prompts/README]] `L353-354` — API 构建技能
[^19]: [[sources/claude-code-system-prompts/README]] `L355-358` — 验证技能
[^20]: [[sources/claude-code-system-prompts/README]] `L362-363` — 配置技能
[^21]: [[sources/claude-code-system-prompts/README]] `L229-235` — 文件事件类
[^22]: [[sources/claude-code-system-prompts/README]] `L250-254` — 模式切换类
[^23]: [[sources/claude-code-system-prompts/README]] `L257-258` — 团队协调类
[^24]: [[sources/claude-code-system-prompts/README]] `L256-262` — 工具提醒类
[^25]: [[sources/claude-code-system-prompts/system-prompts/agent-prompt-explore]] `L1-27` — frontmatter 格式示例
[^26]: [[sources/claude-code-system-prompts/README]] `L49-55` — 条件组合设计
[^27]: [[sources/claude-code-system-prompts/system-prompts/agent-prompt-explore]] `L48-49` — 模板变量示例
[^28]: [[sources/claude-code-system-prompts/README]] `L37-39` — 版本追踪说明
