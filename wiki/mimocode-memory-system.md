---
name: MiMo Code 记忆系统架构
description: MiMo Code 三层记忆文件结构、写入权限、搜索机制、提升逻辑
type: concept
updated: 2026-06-27
tags: mimocode, opencode, memory, architecture, checkpoint
source: [[sources/projects/ECC-MiMo-Code.md]]
---

# MiMo Code 记忆系统架构

## 三层核心记忆文件

| 文件 | 位置 | 谁写入 | 持久性 | 用途 |
|------|------|--------|--------|------|
| **AGENTS.md** | 项目根目录 | 你（开发者） | 永久，提交到 Git | 项目规则、编码标准 |
| **MEMORY.md** | `<data>/memory/projects/<pid>/` | checkpoint-writer + agent（有限） | 跨会话持久 | 持久项目级知识 |
| **checkpoint.md** | `<data>/memory/sessions/<sid>/` | 仅 checkpoint-writer | 会话作用域 | 当前会话状态快照 |
| **notes.md** | `<data>/memory/sessions/<sid>/` | 主代理 | 会话作用域 | 自由格式草稿本 |

## AGENTS.md vs MEMORY.md 关键区别

- **AGENTS.md**：你手动编写，AI 只读取（除非明确要求）。提交到 Git。作为系统提示词层 ② 通过 `instruction.ts:130-151` 加载。首个项目级匹配优先；稀疏 AGENTS.md（<500 字符）触发 CLAUDE.md 回退。
- **MEMORY.md**：AI 通过 checkpoint-writer 写入。不提交到 Git。4 个部分：Project context、Rules、Architecture decisions、Discovered durable knowledge。
- **checkpoint.md**：AI 通过 checkpoint-writer 写入。11 个部分（§1-§11）。用于重启时的会话恢复。

## MEMORY.md 写入权限

在 `memory-path-guard.ts` 中：
- **checkpoint-writer**：可写入 `projects/<pid>/MEMORY.md`、`sessions/<sid>/checkpoint.md`、`sessions/<sid>/notes.md`、`sessions/<sid>/tasks/<TID>/*.md`
- **主代理**：可写入 `projects/<pid>/MEMORY.md`（当用户陈述项目级规则/决策时）和 `sessions/<sid>/notes.md`（草稿本）
- **主代理不能写入**：`sessions/<sid>/tasks/*`（保留给 writer）

## 记忆搜索（FTS5）

- `memory` 工具使用 BM25 全文搜索 markdown 正文
- 记忆布局：`<data>/memory/<scope>/<scope_id>/<key>.md`
- 作用域：global | projects | sessions | cc
- 查询：`memory({ operation: "search", query: "keyword" })`

## Checkpoint-writer 提升逻辑

- 跨任务事实 → §7（会话候选） → MEMORY.md ## Discovered durable knowledge（如果持久）
- 工作风格偏好 → §3（会话）或 MEMORY.md ## Rules（如果项目持久）
- 设计决策 → §10 Design decisions → MEMORY.md ## Architecture decisions（如果跨会话持久）
- Bug+修复 → §8 Errors and fixes

## `@include` 指令 — MiMo Code vs Claude Code

- **MiMo Code**：`@path/to/file` 语法仅在用户输入和命令模板中有效（`config/markdown.ts:6` FILE_REGEX）。CLAUDE.md/AGENTS.md 原样加载，无 `@` 解析。替代方案：使用 `mimocode.json` `instructions` 数组或 HTTP URL。
- **Claude Code**：`@include` 在 CLAUDE.md 文件内有效，有循环引用检测。内容被内联到系统提示词中。
- MiMo Code 还支持模板中的 `!` 反引号 shell 命令：`` !`git branch --show-current` ``