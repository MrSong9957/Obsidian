---
name: Learn ShareAI（s01-s19）
description: 从零构建 Claude Code-like Agent——19 章学习笔记
type: concept
updated: 2026-05-07
tags: agent, learn-shareai
source: "[[sources/agent/learn-claude-code/s01-Agent循环]]"
---

# Learn ShareAI（s01-s19）

learn.shareai.run 全 19 章学习笔记，从零构建 Claude Code-like Agent。每一章只加一个机制。

## 核心闭环（s01-s05）

s01-s05 建立了 agent 的最小能力骨架。

### s01 Agent 循环

**一句话**：把"模型 + 工具"连接成能持续推进任务的主循环。

**核心要点**：
- Agent 本质是 `while stop_reason == "tool_use"` 循环
- 关键不在 while True，而在 **tool_result 必须重新进入消息历史**
- `messages` 不是聊天记录，而是下一轮工作输入
- 数据结构：Message、ToolResultBlock、LoopState（messages + turn_count + transition_reason）

**关键代码**：`agent_loop()` 中 messages → model → tool_result → next turn

→ 详见 [[sources/agent/learn-shareai/s01-Agent循环.md|s01 Agent 循环]]

### s02 工具调用

**一句话**：加工具只加 handler，循环不用动——dispatch map 解耦。

**核心要点**：
- dispatch map `{tool_name: handler}` 替代 if/elif
- 从 1 个 bash 扩展到 4 个工具（bash/read/write/edit）
- `safe_path()` 路径沙箱防止逃逸工作区
- 循环体与 s01 完全一致，只改 dispatch 层

→ 详见 [[sources/agent/learn-shareai/s02-工具调用.md|s02 工具调用]]

### s03 进度追踪

**一句话**：先列步骤再动手，完成率翻倍——TodoManager + nag reminder。

**核心要点**：
- TodoManager 带状态（pending/in_progress/completed），同一时间只允许一个 in_progress
- `todo` 工具加入 dispatch map，和其他工具地位平等
- nag reminder：连续 3 轮不更新 todo 时注入 `<reminder>`
- rounds_since_todo 计数器接入主循环

→ 详见 [[sources/agent/learn-shareai/s03-进度追踪.md|s03 进度追踪]]

### s04 子代理

**一句话**：大任务拆小，每个小任务干净的上下文——子代理用独立 messages[]，不污染主对话。

**核心要点**：
- 父智能体有 `task` 工具，子智能体有除 task 外的所有基础工具（禁止递归）
- 子智能体以 messages=[] 启动，跑自己的循环
- 整个消息历史直接丢弃，父智能体只收到摘要文本
- 本质是一次性外包，不是长期队友

→ 详见 [[sources/agent/learn-shareai/s04-子代理.md|s04 子代理]]

### s05 技能系统

**一句话**：不是把所有知识永远塞进 prompt，而是按需加载正确那一份。

**核心要点**：
- 两层架构：轻量目录（discovery）+ 按需正文（loading）
- SkillRegistry 统一管理：有哪些可用、某 skill 的完整内容是什么
- system prompt 放目录信息，`load_skill` 工具按需加载正文
- 与 memory/CLAUDE.md 的边界：skill=可选知识包，memory=跨会话事实，CLAUDE.md=全局规则

→ 详见 [[sources/agent/learn-shareai/s05-技能系统.md|s05 技能系统]]

### 小结

1. **循环**（s01）：模型输出 → 工具执行 → 结果回喂 → 继续推理
2. **工具扩展**（s02）：dispatch map 让加工具不改循环
3. **规划**（s03）：todo 追踪 + nag 问责
4. **隔离**（s04）：子代理用独立上下文探索
5. **知识管理**（s05）：可选知识从常驻 prompt 拆成按需加载

后续所有章节（权限、hook、压缩、任务、团队）本质上都是往这个循环里增加新的状态、分支和执行能力。

## 系统加固（s06-s08）

s06-s08 为 agent 添加了"长期稳定工作"的基础保障。

### s06 上下文压缩

**一句话**：不是尽量少字，而是让模型在更短的活跃上下文里保住继续工作的连续性。

