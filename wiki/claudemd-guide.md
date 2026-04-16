---
name: CLAUDE.md 指南
description: CLAUDE.md 持久化指令文件完整参考 — 编写或优化 CLAUDE.md 时参考
type: summary
sources:
  - "[[sources/claude-code-system-prompts/README]]"
updated: 2026-04-05
tags: [claude-code, configuration]
---

# CLAUDE.md 完整指南

> 官方文档：https://code.claude.com/docs/en/memory
> 实践参考：https://joseparreogarcia.substack.com/p/how-claude-code-rules-actually-work

---

## 是什么

CLAUDE.md 是你写给 Claude 的持久化指令文件。Claude 在每次会话开始时读取，作为行为指南。

**核心认知：CLAUDE.md 是上下文（context），不是强制执行（enforcement）。** Claude 会尽力遵守，但不保证 100% 合规，特别是指令模糊或互相矛盾时。

---

## 三级作用域

| 作用域 | 位置 | 优先级 | 用途 |
|--------|------|--------|------|
| 组织级 | Windows: `C:\Program Files\ClaudeCode\CLAUDE.md` | 最高 | 公司级编码标准、安全政策 |
| 用户级 | `~/.claude/CLAUDE.md` | 中 | 个人偏好，跨项目生效 |
| 项目级 | `./CLAUDE.md` 或 `./.claude/CLAUDE.md` | 低 | 项目架构、构建命令、团队约定 |

**优先级规则：更具体的位置覆盖更宽泛的。** 项目级 > 用户级 > 组织级。

---

## 加载机制

### 何时加载

**每次会话开始时完整加载。** 与 MEMORY.md 的 200 行限制不同，CLAUDE.md 没有截断限制（全文加载）。

### 加载顺序

```
1. 从工作目录向上遍历目录树，加载每层的 CLAUDE.md
2. 子目录的 CLAUDE.md 在 Claude 读取该子目录文件时按需加载
3. 用户级 ~/.claude/CLAUDE.md 始终加载
4. 组织级 CLAUDE.md 始终加载，且无法被排除
```

### Compaction 后的行为（关键）

**CLAUDE.md 完整存活于 compaction。** 执行 `/compact` 后，Claude 会从磁盘重新读取 CLAUDE.md 并重新注入上下文。

> 官方原话："CLAUDE.md fully survives compaction. After /compact, Claude re-reads your CLAUDE.md from disk and re-injects it fresh."

这意味着：如果你发现 compaction 后某条指令消失了，说明那条指令只存在于对话中，没有写入 CLAUDE.md。

---

## 导入机制

用 `@path` 语法在 CLAUDE.md 中引用其他文件：

```markdown
# 项目说明
见 @README.md 了解项目概述
见 @package.json 了解可用命令

# 个人偏好（不提交到 git）
@~/.claude/my-project-instructions.md
```

- 支持相对路径和绝对路径
- 相对路径基于包含导入的文件所在目录
- 最多递归 5 层导入
- AGENTS.md 兼容：`@AGENTS.md` 然后追加 Claude 专属指令

---

## 最佳使用方法

### 写什么

CLAUDE.md 的职责是**防止全局误解**（prevent global misunderstandings）：

| 适合写 | 不适合写 |
|--------|---------|
| 编码规范（缩进、类型要求） | 逐步骤工作流 |
| 构建和测试命令 | 只在特定目录才适用的规则 |
| 架构决策和项目结构 | 实验性或经常变化的实践 |
| 命名约定 | 外部文档的完整内容 |
| 常用工作流（简短描述） | 超过 200 行的详细文档 |

### 写作原则

**1. 精简：每个文件不超过 200 行**

长文件消耗上下文空间，降低遵守率。超长内容用 `@path` 导入或拆分到 `.claude/rules/`。

**2. 具体：可验证的指令**

```markdown
# 差 — 模糊
格式化代码

# 好 — 具体可验证
使用 2 空格缩进
```

```markdown
# 差
测试你的更改

# 好
提交前运行 npm test
```

**3. 结构化：用 Markdown 标题和列表**

Claude 扫描结构的方式和人类一样：有组织的分节比密集段落更容易遵守。

**4. 无矛盾：定期审查**

如果两个文件对同一行为给出不同指导，Claude 可能随机选一个。用 `/memory` 命令查看当前加载了哪些文件，检查冲突。

**5. 注释给人类看**

HTML 注释 `<!-- 给人看的备注 -->` 会被 Claude 过滤，不消耗上下文：

```markdown
<!-- 下次重构时记得更新这个部分 -->
- API 端点必须包含输入验证
```

### CLAUDE.md 作为索引

一个高阶用法：在项目 CLAUDE.md 中写"路线图"，引导 Claude 在不同场景下查阅不同规则：

```markdown
## 如何在此仓库工作

### 实验性工作
- 查阅 `.claude/rules/experiments.md`
- 约束较松，快速迭代

### 生产代码修改
- `src/models/` 和 `src/pipelines/` 下有严格的 scoped rules
- 可重现性、测试、评估是强制的
```

这不是强制执行，但会**影响 Claude 的规划步骤**，让它更可能做"正确的事"。

---

## 大型项目技巧

### monorepo 排除不相关文件

```json
// .claude/settings.local.json
{
  "claudeMdExcludes": [
    "**/other-team/CLAUDE.md",
    "/home/user/monorepo/legacy/.claude/rules/**"
  ]
}
```

### 附加目录加载 CLAUDE.md

默认 `--add-dir` 的目录不加载 CLAUDE.md，加环境变量启用：

```bash
CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1 claude --add-dir ../shared-config
```

### 子目录 CLAUDE.md

子目录中的 CLAUDE.md 不会在启动时加载，只有当 Claude 读取该子目录的文件时才加载。适合模块级指令：

```
project/
├── CLAUDE.md              # 项目级（启动时加载）
├── src/
│   ├── billing/
│   │   └── CLAUDE.md      # 只在编辑 billing/ 时加载
│   └── auth/
│       └── CLAUDE.md      # 只在编辑 auth/ 时加载
```

---

## 排错

| 问题 | 检查方法 |
|------|---------|
| Claude 没遵守指令 | 运行 `/memory` 确认文件已加载 |
| 指令 compaction 后消失 | 该指令只存在于对话中，未写入 CLAUDE.md |
| 两个指令互相冲突 | 检查全局 + 项目 + 子目录的所有 CLAUDE.md |
| 指令太长被忽略 | 缩短到 200 行以内，拆分到 rules/ |
