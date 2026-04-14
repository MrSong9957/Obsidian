# Obsidian 知识库

## 仓库概览

```
obsidian-vault/
├── MEMORY.md                    # 记忆索引 + 管理规则
├── README.md                    # 知识库导航索引
├── sources/                     # 原始资料层（LLM 只读）
│   ├── claude-code-system-prompts/  # 系统提示词提取（250 篇）
│   └── learn-claude-code/       # 学习笔记（s01-s12）
├── wiki/                        # Wiki 层（LLM 可读写）
│   ├── *.md                     # 概念页 + 摘要页
│   └── contradictions/          # 矛盾标注
└── templates/                   # 页面模板
```

---

## 快速导航

### Claude Code 技能系统

| 文档 | 说明 |
|------|------|
| [[wiki/Everything-Claude-Code\|ECC 技能集]] | 110+ Skills 完整索引 |
| [[wiki/ECC-Agent-Usage-Instructions\|ECC 智能体]] | 25 个专业智能体使用说明 |
| [[wiki/Superpowers\|Superpowers]] | 开发流程技能系统 |

### 安装与配置

| 文档 | 说明 |
|------|------|
| [[wiki/ecc-install\|ECC 安装]] | 安装指南 |
| [[wiki/statusline-install\|状态栏]] | 状态栏配置 |
| [[wiki/statusline-setup\|状态栏 Python 版]] | Python 版状态栏配置 |
| [[wiki/safety-net-guide\|Safety Net]] | 安全插件指南 |
| [[wiki/auto-memory-guide\|Auto Memory 指南]] | MEMORY.md 完整参考 |
| [[wiki/claudemd-guide\|CLAUDE.md 指南]] | 加载机制、三级作用域 |
| [[wiki/claude-rules-guide\|.claude/rules/ 指南]] | 规则触发机制 |

### 开发指南

| 文档 | 说明 |
|------|------|
| [[wiki/ui-design\|UI 设计]] | 前端设计原则 |
| [[wiki/playwright-cli-guide\|Playwright]] | 测试指南 |
| [[wiki/harness-engineering\|Harness Engineering]] | AI Agent-First 开发方法论 |
| [[wiki/natural-language-programming\|人话编程]] | 自然语言编程规范 |
| [[wiki/terminal-ui-testing\|终端 UI 测试]] | VHS + AI 评估器方案 |
| [[wiki/cli-tools-productivity\|CLI 工具]] | 10 个提升生产力的 CLI 工具 |

### 学习笔记

| 文档 | 说明 |
|------|------|
| [[wiki/summary-learn-claude-code\|Learn Claude Code 摘要]] | 12 个渐进式脚本的 4 阶段摘要 |
| [[wiki/summary-claude-code-system-prompts\|系统提示词摘要]] | 250 个提示词的 6 大分类与格式 |
| [[wiki/claude-code-agent-architecture\|Agent 架构]] | 教学版 vs 生产版交叉验证 |
| [[wiki/multi-agent-framework\|多智能体框架]] | 迭代框架精要 |
| [[wiki/ai-era-skills\|AI 时代技能]] | L2-L6 技能体系 |
| [[wiki/learning-methods\|学习方法论]] | 学习路径与螺旋上升 |

### 经验与偏好

| 文档 | 说明 |
|------|------|
| [[wiki/debugging-lessons\|调试经验]] | 调试教训记录 |
| [[wiki/frontend-aesthetics\|前端美学]] | 设计原则 |
| [[wiki/code-quality\|代码质量]] | 质量约束 |
| [[wiki/terminal-ui-testing-memory\|终端 UI 测试笔记]] | 测试方案记忆 |

### 原始资料

> 只读层：外部仓库和原始学习材料

[[sources/claude-code-system-prompts/README\|Claude Code 系统提示词]]（250 篇）

---

## 管理方法

### 三层架构

1. **原始资料层** (`sources/`) — 外部仓库、原始材料，LLM 只读
2. **Wiki 层** (`wiki/`) — 结构化知识，LLM 可读写
3. **Schema 层** (`.claude/CLAUDE.md`) — LLM 行为规则

详见 [[LLM-WIKI-ARCHITECTURE|架构设计文档]]
