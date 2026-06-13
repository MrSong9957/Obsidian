---
name: Claude Code 动态工作流（Dynamic Workflows）
description: 2.1.154 引入的多代理并行执行功能 — 触发方式、使用方法、版本演进
type: guide
updated: 2026-06-08
tags: [claude-code, workflow, ultracode, agents, parallel]
source: "~/.claude/cache/changelog.md"
---

# Claude Code 动态工作流（Dynamic Workflows）

## 概述

动态工作流是 Claude Code 2.1.154 引入的全新功能，允许 Claude 在后台协调**数十到数百个代理（agents）**来并行执行复杂任务。[^1]

核心理念：将大任务拆分成多个子任务，由多个代理同时处理。

## 触发方式

### 方式一：直接请求

在对话中描述你需要的工作流，Claude 会自动识别并创建工作流：[^1]

- "Create a workflow to refactor all API endpoints"
- "Set up a workflow to add tests to every module"
- "用工作流帮我重构所有 API 端点"

### 方式二：触发词（Keyword Trigger）

| 版本范围 | 触发词 | 说明 |
|----------|--------|------|
| 2.1.154 - 2.1.156 | `workflow` | 输入包含 "workflow" 自动触发 |
| 2.1.157+ | `ultracode` | 触发词改为 ultracode，避免误触发 |

触发词在输入框中以紫色高亮显示。[^2]

### 方式三：禁用触发词

在 `/config` 中设置 "Workflow keyword trigger" 可完全禁用触发词自动触发。[^3] 禁用后仍可通过方式一（直接描述需求）触发。


## 使用命令

| 命令 | 作用 |
|------|------|
| `/workflows` | 查看当前运行的工作流 |
| `/effort ultracode` | 设置工作流的努力级别（需模型支持 xhigh） |

## 版本演进

| 版本 | 变更 |
|------|------|
| 2.1.154 | 引入动态工作流，触发词为 "workflow" [^1] |
| 2.1.157 | 触发词改为 "ultracode"，添加 Workflow keyword trigger 设置 [^2][^3] |
| 2.1.160 | 修复 `/effort ultracode` 在模型不支持 xhigh 时的错误提示 [^5] |
| 2.1.162 | 修复工作流动画未遵守 "Reduce motion" 设置 [^6] |
| 2.1.163 | 修复 worktree 隔离的工作流代理无法编辑自身 worktree 内文件 [^7] |

## 相关配置

| 配置项 | 说明 |
|--------|------|
| `Workflow keyword trigger`（`/config`） | 控制触发词是否生效 |
| `/effort ultracode` | 设置工作流努力级别 |

## 引用索引

[^1]: [[~/.claude/cache/changelog.md]] `L198-206` — 2.1.154 版本更新：引入动态工作流、Opus 4.8
[^2]: [[~/.claude/cache/changelog.md]] `L148` — 2.1.157 版本：触发词从 workflow 改为 ultracode
[^3]: [[~/.claude/cache/changelog.md]] `L192` — 2.1.157 版本：添加 Workflow keyword trigger 设置
[^4]: [[~/.claude/cache/changelog.md]] `L2084` — 2.1.165 版本：/plan 命令支持附加描述参数
[^5]: [[~/.claude/cache/changelog.md]] `L139` — 2.1.160 版本：修复 /effort ultracode 错误提示
[^6]: [[~/.claude/cache/changelog.md]] `L102` — 2.1.162 版本：修复工作流动画 motion 设置
[^7]: [[~/.claude/cache/changelog.md]] `L111` — 2.1.163 版本：修复 worktree 隔离问题