**核心要点**：
- 三层压缩策略：大结果先落盘 + 旧结果缩短 + 整体过长再摘要
- CompactState：has_compacted、last_summary、recent_files
- micro_compact：只保留最近 3 个工具结果完整内容
- compact_history：保住目标、已完成动作、改过文件、关键决定、下一步
- 压缩 ≠ 删除，是换一种表示方式

**关键数据结构**：PersistedOutputMarker、CompactState、micro_compact boundary

→ 详见 [[sources/agent/learn-shareai/s06-上下文压缩.md|s06 上下文压缩]]

### s07 权限系统

**一句话**：任何工具调用都不应该直接执行，中间必须先过一条权限管道。

**核心要点**：
- 四步漏斗：deny rules → mode check → allow rules → ask user
- deny 优先于 allow（危险命令优先挡掉）
- 3 种模式：default（问用户）、plan（只读）、auto（安全操作自动过）
- bash 需要特殊对待（sudo、rm -rf、命令替换等最小检查）
- 拒绝计数：连续被拒绝时提示切 plan 或重新澄清

**关键数据结构**：PermissionRule（tool/behavior/path/content）、mode、decision

→ 详见 [[sources/agent/learn-shareai/s07-权限系统.md|s07 权限系统]]

### s08 Hook 系统

**一句话**：hook 不是主循环的替代品，而是主循环在固定时机对外发出的调用。

**核心要点**：
- 3 个教学事件：SessionStart、PreToolUse、PostToolUse
- 统一返回约定：0（继续）、1（阻止）、2（注入补充消息）
- hook 的 3 种作用：观察、拦截、补充
- HookRunner 统一调度，主循环只知道事件名
- 先学统一模型，再扩事件种类

**关键数据结构**：HookEvent（name + payload）、HookResult（exit_code + message）

→ 详见 [[sources/agent/learn-shareai/s08-Hook系统.md|s08 Hook 系统]]

### 小结

1. **压缩**（s06）：agent loop 开始同时维护"任务推进"和"上下文预算"
2. **权限**（s07）：工具意图不能直接执行，必须经过 allow/ask/deny 管道
3. **Hook**（s08）：不改主循环也能在关键时机插入行为，系统可扩展

三者的关系：权限决定"能不能执行"，hook 决定"执行前后还能做什么"，压缩确保"上下文不会爆"。

## 任务运行时（s13-s14）

s13-s14 解决了"执行时机"问题。

### s13 后台任务

**一句话**：并行的是等待与执行槽位，不是主循环本身。

**核心要点**：
- 慢命令（npm install、pytest、docker build）移到后台，主循环继续推进
- 最小架构：BackgroundManager（tasks + notifications + lock）
- 后台执行线负责产出结果，通知队列负责把结果送回主循环
- 主循环下一轮前 drain_notifications()，把摘要注入 messages
- 通知只放摘要，完整输出写磁盘，模型需要时再 read_file

**关键数据结构**：RuntimeTaskRecord（id/command/status/preview/output_file）、Notification

**与任务系统的边界**：
- s12 的 task = 工作目标（要做什么、谁依赖谁）
- s13 的 background task = 运行中的执行单元（哪个命令在跑、跑到什么状态）

→ 详见 [[sources/agent/learn-shareai/s13-后台任务.md|s13 后台任务]]

### s14 定时调度

**一句话**：后台任务是在"等结果"，定时调度是在"等开始"。

**核心要点**：
- cron 表达式定义定时规则，ScheduleRecord 记住未来何时开工
- 后台检查器每分钟看一次是否匹配，匹配就发通知
- 通知最终回到同一条主循环，不是另一套 agent
- recurring 控制是否重复触发，durable 控制是否落盘持久化
- last_fired_at 防止短时间内重复触发

**关键数据结构**：ScheduleRecord（id/cron/prompt/recurring/durable/last_fired_at）

**统一模式**：后台任务完成通知 + 定时调度触发通知，都走通知队列，在下次模型调用前统一注入。

→ 详见 [[sources/agent/learn-shareai/s14-定时调度.md|s14 定时调度]]

### 小结

1. **异步执行**（s13）：慢工作在后台跑，主循环保持单主线
2. **定时触发**（s14）：未来任务先记下来，时间到了再放回主循环

核心心智：定时调度不是另一套 agent，而是多了一种"开始入口"，最终仍回到同一条主循环。

