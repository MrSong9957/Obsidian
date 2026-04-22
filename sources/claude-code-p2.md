---
name: Claude Code 工具（完整参考）
description: ECC技能集/Superpowers/SafetyNet/Statusline的完整参考文档，含完整技能列表、安装步骤和代码示例
updated: 2026-04-17
tags: [claude-code, plugins, ecc, superpowers, safety-net, statusline]
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# Claude Code 工具（完整参考）

> 本文档为 wiki/claude-code-p2.md 的详细版本，包含完整技能列表、安装步骤和代码示例。

---

## Everything Claude Code (ECC)

> Claude Code 扩展技能集，包含 110+ Skills、Rules、MCP 配置和项目模板。

### Skills (112个)

#### ECC 核心技能 (110个)

| 名称 | 说明 |
|------|------|
| agent-harness-construction | 设计和优化 AI agent 动作空间、工具定义和观察格式 |
| agentic-engineering | 使用 eval 优先执行、分解和成本感知模型路由的 agentic 工程师 |
| ai-first-engineering | AI agent 生成大量实现输出的工程运营模式 |
| ai-regression-testing | AI 辅助开发的回归测试策略，沙盒模式 API 测试、自动化 bug 检查工作流 |
| android-clean-architecture | Android 和 Kotlin Multiplatform 项目的 Clean Architecture 模式 |
| api-design | REST API 设计模式 - 资源命名、状态码、分页、过滤、错误响应 |
| article-writing | 文章、指南、博客、教程、简报等长篇内容撰写 |
| autonomous-loops | 自主 Claude Code 循环的模式和架构 |
| backend-patterns | 后端架构模式、API 设计、数据库优化 |
| blueprint | 将单行目标转化为多会话、多 agent 工程项目的分步构建计划 |
| bun-runtime | Bun 运行时、包管理器、打包器和测试运行器，Bun vs Node 选择指南 |
| carrier-relationship-management | 承运商组合管理、运费谈判、承运商绩效跟踪、货运分配策略 |
| clickhouse-io | ClickHouse 数据库模式、查询优化、分析数据工程 |
| claude-api | Anthropic Claude API 模式（Python/TypeScript）、Messages API、流式传输、工具调用、视觉、Claude Agent SDK |
| claude-devfleet | 通过 Claude DevFleet 编排多 agent 编码任务，在隔离工作树中调度并行 agent |
| coding-standards | TypeScript、JavaScript、React、Node.js 通用编码标准 |
| compose-multiplatform-patterns | Compose Multiplatform 和 Jetpack Compose 模式 - 状态管理、导航、主题 |
| configure-ecc | Everything Claude Code 交互式安装程序 |
| content-engine | X、LinkedIn、TikTok、YouTube、简报等多平台内容系统 |
| content-hash-cache-pattern | 使用 SHA-256 内容哈希缓存昂贵文件处理结果 |
| continuous-agent-loop | 带质量门、评估和恢复控制的自主 agent 循环 |
| continuous-learning | 自动从 Claude Code 会话提取可重用模式 |
| continuous-learning-v2 | 基于本能的学习系统 |
| cost-aware-llm-pipeline | LLM API 使用成本优化 - 模型路由、预算跟踪、提示缓存 |
| cpp-coding-standards | C++ 编码标准，基于 C++ Core Guidelines |
| cpp-testing | C++ 测试配置，使用 GoogleTest/CTest |
| crosspost | 跨 X、LinkedIn、Threads、Bluesky 的多平台内容分发，自动适配各平台 |
| customs-trade-compliance | 海关文档、关税分类、关税优化、限制方筛查、多司法管辖区合规 |
| data-scraper-agent | 构建全自动 AI 驱动数据采集 agent，定时抓取、LLM 增强、GitHub Actions 免费运行 |
| database-migrations | 数据库迁移最佳实践 |
| deep-research | 使用 firecrawl 和 exa MCP 的多源深度研究，生成带引用的报告 |
| deployment-patterns | 部署工作流、CI/CD、Docker、健康检查 |
| django-patterns | Django 架构模式、REST API、ORM 最佳实践 |
| django-security | Django 安全最佳实践 |
| django-tdd | Django 测试策略 |
| django-verification | Django 项目验证循环 |
| dmux-workflows | 使用 dmux（tmux 窗格管理器）的多 agent 编排，支持 Claude Code、Codex、OpenCode 并行工作流 |
| docker-patterns | Docker 和 Docker Compose 模式 |
| documentation-lookup | 通过 Context7 MCP 获取最新库和框架文档，替代训练数据 |
| e2e-testing | Playwright E2E 测试模式 |
| energy-procurement | 电力和天然气采购、关税优化、需求费用管理、可再生能源 PPA 评估 |
| enterprise-agent-ops | 长时间运行的 agent 工作负载运维 |
| eval-harness | Claude Code 会话正式评估框架 |
| exa-search | 通过 Exa MCP 进行神经搜索 - 网页、代码、公司研究、AI 驱动深度研究 |
| fal-ai-media | 通过 fal.ai MCP 统一媒体生成 - 图像、视频、音频（Nano Banana、Seedance、Kling 等） |
| foundation-models-on-device | Apple FoundationModels 设备端 LLM |
| frontend-patterns | 前端开发模式 - React、Next.js、状态管理 |
| frontend-slides | 创建动画丰富的 HTML 演示文稿 |
| golang-patterns | Go 惯用模式 |
| golang-testing | Go 测试模式 - 表驱动测试、基准测试、模糊测试 |
| inventory-demand-planning | 需求预测、安全库存优化、补货计划、促销提升估算 |
| investor-materials | 创建和更新融资材料 - 路演文稿、一页纸、投资者备忘录、财务模型 |
| investor-outreach | 起草投资者外联邮件 - 冷邮件、简介介绍、后续跟进、投资者沟通 |
| iterative-retrieval | 渐进式细化上下文检索模式 |
| java-coding-standards | Java 编码标准 - Spring Boot 服务 |
| jpa-patterns | JPA/Hibernate 模式 |
| kotlin-coroutines-flows | Kotlin 协程和 Flow 模式 - 结构化并发、Flow 操作符、StateFlow、错误处理 |
| kotlin-exposed-patterns | JetBrains Exposed ORM 模式 - DSL 查询、DAO 模式、事务、连接池 |
| kotlin-ktor-patterns | Ktor 服务器模式 - 路由 DSL、插件、认证、Koin DI、WebSockets |
| kotlin-patterns | Kotlin 惯用模式、最佳实践 - 协程、空安全、DSL 构建器 |
| kotlin-testing | Kotlin 测试模式 - Kotest、MockK、协程测试、属性测试 |
| laravel-patterns | Laravel 架构模式、Eloquent ORM、服务层、队列、事件、缓存 |
| laravel-security | Laravel 安全最佳实践 - 认证授权、验证、CSRF、文件上传 |
| laravel-tdd | Laravel TDD 策略 - PHPUnit、Pest、工厂、数据库测试 |
| laravel-verification | Laravel 项目验证循环 - 环境检查、静态分析、测试覆盖、安全扫描 |
| liquid-glass-design | iOS 26 Liquid Glass 设计系统 |
| logistics-exception-management | 货运异常处理、发货延迟、损坏、丢失、承运商争议处理 |
| market-research | 市场研究、竞争分析、投资尽职调查 |
| mcp-server-patterns | 使用 Node/TypeScript SDK 构建 MCP 服务器 - 工具、资源、提示、Zod 验证 |
| nanoclaw-repl | NanoClaw v2 运维和扩展 |
| nextjs-turbopack | Next.js 16+ 和 Turbopack - 增量打包、FS 缓存、开发速度优化 |
| nutrient-document-processing | 文档处理、OCR、提取、编辑、签名 |
| perl-patterns | 现代 Perl 5.36+ 惯用模式、最佳实践 |
| perl-security | Perl 安全 - 污点模式、输入验证、安全进程执行、DBI 参数化查询 |
| perl-testing | Perl 测试模式 - Test2::V0、Test::More、prove 运行器、mocking、覆盖率 |
| plankton-code-quality | 写入时代码质量 enforcement |
| postgres-patterns | PostgreSQL 数据库模式 |
| production-scheduling | 生产排程、作业排序、产线平衡、换型优化、瓶颈解决 |
| project-guidelines-example | 项目特定技能模板示例 |
| prompt-optimizer | 分析原始提示词、识别意图和差距、匹配 ECC 组件、输出优化提示词 |
| python-patterns | Pythonic 惯用模式、PEP 8 标准 |
| python-testing | Python 测试策略 - pytest、TDD |
| quality-nonconformance | 质量控制、不合格调查、根因分析、纠正措施、供应商质量管理 |
| ralphinho-rfc-pipeline | RFC 驱动的多 agent DAG 执行模式 |
| regex-vs-llm-structured-text | 正则 vs LLM 解析结构化文本的决策框架 |
| returns-reverse-logistics | 退货授权、收货检验、处置决策、退款处理、欺诈检测 |
| rust-patterns | Rust 惯用模式、所有权、错误处理、Traits、并发 |
| rust-testing | Rust 测试模式 - 单元测试、集成测试、异步测试、属性测试、mocking |
| search-first | 研究优先工作流 - 先搜索现有工具和模式 |
| security-review | 安全审查技能 |
| security-scan | Claude Code 配置安全扫描 |
| skill-stocktake | Claude Skills 和 Commands 审计 |
| springboot-patterns | Spring Boot 架构模式 |
| springboot-security | Spring Security 最佳实践 |
| springboot-tdd | Spring Boot TDD |
| springboot-verification | Spring Boot 项目验证循环 |
| strategic-compact | 手动上下文压缩建议 |
| swift-actor-persistence | Swift actor 线程安全数据持久化 |
| swift-concurrency-6-2 | Swift 6.2 并发 |
| swift-protocol-di-testing | Swift 协议式依赖注入测试 |
| swiftui-patterns | SwiftUI 架构模式 |
| team-builder | 交互式 agent 选择器，用于组建和调度并行团队 |
| tdd-workflow | TDD 工作流 - 80%+ 覆盖率 |
| verification-loop | Claude Code 会话综合验证系统 |
| video-editing | AI 辅助视频编辑工作流 - FFmpeg、Remotion、ElevenLabs、fal.ai、Descript |
| videodb | 视频/音频理解与处理 - 帧提取、索引、搜索、转码、时间线编辑 |
| visa-doc-translate | 签证文件翻译 |
| x-api | X/Twitter API 集成 - 发推、线程、读取时间线、搜索、分析 |

