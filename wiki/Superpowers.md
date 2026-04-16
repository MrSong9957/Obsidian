---
name: Superpowers 技能系统
description: 预定义技能集，规范需求分析到代码审查完整开发流程 — 配置或使用 Superpowers 技能时参考
type: summary
sources:
  - "[[sources/claude-code-system-prompts/README]]"
updated: 2026-04-05
tags: [claude-code, skills]
---

# Superpowers 使用指南

> 相关文档：[[Everything-Claude-Code.md|ECC 技能集总览]]

## 1. 什么是 Superpowers？

Superpowers 是一个基于 AI 辅助开发的技能系统，通过一系列预定义的技能（skills）来规范开发流程，提高开发效率和代码质量。它包含了从需求分析到代码实现、测试、审查等完整开发生命周期的技能集。

## 2. 技能列表

| 技能名称 | 描述 | 核心功能 |
|---------|------|----------|
| using-superpowers | 基础技能，指导如何访问和使用其他技能 | 技能调用规则、优先级、流程 |
| writing-plans | 编写实现计划 | 创建详细的分步实现计划 |
| executing-plans | 执行实现计划 | 按计划分批执行任务，定期汇报 |
| systematic-debugging | 系统化调试 | 高效定位和修复 bug |
| test-driven-development | 测试驱动开发 | 遵循 TDD 流程开发代码 |
| brainstorming | 头脑风暴 | 需求分析和解决方案设计 |
| subagent-driven-development | 子代理驱动开发 | 分任务调度子代理完成开发 |
| requesting-code-review | 请求代码审查 | 准备和提交代码审查请求 |
| receiving-code-review | 接收代码审查 | 处理和响应代码审查反馈 |
| finishing-a-development-branch | 完成开发分支 | 最终验证和分支处理 |
| verification-before-completion | 完成前验证 | 全面验证确保功能正确性 |
| dispatching-parallel-agents | 并行代理调度 | 调度多个代理并行工作 |
| ui-ux-pro-max | UI/UX 设计 | 专业 UI/UX 设计支持 |
| using-git-worktrees | Git Worktrees 使用 | 高效管理多个开发分支 |
| writing-skills | 编写技能 | 创建新的 Superpowers 技能 |

## 3. 安装与设置

### 3.1 环境要求
- Windows 11 操作系统
- Git 已安装
- Python 3.8+（可选，部分技能需要）

### 3.2 安装步骤
1. 克隆仓库：
   ```bash
   git clone https://github.com/yourusername/superpowers.git
   cd superpowers
   ```

2. （可选）创建并激活虚拟环境：
   ```bash
   python -m venv .venv
   .venv\Scripts\Activate.ps1
   ```

## 4. 零基础使用教程

### 4.1 基本使用流程

Superpowers 的核心使用流程如下：

1. **选择合适的技能**：根据当前任务选择对应的技能
2. **调用技能**：按照技能要求的方式调用
3. **遵循技能指南**：严格按照技能文档的步骤执行
4. **验证结果**：执行技能中的验证步骤确保结果正确

### 4.2 第一步：使用 using-superpowers 技能

**这是所有任务的起点！**

无论你要做什么，首先必须检查是否有适用的技能，使用 `using-superpowers` 技能来指导你：

1. **核心规则**：如果有 1% 的可能适用某个技能，就必须调用它
2. **技能调用优先级**：
   - 先使用流程技能（如 brainstorming、systematic-debugging）
   - 再使用实现技能（如 frontend-design、mcp-builder）

### 4.3 第二步：编写实现计划

当你有一个多步骤任务时，使用 `writing-plans` 技能：

1. **创建工作目录**：在 `docs/plans/` 目录下创建计划文件
2. **编写计划**：按照模板编写详细的分步计划
3. **计划结构**：
   - 目标：一句话描述要构建的功能
   - 架构：2-3 句话说明实现方法
   - 技术栈：列出关键技术和库
   - 任务列表：每个任务包含具体步骤、代码示例和验证方法

**示例计划文件**：`docs/plans/2024-01-12-hello-world.md`

