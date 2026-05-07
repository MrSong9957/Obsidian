# Obsidian 知识库

> 本文档是知识库的**冷数据导航索引**。当 MEMORY.md（热数据）中未找到相关信息时，在此处按领域浏览查找。

## 仓库概览

```
obsidian/
├── MEMORY.md                        # 记忆索引 + 管理规则（热数据）
├── README.md                        # 知识库导航索引（冷数据）
├── LLM-WIKI-ARCHITECTURE.md         # 三层架构设计文档
├── wiki/                            # Wiki 层（16 个文件，按领域组织）
├── sources/                         # 原始资料层（18 个文件 + 2 个子仓库）
│   ├── claude-code-system-prompts/  # 系统提示词提取（250 篇）
│   └── learn-shareai/              # 学习笔记（s01-s19）
├── skills/                          # 技能定义（7 个）
└── templates/                       # 页面模板（3 个）
```

---

## Wiki 导航

### Claude Code

| 文档 | 说明 |
|------|------|
| [[wiki/claude-code-p1.md\|配置]] | settings.json、CLAUDE.md、rules、记忆系统 |
| [[wiki/claude-code-p2.md\|工具]] | Superpowers、SafetyNet、Statusline |
| [[wiki/claude-md-base.md\|CLAUDE.md 模板]] | 双层架构模板（全局 + 项目级） |

### 开发实践

| 文档 | 说明 |
|------|------|
| [[wiki/dev-practice.md\|开发实践]] | TDD、代码质量、调试、提示词技巧、KV 缓存 |
| [[wiki/frontend.md\|前端与 UI]] | UI 设计、终端 UI 测试、Playwright、技术栈策略 |
| [[wiki/git-security.md\|Git 安全]] | 认证安全、token 管理、credential 配置 |

### 架构与学习

| 文档 | 说明 |
|------|------|
| [[wiki/agent-arch-p1.md\|Agent 架构（上）]] | Agent 循环、多智能体、系统提示词 |
| [[wiki/agent-arch-p2.md\|Agent 架构（下）]] | Harness 工程方法论、人话编程规范 |
| [[wiki/learn-shareai.md\|Learn ShareAI]] | s01-s19 从零构建 Agent 学习笔记 |
| [[wiki/learning.md\|学习方法]] | AI 时代技能、学习方法论、认知提升 |
| [[wiki/mental-models.md\|思维模型]] | 跨学科思维模型（芒格 100 模型） |

### 工具指南

| 文档 | 说明 |
|------|------|
| [[wiki/cli-tools-productivity.md\|CLI 工具]] | 10 个提升生产力的 CLI 工具推荐 |
| [[wiki/container-python-setup.md\|容器 Python]] | 容器网络受限环境安装 Python |
| [[wiki/obsidian.md\|Obsidian]] | Obsidian CLI、知识仓库管理 |
| [[wiki/yazi.md\|Yazi]] | 文件管理器配置、主题、配色 |
| [[wiki/zhipu-mcp.md\|智谱 MCP]] | MCP 服务器配置、Web 抓取问题排查 |

---

## 原始资料（sources/）

### 子仓库

| 资料集 | 说明 |
|--------|------|
| [[sources/claude-code-system-prompts/\|Claude Code 系统提示词]] | 系统提示词 + 工具定义（250 篇） |
| [[sources/learn-shareai/\|Learn ShareAI 原始笔记]] | s01-s19 逐章学习笔记 |

### 独立资料

| 资料文件 | 说明 |
|----------|------|
| [[sources/claude-code-p1.md\|Claude Code 配置原始资料]] | 配置系统的完整调试记录 |
| [[sources/claude-code-cli-path-fix.md\|CLI 路径修复]] | CLI 路径问题排查 |
| [[sources/claude-code-first-time-setup.md\|首次配置]] | 完整首次安装流程 |
| [[sources/claude-code-statusline-setup.md\|状态栏配置]] | 状态栏安装调试记录 |
| [[sources/agent-arch-p1.md\|Agent 架构（上）原始资料]] | Agent 循环与多智能体学习过程 |
| [[sources/agent-arch-p2.md\|Agent 架构（下）原始资料]] | Harness 工程学习过程 |
| [[sources/dev-practice.md\|开发实践原始资料]] | TDD/调试等方法论的完整记录 |
| [[sources/frontend.md\|前端原始资料]] | UI/测试/技术栈选择的完整过程 |
| [[sources/claude-collaboration-prompting.md\|协作提示词]] | 协作场景提示词设计 |
| [[sources/obsidian.md\|Obsidian 原始资料]] | Obsidian 使用过程记录 |
| [[sources/obsidian-knowledge-system.md\|知识系统设计]] | 知识管理系统架构讨论 |
| [[sources/charlie-munger-mental-models.md\|芒格思维模型]] | 芒格 100 思维模型原始资料 |
| [[sources/learning.md\|学习原始资料]] | 学习方法论完整记录 |
| [[sources/openai-harness-engineering.md\|Harness Engineering 原始资料]] | OpenAI Harness 工程完整讨论 |
| [[sources/stop-hook-design.md\|Stop Hook 设计]] | Stop Hook 设计过程 |
| [[sources/yazi-theme-setup.md\|Yazi 主题配置]] | Yazi 主题安装调试过程 |
| [[sources/zhipu-mcp.md\|智谱 MCP 原始资料]] | 智谱 MCP 配置调试记录 |
| [[sources/feedback_git-token-security.md\|Git Token 安全事件]] | Git token 泄露教训记录 |

---

## 技能（skills/）

| 技能 | 用途 |
|------|------|
| [[skills/agent-builder/SKILL.md\|agent-builder]] | 构建 AI Agent |
| [[skills/auto-optimize/SKILL.md\|auto-optimize]] | 自动优化技能 |
| [[skills/docker-dev-container/SKILL.md\|docker-dev-container]] | Docker 容器开发 |
| [[skills/mcp-builder/SKILL.md\|mcp-builder]] | MCP 服务器构建 |
| [[skills/obsidian-link/SKILL.md\|obsidian-link]] | Obsidian 仓库链接 |
| [[skills/obsidian-remember/SKILL.md\|obsidian-remember]] | 知识仓库管理 |
| [[skills/pdf/SKILL.md\|pdf]] | PDF 处理 |

---

## 管理方法

### 三层架构

1. **原始资料层** (`sources/`) — 外部仓库、原始材料，可读写，禁止删除
2. **Wiki 层** (`wiki/`) — 结构化知识，LLM 可读写
3. **Schema 层** (`.claude/CLAUDE.md`) — LLM 行为规则

### 记忆查找优先级

1. **MEMORY.md**（热数据）— 始终在上下文中，优先查找
2. **README.md**（冷数据）— MEMORY.md 未命中时，按领域导航查找
3. **wiki/**（详情层）— 通过 MEMORY.md 或 README.md 的索引定位

详见 [[LLM-WIKI-ARCHITECTURE.md|架构设计文档]]
