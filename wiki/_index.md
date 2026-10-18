---
type: index
updated: 2026-10-18
---

# Wiki 索引

| 页面 | 说明 | 标签 |
|------|------|------|
| [[wiki/agent-arch\|Agent 架构]] | Agent 循环、多智能体框架、Harness 工程方法论 | agent, architecture, multi-agent, harness |
| [[wiki/claude-code\|Claude Code 配置与工作流]] | settings.json/CLAUDE.md/rules/记忆系统/插件/Hooks/子代理/MCP | claude-code, settings, claudemd, memory, rules, hooks, subagents, mcp |
| [[wiki/claude-code-setup\|Claude Code 当前配置清单]] | 本机完整配置 + 自检清单（插件/Hooks/状态栏/MCP/技能/代理/规则） | claude-code, setup, plugins, hooks, mcp, config |
| [[wiki/claude-md-base\|CLAUDE.md 双层架构模板]] | 全局（karpathy 行为准则）+ 项目级（开发规范与测试实现标准）双层 CLAUDE.md，编排职责已迁出 | claude-md, template, karpathy |
| [[wiki/大脑模型-v3\|大脑模型 v3]] | Superpowers + goal 版——S/A/B/C 任务分级、Skill 调度、返回门禁、L1/L2/L3 测试证据、Security Testing Gate、verify:full | agent, orchestration, decision-control, superpowers, testing-gates, goal |
| [[wiki/大脑模型-v4\|大脑模型 v4]] | grill-me + goal 版——Supervisor 职责边界、Matt Skills 地图、/goal 自治交付、返回门禁与 delivery decision | agent, orchestration, decision-control, grill-me, goal, matt-skills |
| [[wiki/大脑模型-v4-规格驱动版\|大脑模型 v4 规格驱动版]] | v4 演进稿——统一流程（规格→分支→/goal→用户批准→交付）、五节规格实战法收编、保留 4 个 Skill、返回门禁与 L1-L3 验证范围 | agent, orchestration, decision-control, spec-driven, goal, supervisor |
| [[wiki/cli-tools-productivity\|十个提升生产力的 CLI 工具]] | 10 个提升开发生产力的 CLI 工具推荐 | cli, productivity |
| [[wiki/container-python-setup\|Docker 容器内安装 Python 版本]] | 容器网络受限环境下安装 Python 版本的经验路线图 | docker, python, networking |
| [[wiki/dev-practice\|开发实践]] | TDD/代码质量/调试方法论/提示词技巧/协作提示词 | dev, tdd, debugging, prompt, code-quality |
| [[wiki/ecc\|ECC 快速指南]] | ECC 插件安装方式、Hooks 配置档位、隐藏组件、故障排除 | ecc, hooks, rules, agents, installation |
| [[wiki/email-assistant\|Email Assistant]] | Email Assistant 项目核心文档 | email, python, assistant |
| [[wiki/frontend\|前端与 UI]] | UI 设计原则/终端 UI 测试/Playwright 自动化/技术栈策略 | frontend, ui, testing, playwright |
| [[wiki/git-quick-reference\|Git 快速参考]] | Git 常用命令速查 | git, cli, reference |
| [[wiki/learn-shareai\|Learn ShareAI]] | 从零构建 Claude Code-like Agent——19 章学习笔记 | agent, learn-shareai |
| [[wiki/mental-models\|思维模型]] | 跨学科思维模型框架（芒格 100 模型） | mental-models, decision-making, cognition |
| [[wiki/new-project-setup\|新项目开发环境初始化]] | Docker 容器启动、镜像配置、首次配置、Obsidian 记忆链接 | docker, setup, claude-md, configuration |
| [[wiki/obsidian\|Obsidian]] | Obsidian CLI 工具使用/知识仓库管理 | obsidian, knowledge-management |
| [[wiki/opencode-bridge\|OpenCode 桥接与 MiMo-Code]] | OpenCode plugin 行为/ECC 桥接/MiMo-Code 架构 | opencode, mimocode, claude-code, hooks, bridge |
| [[wiki/token-management\|Token 管理]] | Token 使用与成本优化 | token, cost, optimization |
| [[wiki/yazi\|Yazi 文件管理器]] | Yazi 配置经验——主题、配色、Windows 路径 | cli, yazi, config, theme |
| [[wiki/zhipu-mcp\|智谱 MCP]] | 智谱 MCP 服务器配置速查/Web 抓取缓存问题 | zhipu, mcp, web-scraping |
| [[wiki/wiki-query-deploy\|wiki-query MCP 部署]] | wiki-query MCP 安装/配置/跨平台分发（GitHub Actions）与踩坑 | wiki-query, mcp, deploy, github-actions, release |
| [[wiki/mimocode-prompt-architecture\|MiMo Code 提示词注入架构]] | MiMo Code 系统提示词 4 层管道、权限系统、工具限制 | mimocode, prompt, architecture |
| [[wiki/claude-code-prompt-architecture\|Claude Code 提示词架构]] | Claude Code 10 层治理架构、系统提示词构造、自动模式分类器 | claude-code, prompt, architecture |
| [[wiki/mimocode-memory-system\|MiMo Code 记忆系统架构]] | 三层记忆文件结构、写入权限、搜索机制、提升逻辑 | mimocode, memory, architecture |
| [[wiki/streaming-architecture\|流式架构模式]] | EventBus 模式、渲染器分离、压缩集成、4 层流式管道 | streaming, architecture, event-bus |
| [[wiki/mimocode-ui-rendering\|MiMo Code UI 渲染管道]] | 屏幕缓冲区 + diff 系统、布局计算、光标定位 | mimocode, ui, rendering, terminal |
| [[wiki/how-openai-uses-codex\|How OpenAI uses Codex]] | OpenAI 7 类 Codex 用法 + 6 条最佳实践入口；细节见 sources 全量中译 | codex, openai, coding-agent, workflow, best-practices |
| [[wiki/how-anthropic-teams-use-claude-code\|How Anthropic teams use Claude Code]] | Anthropic 10 团队案例入口——含非技术岗；细节见 sources 全量中译 | claude-code, anthropic, case-study, workflow |
| [[wiki/大模型API上下文结构\|大模型 API 上下文结构]] | 四种消息角色 + tools 字段、API 无状态原理与 Harness 状态保存 | ai-agent, context-engineering, llm-api, messages, tool-calling |
| [[wiki/Agent核心循环\|Agent 核心循环]] | 组装请求、提取模型消息、执行工具并回传结果的 Python 完整循环 | ai-agent, context-engineering, tool-calling, python |
| [[wiki/Agent工具调用的API循环\|Agent 工具调用的 API 循环]] | 模型决策与 Harness 执行的职责边界、tool_call_id 配对、串并行判断 | ai-agent, tool-calling, react, context-engineering, llm-api |
| [[wiki/Agent上下文布局与本地模型工具调用\|Agent 上下文布局与本地模型工具调用]] | 稳定前缀 + 动态轨迹、预算压缩伪代码、本地模型工具调用与首 token 延迟测量 | ai-agent, context-engineering, kv-cache, tool-calling, local-llm |
| [[wiki/API消息如何变成模型输入\|Chat Template：API 消息如何变成模型输入]] | 消息到 token 的排列规则；工具结果回传格式、思考内容保留与缓存前缀影响 | llm, chat-template, tokenization, tool-calling, kv-cache |
| [[wiki/KV-Cache友好的上下文设计与注意力机制\|KV Cache 友好的上下文设计与注意力机制]] | Q/K/V 与因果遮罩、缓存复用原理、三条上下文布局原则、注意力可视化 | context-engineering, kv-cache, attention, transformer, prompt-cache |
| [[wiki/KV Cache、Prompt Cache 与缓存友好的 Agent 架构\|KV Cache、Prompt Cache 与缓存友好的 Agent 架构]] | 两层缓存区分、Codex 上下文组装顺序与源码证据、缓存边界与子 Agent 前缀对齐、可编辑/可组合 KV 研究边界 | context-engineering, kv-cache, prompt-cache, agent, architecture, codex |
| [[wiki/Obsidian仓库大脑模型\|Obsidian 仓库大脑模型]] | 网页 ChatGPT 仓库管理大脑提示词：三层管理、ADD/UPDATE/NOOP/CONTRADICT 裁决、/goal 与 CONTINUE/DONE 状态机 | brain-model, decision-control, agent-orchestration, obsidian, wiki-management, goal |
| [[wiki/1个月把技能练到前1%\|1个月技能精通3-2-1法则]] | Dan Koe提出的技能快速精通框架，通过3种资源输入、2个输出出口、1个月验证周期，帮助区分真兴趣与伪需求，实现从学习到实践的闭环 | skill-mastery, learning-framework, output-driven, deliberate-practice, personal-growth |
| [[wiki/从Prompt到Graph\|从 Prompt 到 Graph：Agent 工程关注范围如何逐层扩大]] | 理解 AI 工程如何从优化提示词，演进到管理 Agent 循环、长期自治和执行图。 | agent, prompt-engineering, context-engineering, harness, loop-engineering, graph-engineering |
| [[wiki/第一性原理\|第一性原理：从底层事实重新构建解决方案]] | 通过事实拆解、假设验证和重新推导，突破经验限制，解决复杂问题。 | first-principles, thinking, decision-making, problem-solving |
| [[wiki/工程化训练\|工程化训练]] | 工程化流程训练图——brainstorming 到 finishing 的全链路（含 Security Testing Gate 分支）、软件工程生命周期与测试需求文档（Test Spec）四要素写法 | engineering, test-design, test-spec, brainstorming, workflow |
| [[wiki/工具定义、按需加载与提示工程消融实验\|工具定义、按需加载与提示工程消融实验]] | 设计易于正确使用的工具说明，理解按需加载如何保留缓存，并通过对照实验检验提示词效果。 | tool-use, tool-search, context-engineering, prompt-caching, ablation-study, tau-bench |
| [[wiki/工作流编排\|工作流编排：用确定性流程控制 LLM 与工具]] | 判断什么时候该用单次调用、固定工作流，什么时候才需要自主 Agent。 | agent, workflow, orchestration, harness, llm, adaptation-layer |
| [[wiki/构建有效Agent\|构建有效 Agent：简单、透明、工具友好，并按真实任务选模型]] | 理解 Agent 设计的三个核心原则，以及模型选型真正该看哪些指标。 | agent, harness, aci, model-selection, reasoning, tool-calling |
| [[wiki/观察空间与动作空间决定Agent能力\|观察空间与动作空间决定 Agent 能看什么、能做什么]] | 理解 Agent 如何通过观察和行动接口连接外部世界，以及为什么扩展接口能直接提升任务能力。 | agent, observation-space, action-space, harness, tools, context |
| [[wiki/Skill 编写方法、上下文位置与缓存成本\|Skill 编写方法、上下文位置与缓存成本]] | 把经验写成可执行 Skill 的四部分结构、真实任务迭代法、各框架加载位置与 emit-once 缓存辨析 | agent-skills, prompt-engineering, context-engineering, prompt-caching |
| [[wiki/Skills 与工具的协作方式及实验设计\|Skills 与工具的协作方式及实验设计]] | Skill/工具/脚本职责划分、通用执行器思路，及论文转 PPT 与个人写作两个实验方案 | agent-skills, tool-use, progressive-disclosure, skill-evaluation |
| [[wiki/上下文压缩：减少信息负担，让已有知识更容易使用\|上下文压缩：减少信息负担，让已有知识更容易使用]] | 压缩三动机（容量/Context Rot/提前收尾）、提炼保留原则与上下文学习不等于长期记忆 | ai-agent, context-engineering, context-compression, context-rot, in-context-learning |
| [[wiki/上下文压缩与缓存复用\|上下文压缩与缓存复用：六种策略及其取舍]] | 压缩对缓存的影响、六种压缩策略取舍、实验统计口径辨析与自适应窗口三机制 | ai-agent, context-compression, kv-cache, prompt-cache, context-engineering |
| [[wiki/上下文管理：分层压缩、信息保留与子 Agent 隔离\|上下文管理：分层压缩、信息保留与子 Agent 隔离]] | 五种可组合处理方式、压缩保住关键知识的四原则、进度记录与子 Agent 隔离的收益边界 | ai-agent, context-engineering, context-compression, subagents, information-management |
| [[wiki/上下文工程\|上下文工程：决定 Agent 每一步能看到什么]] | 解释为什么 Agent 能力高度依赖上下文，以及 Harness 如何为每次模型调用重建有效信息。 | agent, context-engineering, harness, react, llm |
| [[wiki/第二章：上下文工程 章节结束的深度思考题\|第二章：上下文工程 章节结束的深度思考题]] | 九道章末思考题与解答——缓存友好压缩、思维链保留、元信息可靠性、提示词防熵增、Skills 元认知与上下文布局 | context-engineering, kv-cache, prompt-cache, agent-skills, reasoning, exercises |
| [[wiki/十二种深度思考训练法\|十二种深度思考训练法：从表象到本质的认知工具集]] | 提供12套可刻意练习的本质思考工具，涵盖溯源、拆解、模式识别等，助用户建立结构化洞察力。 | deep-thinking, cognitive-tools, problem-solving, mental-models, decision-making |
| [[wiki/时空可组合性\|时空可组合性：让动态系统安全地变化]] | 提炼 Cordis 如何用可撤销副作用和响应式依赖，让组件能够安全加入、移除、替换和重新组合。 | cordis, composability, runtime, hmr, agent-harness |
| [[wiki/提示词注入威胁\|提示注入：上下文安全的核心威胁]] | 理解恶意文字怎样诱使 Agent 越权，怎样分层防御，以及如何设计实验检查效果。 | prompt-injection, agent-security, context-engineering, tool-use, memory-poisoning |
| [[wiki/系统提示词设计\|系统提示词设计：明确要求、组织结构与执行流程]] | 写清 Agent 的职责、表达方式和工作步骤，减少理解歧义与执行遗漏。 | prompt-engineering, system-prompt, agent, workflow, xml |
| [[wiki/系统提示词中的业务规则、示例与缓存设计\|系统提示词中的业务规则、Few-shot 示例与缓存设计]] | 把业务判断写成可执行规则，用少量示例说明边界，并兼顾跨请求缓存复用。 | prompt-engineering, business-rules, few-shot, prompt-caching, agent |
| [[wiki/知识复利三阶跃迁模型\|知识复利三阶跃迁模型]] | 将碎片信息转化为个人资产的三级加工框架：从原文摘录到概念转译再到体系缝合，解决“收藏即遗忘”的知识管理失效问题 | knowledge-compound-interest, zettelkasten, second-brain, anti-collection-fallacy, personal-knowledge-base, note-taking |
| [[wiki/自主-Agent\|自主 Agent：用 ReAct 循环动态决定下一步]] | 理解自主 Agent 如何规划、使用工具、处理失败，以及什么时候该和工作流混合使用。 | agent, autonomous-agent, react, workflow, harness, orchestration |
| [[wiki/Agent-工具设计\|Agent 工具设计：连接世界、扩大能力，也扩大风险]] | 理解 Agent 工具的类型、调用流程，以及通用工具和专用工具该怎么选。 | agent, tools, tool-calling, mcp, sandbox, aci |
| [[wiki/Agent Skills\|Agent Skills：按任务加载操作说明]] | Skills 三层渐进式加载、description 触发边界、用户指定与模型选择两种触发方式，及其与提示缓存的配合 | agent-skills, prompt-engineering, progressive-disclosure, context-engineering, prompt-caching |
| [[wiki/Agent 状态栏：让模型直接掌握任务进度和调用次数\|Agent 状态栏：让模型直接掌握任务进度和调用次数]] | 为什么完整历史仍不够、框架怎样维护状态摘要，以及状态栏减少重复统计与超限操作的作用 | ai-agent, context-engineering, state-tracking, tool-use |
| [[wiki/Agent 状态栏：信息构成、消息位置与缓存复用\|Agent 状态栏：信息构成、消息位置与缓存复用]] | 状态栏三类信息构成、user 消息承载方式，以及末尾追加对 KV Cache 复用的意义 | ai-agent, context-engineering, state-tracking, kv-cache |
| [[wiki/Agent 状态栏：更新策略、缓存成本与可靠维护\|Agent 状态栏：更新策略、缓存成本与可靠维护]] | 替换与追加两种状态更新的费用权衡、五种实用状态信息与维护准则 | ai-agent, context-engineering, prompt-cache, state-tracking, error-recovery |
| [[wiki/Agent 记忆分层与四种存储格式\|Agent 记忆分层与四种存储格式]] | 轨迹、长期记忆与业务状态三层职责，Simple Notes / Enhanced Notes / JSON Cards / Advanced JSON Cards 四种存储格式的取舍与实验 3-2 定性观察 | agent-memory, memory-architecture, memory-storage, json, trajectory |
| [[wiki/Agent-护栏\|Agent 护栏：把安全控制下沉到模型之外]] | 用上下文、执行、数据三层护栏限制 Agent 的可见信息、可执行动作和最终数据权限。 | agent, guardrails, security, prompt-injection, human-in-the-loop, access-control |
| [[wiki/Agent-上下文\|Agent 上下文决定它能看到什么]] | 理解 Agent 每次决策依赖哪些信息，以及上下文缺失为什么会导致错误行动。 | agent, context, llm, tool-calling, reasoning, memory |
| [[wiki/Agent的5个通用设计模式与第一章总框架\|Agent 的 5 个通用设计模式与第一章总框架]] | 用 5 个可复用模式理解 Agent 如何审查、加载信息、保存状态、验证修改和安全演进。 | agent, harness, design-patterns, react, guardrails, context-engineering |
| [[wiki/Agent驱动Rust学习\|Agent驱动Rust学习：以Codex为地图构建Coding Agent Runtime能力]] | 面向Agent/Coding Agent工程师的Codex源码学习路线，强调“用Codex学架构，让Rust被迫升级” | codex, rust, agent-runtime, coding-agent, just-in-time-learning, mini-codex, openai |
| [[wiki/AI-Agent-的最小结构\|AI Agent 的最小结构：Model、Context、Tools 与 Environment]] | 理解现代 Agent 由什么组成，以及 Agent 与外部环境之间如何形成行动闭环。 | agent, llm, context, tools, harness, environment |
| [[wiki/AI时代知识管理防御体系与核心书单\|AI时代知识管理防御体系与核心书单]] | 构建“信源过滤-深度加工-输出验证”三层防线抵御AI噪声，并附6本搭建底层认知结构的必读书单及适用场景 | pkm, ai-era, information-defense, book-list, cognitive-framework, anti-algorithm-bias, engineering-thinking |
| [[wiki/architecture-migration-semantic-boundary\|架构迁移语义边界准则]] | 从"renderer 控制视觉"迁移到"message store 控制语义"时,必须区分视觉控制指令与真实数据,否则旧时代的视觉指令会污染新时代的语义数据 | architecture, migration, semantic-boundary, tui, message-store, renderer, anti-pattern |
| [[wiki/Claude-Code-Statusline-部署与故障恢复\|Claude Code Statusline 部署与故障恢复]] | 状态栏运维手册——canonical 源为准的部署结构、Linux 容器实测恢复流程、边界矩阵验证与快速排障（2026-10-18 起不集成 Safety-net） | claude-code, statusline, ops |
| [[wiki/CLI 备用屏架构设计\|CLI 备用屏架构设计]] | DeepSeek Harness CLI 备用屏架构设计——启动即备用屏、职责划分与受限布局、聊天数据与画面分离、缩放重建七步流程、/trace 与退出恢复、三层验收门禁 | cli, terminal, alternate-screen, ink, rendering, deepseek-harness, design |
| [[wiki/终端重复绘制：原因与备用屏方案\|终端重复绘制：原因与备用屏方案]] | 终端重复绘制的根因与备用屏方案——旧画面擦除位置判断错误、全时备用屏、缩放清屏完整重画与真实终端验收要点 | cli, terminal, alternate-screen, ink, rendering, deepseek-harness, design |
| [[wiki/Harness-工程\|Harness 工程：让 Agent 从“能做”变成“可靠地做”]] | 理解 Harness 如何通过上下文、工具、约束、验证和纠正，让 Agent 在生产环境中可靠运行。 | agent, harness, context, tools, reliability, verification |
| [[wiki/Harness五要素\|Harness 五要素：Agent 构建阶段的工程视角]] | 理清 Agent 总公式与 Harness 五要素的关系，以及各要素在全书中的位置。 | agent, harness, context-engineering, tools, validation, security |
| [[wiki/inline-grid-footer-render\|Inline Grid Footer 渲染架构]] | cell 级双缓冲 + 绝对坐标定位彻底解决 inline 模式 resize 堆叠——footerTopRow 计算、resize 链路、emit yBias、InlineGridRenderer 组件职责 | tui, inline-renderer, grid, double-buffer, resize, footer, conhost, reflow, absolute-positioning |
| [[wiki/karpathy-methodology\|Karpathy 方法论]] | Andrej Karpathy 的 5 套核心方法论 — 痛苦驱动学习、解释倒逼输入、代码至上、极小闭环、提示词优化法 | karpathy, learning, methodology, ai-workflow, programming, prompt |
| [[wiki/拼贴编程法\|拼贴编程法]] | Karpathy 拼贴编程法——半代码半注释写清每步数据类型与状态变化，Agent 生成具体代码，人掌握数据流与关键规则并验收 | karpathy, programming-method, ai-workflow, code-comments, agent-collaboration |
| [[wiki/可执行记忆与记忆内容的三种类型\|可执行记忆与记忆内容的三种类型]] | User as Code 用对象存状态、函数执行规则（日志+检查点），情景/语义/程序三类认知记忆与三套分类体系的组合使用 | agent-memory, executable-memory, user-as-code, knowledge-representation, cognitive-science |
| [[wiki/LLM是Agent的决策核心\|LLM 是 Agent 的决策核心，能力通过三条路径持续更新]] | 理解模型如何驱动 Agent 决策，以及能力如何从临场适应逐步沉淀为持久能力。 | agent, llm, harness, model-as-agent, learning, reinforcement-learning |
| [[wiki/Mem0 与 Memobase\|Mem0 与 Memobase：记忆更新、检索和用户画像的设计取舍]] | Mem0 v2 写入时决策与 v3 只追加+查询时判断的演进、官方评测数字的来源辨析，Memobase 画像+事件时间线与缓冲批处理 | agent-memory, mem0, memobase, memory-retrieval, memory-architecture |
| [[wiki/mi-code-tui-experience\|MiCode TUI 开发经验汇总]] | mi-code 项目从 Ink/React 迁移到自研差量渲染引擎的实战记录——架构演进、渲染优化与踩坑经验 | tui, mimocode, renderer, ink, migration, experience |
| [[wiki/mimocode-mcp-config\|MiMoCode MCP 配置指南]] | MiMoCode 与 Claude Code 的 MCP 配置差异、wiki-query 接入流程、常见问题排查 | mimocode, mcp, wiki-query, configuration, troubleshooting |
| [[wiki/Obsidian-AI-Skill-实测清单\|Obsidian AI Skill 实测清单：9大Agent联动工具与本地知识库安全实践]] | 提供9个高星Obsidian AI Skill实测排名、安装命令及Git安全回滚策略，解决本地知识库AI自动化与安全管控问题。 | obsidian, ai-skill, agent, knowledge-management, tool-list, local-first, git-safety |
| [[wiki/pr-merge-local-sync\|PR merge 后本地分支的安全同步]] | 本地 master 有未推送提交 + 工作区有脏文件时,PR squash merge 后不能机械用 pull/reset --hard;先看拓扑和最终树内容,再做最小受控同步——保住脏文件、对齐到 merge commit | git, pr-merge, squash-merge, local-sync, dirty-worktree, safe-reset |
| [[wiki/ReAct-循环\|ReAct 循环：Agent 如何通过轨迹持续思考和行动]] | 理解 Agent 如何把模型、上下文和工具串成“想→做→看”的执行闭环。 | agent, react, trajectory, harness, tool-calling, context |
| [[wiki/terminal-inline-renderer-principles\|Terminal Inline Renderer 架构原则]] | Application-driven Layout 底层约束——DECAWM、wordWrap、宽度计算、ANSI 处理等不变原则 + 架构演进记录（cursorUp 覆写 → cell 级双缓冲） | tui, inline-renderer, decawm, wordwrap, ansi, cursor, layout-engine, terminal, architecture-principles, grid, double-buffer |
| [[wiki/记忆能力评估\|记忆能力评估：基础回忆、多会话检索与主动服务]] | 三层记忆能力框架（基础回忆/多会话检索/主动服务）、八项能力归纳与 LLM-as-a-judge 逐次更新评估设计 | agent-memory, memory-evaluation, methodology, locomo |
| [[wiki/记忆压缩与本地日志脱敏\|记忆压缩与本地日志脱敏]] | 三层记忆整理（重要性筛选/聚类合并/抽象规律）、PII 日志脱敏四步流程与 Ollama 本地模型、"原文报告"数字的引用纪律 | agent-memory, memory-compression, log-sanitization, pii, ollama, qwen3 |
| [[wiki/用户记忆与知识库\|用户记忆与知识库：让 Agent 跨会话记住信息]] | 用户记忆与知识库的分工、对话提取记忆的三条规则（选择性/抽象化/结构化）、RAG 与双层记忆架构预告 | agent-memory, knowledge-base, rag, memory-extraction |
| [[wiki/Rust 程序设计语言/Rust 入门：核心价值、开发工具与学习方法\|Rust 入门：核心价值、开发工具与学习方法]] | Rust 编译期检查（所有权/unsafe）减少底层错误、零成本抽象边界、Cargo/rustfmt/rust-analyzer 工具分工与官方书学习方法（含 Ferris 标记含义） | rust, memory-safety, systems-programming, cargo, learning-guide |
| [[wiki/Docker容器开发方案\|Docker 容器开发方案]] | Windows Docker Desktop（WSL2）数据盘迁移 D:\AgentLab、agentlab-base 基础镜像＋项目镜像分层、Rust MCP 以预编译二进制运行于容器、Compose 持久卷与缓存规划、多容器共享 Obsidian 仓库 /vault、Agent 优先执行（命令→桌面自动化→用户兜底） | Docker, WSL2, Agent, 缓存管理 |
| [[wiki/5步学习法第1步：预提问，带着问题去学习\|5步学习法第1步：预提问，带着问题去学习]] | 学习前先提问激活已有知识网络，让大脑进入搜索模式，提升输入效率 | pre-questioning, active-learning, reading-strategy, knowledge-connection, first-principles |
| [[wiki/5步学习法第2步：最小输入，一次只学一个知识点\|5步学习法第2步：最小输入，一次只学一个知识点]] | 利用大脑工作记忆容量有限的规律，用最小知识单元和块状学习替代贪多求快 | minimal-input, working-memory, focused-learning, chunk-learning, first-principles |
| [[wiki/5步学习法第3步：主动提取，合上书才是学习的开始\|5步学习法第3步：主动提取，合上书才是学习的开始]] | 用回忆和讲解代替被动输入，让大脑真正建立并强化神经连接 | active-recall, feynman-technique, learning-method, memory-consolidation, first-principles |
| [[wiki/5步学习法第4步：间隔复习，用最少时间获得最牢记忆\|5步学习法第4步：间隔复习，用最少时间获得最牢记忆]] | 利用艾宾浩斯遗忘曲线的规律，在最佳时间点复习，用最少时间让记忆永久固化 | spaced-repetition, forgetting-curve, memory-consolidation, review-method, active-learning |
| [[wiki/5步学习法第5步：迁移应用，让知识真正变成你的能力\|5步学习法第5步：迁移应用，让知识真正变成你的能力]] | 学习的终极目的是解决从未见过的新问题——迁移能力的含义、培养方法与完整学习闭环 | transfer-learning, knowledge-application, metacognition, learning-system, first-principles |
| [[wiki/90%的人都在犯的5个学习误区，正在拖垮你的努力\|90%的人都在犯的5个学习误区，正在拖垮你的努力]] | 90%的人在犯的5个学习错误，每个都违背大脑底层规律 | learning-myths, mistake-analysis, first-principles, study-habits, brain-science |
| [[wiki/公理1：大脑没有硬盘，学习就是给神经元牵线搭桥\|公理1：大脑没有硬盘，学习就是给神经元牵线搭桥]] | 学习的本质是建立神经元之间的突触连接而非存储信息，附类比法与举例法两个实操方法 | neural-connection, memory-consolidation, analogy, active-learning, first-principles |
| [[wiki/公理2：被动输入都是无用功，主动提取才是学习本身\|公理2：被动输入都是无用功，主动提取才是学习本身]] | 突触连接强度与主动提取次数正相关，与被动输入次数几乎无关，附笔记法与费曼技巧 | active-recall, feynman-technique, note-taking, memory-consolidation, first-principles |
| [[wiki/公理3：你的大脑只能同时处理4件事，贪多必忘\|公理3：你的大脑只能同时处理4件事，贪多必忘]] | 大脑只能同时处理4±1个信息块，超出就会溢出——机制、常见误区与组块化/单任务两个方法 | working-memory, chunking, single-task, block-learning, first-principles |
| [[wiki/公理4：大脑只会记住有用的知识，孤立信息会被自动删除\|公理4：大脑只会记住有用的知识，孤立信息会被自动删除]] | 大脑只保留与已有知识网络建立有意义连接的信息，孤立信息会被删除——机制与提问/关联/应用三步骤 | meaning-encoding, memory-consolidation, knowledge-connection, learning-method, first-principles |
| [[wiki/公理5：睡眠不是浪费时间，是学习的最后一步\|公理5：睡眠不是浪费时间，是学习的最后一步]] | 清醒时学习只是打草稿，睡眠才是大脑真正把知识砌成墙的过程——睡眠固化机制与三条睡眠法则 | sleep-consolidation, memory, learning-method, first-principles, study-habits |
| [[wiki/用第一性原理学数学\|用第一性原理学数学]] | 从定义出发理解数学底层逻辑，替代背公式和盲目刷题 | math-learning, first-principles, definition-driven, logical-thinking, active-learning |
| [[wiki/用第一性原理学文科\|用第一性原理学文科]] | 用认知科学的底层规律重新设计文科学习方式，替代死记硬背 | liberal-arts, learning-method, knowledge-system, feynman-technique, first-principles |
| [[wiki/用第一性原理学物理\|用第一性原理学物理]] | 基于底层原理推导物理公式、拆解题目而非死记硬背或盲目刷题 | physics-learning, first-principles, derivation, active-learning, error-analysis |
| [[wiki/用第一性原理学英语\|用第一性原理学英语]] | 基于认知科学规律重构英语学习方式，替代死记硬背的传统路径 | english-learning, learning-method, spaced-repetition, active-recall, first-principles |
| [[wiki/从零实现Agent\|从零实现Agent]] | 34 步迭代表从零实现终端编程 Agent——能聊天、会用工具、能改代码、稳定持续工作、可扩展发布（Rust，主参考 Codex 与 DeepSeek Harness 源码） | agent, coding-agent, rust, roadmap, tool-calling, codex, deepseek-harness |
| [[wiki/Agent工具结果与上下文压缩：裁剪、摘要与证据补读\|Agent 工具结果与上下文压缩]] | 工具结果先代码减量/落盘/单条摘要再进上下文、历史到阈值集中压缩、证据不足按需补读原文，附 DeepSeek Harness/Codex/Claude Code 三家做法对比 | agent, context-management, tool-output, summarization, prompt-cache, rag |
| [[wiki/假说驱动：先给答案再验证，而不是先收集全部信息\|假说驱动：先给答案再验证]] | 三步把模糊难题变成可验证猜想（第一判断、议题树拆解、优先证伪），靠证伪快速逼近真相 | hypothesis-driven, problem-solving, decision-making, verification, consulting |
| [[wiki/规格文档实战法\|规格文档实战法]] | 学习编程主方法——每个功能一份规格文档：五步流程（想清楚→预测→实测→关键判断→坑记录），规格是母版、代码随用随生成 | spec-driven, learning-method, programming-method, ai-era, knowledge-management |
| [[wiki/多厂商LLM接入架构\|多厂商LLM接入架构]] | Rust CLI 多厂商 LLM 接入——Harness 厂商适配器 + Codex 请求分层，五类职责划分、一次对话数据流、错误/重试/断流/取消规则与验收；对应从零实现 Agent 第 28 步 | rust, llm, provider, adapter, retry, streaming, codex, deepseek-harness |
| [[wiki/终端UI方案：Codex复用与首版规格\|终端UI方案：Codex复用与首版规格]] | Rust CLI 终端 UI——Codex 源码按模块复用边界、Ratatui/Crossterm/ratatui-textarea 库选型与首版（MVP）渲染规格草稿；对应从零实现 Agent 第 27 步 | rust, tui, codex, ratatui, crossterm, terminal, spec |
| [[wiki/Git 幻影修改：Windows Git 与容器 Git 不一致\|Git 幻影修改：Windows Git 与容器 Git 不一致]] | 同一仓库被两侧 git 共用结论不一致——autocrlf 行尾与 filemode 两类幻影的成因、判断方法与 .gitattributes／filemode 根治原则 | git, windows, wsl, autocrlf, filemode, vscode, line-endings |
| [[wiki/上下文嵌入与近似向量检索\|上下文嵌入与近似向量检索]] | 理解词向量如何随语境变化，以及 Annoy、HNSW 怎样加快向量查找。 | embeddings, word2vec, bert, bge-m3, ann, annoy, hnsw |
| [[wiki/稀疏检索与 BM25：按词匹配，按重要性排序\|稀疏检索与 BM25：按词匹配，按重要性排序]] | 理解稀疏向量、TF-IDF、BM25 和倒排索引如何共同完成关键词检索。 | sparse-retrieval, bow, tf-idf, bm25, inverted-index |
| [[wiki/混合检索与重排序：合并候选，再细排相关性\|混合检索与重排序：合并候选，再细排相关性]] | 理解两路检索、RRF 融合、跨编码器重排序的分工，以及检索质量指标。 | hybrid-search, bm25, rrf, cross-encoder, reranking, evaluation |
| [[wiki/知识组织与检索：从零散片段到统计与规则\|知识组织与检索：从零散片段到统计与规则]] | 解释为什么相关片段不足以支持完整回答，以及如何在索引阶段补上统计、规则和知识关系。 | rag, indexing, knowledge-organization, retrieval, knowledge-distillation |
| [[wiki/工具调用闭环：Codex复用与参考范围\|工具调用闭环：Codex复用与参考范围]] | Codex 工具定义机制（Rust 规格 → JSON 声明、自然语言写在 description）与工具调用闭环复用——固定快照 2fdf047c 文件级参考表、测试夹具、Chat Completions／apply_patch 适配点；对应从零实现 Agent 工具闭环里程碑 | codex, agent, tool-calling, rust, harness |
| [[wiki/让模型输出人能看懂、能验证的四种形式\|让模型输出人能看懂、能验证的四种形式]] | Karpathy 的提效思路：用受控语言、图、交互网页和定制讲解视频替代长文，把读结论变成验证结论。 | prompt-engineering, llm-output, controlled-language, visualization, explainer-video |
| [[wiki/codex-memory-system\|Codex 记忆系统]] | 两阶段记忆管线（逐会话提炼→全局合并）、SQLite+Markdown 双存储、developer 级摘要注入与按需文件搜索、全量配置项与 V2 差异——使用、排障与移植入口 | codex, agent-memory, memory, sqlite, markdown, progressive-disclosure |
