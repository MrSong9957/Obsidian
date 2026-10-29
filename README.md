# Obsidian 知识库

> 本文档是知识库的**导航索引**。检索优先使用 wiki MCP（`wiki_search`）；无 MCP 时在此按领域浏览查找。根目录 [MEMORY.md](MEMORY.md) 是文件记忆入口，仅服务 Junction/链接方式使用本 vault 的外部项目会话。

## 仓库概览

```
obsidian/
├── CLAUDE.md / AGENTS.md            # Schema 层（LLM 行为规则 + 分类规则）
├── MEMORY.md                        # 文件记忆入口（少量偏好与知识指针）
├── README.md                        # 知识库导航索引
├── LLM-WIKI-ARCHITECTURE.md         # 架构决策记录（ADR）
├── wiki/                            # Wiki 层（按领域组织）
├── sources/                         # 原始资料层（按子目录分类）
│   ├── agent/                       # Agent 架构、多代理系统
│   ├── claude-code/                 # Claude Code 配置、CLI、MCP
│   ├── dev/                         # 编程语言、开发实践
│   ├── learning/                    # 学习方法论、思维模型
│   ├── projects/                    # 项目规划、进度记录
│   ├── security/                    # 安全实践、密钥管理
│   └── tools/                       # CLI 工具、编辑器配置
├── skills/                          # 技能定义（17 个）
└── templates/                       # 页面模板（4 个）
```

---

## Wiki 导航

### Claude Code

| 文档 | 说明 |
|------|------|
| [[wiki/claude-code.md\|配置与工作流]] | settings.json、CLAUDE.md、rules、记忆系统、插件、Hooks、子代理、MCP |
| [[wiki/claude-code-setup.md\|当前配置清单]] | 本机完整配置 + 自检清单 + 缓存优化 + cc switch 模板 |
| [[wiki/AGENTS-md-base.md\|AGENTS.md 模板]] | 双层架构模板（全局 + 项目级） |
| [[wiki/agents-md-claude-md.md\|AGENTS.md 与 CLAUDE.md 配置关系]] | 四层级放置规则、AGENTS.md 默认替补逻辑、四档配置、import 共存方案 |

### 开发实践

| 文档 | 说明 |
|------|------|
| [[wiki/dev-practice.md\|开发实践]] | TDD、代码质量、调试、提示词技巧、KV 缓存 |
| [[wiki/frontend.md\|前端与 UI]] | UI 设计、终端 UI 测试、Playwright、技术栈策略 |
| [[wiki/dev-practice.md\|Git 安全]] | 认证安全、token 管理、credential 配置 |

### 架构与学习

| 文档 | 说明 |
|------|------|
| [[wiki/agent-arch.md\|Agent 架构]] | Agent 循环、多智能体、Harness 工程方法论 |
| [[wiki/learn-shareai.md\|Learn ShareAI]] | s01-s19 从零构建 Agent 学习笔记 |
| [[wiki/规格文档.md\|规格文档]] | 学习编程主方法——每个功能一份规格提示词，预测→实测验收，规格是母版、代码随用随生成 |
| [[wiki/mental-models.md\|思维模型]] | 跨学科思维模型（芒格 100 模型） |

### 工具指南

| 文档 | 说明 |
|------|------|
| [[wiki/cli-tools-productivity.md\|CLI 工具]] | 10 个提升生产力的 CLI 工具推荐 |
| [[wiki/container-python-setup.md\|容器 Python]] | 容器网络受限环境安装 Python |
| [[wiki/obsidian.md\|Obsidian]] | Obsidian CLI、知识仓库管理 |
| [[wiki/yazi.md\|Yazi]] | 文件管理器配置、主题、配色 |
| [[wiki/zhipu-mcp.md\|智谱 MCP]] | MCP 服务器配置、Web 抓取问题排查 |
| [[wiki/wiki-query-deploy.md\|wiki-query MCP 部署]] | wiki-query MCP 安装/配置/跨平台分发（GitHub Actions）与踩坑 |

---

## 原始资料（sources/）

按子目录分类组织，详见 [[sources/_index.md|原始资料索引]]。

| 子目录 | 用途 | 代表资料 |
|--------|------|----------|
| `agent/` | Agent 架构、多代理系统 | agent-arch-p1/p2、learn-shareai、openai-harness |
| `claude-code/` | Claude Code 配置、CLI、MCP | claude-code-p1、ecc、zhipu-mcp、stop-hook-design |
| `DeepSeek-Harness/` | DeepSeek Harness、AI IDE 设计资料 | deepseek-harness-context-essentials、ai-ide、vs-code-插件 |
| `dev/` | 编程语言、开发实践 | dev-practice、frontend、rust |
| `learning/` | 学习方法论、思维模型 | charlie-munger-mental-models、learning |
| `projects/` | 项目规划、进度记录 | email-assistant-journey、project-roadmap、publish-plan、mimocode-reading-guide、wiki-query-deploy-references |
| `security/` | 安全实践、密钥管理 | feedback_git-token-security |
| `tools/` | CLI 工具、编辑器配置 | obsidian、opencode、yazi-*、token-management |

---

## 技能（skills/）

| 技能 | 用途 |
|------|------|
| [[skills/agent-builder/SKILL.md\|agent-builder]] | 构建 AI Agent |
| [[skills/ai-era-fundamentals-learning/SKILL.md\|ai-era-fundamentals-learning]] | AI 时代底层原理学习法辅助学习编程语言 |
| [[skills/auto-optimize/SKILL.md\|auto-optimize]] | 自动优化技能 |
| [[skills/claude-code-config-health/SKILL.md\|claude-code-config-health]] | Claude Code 配置健康检查 |
| [[skills/comprehensive-testing/SKILL.md\|comprehensive-testing]] | 系统性回归强化测试 |
| [[skills/docker-dev-container/SKILL.md\|docker-dev-container]] | Docker 容器开发 |
| [[skills/obsidian-link/SKILL.md\|obsidian-link]] | Obsidian 仓库链接 |
| [[skills/obsidian-remember/SKILL.md\|obsidian-remember]] | 知识仓库管理 |
| [[skills/pdf/SKILL.md\|pdf]] | PDF 处理 |
| [[skills/playwright-cli/SKILL.md\|playwright-cli]] | 浏览器自动化测试 |
| [[skills/plain-language-knowledge/SKILL.md\|plain-language-knowledge]] | 大白话知识萃取与通俗改写 |
| [[skills/skill-creator/SKILL.md\|skill-creator]] | 创建与优化技能 |
| [[skills/wiki-mcp/SKILL.md\|wiki-mcp]] | wiki-query MCP 知识库操作 |

---

## 管理方法

### 三层架构

1. **原始资料层** (`sources/`) — 外部仓库、原始材料，可读写，禁止删除
2. **Wiki 层** (`wiki/`) — 结构化知识，LLM 可读写
3. **Schema 层** (`CLAUDE.md` / `AGENTS.md`) — LLM 行为规则

### 检索方式

1. **wiki MCP**（可用时）— `wiki_search` 全文检索 wiki/ 与 sources/
2. **README.md** — 无 MCP 时按领域导航定位 wiki/ 文件
3. **文件搜索** — 必要时 rg / glob 兜底

根目录 [MEMORY.md](MEMORY.md) 是**文件记忆入口**：仅服务通过 Junction/链接把本 vault 用作项目记忆的场景，维护少量长期有效的偏好与知识指针，不承担全库检索。

详见 [[LLM-WIKI-ARCHITECTURE.md|架构决策记录]]
