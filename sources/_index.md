---
type: index
updated: 2026-08-30
---

# 原始资料索引

## 目录结构

| 子目录 | 用途 |
|--------|------|
| `agent/` | Agent 架构、多代理系统、ShareAI 系列 |
| `claude-code/` | Claude Code 配置、CLI、MCP 集成、system-prompts |
| `DeepSeek-Harness/` | DeepSeek Harness、AI IDE 设计资料 |
| `dev/` | 编程语言、开发实践、前端/后端技术 |
| `learning/` | 学习方法论、思维模型、认知科学 |
| `projects/` | 项目规划、进度记录、发布方案 |
| `security/` | 安全实践、密钥管理、漏洞分析 |
| `tools/` | CLI 工具、编辑器、开发环境配置 |

## agent/

| 文件 | 说明 |
|------|------|
| [[sources/agent/agent-arch-references\|Agent 架构详细参考]] | 详细内容索引（指向 p1/p2 文件） |
| [[sources/agent/agent-arch-p1\|Agent 架构（上）]] | Agent循环/多智能体框架/系统提示词/教学脚本（详细内容） |
| [[sources/agent/agent-arch-p2\|Agent 架构（下）]] | Harness工程方法论/人话编程规范（详细内容） |
| [[sources/agent/openai-harness-engineering\|Harness Engineering]] | OpenAI 团队用 0 行人工代码构建产品的实验总结 |
| [[sources/agent/how-openai-uses-codex\|How OpenAI uses Codex]] | OpenAI 内部 7 类 Codex 用法与 6 条最佳实践全量中译（官方手册 PDF） |

## claude-code/

| 文件 | 说明 |
|------|------|
| [[sources/claude-code/claude-code-references\|Claude Code 配置详细参考]] | CLAUDE.md/rules/Memory/StatusLine/诊断/子代理/Hooks/Token经济学/进阶技术 |
| [[sources/claude-code/claude-code-setup-references\|配置清单详细参考]] | cc switch诊断/状态栏源码/Hooks机制/Rules安装/缓存优化/MiMoCode链接 |
| [[sources/claude-code/claude-code-first-time-setup\|首次安装配置指南]] | 首次安装或初始化 Claude Code 后的完整配置流程 |
| [[sources/claude-code/new-project-setup-references\|新项目初始化详细参考]] | 基础镜像更新命令清单、构建故障排查、Docker Hub Push 代理配置 |
| [[sources/claude-code/claude-code-cli-path-fix\|CLI 路径修复]] | CLI 路径问题排查 |
| [[sources/claude-code/claude-code-statusline-setup\|状态栏配置]] | 状态栏脚本配置 |
| [[sources/claude-code/2026-05-13-safety-net-hook-fix\|Safety Net Hook 修复]] | Safety Net 集成修复记录 |
| [[sources/claude-code/claude-collaboration-prompting\|协作提示词技巧]] | 10 条从协作角度优化 Claude 输出质量的方法 |
| [[sources/claude-code/stop-hook-design\|Stop Hook 设计文档]] | V4 最终方案（关键词触发 + 后台 agent） |
| [[sources/claude-code/subagent-skill-test-2026-05-10\|子代理 Skill 测试]] | 子代理/团队成员能否使用 Skill 工具的实测 |
| [[sources/claude-code/zhipu-mcp\|智谱 MCP 安装指南]] | 智谱 AI 4 个 MCP 服务器安装与配置 |
| [[sources/claude-code/creating-subagents\|创建子代理]] | 子代理创建流程 |
| [[sources/claude-code/dynamic-workflows\|动态工作流]] | 动态工作流配置与实践 |
| [[sources/claude-code/opencode-auto-mode\|OpenCode 自动模式]] | OpenCode 自动模式配置 |
| [[sources/claude-code/opencode-bridge-v2\|OpenCode 桥接 v2]] | OpenCode 桥接 v2 实现 |
| [[sources/claude-code/settings-reference\|Settings 参考]] | settings.json 完整参考 |
| [[sources/claude-code/statusline.py\|状态栏脚本 statusline.py]] | 综合版状态栏 Python 脚本（模型/目录/分支/进度条/任务/成本指标） |
| [[sources/claude-code/cc-switch-template.json\|cc switch 配置模板]] | cc switch 切换 API KEY 时重写 settings.json 的完整 JSON 模板 |
| [[sources/claude-code/ecc-references\|ECC Hooks 完整参考]] | 所有 Hook 作用、ECC_HOOK_PROFILE 档位对照 |
| [[sources/claude-code/mimocode-mcp-config\|MiMoCode MCP 配置详解]] | MiMoCode 与 Claude Code 的 MCP 配置差异、wiki-query 接入完整流程 |
| [[sources/claude-code/mimocode-mcp-config-references\|MiMoCode MCP 配置详细参考]] | 完整的 MiMoCode MCP 配置调试过程、配置文件位置对比、验证步骤 |
| [[sources/claude-code/how-anthropic-teams-use-claude-code\|How Anthropic teams use Claude Code]] | Anthropic 10 团队使用案例全量中译（官方手册 PDF） |