> 注：s09-s12 为旧版 12 章的紧凑内容（持久化队友、协商协议、自组织代理、Worktree 绑定），s15-s18 提供了更详细的扩展版本。

## 多 Agent 平台（s15-s19）

s15-s19 完成了从单 agent 到多 agent 平台的跃迁。

### s15 Agent 团队

**一句话**：teammate 的核心不是"多一个模型调用"，而是"多一个长期存在的执行者"。

**核心要点**：
- 与 s04 subagent 的根本区别在生命周期：subagent 一次性消失，teammate 长期存在
- 最小三件套：名册（config.json）+ 邮箱（JSONL 收件箱）+ 独立循环
- TeammateManager 管理 spawn/status，MessageBus 处理 send/drain
- 每个队友有自己的 messages、inbox、agent loop

**关键区分**：
| 机制 | 生命周期 | 关键边界 |
| --- | --- | --- |
| subagent | 干完就结束 | 隔离探索性上下文 |
| runtime task | 跑完或取消就结束 | 慢任务稍后回来 |
| teammate | 可反复接任务 | 有名字、有邮箱、有独立循环 |

→ 详见 [[sources/agent/learn-shareai/s15-Agent团队.md|s15 Agent 团队]]

### s16 团队协议

**一句话**：团队协议的核心，是"请求-响应 + request_id + 状态表"。

**核心要点**：
- 自由文本无法稳定处理"批准/拒绝"，需要结构化协议
- ProtocolEnvelope（type + request_id + payload）区分于普通 MessageEnvelope
- 两种协议模板：优雅关机 + 计划审批，底层是同一个 request-response FSM
- RequestRecord 落盘到 `.team/requests/`，保证可恢复
- 状态机：pending → approved / rejected / expired

→ 详见 [[sources/agent/learn-shareai/s16-团队协议.md|s16 团队协议]]

### s17 自主代理

**一句话**：自治不是让 agent 乱跑，而是让它在清晰规则下自己接住下一份工作。

**核心要点**：
- WORK → IDLE 双阶段循环：idle 时看邮箱 + 看任务板
- Claimable Predicate：pending + 无 owner + 无 blockedBy + 角色匹配，缺一不可
- 认领是原子动作（claim_lock 防重复抢）
- claim_source: "auto" vs "manual"，事件日志记录认领过程
- 身份重注入：上下文压缩后补回 who-am-I 块

→ 详见 [[sources/agent/learn-shareai/s17-自主代理.md|s17 自主代理]]

### s18 Worktree 隔离

**一句话**：任务系统管"做什么"，worktree 系统管"在哪做且互不干扰"。

**核心要点**：
- 任务板 + worktree 注册表通过 task_id 绑定
- WorktreeRecord 不只是路径映射，还包含 last_entered_at、last_command_preview
- closeout 动作：keep（保留供审阅）或 remove（删除并完结任务）
- worktree_state（active/kept/removed）与 task status 分开管理
- events.jsonl 记录完整生命周期，支持崩溃恢复

→ 详见 [[sources/agent/learn-shareai/s18-Worktree隔离.md|s18 Worktree 隔离]]

### s19 MCP 与插件

**一句话**：MCP 的本质，不是协议名词堆砌，而是把外部工具安全、统一地接进你的 agent。

**核心要点**：
- 三层：plugin manifest（发现）→ MCP server（连接）→ MCP tool（调用）
- 工具名前缀 `mcp__{server}__{tool}` 避免命名冲突
- 统一路由器：本地工具走 native handler，MCP 工具走 MCP client
- **MCP 工具也必须走同一条权限管道**，不能绕开
- 结果标准化回同一条 tool_result 格式

→ 详见 [[sources/agent/learn-shareai/s19-MCP与插件.md|s19 MCP 与插件]]

### 小结

1. **团队**（s15）：长期存在的队友，有名字、邮箱、独立循环
2. **协议**（s16）：结构化协作（请求-响应 + request_id + 状态机）
3. **自治**（s17）：空闲队友自己扫描任务板、按角色认领
4. **隔离**（s18）：每个任务独立 worktree 目录，互不污染
5. **扩展**（s19）：外部工具通过 MCP 安全统一接入

整体演进线：s15 建实体 → s16 建规则 → s17 加自治 → s18 加隔离 → s19 向外开放。
