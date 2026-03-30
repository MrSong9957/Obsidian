# Obsidian 知识库

## 仓库概览

```
obsidian-vault/
├── MEMORY.md                    # 全仓库文档索引
├── References/                  # 所有参考材料 + 学习笔记（24 篇）
└── claude-code-system-prompts/  # Claude Code 系统提示词提取（110+ 篇）
```

---

## 快速导航

### 参考文档

#### Claude Code 技能系统

| 文档 | 说明 |
|------|------|
| [[References/Everything-Claude-Code\|ECC 技能集]] | 110+ Skills 完整索引 |
| [[References/ECC-Agent-Usage-Instructions\|ECC 智能体]] | 25 个专业智能体使用说明 |
| [[References/Superpowers\|Superpowers]] | 开发流程技能系统 |

#### 安装与配置

| 文档 | 说明 |
|------|------|
| [[References/ecc-install\|ECC 安装]] | 安装指南 |
| [[References/statusline-install\|状态栏]] | 状态栏配置 |
| [[References/statusline-setup\|状态栏 Python 版]] | Python 版状态栏配置 |
| [[References/safety-net-guide\|Safety Net]] | 安全插件指南 |
| [[References/auto-memory-guide\|Auto Memory 指南]] | MEMORY.md 完整参考：加载机制、存储规则 |
| [[References/claudemd-guide\|CLAUDE.md 指南]] | 加载机制、三级作用域、compaction 行为、最佳实践 |
| [[References/claude-rules-guide\|.claude/rules/ 指南]] | 无条件/Scoped 规则、触发机制、glob 模式、最佳实践 |

#### 开发指南

| 文档 | 说明 |
|------|------|
| [[References/ui-design\|UI 设计]] | 前端设计原则 |
| [[References/playwright-cli-guide\|Playwright]] | 测试指南 |
| [[References/harness-engineering\|Harness Engineering]] | AI Agent-First 开发方法论 |
| [[References/terminal-ui-testing\|终端 UI 测试]] | VHS + AI 评估器方案 |
| [[References/cli-tools-productivity\|CLI 工具]] | 10 个提升生产力的 CLI 工具 |

### 学习笔记

| 文档 | 说明 |
|------|------|
| [[References/learn-claude-code\|Learn Claude Code]] | shareAI-lab 仓库学习笔记 |
| [[References/multi-agent-framework\|多智能体框架]] | 迭代框架精要 |
| [[References/ai-era-skills\|AI 时代技能]] | L2-L6 技能体系 |
| [[References/科学提升智商：六大核心习惯与实践指南\|科学提升智商]] | 六大核心习惯与实践指南 |

### 经验与偏好

| 文档 | 说明 |
|------|------|
| [[References/debugging-lessons\|调试经验]] | 调试教训记录 |
| [[References/frontend-aesthetics\|前端美学]] | 设计原则 |
| [[References/code-quality\|代码质量]] | 质量约束 |
| [[References/terminal-ui-testing-memory\|终端 UI 测试笔记]] | 测试方案记忆 |
| [[References/claude-memory-old\|旧 Memory 索引]] | 历史记忆索引存档 |

### Claude Code 系统提示词

> 外部仓库：110+ Claude Code 系统提示词提取

[[claude-code-system-prompts/README|查看完整目录]]

---

## 管理方法

### 核心原则

1. **根目录保持简洁** -- MEMORY.md + README.md
2. **单一文档池** -- 所有文档放 References/，通过 MEMORY.md 索引分类
3. **索引驱动** -- MEMORY.md 是唯一的文档入口
