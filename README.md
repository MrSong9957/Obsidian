# Obsidian 知识库

## 仓库概览

```
memory/
├── MEMORY.md                    # 记忆索引 + 管理规则
├── README.md                    # 知识库导航索引
├── LLM-WIKI-ARCHITECTURE.md     # 三层架构设计文档
├── sources/                     # 原始资料层（LLM 只读）
│   ├── claude-code-system-prompts/  # 系统提示词提取（250 篇）
│   └── learn-claude-code/       # 学习笔记（s01-s12）
├── wiki/                        # Wiki 层（LLM 可读写）
│   ├── *.md                     # 概念页 + 摘要页
│   └── contradictions/          # 矛盾标注
└── skills/                      # 技能定义
```

---

## 快速导航

### Claude Code 技能系统

| 文档 | 说明 |
|------|------|
| [[wiki/Everything-Claude-Code.md\|ECC 技能集]] | 110+ Skills 完整索引 |
| [[wiki/ECC-Agent-Usage-Instructions.md\|ECC 智能体]] | 25 个专业智能体使用说明 |
| [[wiki/Superpowers.md\|Superpowers]] | 开发流程技能系统 |

### 安装与配置

| 文档 | 说明 |
|------|------|
| [[wiki/ecc-install.md\|ECC 安装]] | 安装指南 |
| [[wiki/statusline-install.md\|状态栏]] | 状态栏配置 |
| [[wiki/statusline-setup.md\|状态栏 Python 版]] | Python 版状态栏配置 |
| [[wiki/safety-net-guide.md\|Safety Net]] | 安全插件指南 |
| [[wiki/auto-memory-guide.md\|Auto Memory 指南]] | MEMORY.md 完整参考 |
| [[wiki/claudemd-guide.md\|CLAUDE.md 指南]] | 加载机制、三级作用域 |
| [[wiki/claude-rules-guide.md\|.claude/rules/ 指南]] | 规则触发机制 |
| [[wiki/obsidian-cli-guide.md\|Obsidian CLI]] | 命令行工具速查 |
| [[wiki/zhipu-mcp-setup.md\|智谱 MCP]] | 智谱 AI 4 个 MCP 服务器安装 |

### 开发指南

| 文档 | 说明 |
|------|------|
| [[wiki/harness-engineering.md\|Harness Engineering]] | AI Agent-First 开发方法论 |
| [[wiki/natural-language-programming.md\|人话编程]] | 自然语言编程规范 |
| [[wiki/terminal-ui-testing.md\|终端 UI 测试]] | VHS + AI 评估器方案 |
| [[wiki/ui-design.md\|UI 设计]] | 前端设计原则 |
| [[wiki/playwright-cli-guide.md\|Playwright]] | 测试指南 |
| [[wiki/cli-tools-productivity.md\|CLI 工具]] | 10 个提升生产力的 CLI 工具 |
| [[wiki/container-python-setup.md\|容器 Python]] | 容器网络受限环境安装 Python |

### 架构与学习笔记

| 文档 | 说明 |
|------|------|
| [[wiki/s_full_architecture.md\|s_full 架构]] | 完整底层逻辑中文文档（s01-s11 整合） |
| [[wiki/claude-code-agent-architecture.md\|Agent 架构]] | 教学版 vs 生产版交叉验证 |
| [[wiki/summary-learn-claude-code.md\|Learn Claude Code 摘要]] | 12 个渐进式脚本的 4 阶段摘要 |
| [[wiki/summary-claude-code-system-prompts.md\|系统提示词摘要]] | 250 个提示词的 6 大分类与格式 |
| [[wiki/multi-agent-framework.md\|多智能体框架]] | 迭代框架精要 |
| [[wiki/ai-era-skills.md\|AI 时代技能]] | L2-L6 技能体系 |
| [[wiki/learning-methods.md\|学习方法论]] | 学习路径与螺旋上升 |

### 经验与偏好

| 文档 | 说明 |
|------|------|
| [[wiki/feedback_three_stage_observability.md\|三段式可观测性]] | before/after/failed 事件日志调试法 |
| [[wiki/feedback_tdd.md\|TDD 实践]] | 安全/工具函数先写边界测试再实现 |
| [[wiki/feedback_code_teaching.md\|代码讲解策略]] | 底层逻辑优先，遇新概念零基础教学 |
| [[wiki/feedback_claudemd_layout.md\|CLAUDE.md 编排]] | 稳定前置、精简可推导内容的 KV 缓存优化 |
| [[wiki/kv-cache-prompt-design.md\|KV 缓存提示词]] | 提示词前缀稳定性与缓存命中率设计 |
| [[wiki/tech_stack_strategy.md\|技术栈策略]] | 原型 HTML/JS，正式 Flutter，高性能 Rust |
| [[wiki/frontend-aesthetics.md\|前端美学]] | 避免 AI 泛滥设计 |
| [[wiki/code-quality.md\|代码质量]] | 质量约束 |
| [[wiki/debugging-lessons.md\|调试经验]] | 调试教训记录 |
| [[wiki/terminal-ui-testing-memory.md\|终端 UI 测试笔记]] | 测试方案记忆 |

### 原始资料

> 只读层：外部仓库和原始学习材料

[[sources/claude-code-system-prompts/README.md\|Claude Code 系统提示词]]（250 篇）

---

## 管理方法

### 三层架构

1. **原始资料层** (`sources/`) — 外部仓库、原始材料，LLM 只读
2. **Wiki 层** (`wiki/`) — 结构化知识，LLM 可读写
3. **Schema 层** (`.claude/CLAUDE.md`) — LLM 行为规则

详见 [[LLM-WIKI-ARCHITECTURE.md|架构设计文档]]
