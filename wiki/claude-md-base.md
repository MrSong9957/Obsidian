---
name: CLAUDE.md 模板与编排
description: CLAUDE.md 通用规则模板 + 全局 CLAUDE.md 创建方法（ECC 前置 + 偏好后置结构）
type: reference
updated: 2026-04-23
tags: [claude-md, template, universal-rules, kv-cache, ecc]
---
# CLAUDE.md 通用基础规则

## 协作方式
- 语言：中文
- 小步迭代，每步可验证
- 先观察再修改，创建最小诊断脚本确认问题
- 自主闭环，多次失败才请求介入

## 行为约束
- 禁止修改未读取的文件
- 禁止添加未被要求的功能
- 禁止猜测不确定的信息，超出知识范围直接说明，从权威渠道（官方文档、Web 搜索）获取支持后再回答
- 指出方案缺陷不迎合不软化，因沉默的同意比刺耳的真话代价更大
- 基于假设行动时显式标出假设，因隐含假设错误会导致整个方案静默失败
- 子代理：可并行或需隔离时用；简单操作直接做

## 编码风格 & 重构规则
- 节分隔符：`# --- 名称 ---`
- 保持方案简单。加个参数优于创建重复函数。扩展优于抽象。
- 不要过度设计。用户要求小改动就做最小的改动，复用成熟方案。
- 优先使用 Python。Python 3.13 + venv 优于 conda/miniconda。
- UI 工作可以用 HTML 原型。

## 评估理念
- 用以用户为中心的标准评估 UI/UX，而非代码指标（不用行数、功能密度、token 频率来衡量面向用户的输出）。

## Git 规范
- 身份信息从 `git log --format='%an <%ae>' -1` 获取（与历史提交一致）
- 提交时使用 `-c user.name=... -c user.email=...` 避免修改全局配置

---

# 全局 CLAUDE.md 创建方法

> 文件位置：`~/.claude/CLAUDE.md`（用户级，跨项目生效）

## 结构原则：ECC 前置 + 偏好后置

利用 Anthropic prompt caching 的前缀匹配特性——前缀不变时缓存命中，修改后缀不会使前缀缓存失效：

```
~/.claude/CLAUDE.md
├── [前缀] ECC 模板内容（不常变，缓存命中率高）
│   ├── Core Philosophy
│   ├── Modular Rules → ~/.claude/rules/common/*.md
│   ├── Available Agents → ~/.claude/agents/*.md
│   ├── Privacy
│   ├── Git
│   └── Success Metrics
├── ---（分隔符，标记缓存边界）
└── [后缀] 个人偏好（可按需调整，修改不影响上方缓存）
    ├── 协作方式
    ├── 行为约束
    ├── 评估理念
    ├── 技术栈偏好
    ├── 知识管理
    └── 成功标准
```

## 创建步骤

1. **安装 ECC 插件** → 自动生成前半部分模板（Core Philosophy / Rules 表 / Agents 表 / Privacy / Git / Metrics）
2. **在模板末尾追加 `---` 分隔符**
3. **追加个人偏好**：语言、协作方式、行为约束、技术栈等
4. **将详细规则拆分到 `~/.claude/rules/common/` 目录**，CLAUDE.md 中只放索引表（不内联长文本）

## 配套文件结构

```
~/.claude/
├── CLAUDE.md                    # 全局指令（ECC模板 + 个人偏好）
├── rules/common/                # 详细规则文件
│   ├── development-workflow.md  # 开发流程
│   ├── code-review.md           # 代码审查
│   ├── security.md              # 安全检查
│   ├── coding-style.md          # 编码风格
│   ├── testing.md               # 测试要求
│   ├── git-workflow.md          # Git 工作流
│   ├── agents.md                # Agent 编排
│   ├── patterns.md              # 设计模式
│   ├── performance.md           # 性能优化
│   └── hooks.md                 # Hooks 系统
├── agents/                      # Agent 定义文件
└── skills/                      # 技能定义
```

## 关键设计决策

| 决策 | 原因 |
|------|------|
| 模板用英文 | ECC 插件生成的标准模板，减少维护负担 |
| 偏好用中文 | 个人内容用母语更精确、更易维护 |
| 详细规则外置到 rules/ | CLAUDE.md 保持精简（< 200 行），rules 按需加载 |
| `---` 分隔缓存边界 | 修改偏好时不使模板部分的缓存失效 |
