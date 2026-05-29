---
type: index
updated: 2026-05-30
---

# 原始资料索引

## 目录结构

| 子目录 | 用途 |
|--------|------|
| `agent/` | Agent 架构、多代理系统、ShareAI 系列 |
| `claude-code/` | Claude Code 配置、CLI、MCP 集成、system-prompts |
| `dev/` | 编程语言、开发实践、前端/后端技术 |
| `feedback/` | 用户反馈、使用经验、改进建议 |
| `learning/` | 学习方法论、思维模型、认知科学 |
| `projects/` | 项目规划、进度记录、发布方案 |
| `security/` | 安全实践、密钥管理、漏洞分析 |
| `tools/` | CLI 工具、编辑器、开发环境配置 |

## agent/

| 文件 | 说明 |
|------|------|
| [[sources/agent/agent-arch-p1\|Agent 架构（上）]] | Agent循环/多智能体框架/系统提示词/教学脚本 |
| [[sources/agent/agent-arch-p2\|Agent 架构（下）]] | Harness工程方法论/人话编程规范 |
| [[sources/agent/openai-harness-engineering\|Harness Engineering]] | OpenAI 团队用 0 行人工代码构建产品的实验总结 |

## claude-code/

| 文件 | 说明 |
|------|------|
| [[sources/claude-code/claude-code-p1\|Claude Code 配置（完整参考）]] | settings.json/CLAUDE.md/rules/记忆系统/技能位置 |
| [[sources/claude-code/claude-code-first-time-setup\|首次安装配置指南]] | 首次安装或初始化 Claude Code 后的完整配置流程 |
| [[sources/claude-code/claude-code-cli-path-fix\|CLI 路径修复]] | CLI 路径问题排查 |
| [[sources/claude-code/claude-code-statusline-setup\|状态栏配置]] | 状态栏脚本配置 |
| [[sources/claude-code/2026-05-13-safety-net-hook-fix\|Safety Net Hook 修复]] | Safety Net 集成修复记录 |
| [[sources/claude-code/ecc\|ECC 完整参考]] | Skills/Hooks/Subagents/MCPs/Plugins/Session Memory |
| [[sources/claude-code/claude-collaboration-prompting\|协作提示词技巧]] | 10 条从协作角度优化 Claude 输出质量的方法 |
| [[sources/claude-code/stop-hook-design\|Stop Hook 设计文档]] | V4 最终方案（关键词触发 + 后台 agent） |
| [[sources/claude-code/subagent-skill-test-2026-05-10\|子代理 Skill 测试]] | 子代理/团队成员能否使用 Skill 工具的实测 |
| [[sources/claude-code/zhipu-mcp\|智谱 MCP 安装指南]] | 智谱 AI 4 个 MCP 服务器安装与配置 |

## dev/

| 文件 | 说明 |
|------|------|
| [[sources/dev/dev-practice\|开发实践]] | TDD/调试案例/代码讲解/提示词技巧 |
| [[sources/dev/frontend\|前端与 UI]] | UI设计原则/终端UI测试/Playwright自动化/技术栈策略 |
| [[sources/dev/rust\|Rust 学习]] | Rust 语言学习资料 |

## feedback/

（暂无独立文件，反馈内容归入 security/ 等对应目录）

## learning/

| 文件 | 说明 |
|------|------|
| [[sources/learning/charlie-munger-mental-models\|芒格100思维模型]] | 跨学科思维模型完整参考（8大类100个模型） |
| [[sources/learning/learning\|学习方法]] | AI时代技能发展/学习方法论/认知提升习惯 |

## projects/

| 文件 | 说明 |
|------|------|
| [[sources/projects/email-assistant-journey\|Email Assistant 开发历程]] | 从零到一的完整开发过程、调试故事、踩坑记录 |
| [[sources/projects/project-roadmap\|项目发展规划]] | 项目发展方向三阶段路线图（开源→框架→SaaS） |
| [[sources/projects/publish-plan\|发布方案]] | Email Assistant 多平台发布方案与一键发布工具 |
| [[sources/projects/2026-05-14-persona-style-progress\|人格化回复进度]] | 人格化邮件回复功能开发进度（已合并 master） |

## security/

| 文件 | 说明 |
|------|------|
| [[sources/security/feedback_git-token-security\|Git Token 安全]] | 避免明文 token，改用 credential store |

## tools/

| 文件 | 说明 |
|------|------|
| [[sources/tools/obsidian\|Obsidian CLI]] | Obsidian CLI 完整参考/快捷键/自动化脚本 |
| [[sources/tools/obsidian-knowledge-system\|Obsidian 知识体系]] | Obsidian 知识管理系统 |
| [[sources/tools/opencode\|OpenCode 配置]] | OpenCode 配置、MCP、provider |
| [[sources/tools/2026-05-15-token-management\|Token 管理]] | Token 获取与集中管理 |
| [[sources/tools/yazi-preview-line-numbers\|Yazi 行号预览]] | Yazi 预览行号方案探索与实现 |
| [[sources/tools/yazi-theme-setup\|Yazi 主题配置]] | Yazi 主题安装与配色配置 |
| [[sources/tools/yazi-wrap-fix\|Yazi 换行修复]] | Yazi 预览不换行问题排查 |

## 集合

| 路径 | 文件数 | 说明 |
|------|--------|------|
| [[sources/claude-code/claude-code-system-prompts/\|claude-code-system-prompts]] | 252 | Claude Code System Prompts |
| [[sources/agent/learn-shareai/\|learn-shareai]] | 19 | Agent 核心循环系列 |