```markdown
# Hello World 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 创建一个简单的 Hello World 程序

**Architecture:** 使用 Python 编写一个单文件程序，输出 "Hello, Superpowers!"

**Tech Stack:** Python 3.8+

---

### Task 1: 创建测试文件

**Files:**
- Create: `tests/test_hello.py`

**Step 1: 编写测试代码**
```python
def test_hello():
    from src.hello import hello
    assert hello() == "Hello, Superpowers!"
```

**Step 2: 运行测试（预期失败）**
```bash
pytest tests/test_hello.py -v
```
Expected: FAIL with "ModuleNotFoundError: No module named 'src.hello'"

### Task 2: 实现功能

**Files:**
- Create: `src/hello.py`

**Step 1: 编写实现代码**
```python
def hello():
    return "Hello, Superpowers!"
```

**Step 2: 运行测试（预期通过）**
```bash
pytest tests/test_hello.py -v
```
Expected: PASS

**Step 3: 提交代码**
```bash
git add src/hello.py tests/test_hello.py
git commit -m "feat: add hello world function"
```

### 4.4 第三步：执行实现计划

使用 `executing-plans` 技能来执行你的计划：

1. **加载并审查计划**：仔细阅读计划，确保理解每个步骤
2. **分批执行任务**：默认每次执行前 3 个任务
3. **执行每个步骤**：
   - 标记任务为 in_progress
   - 严格按照计划步骤执行
   - 运行验证步骤
   - 标记任务为 completed
4. **汇报进度**：每批任务完成后汇报结果，等待反馈
5. **处理阻塞**：遇到问题立即停止，寻求帮助

### 4.5 第四步：完成开发分支

当所有任务完成后，使用 `finishing-a-development-branch` 技能：

1. **验证所有测试**：确保所有测试通过
2. **检查代码质量**：运行代码质量检查工具
3. **选择后续操作**：合并分支、删除分支或继续开发

## 5. 常用技能使用示例

### 5.1 系统化调试

使用 `systematic-debugging` 技能来定位和修复 bug：

1. **复现问题**：确保能够稳定复现 bug
2. **收集信息**：查看日志、调试信息等
3. **定位根因**：使用各种调试技术找到问题根源
4. **修复验证**：编写测试验证修复效果
5. **预防回归**：添加测试防止问题再次出现

### 5.2 测试驱动开发

使用 `test-driven-development` 技能遵循 TDD 流程：

1. **编写失败测试**：先编写测试用例，预期失败
2. **实现最小功能**：编写足够的代码使测试通过
3. **重构代码**：优化代码结构，保持测试通过
4. **重复迭代**：继续添加新功能，重复上述步骤

### 5.3 请求代码审查

使用 `requesting-code-review` 技能请求代码审查：

1. **准备审查请求**：确保代码符合项目规范
2. **提交审查请求**：使用合适的格式提交
3. **等待反馈**：耐心等待审查者的反馈
4. **处理反馈**：根据反馈修改代码

## 6. 最佳实践

1. **严格遵循技能指南**：技能设计经过实践验证，严格遵循能提高效率
2. **优先使用流程技能**：先确定方法，再执行实现
3. **保持任务粒度**：每个任务控制在 2-5 分钟，便于管理和验证
4. **频繁验证**：每个步骤都要验证，确保结果正确
5. **遇到问题立即停止**：不要猜测，寻求帮助或使用调试技能
6. **保持代码简洁**：遵循极简主义设计，用最少的代码实现功能

## 7. 常见问题

### Q: 如何选择合适的技能？
A: 根据当前任务类型选择：
- 需求分析：brainstorming
- 编写计划：writing-plans
- 执行计划：executing-plans
- 调试问题：systematic-debugging
- 开发功能：test-driven-development

### Q: 技能调用的优先级是什么？
A: 先使用流程技能（如 brainstorming、debugging），再使用实现技能（如 frontend-design）。

### Q: 遇到技能不适用怎么办？
A: 如果没有完全匹配的技能，可以选择最接近的技能，或使用基础技能 using-superpowers 指导。

## 8. 总结

Superpowers 是一个强大的 AI 辅助开发系统，通过预定义的技能规范开发流程，提高开发效率和代码质量。零基础用户可以按照本指南的步骤，从使用基础技能开始，逐步掌握各个技能的使用方法，最终实现高效的 AI 辅助开发。

记住：**技能是工具，关键是合理使用它们来解决实际问题**。