#### 官方 Skill 插件 (2个)

| 名称 | 说明 |
|------|------|
| skill-creator | 创建新技能、改进现有技能、测量技能性能 |
| frontend-design | 前端设计模式 - UI/UX、响应式设计、组件设计 |

### 快速索引

#### 按功能分类

**流程与工作流**
- tdd-workflow, verification-loop, continuous-learning, autonomous-loops, strategic-compact
- blueprint

**代码开发**
- coding-standards, python-patterns, python-testing, golang-patterns, golang-testing
- cpp-coding-standards, cpp-testing, java-coding-standards, perl-patterns, perl-testing
- frontend-patterns, backend-patterns, docker-patterns, bun-runtime, nextjs-turbopack
- kotlin-patterns, kotlin-testing, rust-patterns, rust-testing
- laravel-patterns, laravel-tdd

**测试**
- e2e-testing, python-testing, golang-testing, cpp-testing, kotlin-testing, rust-testing, perl-testing
- tdd-workflow, ai-regression-testing

**安全**
- security-review, security-scan, django-security, springboot-security, laravel-security, perl-security

**数据库**
- postgres-patterns, clickhouse-io, jpa-patterns, database-migrations, kotlin-exposed-patterns

**框架**
- django-patterns, django-tdd, django-security, django-verification
- springboot-patterns, springboot-tdd, springboot-security, springboot-verification
- laravel-patterns, laravel-tdd, laravel-security, laravel-verification
- kotlin-ktor-patterns, kotlin-coroutines-flows

**前端**
- frontend-patterns, frontend-slides
- swiftui-patterns, liquid-glass-design
- compose-multiplatform-patterns, android-clean-architecture

**Agent 工程**
- agentic-engineering, agent-harness-construction, enterprise-agent-ops
- continuous-agent-loop, autonomous-loops, claude-devfleet, dmux-workflows
- team-builder, data-scraper-agent

**内容与营销**
- content-engine, article-writing, market-research, deep-research
- investor-materials, investor-outreach, crosspost, x-api

**iOS/Swift**
- swiftui-patterns, swift-concurrency-6-2, swift-actor-persistence
- swift-protocol-di-testing, foundation-models-on-device, liquid-glass-design

**媒体处理**
- video-editing, videodb, fal-ai-media

**API 与 MCP**
- claude-api, mcp-server-patterns, exa-search, documentation-lookup

**供应链与物流**
- carrier-relationship-management, customs-trade-compliance, logistics-exception-management
- inventory-demand-planning, production-scheduling, quality-nonconformance, returns-reverse-logistics
- energy-procurement

### Rules 编码规范

来自 `everything-claude-code/rules/`

#### 通用规范 (common/)

| 文件 | 内容 |
|------|------|
| agents.md | Agent 编排规则 - 何时委派、如何并行执行 |
| coding-style.md | 代码风格 - 不可变性、文件组织、错误处理 |
| development-workflow.md | 开发工作流 - 功能实现流程 |
| git-workflow.md | Git 工作流 - 提交格式、PR 流程 |
| hooks.md | Hooks 系统 - PreToolUse/PostToolUse 配置 |
| patterns.md | 常见模式 - API 响应、自定义 Hooks、Repository |
| performance.md | 性能优化 - 模型选择、上下文管理 |
| security.md | 安全指南 - 密钥管理、输入验证、SQL 注入防护 |
| testing.md | 测试要求 - 80% 覆盖率、TDD 流程 |

#### 语言特定规范

| 语言 | 规范文件 |
|------|----------|
| TypeScript | coding-style, hooks, patterns, security, testing |
| Python | coding-style, hooks, patterns, security, testing |
| Go | coding-style, hooks, patterns, security, testing |
| Kotlin | coding-style, hooks, patterns, security, testing |
| Swift | coding-style, hooks, patterns, security, testing |
| Perl | coding-style, hooks, patterns, security, testing |
| PHP | coding-style, hooks, patterns, security, testing |
| C++ | coding-style, hooks, patterns, security, testing |

