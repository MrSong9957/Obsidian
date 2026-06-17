---
name: Agent 架构详细参考
description: Agent循环/多智能体/Harness工程/人话编程的完整详细内容，含代码示例、实战验证
type: reference
updated: 2026-06-17
tags: agent, architecture, multi-agent, harness, system-prompts
source: https://openai.com/index/harness-engineering/
---

# Agent 架构详细参考

> 入口文档：[[wiki/agent-arch.md|Agent 架构核心]]

本文档为 Agent 架构的详细参考资料索引。内容按主题组织在以下文件中：

## Part 1：Agent 循环与多智能体

**文件**：`sources/agent/agent-arch-p1.md`（1035 行）

包含：
- Agent Loop 详细说明（教学版 vs 生产版）
- s_full.py 底层逻辑全景（启动初始化、REPL 主循环、agent_loop 内部流程、六大支撑机制）
- 多智能体迭代框架精要（执行约束、核心原则、目录结构、工作流配置、四角色迭代流程）
- Learn Claude Code 摘要（s01-s12 四阶段递进）
- Claude Code System Prompts 摘要

## Part 2：Harness 工程与人话编程

**文件**：`sources/agent/agent-arch-p2.md`（1098 行）

包含：
- Harness Engineering 原文核心内容（中文摘要）
- 结构化解读（实验方法、关键教训、工程师启示）
- 实践清单
- 人话编程规范详细内容（基本规则、基础语法元素、高级语法元素、完整示例、反向翻译规则、写作指南）
