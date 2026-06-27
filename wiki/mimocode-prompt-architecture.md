---
name: MiMo Code 提示词注入架构
description: MiMo Code 系统提示词 4 层管道、权限系统、工具限制、插件转换钩子
type: concept
updated: 2026-06-27
tags: mimocode, opencode, prompt, architecture, system-prompt
source: [[sources/projects/ECC-MiMo-Code.md]]
---

# MiMo Code 提示词注入架构

## 4 层系统提示词管道

在 `packages/opencode/src/session/llm.ts:234-296` 的 `buildSystemArray()` 函数中：

```
┌─────────────────────────────────────────────────────────┐
│  1. Provider-specific base prompt                      │
│     gpt-4/o1/o3 → beast.txt                            │
│     claude → anthropic.txt                             │
│     gemini → gemini.txt                                │
│     fallback → default.txt                             │
├─────────────────────────────────────────────────────────┤
│  2. Environment + skills injection                     │
│     工作目录、平台、日期、模型名称、可用技能          │
├─────────────────────────────────────────────────────────┤
│  3. Agent-specific prompt override                     │
│     如果 agent 有 prompt 字段，替换 provider prompt    │
├─────────────────────────────────────────────────────────┤
│  4. Memory instructions                                │
│     教 LLM 关于 MEMORY.md、checkpoint.md 等文件       │
└─────────────────────────────────────────────────────────┘
```

## 权限系统

- **Agent permission ruleset**：每个 agent 有自己的权限规则
- **hardPermission**：不可覆盖的硬约束（如 plan mode 阻止编辑非计划文件）
- **runtimePermission()**：合并 agent.permission + userPermission + agent.hardPermission

## 工具限制

- **toolAllowlist**：限制 agent 可用工具（如 explore agent 只有 grep/glob/list/bash/webfetch/read）

## 插件转换钩子

- **experimental.chat.system.transform**：允许插件在 LLM 调用前修改系统提示词数组

## 指令文件加载

在 `instruction.ts:124-179`：
- 从项目根目录加载 AGENTS.md/CLAUDE.md
- 从全局配置目录加载
- 用户指定的 `instructions` 路径
- 首个项目级匹配优先；稀疏 AGENTS.md（<500 字符）触发 CLAUDE.md 回退

## Plan Mode 系统提醒注入

在 `prompt.ts:536-618`：
- 当处于 plan mode 时，`<system-reminder>` 块被添加到用户消息前
- 包含 5 阶段工作流、工具限制、计划文件路径

## 动态工具描述

在 `registry.ts:366-371`：
- Actor 工具：附加 `describeTask()`（列出可用子代理类型 + 工具）
- Skill 工具：附加 `describeSkill()`（列出所有可用技能）
- Workflow 工具：附加 `describeWorkflow()`（列出内置工作流）

## 自定义工具注册

插件中使用 `hooks["tool"] = { "tool-name": { description, parameters, execute } }` 注册可调用工具