### MCP 服务配置

来自 `everything-claude-code/mcp-configs/mcp-servers.json`

#### 开发工具

| 服务 | 说明 |
|------|------|
| github | GitHub 操作 - PR、Issue、仓库管理 |
| filesystem | 文件系统操作 |
| memory | 跨会话持久化记忆 |
| sequential-thinking | 链式思维推理 |

#### 数据库与存储

| 服务 | 说明 |
|------|------|
| supabase | Supabase 数据库操作 |
| clickhouse | ClickHouse 分析查询 |

#### 部署平台

| 服务 | 说明 |
|------|------|
| vercel | Vercel 部署和项目 |
| railway | Railway 部署 |
| cloudflare-docs | Cloudflare 文档搜索 |
| cloudflare-workers-* | Cloudflare Workers 构建/绑定/可观测性 |

#### 搜索与研究

| 服务 | 说明 |
|------|------|
| exa-web-search | 网页搜索、研究、数据采集 |
| firecrawl | 网页抓取和爬取 |
| context7 | 实时文档查找 |

#### 浏览器自动化

| 服务 | 说明 |
|------|------|
| playwright | 浏览器自动化和测试 |
| browserbase | 云端浏览器会话 |
| browser-use | AI 浏览器 Agent |

#### AI 与媒体

| 服务 | 说明 |
|------|------|
| fal-ai | AI 图像/视频/音频生成 |
| magic | Magic UI 组件 |

#### 协作与监控

| 服务 | 说明 |
|------|------|
| confluence | Confluence Cloud 集成 |
| devfleet | 多 Agent 编排 - 并行 Claude Code 调度 |
| insaits | AI 安全监控 - 异常检测、凭证暴露检查 |
| token-optimizer | Token 优化，95%+ 上下文压缩 |

> **注意**: 建议保持启用 MCP < 10 个，以保护上下文窗口

### 项目模板 Examples

来自 `everything-claude-code/examples/`

| 模板 | 技术栈 | 适用场景 |
|------|--------|----------|
| **saas-nextjs-CLAUDE.md** | Next.js 15 + Supabase + Stripe + Tailwind | SaaS 应用 |
| **django-api-CLAUDE.md** | Django 5.x + DRF + PostgreSQL + Celery | REST API |
| **go-microservice-CLAUDE.md** | Go + gRPC + PostgreSQL + Wire | 微服务 |
| **rust-api-CLAUDE.md** | Rust + Axum + SQLx + PostgreSQL | 高性能 API |
| **laravel-api-CLAUDE.md** | Laravel 11.x + PostgreSQL + Redis | PHP API |
| **user-CLAUDE.md** | 通用用户级配置模板 | 个人项目 |

#### 使用方法

```bash
# 复制模板到项目根目录
cp examples/saas-nextjs-CLAUDE.md your-project/CLAUDE.md

# 根据项目自定义配置
# 编辑 CLAUDE.md 修改技术栈、规则、文件结构等
```

#### 模板核心内容

每个模板包含：
- **项目概述**: 技术栈和架构说明
- **关键规则**: 数据库、认证、计费、代码风格
- **文件结构**: 推荐的目录组织
- **测试要求**: 覆盖率和测试策略
- **部署配置**: CI/CD 和环境变量

---

## 全能 Claude 代码框架（ECC）——智能体使用说明

这是一款**可用于生产环境的 AI 编程插件**，提供 25 个专业智能体、108 项技能、57 条指令，以及面向软件开发的自动化钩子工作流。

### 核心原则
1. **智能体优先** —— 将领域任务委派给专业智能体处理
2. **测试驱动** —— 实现功能前先编写测试，要求覆盖率不低于 80%
3. **安全优先** —— 绝不妥协安全要求，对所有输入进行校验
4. **不可变性** —— 始终创建新对象，绝不修改已有对象
5. **先规划后执行** —— 编写代码前先规划复杂功能

### 可用智能体

| 智能体 | 用途 | 使用场景 |
|--------|------|----------|
| planner | 制定实施方案 | 复杂功能开发、代码重构 |
| architect | 系统设计与扩展性规划 | 架构决策 |
| tdd-guide | 测试驱动开发指导 | 新功能开发、缺陷修复 |
| code-reviewer | 代码质量与可维护性审查 | 编写或修改代码后 |
| security-reviewer | 漏洞检测 | 提交代码前、敏感代码处理 |
| build-error-resolver | 修复构建/类型错误 | 构建失败时 |
| e2e-runner | 基于 Playwright 的端到端测试 | 核心用户流程 |
| refactor-cleaner | 清理无效代码 | 代码维护 |
| doc-updater | 文档与代码图谱更新 | 文档修订 |
| go-reviewer | Go 代码审查 | Go 语言项目 |
| go-build-resolver | Go 构建错误修复 | Go 项目构建失败 |
| kotlin-reviewer | Kotlin 代码审查 | Kotlin/Android/KMP 项目 |
| kotlin-build-resolver | Kotlin/Gradle 构建错误修复 | Kotlin 项目构建失败 |
| database-reviewer | PostgreSQL/Supabase 专项专家 | 表结构设计、查询优化 |
| python-reviewer | Python 代码审查 | Python 项目 |
| java-reviewer | Java 与 Spring Boot 代码审查 | Java/Spring Boot 项目 |
| java-build-resolver | Java/Maven/Gradle 构建错误修复 | Java 项目构建失败 |
| chief-of-staff | 沟通梳理与文案起草 | 邮件、Slack、LINE、Messenger 多渠道沟通 |
| loop-operator | 自主循环执行 | 安全运行循环、监控阻塞、介入干预 |
| harness-optimizer | 执行配置调优 | 可靠性、成本、吞吐量优化 |
| rust-reviewer | Rust 代码审查 | Rust 项目 |
| rust-build-resolver | Rust 构建错误修复 | Rust 项目构建失败 |

### 智能体调度
无需用户提示，主动调用智能体：
- 复杂功能需求 → **planner**
- 刚编写/修改的代码 → **code-reviewer**
- 缺陷修复或新功能开发 → **tdd-guide**
- 架构决策 → **architect**
- 安全敏感代码 → **security-reviewer**
- 多渠道沟通梳理 → **chief-of-staff**
- 自主循环 / 循环监控 → **loop-operator**
- 执行配置可靠性与成本优化 → **harness-optimizer**

对独立操作采用并行执行，可同时启动多个智能体。

### 安全规范
**任何提交前必须检查：**
- 禁止硬编码密钥（API 密钥、密码、令牌）
- 所有用户输入均经过校验
- 防止 SQL 注入（使用参数化查询）
- 防止 XSS 攻击（HTML 内容净化）
- 开启 CSRF 防护
- 完成身份认证与权限校验
- 所有接口实现限流
- 错误信息不泄露敏感数据

**密钥管理：** 严禁硬编码密钥，使用环境变量或密钥管理服务。启动时校验必填密钥，一旦密钥泄露立即轮换。

**发现安全问题时：** 立即停止操作 → 调用 security-reviewer 智能体 → 修复高危问题 → 轮换泄露密钥 → 全面排查同类问题。

