---
name: 子代理 Skill 工具可用性测试
description: 2026-05-10 实测子代理/团队成员能否使用 Skill 工具，记录所有尝试方案和结果
type: feedback
updated: 2026-05-10
tags: [claude-code, subagent, skill, test]
---

# 子代理 Skill 工具可用性测试

测试日期：2026-05-10
环境：Docker 容器，Claude Code 2.1.132，模型 glm-5.1（智谱 AI API），ANTHROPIC_BASE_URL=open.bigmodel.cn

## 测试背景

CLAUDE.md 中写有子代理派发规则要求子代理调用 Skill 工具加载技能。需验证子代理/团队成员是否能实际使用 Skill 工具。

## 测试结果汇总

### 方案1：子代理直接调用 Skill

```bash
claude --print -p '调用 Skill 工具加载 brainstorming 技能'
```

**结果：失败。** 子代理工具列表中无 Skill，ToolSearch 延迟列表中也无 Skill。

### 方案2：ToolSearch 加载 Skill

```bash
# 子代理内执行
ToolSearch("select:Skill")
```

**结果：失败。** ToolSearch 返回 "Tool loaded." 但无 schema，实际无法调用。假阳性响应。

### 方案3：--allowedTools 指定 Skill

```bash
claude --print --agents '{"t":{"tools":["*"]}}' --agent t -p '...'
```

**结果：失败。** allowedTools 设置不影响 Skill 工具的可用性。

### 方案4：--plugin-dir 加载插件

```bash
claude --print --plugin-dir ~/.claude/plugins -p '...'
```

**结果：失败。** 插件提供技能内容（SKILL.md 文件），不提供 Skill 工具本身。

### 方案5：智能体团队成员

```bash
claude --print --agents '{"team-test":{...}}' --agent team-test -p '检查 Skill 可用性'
```

**结果：失败。** 团队成员与普通子代理工具列表一致，无 Skill。

### 方案6（成功）：父代理预加载 + prompt 注入

```bash
# 父代理先读取技能文件
TDD_CONTENT=$(cat .../skills/test-driven-development/SKILL.md)

# 子代理：注入 -p 参数
claude --print -p "技能规则：---TDD START--- $TDD_CONTENT ---TDD END--- 任务：..."

# 团队成员：注入 prompt 字段
claude --print --agents '{"a":{"prompt":"---TDD START--- ... ---TDD END---"}}' --agent a -p '...'
```

**结果：成功。** 子代理和团队成员均能准确遵循注入的技能规则。

## 根因分析

Skill 是 Claude Code 运行时内置工具（在 250MB 编译二进制中注册），仅对主会话可用。子代理和团队成员运行在独立上下文中，不继承主会话的工具集。插件系统只提供技能内容文件，不提供 Skill 工具。

## 结论

→ 详见 [[wiki/claude-code-p2.md]] 和 memory/feedback-subagent-skill-injection.md