## DeepSeek-Harness/

| 文件 | 说明 |
|------|------|
| [[sources/DeepSeek-Harness/AI-IDE\|AI IDE 产品需求文档（PRD）]] | Rust 原生 AI IDE 设计：重构人/AI/代码/文档/插件协作方式，组件级动态插件架构 |
| [[sources/DeepSeek-Harness/DeepSeek-Harness-Context-Essentials\|DeepSeek Harness Context Essentials]] | 经 deepseek-ai/deepseek-harness 源码核对的上下文保存/组装/压缩 5 机制 |
| [[sources/DeepSeek-Harness/VS Code 插件\|VS Code 插件 MVP：结构化意图 → 代码]] | 验证"结构化意图描述比聊天 Prompt 更稳定生成代码"的插件 MVP 设计 |

> 另含架构图 image.png。

## dev/

| 文件 | 说明 |
|------|------|
| [[sources/dev/dev-practice\|开发实践]] | TDD/调试案例/代码讲解/提示词技巧 |
| [[sources/dev/frontend\|前端与 UI]] | UI设计原则/终端UI测试/Playwright自动化/技术栈策略 |
| [[sources/dev/rust\|Rust 学习]] | Rust 语言学习资料 |

## learning/

| 文件 | 说明 |
|------|------|
| [[sources/learning/charlie-munger-mental-models\|芒格100思维模型]] | 跨学科思维模型完整参考（8大类100个模型） |
| [[sources/learning/learning\|学习方法]] | AI时代技能发展/学习方法论/认知提升习惯 |
| [[sources/learning/learning-references\|学习方法参考]] | 四层认知分离法详解、编程第一性原理、AI 时代语言学习哲学 |

## projects/

| 文件 | 说明 |
|------|------|
| [[sources/projects/email-assistant-journey\|Email Assistant 开发历程]] | 从零到一的完整开发过程、调试故事、踩坑记录 |
| [[sources/projects/email-assistant-references\|Email Assistant 参考资料]] | 开发历程、技术细节、QQ 邮箱特性、项目路线图 |
| [[sources/projects/project-roadmap\|项目发展规划]] | 项目发展方向三阶段路线图（开源→框架→SaaS） |
| [[sources/projects/publish-plan\|发布方案]] | Email Assistant 多平台发布方案与一键发布工具 |
| [[sources/projects/2026-05-14-persona-style-progress\|人格化回复进度]] | 人格化邮件回复功能开发进度（已合并 master） |
| [[sources/projects/mimocode-reading-guide\|MiMo-Code 源码阅读指南]] | MiMo-Code（OpenCode fork）源码阅读顺序、核心架构、关键技术栈 |
| [[sources/projects/wiki-query-deploy-references\|wiki-query 跨平台部署记录]] | wiki-query MCP 从单机配置到 GitHub Actions 三平台分发的完整过程、踩坑与调试故事 |

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
| [[sources/tools/mimo-provider-debug\|MiMo Provider 调试]] | MiMo Provider 问题排查 |

## 集合

| 路径 | 文件数 | 说明 |
|------|--------|------|
| [[sources/claude-code/claude-code-system-prompts/\|claude-code-system-prompts]] | 252 | Claude Code System Prompts |
| [[sources/agent/learn-shareai/\|learn-shareai]] | 19 | Agent 核心循环系列 |