### 编码风格
**不可变性（核心要求）：** 始终创建新对象，禁止修改原对象，返回应用变更后的新副本。

**文件组织：** 优先拆分为多个小文件，避免大文件。单文件常规行数 200–400 行，最大不超过 800 行。按功能/业务域划分，而非按文件类型划分。高内聚、低耦合。

**错误处理：** 全层级处理异常，UI 层展示用户友好提示，服务端记录详细上下文。绝不静默忽略错误。

**输入校验：** 在系统边界校验所有用户输入，使用基于 schema 的校验方式，快速失败并返回清晰提示。绝不信任外部数据。

**代码质量检查清单：**
- 函数精简（<50 行），文件职责单一（<800 行）
- 避免深层嵌套（>4 层）
- 完善的错误处理，无硬编码常量
- 代码可读，命名规范清晰

### 测试要求
**最低覆盖率：80%**

必须覆盖的测试类型：
1. **单元测试** —— 独立函数、工具类、组件
2. **集成测试** —— API 接口、数据库操作
3. **端到端测试** —— 核心用户流程

**测试驱动开发工作流（强制执行）：**
1. 先编写测试（红态）—— 测试应失败
2. 编写最简实现（绿态）—— 测试应通过
3. 代码重构（优化）—— 确认覆盖率 ≥80%

故障排查：检查测试隔离性 → 校验模拟对象 → 修复实现逻辑（非测试用例，测试用例错误除外）。

### 开发工作流
1. **规划** —— 使用 planner 智能体，明确依赖与风险，拆分阶段
2. **测试驱动开发** —— 使用 tdd-guide 智能体，先写测试，再实现，后重构
3. **审查** —— 立即调用 code-reviewer 智能体，修复高危/高优先级问题
4. **合理沉淀知识**
   - 个人调试笔记、偏好、临时上下文 → 自动记忆
   - 团队/项目知识（架构决策、API 变更、运行手册）→ 项目现有文档结构
   - 若当前任务已生成相关文档或代码注释，不在其他位置重复记录
   - 若无明确的项目文档位置，创建新顶层文件前先询问
5. **提交** —— 遵循约定式提交格式，编写完整的 PR 说明

### Git 工作流
**提交格式：** `<类型>: <描述>`
类型包括：feat（功能）、fix（修复）、refactor（重构）、docs（文档）、test（测试）、chore（杂项）、perf（性能）、ci（持续集成）

**PR 工作流：** 分析完整提交历史 → 编写完整摘要 → 包含测试方案 → 带 `-u` 参数推送。

### 架构模式
**API 响应格式：** 统一封装结构，包含成功标识、数据载体、错误信息与分页元数据。

**仓储模式：** 通过标准接口封装数据访问（findAll、findById、create、update、delete）。业务逻辑依赖抽象接口，而非具体存储实现。

**项目骨架：** 检索经过实战验证的模板，通过并行智能体评估（安全性、可扩展性、适用性），克隆最优模板，在成熟结构上迭代开发。

### 性能优化
**上下文管理：** 大型重构与多文件功能开发，避免占用上下文窗口最后 20% 空间。低敏感度任务（单次编辑、文档、简单修复）可容忍更高使用率。

**构建故障排查：** 调用 build-error-resolver 智能体 → 分析错误 → 逐步修复 → 每次修复后验证。

### 项目结构
```
agents/          —— 25 个专业子智能体
skills/          —— 102 项工作流技能与领域知识
commands/        —— 57 条斜杠指令
hooks/           —— 基于触发器的自动化逻辑
rules/           —— 必须遵守的规范（通用规范 + 各语言规范）
scripts/         —— 跨平台 Node.js 工具脚本
mcp-configs/     —— 14 套 MCP 服务端配置
tests/           —— 测试套件
```

### 成功指标
- 所有测试通过，覆盖率 ≥80%
- 无安全漏洞
- 代码可读、可维护
- 性能符合预期
- 满足用户需求

---

## Superpowers 使用指南

### 1. 什么是 Superpowers？

Superpowers 是一个基于 AI 辅助开发的技能系统，通过一系列预定义的技能（skills）来规范开发流程，提高开发效率和代码质量。它包含了从需求分析到代码实现、测试、审查等完整开发生命周期的技能集。

### 2. 技能列表

| 技能名称 | 描述 | 核心功能 |
|---------|------|----------|
| using-superpowers | 基础技能，指导如何访问和使用其他技能 | 技能调用规则、优先级、流程 |
| writing-plans | 编写实现计划 | 创建详细的分步实现计划 |
| executing-plans | 执行实现计划 | 按计划分批执行任务，定期汇报 |
| systematic-debugging | 系统化调试 | 高效定位和修复 bug |
| test-driven-development | 测试驱动开发 | 遵循 TDD 流程开发代码 |
| brainstorming | 头脑风暴 | 需求分析和解决方案设计 |
| subagent-driven-development | 子代理驱动开发 | 分任务调度子代理完成开发 |
| requesting-code-review | 请求代码审查 | 准备和提交代码审查请求 |
| receiving-code-review | 接收代码审查 | 处理和响应代码审查反馈 |
| finishing-a-development-branch | 完成开发分支 | 最终验证和分支处理 |
| verification-before-completion | 完成前验证 | 全面验证确保功能正确性 |
| dispatching-parallel-agents | 并行代理调度 | 调度多个代理并行工作 |
| ui-ux-pro-max | UI/UX 设计 | 专业 UI/UX 设计支持 |
| using-git-worktrees | Git Worktrees 使用 | 高效管理多个开发分支 |
| writing-skills | 编写技能 | 创建新的 Superpowers 技能 |

### 3. 安装与设置

#### 3.1 环境要求
- Windows 11 操作系统
- Git 已安装
- Python 3.8+（可选，部分技能需要）

#### 3.2 安装步骤
1. 克隆仓库：
   ```bash
   git clone https://github.com/yourusername/superpowers.git
   cd superpowers
   ```

2. （可选）创建并激活虚拟环境：
   ```bash
   python -m venv .venv
   .venv\Scripts\Activate.ps1
   ```

### 4. 零基础使用教程

#### 4.1 基本使用流程

Superpowers 的核心使用流程如下：

1. **选择合适的技能**：根据当前任务选择对应的技能
2. **调用技能**：按照技能要求的方式调用
3. **遵循技能指南**：严格按照技能文档的步骤执行
4. **验证结果**：执行技能中的验证步骤确保结果正确

#### 4.2 第一步：使用 using-superpowers 技能

**这是所有任务的起点！**

无论你要做什么，首先必须检查是否有适用的技能，使用 `using-superpowers` 技能来指导你：

1. **核心规则**：如果有 1% 的可能适用某个技能，就必须调用它
2. **技能调用优先级**：
   - 先使用流程技能（如 brainstorming、systematic-debugging）
   - 再使用实现技能（如 frontend-design、mcp-builder）

#### 4.3 第二步：编写实现计划

当你有一个多步骤任务时，使用 `writing-plans` 技能：

