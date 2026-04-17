---
name: Agent 架构（上）
description: Agent循环/多智能体框架/系统提示词/教学脚本 — 遇到Agent架构设计、多智能体协作、系统提示词问题时参考
type: concept
updated: 2026-04-17
tags: [agent, architecture, multi-agent, system-prompts]
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# Agent 架构（上）

> 续篇：[[wiki/agent-arch-p2.md|Agent架构（下）]]
> 详细内容见 [[sources/agent-arch-p1.md]]

## Agent 核心架构

- **Agent = while 循环 + 工具分发 + 上下文管理 + 子代理**，四个机制组合，缺一不可
- 循环本质：`while stop_reason == "tool_use"` 把工具结果喂回模型，直到模型自己停下来
- 工具分发：`TOOL_HANDLERS` 字典映射 tool_name → handler，新增工具只需注册
- 上下文管理：三层压缩（micro_compact 每轮静默清理 → auto_compact 超阈值 LLM 摘要 → compact 工具模型主动触发）
- 子代理：全新 messages 列表 + 受限工具集（防止递归）+ 只返回摘要（父代理上下文保持干净）
- 技能按需加载：系统提示词只放 name+description（~100 tokens），调用时才加载完整内容
- 生产版核心洞察：循环本身不变，通过 250 个提示词片段控制循环内的决策质量

> 详细内容见 [[sources/agent-arch-p1.md]]

## s_full.py 核心结论

- **模型本身就是 Agent，代码是 harness** — 代码不决定"做什么"，只提供工具、观测和动作接口
- s_full 整合了 s01-s11 全部机制：工具分发、TodoManager、Subagent、SkillLoader、压缩管线、TaskManager、BackgroundManager、TeammateManager + MessageBus
- Lead agent 有 25 个工具运行在主线程，Teammate 只有 7 个工具运行在独立 daemon 线程
- 通信通过 MessageBus（文件系统 JSONL）和共享 TaskManager（.tasks/ 目录）
- s_full 局限：多线程共享同一工作目录，文件操作无锁，同时写同一文件会丢失数据 → s12 用 git worktree 物理隔离解决

> 详细内容见 [[sources/agent-arch-p1.md]]

## 多智能体四角色框架

- **四角色**：研究者（只读分析+验收标准）→ 实现者（可编辑）→ 测试者（跑测试+截图+报告）→ 评估者（多模态审查，守门员）
- 评估者是最严格守门员：零问题才 PASS，有任何问题都 CONTINUE
- CONTINUE 时按问题类型决定循环起点：代码 Bug → 实现者，需求理解错误 → 研究者
- 协调器禁止直接修改代码，必须通过 Agent 工具启动子代理
- 框架不硬编码智能体，动态读取 CLAUDE.md 索引发现可用资源
- 实战验证：评估者捕获了测试者和实现者都遗漏的 API 协议合规性问题，证实"守门员"价值

> 详细内容见 [[sources/agent-arch-p1.md]]

## System Prompts 六大分类

- **System Prompts（~101 个）**：行为约束、任务执行、子代理管理、记忆系统、模式控制
- **Tool Descriptions（~75 个）**：核心工具声明 + Bash sandbox 子规则（~20 个片段）
- **Agent Prompts（~34 个）**：子代理（Explore/Plan/General-purpose）、创建助手、Slash 命令、AI 工具函数
- **Data（~27 个）**：SDK 参考、概念文档，按需注入
- **Skills（~13 个）**：内置技能 prompt，slash 命令触发时加载
- **System Reminders（~40 个）**：运行时动态注入的环境状态通知
- 关键设计：条件组合（运行时按环境动态拼接）+ 模板变量 + 版本追踪

> 详细内容见 [[sources/agent-arch-p1.md]]

## s01-s12 学习路径结论

- **阶段一（s01-s02）核心循环**：while 循环 + 工具分发字典，约 150 行 Python
- **阶段二（s03-s06）状态与压缩**：TodoWrite 进度追踪、子代理隔离、技能两层按需加载、三层上下文压缩
- **阶段三（s07-s10）持久化与协作**：任务持久化到文件、后台线程、持久化 teammate + 消息通道、自然语言团队协议
- **阶段四（s11-s12）自治与隔离**：空闲轮询自动认领任务、git worktree 物理隔离避免文件冲突
- 关键数据：子代理上限 30 轮，压缩阈值 50000 tokens，保留最近 3 条工具结果

> 详细内容见 [[sources/agent-arch-p1.md]]
