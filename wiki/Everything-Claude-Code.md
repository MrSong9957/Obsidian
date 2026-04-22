---
name: ECC 技能集总览
description: Claude Code 扩展技能集，110+ Skills/Rules/MCP 配置 — 需要了解技能全貌或查找特定技能时查阅
type: summary
sources:
  - "[[sources/claude-code-system-prompts/README]]"
updated: 2026-04-05
tags: [claude-code, skills]
status: deprecated
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# Everything Claude Code (ECC)

> 内容已迁移至 [[wiki/claude-code-p2.md|claude-code-p2]]。

> Claude Code 扩展技能集，包含 110+ Skills、Rules、MCP 配置和项目模板。

**相关文档**：[[ECC-Agent-Usage-Instructions.md|ECC 智能体使用]] | [[Superpowers.md|Superpowers 技能系统]] | [[ecc-install.md|安装指南]]

---

## Skills (112个)

### ECC 核心技能 (110个)

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

### 官方 Skill 插件 (2个)

| 名称 | 说明 |
|------|------|
| skill-creator | 创建新技能、改进现有技能、测量技能性能 |
| frontend-design | 前端设计模式 - UI/UX、响应式设计、组件设计 |

---

## 快速索引

### 按功能分类

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

---

## Rules 编码规范

来自 `everything-claude-code/rules/`

### 通用规范 (common/)

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

### 语言特定规范

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

---

## MCP 服务配置

来自 `everything-claude-code/mcp-configs/mcp-servers.json`

### 开发工具

| 服务 | 说明 |
|------|------|
| github | GitHub 操作 - PR、Issue、仓库管理 |
| filesystem | 文件系统操作 |
| memory | 跨会话持久化记忆 |
| sequential-thinking | 链式思维推理 |

### 数据库与存储

| 服务 | 说明 |
|------|------|
| supabase | Supabase 数据库操作 |
| clickhouse | ClickHouse 分析查询 |

### 部署平台

| 服务 | 说明 |
|------|------|
| vercel | Vercel 部署和项目 |
| railway | Railway 部署 |
| cloudflare-docs | Cloudflare 文档搜索 |
| cloudflare-workers-* | Cloudflare Workers 构建/绑定/可观测性 |

### 搜索与研究

| 服务 | 说明 |
|------|------|
| exa-web-search | 网页搜索、研究、数据采集 |
| firecrawl | 网页抓取和爬取 |
| context7 | 实时文档查找 |

### 浏览器自动化

| 服务 | 说明 |
|------|------|
| playwright | 浏览器自动化和测试 |
| browserbase | 云端浏览器会话 |
| browser-use | AI 浏览器 Agent |

### AI 与媒体

| 服务 | 说明 |
|------|------|
| fal-ai | AI 图像/视频/音频生成 |
| magic | Magic UI 组件 |

### 协作与监控

| 服务 | 说明 |
|------|------|
| confluence | Confluence Cloud 集成 |
| devfleet | 多 Agent 编排 - 并行 Claude Code 调度 |
| insaits | AI 安全监控 - 异常检测、凭证暴露检查 |
| token-optimizer | Token 优化，95%+ 上下文压缩 |

> **注意**: 建议保持启用 MCP < 10 个，以保护上下文窗口

---

## 项目模板 Examples

来自 `everything-claude-code/examples/`

| 模板 | 技术栈 | 适用场景 |
|------|--------|----------|
| **saas-nextjs-CLAUDE.md** | Next.js 15 + Supabase + Stripe + Tailwind | SaaS 应用 |
| **django-api-CLAUDE.md** | Django 5.x + DRF + PostgreSQL + Celery | REST API |
| **go-microservice-CLAUDE.md** | Go + gRPC + PostgreSQL + Wire | 微服务 |
| **rust-api-CLAUDE.md** | Rust + Axum + SQLx + PostgreSQL | 高性能 API |
| **laravel-api-CLAUDE.md** | Laravel 11.x + PostgreSQL + Redis | PHP API |
| **user-CLAUDE.md** | 通用用户级配置模板 | 个人项目 |

### 使用方法

```bash
# 复制模板到项目根目录
cp examples/saas-nextjs-CLAUDE.md your-project/CLAUDE.md

# 根据项目自定义配置
# 编辑 CLAUDE.md 修改技术栈、规则、文件结构等
```

### 模板核心内容

每个模板包含：
- **项目概述**: 技术栈和架构说明
- **关键规则**: 数据库、认证、计费、代码风格
- **文件结构**: 推荐的目录组织
- **测试要求**: 覆盖率和测试策略
- **部署配置**: CI/CD 和环境变量