1. **创建工作目录**：在 `docs/plans/` 目录下创建计划文件
2. **编写计划**：按照模板编写详细的分步计划
3. **计划结构**：
   - 目标：一句话描述要构建的功能
   - 架构：2-3 句话说明实现方法
   - 技术栈：列出关键技术和库
   - 任务列表：每个任务包含具体步骤、代码示例和验证方法

**示例计划文件**：`docs/plans/2024-01-12-hello-world.md`

```markdown
# Hello World 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 创建一个简单的 Hello World 程序

**Architecture:** 使用 Python 编写一个单文件程序，输出 "Hello, Superpowers!"

**Tech Stack:** Python 3.8+

---

### Task 1: 创建测试文件

**Files:**
- Create: `tests/test_hello.py`

**Step 1: 编写测试代码**
```python
def test_hello():
    from src.hello import hello
    assert hello() == "Hello, Superpowers!"
```

**Step 2: 运行测试（预期失败）**
```bash
pytest tests/test_hello.py -v
```
Expected: FAIL with "ModuleNotFoundError: No module named 'src.hello'"

### Task 2: 实现功能

**Files:**
- Create: `src/hello.py`

**Step 1: 编写实现代码**
```python
def hello():
    return "Hello, Superpowers!"
```

**Step 2: 运行测试（预期通过）**
```bash
pytest tests/test_hello.py -v
```
Expected: PASS

**Step 3: 提交代码**
```bash
git add src/hello.py tests/test_hello.py
git commit -m "feat: add hello world function"
```

#### 4.4 第三步：执行实现计划

使用 `executing-plans` 技能来执行你的计划：

1. **加载并审查计划**：仔细阅读计划，确保理解每个步骤
2. **分批执行任务**：默认每次执行前 3 个任务
3. **执行每个步骤**：
   - 标记任务为 in_progress
   - 严格按照计划步骤执行
   - 运行验证步骤
   - 标记任务为 completed
4. **汇报进度**：每批任务完成后汇报结果，等待反馈
5. **处理阻塞**：遇到问题立即停止，寻求帮助

#### 4.5 第四步：完成开发分支

当所有任务完成后，使用 `finishing-a-development-branch` 技能：

1. **验证所有测试**：确保所有测试通过
2. **检查代码质量**：运行代码质量检查工具
3. **选择后续操作**：合并分支、删除分支或继续开发

### 5. 常用技能使用示例

#### 5.1 系统化调试

使用 `systematic-debugging` 技能来定位和修复 bug：

1. **复现问题**：确保能够稳定复现 bug
2. **收集信息**：查看日志、调试信息等
3. **定位根因**：使用各种调试技术找到问题根源
4. **修复验证**：编写测试验证修复效果
5. **预防回归**：添加测试防止问题再次出现

#### 5.2 测试驱动开发

使用 `test-driven-development` 技能遵循 TDD 流程：

1. **编写失败测试**：先编写测试用例，预期失败
2. **实现最小功能**：编写足够的代码使测试通过
3. **重构代码**：优化代码结构，保持测试通过
4. **重复迭代**：继续添加新功能，重复上述步骤

#### 5.3 请求代码审查

使用 `requesting-code-review` 技能请求代码审查：

1. **准备审查请求**：确保代码符合项目规范
2. **提交审查请求**：使用合适的格式提交
3. **等待反馈**：耐心等待审查者的反馈
4. **处理反馈**：根据反馈修改代码

### 6. 最佳实践

1. **严格遵循技能指南**：技能设计经过实践验证，严格遵循能提高效率
2. **优先使用流程技能**：先确定方法，再执行实现
3. **保持任务粒度**：每个任务控制在 2-5 分钟，便于管理和验证
4. **频繁验证**：每个步骤都要验证，确保结果正确
5. **遇到问题立即停止**：不要猜测，寻求帮助或使用调试技能
6. **保持代码简洁**：遵循极简主义设计，用最少的代码实现功能

### 7. 常见问题

### Q: 如何选择合适的技能？
A: 根据当前任务类型选择：
- 需求分析：brainstorming
- 编写计划：writing-plans
- 执行计划：executing-plans
- 调试问题：systematic-debugging
- 开发功能：test-driven-development

### Q: 技能调用的优先级是什么？
A: 先使用流程技能（如 brainstorming、debugging），再使用实现技能（如 frontend-design）。

### Q: 遇到技能不适用怎么办？
A: 如果没有完全匹配的技能，可以选择最接近的技能，或使用基础技能 using-superpowers 指导。

### 8. 总结

Superpowers 是一个强大的 AI 辅助开发系统，通过预定义的技能规范开发流程，提高开发效率和代码质量。零基础用户可以按照本指南的步骤，从使用基础技能开始，逐步掌握各个技能的使用方法，最终实现高效的 AI 辅助开发。

记住：**技能是工具，关键是合理使用它们来解决实际问题**。

---

## ECC (everything-claude-code) 安装指南

### 环境要求
- Claude Code 最新版本
- Node.js 18+ (用于执行 hook 脚本)

### 安装步骤

#### 步骤 1：添加插件市场

```bash
/plugin marketplace add affaan-m/everything-claude-code
```

#### 步骤 2：安装插件

```bash
/plugin install everything-claude-code@everything-claude-code
```

#### 步骤 3：配置 settings.json

将以下配置添加到 `~/.claude/settings.json`：

```json
{
  "extraKnownMarketplaces": {
    "everything-claude-code": {
      "source": {
        "source": "directory",
        "path": "C:/Users/<你的用户名>/.claude/plugins/marketplaces/everything-claude-code"
      }
    }
  },
  "enabledPlugins": {
    "everything-claude-code@everything-claude-code": true
  }
}
```

> **实际经验**：官方文档用 `"source": "github"`，但如果 GitHub 无法访问会下载失败。建议用 `"source": "directory"` 指向本地已下载的插件目录。

##### 3.1 插件目录结构说明

Claude Code 插件有两个关键目录：

| 目录 | 作用 |
|------|------|
| `marketplaces/` | 插件源码存放（source） |
| `cache/` | 运行时加载（runtime） |

**如果 hooks 不生效**，可能需要手动将插件复制到 cache 目录：
```bash
# 从 marketplaces 复制到 cache（保留隐藏目录）
cp -r ~/.claude/plugins/marketplaces/everything-claude-code ~/.claude/plugins/cache/
```

#### 步骤 4：手动安装 Rules

> Claude Code 插件系统不支持 rules 通过插件分发，需要手动安装。

##### 4.1 准备工作：备份现有规则

```bash
# 备份现有规则（重要！）
cp -r ~/.claude/rules ~/.claude/rules.backup
```

##### 4.2 安装 ECC 规则

```bash
# 克隆仓库
git clone https://github.com/affaan-m/everything-claude-code.git

# 查看 ECC 提供的规则
ls everything-claude-code/rules/common/
```

##### 4.3 合并规则（重要）

**不要直接覆盖！** 现有规则可能包含你的个人偏好和工作流。

**推荐做法**：
```bash
# 方法1：只复制不存在的文件（保留现有规则）
for f in everything-claude-code/rules/common/*; do
  if [ ! -f "~/.claude/rules/common/$(basename $f)" ]; then
    cp "$f" ~/.claude/rules/common/
  fi
done

# 方法2：手动选择需要的规则文件复制
# 只复制你确实需要的，避免覆盖已有配置
```

**实际经验**：
- 用户通常已有 `agents.md`、`coding-style.md`、`performance.md` 等自定义规则
- ECC 的规则可能与你的现有规则冲突或重复
- 建议先查看 ECC 规则内容，再决定是否需要

##### 4.4 项目级规则（可选）

```bash
# 项目级规则（仅当前项目生效）
mkdir -p .claude/rules
# 只复制你需要的规则文件到这个项目
cp everything-claude-code/rules/common/<需要的文件> .claude/rules/
```

#### 步骤 5：配置 Hooks（重要）

> **警告**：不要直接覆盖用户的现有 hooks！必须保留用户已有的配置。

插件的 hooks 定义在仓库的 `hooks/hooks.json`，需要**追加**到现有 hooks 后面，而不是替换。

##### 5.1 合并 Hooks 配置

**正确做法**：将 ECC 的每个 hook 类型**追加**到用户现有 hooks 的末尾。

```json
{
  "hooks": {
    "PreToolUse": [
      /* 用户的现有 hooks（保留） */
      {"matcher": "...", "hooks": [...]},

      /* ECC hooks（追加到末尾） */
      {"matcher": "...", "hooks": [...]}
    ],
    "PreCompact": [
      /* 用户现有 + ECC */
    ],
    ...
  }
}
```

**错误做法**：直接用 ECC hooks 替换用户的 hooks。

##### 5.2 配置环境变量（Windows 必需）

> **Windows 特有问题**：`${CLAUDE_PLUGIN_ROOT}` 在 Windows 下不会自动解析，必须显式配置。

在 `settings.json` 中添加 `env` 部分：

```json
{
  "env": {
    "CLAUDE_PLUGIN_ROOT": "C:/Users/<你的用户名>/.claude/plugins/cache/everything-claude-code/everything-claude-code/<版本号>"
  }
}
```

**如何查找实际路径**：
```bash
# 查看插件缓存目录
ls -la ~/.claude/plugins/cache/everything-claude-code/
```

### 验证安装

运行以下命令检查 hooks 是否正常工作：

```bash
/compact
```

**预期输出**：
- 成功：`completed successfully`
- 失败：`failed: node: internal/modules/cjs/loader`

### 常见问题

#### Q1: /compact 报错 "Cannot find module"
**原因**：Windows 下 `${CLAUDE_PLUGIN_ROOT}` 未解析。
**解决**：参考「步骤 5.2」配置环境变量。

#### Q2: Hooks 不生效
**原因**：hooks 配置未合并到 settings.json。
**解决**：手动将 `hooks/hooks.json` 内容**追加**到 settings.json 的 `hooks` 字段，保留用户的现有 hooks。

#### Q3: 用户的 hooks 被覆盖了
**原因**：直接用 ECC hooks 替换了用户的 hooks。
**解决**：将用户的现有 hooks 保留，只将 ECC hooks 追加到末尾。参考「步骤 5.1」。

#### Q4: 命令不可用
**原因**：rules 未正确安装。
**解决**：参考「步骤 4」安装 rules。

#### Q5: 我的自定义规则被覆盖了
**原因**：直接用 `cp -r` 复制 ECC 规则，覆盖了现有文件。
**解决**：
```bash
# 如果有备份，从备份恢复
cp -r ~/.claude/rules.backup/* ~/.claude/rules/common/

# 然后用步骤 4.3 的方法合并（只复制不存在的文件）
```

**预防**：以后安装任何规则前，先执行 `cp -r ~/.claude/rules ~/.claude/rules.backup`

### 快速检查清单

- [ ] 插件已安装 (`/plugin list`)
- [ ] settings.json 包含 `enabledPlugins`
- [ ] settings.json 包含 `hooks` 配置
- [ ] **用户的现有 hooks 已被保留**（不是被覆盖）
- [ ] settings.json 包含 `env.CLAUDE_PLUGIN_ROOT`（Windows）
- [ ] rules 已复制到 `~/.claude/rules/`
- [ ] **现有规则已备份**（安装前执行了 `cp -r ~/.claude/rules ~/.claude/rules.backup`）
- [ ] 规则已合并（非直接覆盖）
- [ ] `/compact` 执行成功

---

## Safety Net 使用手册

> Claude Code 插件 - 在危险命令执行前拦截它们

### 简介

Safety Net 是一个 Claude Code 安全插件，用于防止 AI 代理执行破坏性的 git 和文件系统命令。

**核心价值**：`CLAUDE.md` 中的"软"规则无法替代"硬"技术约束。本插件通过 Hook 机制在命令执行前进行语义分析，拦截危险操作。

### 安装

```bash
# 1. 添加市场
/plugin marketplace add kenryu42/cc-marketplace

# 2. 安装插件
/plugin install safety-net@cc-marketplace

# 3. 重载插件生效
/reload-plugins
```

#### 配置状态栏

状态栏会在 Claude Code 界面底部显示 Safety Net 的状态信息。

**方式一：通过技能配置**

```bash
/safety-net:set-statusline
```

按提示选择包管理器（npx 或 bunx），然后选择是否与现有状态栏命令合并。

**方式二：手动配置**

编辑 `~/.claude/settings.json`：

```json
{
  "statusLine": {
    "type": "command",
    "command": "npx -y cc-safety-net --statusline"
  }
}
```

**合并现有状态栏**

如果已有其他状态栏命令，可以用管道合并：

```json
{
  "statusLine": {
    "type": "command",
    "command": "python ~/.claude/scripts/statusline.py | npx -y cc-safety-net --statusline"
  }
}
```

> **注意**：配置变更立即生效，无需重启。

#### 配置 Hook（Windows 必需）

> **Windows 特有问题**：插件系统不会自动将 hooks 合并到 `settings.json`，需要手动配置。

**步骤 1：添加环境变量**

在 `~/.claude/settings.json` 的 `env` 部分添加：

```json
{
  "env": {
    "SAFETY_NET_PLUGIN_ROOT": "C:/Users/<你的用户名>/.claude/plugins/cache/cc-marketplace/safety-net/<版本号>"
  }
}
```

查找实际路径：

```bash
# 查看插件缓存目录
ls ~/.claude/plugins/cache/cc-marketplace/safety-net/
```

**步骤 2：添加 PreToolUse Hook**

在 `hooks.PreToolUse` 数组的**开头**添加（确保优先执行）：

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "node \"${SAFETY_NET_PLUGIN_ROOT}/dist/bin/cc-safety-net.js\" --claude-code",
            "timeout": 15
          }
        ],
        "description": "Safety Net - Block destructive git and filesystem commands"
      },
      // ... 其他 hooks
    ]
  }
}
```

**步骤 3：重新加载配置**

```bash
/reload-plugins
```

或重启 Claude Code。

#### 自动更新

```
/plugin → Marketplaces → cc-marketplace → 启用 auto-update
```

### 可用技能

| 命令 | 用途 |
|------|------|
| `/safety-net:set-custom-rules` | 配置自定义阻止规则 |
| `/safety-net:set-statusline` | 设置状态栏显示 |
| `/safety-net:verify-custom-rules` | 验证规则配置 |

### 被拦截的命令

#### Git 命令

| 命令 | 原因 |
|------|------|
| `git checkout -- files` | 永久丢弃未提交的更改 |
| `git restore files` | 丢弃未提交的更改 |
| `git reset --hard` | 销毁所有未提交的更改 |
| `git clean -f` | 永久删除未跟踪的文件 |
| `git push --force / -f` | 破坏远程历史 |
| `git branch -D` | 强制删除分支（无合并检查） |
| `git stash drop/clear` | 永久删除暂存的更改 |
| `git worktree remove --force` | 强制删除工作树 |

#### 文件系统命令

| 命令 | 原因 |
|------|------|
| `rm -rf` (cwd 外的路径) | 递归删除当前目录外的文件 |
| `rm -rf /` 或 `~` | 极度危险 |
| `find ... -delete` | 永久删除匹配的文件 |
| `xargs rm -rf` | 动态输入导致目标不可预测 |
| `parallel rm -rf` | 同上 |

### 允许的安全命令

| 命令 | 原因 |
|------|------|
| `git checkout -b branch` | 创建新分支 |
| `git restore --staged` | 仅取消暂存 |
| `git branch -d` | 安全删除（有合并检查） |
| `git clean -n` | 仅预览 |
| `git push --force-with-lease` | 安全的强制推送 |
| `rm -rf /tmp/...` | 临时目录 |
| `rm -rf ./...` (cwd 内) | 限制在当前目录 |

### 高级配置

#### 环境变量

```bash
# 严格模式：无法解析的命令将被阻止
export SAFETY_NET_STRICT=1

# 偏执模式：启用所有严格检查
export SAFETY_NET_PARANOID=1

# 单独启用偏执检查
export SAFETY_NET_PARANOID_RM=1           # 阻止 cwd 内的 rm -rf
export SAFETY_NET_PARANOID_INTERPRETERS=1 # 阻止解释器单行命令
```

#### 偏执模式行为

- **rm**: 阻止非临时目录的 `rm -rf`，即使在当前目录内
- **解释器**: 阻止 `python -c`、`node -e`、`ruby -e`、`perl -e` 等单行命令

#### 自定义规则

配置文件位置：

| 范围 | 路径 | 优先级 |
|------|------|--------|
| 用户级 | `~/.cc-safety-net/config.json` | 低 |
| 项目级 | `.safety-net.json` | 高（覆盖用户级） |

示例配置：

```json
{
  "$schema": "https://raw.githubusercontent.com/kenryu42/claude-code-safety-net/main/assets/cc-safety-net.schema.json",
  "version": 1,
  "rules": [
    {
      "name": "block-git-add-all",
      "command": "git",
      "subcommand": "add",
      "block_args": ["-A", "--all", "."],
      "reason": "Use 'git add <specific-files>' instead."
    },
    {
      "name": "block-npm-global",
      "command": "npm",
      "subcommand": "install",
      "block_args": ["-g", "--global"],
      "reason": "Use npx or local install."
    }
  ]
}
```

### CLI 命令

```bash
# 诊断检查
npx -y cc-safety-net doctor

# 解释命令分析过程
npx -y cc-safety-net explain "git push --force"

# 验证配置
npx -y cc-safety-net --verify-config

# 查看自定义规则文档
npx -y cc-safety-net --custom-rules-doc

# 状态栏显示
npx -y cc-safety-net --statusline
```

### 特性

#### Shell 包装检测

递归分析包装在 shell 中的命令：

```bash
bash -c 'git reset --hard'    # 被拦截
sh -lc 'rm -rf /'             # 被拦截
```

#### 解释器单行命令检测

检测隐藏在解释器中的破坏性命令：

```bash
python -c 'import os; os.system("rm -rf /")'  # 被拦截
```

#### 敏感信息脱敏

拦截消息自动脱敏敏感数据（令牌、密码、API 密钥）。

#### 审计日志

所有被拦截的命令记录到 `~/.cc-safety-net/logs/<session_id>.jsonl`：

```json
{"ts": "2025-01-15T10:30:00Z", "command": "git reset --hard", "reason": "...", "cwd": "/path/to/project"}
```

### 被拦截时的处理

当命令被拦截时，你会看到：

```
BLOCKED by Safety Net

Reason: git checkout -- discards uncommitted changes permanently. Use 'git stash' first.

Command: git checkout -- src/main.py

If this operation is truly needed, ask the user for explicit permission and have them run the command manually.
```

**解决方案**：如果确实需要执行该操作，请用户手动运行命令。

### 测试

#### 使用 explain 命令分析

```bash
# 分析危险命令（不实际执行）
npx -y cc-safety-net explain "git reset --hard HEAD"
npx -y cc-safety-net explain "rm -rf /"
npx -y cc-safety-net explain "git checkout -- src/main.py"
```

#### 实际拦截测试

```bash
# 测试拦截 - 以下命令会被 Safety Net 阻止
git checkout -- README.md  # BLOCKED: 丢弃未提交的更改
git reset --hard HEAD      # BLOCKED: 销毁未提交的更改
rm -rf /                   # BLOCKED: 根目录删除

# 测试允许 - 以下命令会被放行
git checkout -b test-branch      # ALLOWED: 创建新分支
git branch -d merged-branch      # ALLOWED: 安全删除已合并分支
rm -rf ./node_modules            # ALLOWED: 当前目录内（非 home 目录）
```

### 相关链接

- 仓库：https://github.com/kenryu42/claude-code-safety-net
- 许可证：MIT

---

## Claude Code 状态栏插件安装指南

### 概述

StatusLine 是 Claude Code 的内置功能，无需额外安装。只需在 `~/.claude/settings.json` 中配置即可。

### 功能

在每次命令执行后显示：
- 当前模型名称
- 当前工作目录（`$HOME` 显示为 `~`）
- Git 分支名称
- 上下文使用进度条（颜色编码：绿<50%，黄<80%，红>=80%）

**示例输出：**
```
MiniMax M2.5-highspeed | sry-opencode/docs | main | ████████░░ 75%
```

### 快速开始（推荐 Python 版本）

> **强烈推荐使用 Python 版本**。Bash 版本在某些环境下可能因转义问题导致状态栏无法显示，Python 版本更稳定可靠。

#### 步骤 1：创建脚本文件

保存到 `~/.claude/scripts/statusline.py`：

```python
#!/usr/bin/env python3
"""Claude Code StatusLine - Python implementation"""

import json
import sys
import os
import subprocess


def get_git_branch(cwd: str) -> str:
    """Get git branch name, searching parent directories if needed."""
    if not cwd:
        return ""

    check_dir = cwd
    while check_dir and check_dir != "/":
        if os.path.isdir(os.path.join(check_dir, ".git")):
            try:
                result = subprocess.run(
                    ["git", "-C", check_dir, "rev-parse", "--abbrev-ref", "HEAD"],
                    capture_output=True,
                    text=True,
                    timeout=2
                )
                if result.returncode == 0:
                    return result.stdout.strip().replace("heads/", "")
                break
            except (subprocess.TimeoutExpired, Exception):
                break
        check_dir = os.path.dirname(check_dir)
    return ""


def get_progress_bar(percentage: int, length: int = 10) -> str:
    """Generate a progress bar."""
    filled = min(max(int(percentage * length / 100), 0), length)
    empty = length - filled
    return "█" * filled + "░" * empty


def shorten_path(path: str, levels: int = 2) -> str:
    """Shorten path to show only last N levels, with ~ for home."""
    if not path:
        return ""

    home = os.environ.get("HOME", "")
    if home and path.startswith(home):
        rest = path[len(home):].lstrip(os.sep)
        if not rest:  # Exactly home
            return "~"
        parts = rest.split(os.sep)
        if len(parts) <= levels:
            return "~/" + "/".join(parts)
        return "/".join(parts[-levels:])
    else:
        parts = path.split(os.sep)
        # Filter empty parts (e.g., from leading /)
        parts = [p for p in parts if p]
        if len(parts) <= levels:
            return path
        return "/".join(parts[-levels:])


def main():
    try:
        input_data = json.load(sys.stdin)
    except json.JSONDecodeError:
        print("Error reading input")
        return

    model = input_data.get("model", {}).get("display_name", "")
    cwd = input_data.get("workspace", {}).get("current_dir", "")
    used_pct = input_data.get("context_window", {}).get("used_percentage")

    # Show only last 2 levels of path
    dir_display = shorten_path(cwd, levels=2)

    # Get git branch
    branch = get_git_branch(cwd)

    # ANSI color codes
    CYAN = "\033[96m"
    BLUE = "\033[94m"
    GREEN = "\033[92m"
    YELLOW = "\033[93m"
    RED = "\033[91m"
    DIM = "\033[2m"
    RESET = "\033[0m"

    # Build context info with progress bar
    if used_pct is not None and isinstance(used_pct, (int, float)):
        if used_pct < 50:
            bar_color = GREEN
        elif used_pct < 80:
            bar_color = YELLOW
        else:
            bar_color = RED
        bar = get_progress_bar(int(used_pct))
        context_info = f"{bar_color}{bar}{RESET} {int(used_pct)}%"
    else:
        context_info = f"{DIM}--{RESET}"

    # Build output
    parts = [f"{CYAN}{model}{RESET}", f"{BLUE}{dir_display}{RESET}"]
    if branch:
        parts.append(f"{GREEN}{branch}{RESET}")
    parts.append(context_info)

    print(" | ".join(parts))


if __name__ == "__main__":
    main()
```

#### 步骤 2：添加执行权限

```bash
chmod +x ~/.claude/scripts/statusline.py
```

#### 步骤 3：配置 settings.json

在 `~/.claude/settings.json` 中添加：

```json
{
  "statusLine": {
    "type": "command",
    "command": "python3 ~/.claude/scripts/statusline.py"
  }
}
```

#### 步骤 4：重启 Claude Code

退出当前会话（`Ctrl+C`），重新运行 `claude` 即可看到状态栏。

### 可用变量

通过 stdin 传入 JSON，支持以下变量：

| 变量 | 说明 |
|------|------|
| `model.display_name` | 模型显示名称 |
| `model.name` | 模型内部名称 |
| `workspace.current_dir` | 当前工作目录 |
| `context_window.used_tokens` | 已用 token 数 |
| `context_window.limit_tokens` | token 限制 |
| `context_window.used_percentage` | 上下文使用百分比 |
| `session.id` | 会话 ID |
| `session.start_time` | 会话开始时间戳 |

### 配置类型说明

#### 1. template（简洁）

适合简单需求，无需 shell：

```json
{
  "statusLine": {
    "type": "template",
    "template": "{{model.display_name}} | {{workspace.current_dir}} | {{context_window.used_percentage}}%"
  }
}
```

#### 2. command（灵活）

适合自定义样式，通过命令处理 stdin JSON 数据。

### Shell 脚本示例（备选）

> **注意**：如果 Python 版本正常工作，不需要使用 Shell 版本。Shell 版本在某些环境下可能因转义问题导致无法显示。

#### 简洁版

```bash
input=$(cat)
model=$(echo "$input" | jq -r '.model.display_name // empty')
cwd=$(echo "$input" | jq -r '.workspace.current_dir // empty')
used_pct=$(echo "$input" | jq -r '.context_window.used_percentage // empty')

dir_display="${cwd/#$HOME/~}"
printf "%s | %s | %s%%" "$model" "$dir_display" "$used_pct"
```

#### 带颜色版

```bash
input=$(cat)
used_pct=$(echo "$input" | jq -r '.context_window.used_percentage')
model=$(echo "$input" | jq -r '.model.display_name')

if (( $(echo "$used_pct > 80" | bc -l) )); then
  color="\033[31m"  # 红色
elif (( $(echo "$used_pct > 50" | bc -l) )); then
  color="\033[33m"  # 黄色
else
  color="\033[32m"  # 绿色
fi

printf "%s\033[0m | %s%%" "$model" "$color" "$used_pct"
```

#### 显示 Git 分支

```bash
input=$(cat)
model=$(echo "$input" | jq -r '.model.display_name')
cwd=$(echo "$input" | jq -r '.workspace.current_dir')

cd "$cwd" 2>/dev/null
branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
dir="${cwd/#$HOME/~}"

if [ -n "$branch" ]; then
  printf "%s | %s (%s)" "$model" "$dir" "$branch"
else
  printf "%s | %s" "$model" "$dir"
fi
```

### 高级用法

#### 自定义脚本文件

```json
{
  "statusLine": {
    "type": "command",
    "command": "/path/to/your/statusline.py"
  }
}
```

#### 错误处理（Python）

```python
try:
    input_data = json.load(sys.stdin)
except json.JSONDecodeError:
    print("Error: Invalid input")
    sys.exit(1)
```

#### 缓存 Git 分支（性能优化）

```python
import time

CACHE_FILE = "/tmp/statusline_branch_cache"
CACHE_TTL = 30  # seconds

def get_git_branch_cached(cwd: str) -> str:
    # Check cache
    try:
        if os.path.exists(CACHE_FILE):
            mtime = os.path.getmtime(CACHE_FILE)
            if time.time() - mtime < CACHE_TTL:
                with open(CACHE_FILE) as f:
                    return f.read().strip()
    except Exception:
        pass

    branch = get_git_branch(cwd)

    # Update cache
    try:
        with open(CACHE_FILE, 'w') as f:
            f.write(branch)
    except Exception:
        pass

    return branch
```

### 性能考虑

1. **避免昂贵操作**：不要执行 `git status`，避免调用外部 API
2. **减少 subprocess**：一次处理多个字段
3. **使用缓存**：Git 分支等信息可以缓存
4. **保持轻量**：每次交互都会更新，脚本越轻响应越快

### 故障排查

| 问题 | 解决方案 |
|------|----------|
| 状态栏不显示 | 优先使用 Python 版本，避免 bash 转义问题 |
| 脚本执行失败 | 检查脚本路径和执行权限 (`chmod +x`) |
| Git 分支不显示 | 确认当前目录在 git 仓库中 |
| 颜色显示异常 | 确认终端支持 ANSI 颜色 |

### 注意事项

- 每次工具执行后自动刷新
- command 类型执行 shell，复杂命令会影响响应速度
- 命令失败时不显示状态栏

### 参考

- 官方文档：https://code.claude.com/docs/zh-CN/statusline
