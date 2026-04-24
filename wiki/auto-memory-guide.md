---
name: Auto Memory 指南
description: Claude Code Auto Memory（MEMORY.md）完整参考：加载机制、存储规则、最佳实践 — 理解或配置记忆系统时参考
type: summary
sources:
  - "[[sources/claude-code-system-prompts/README]]"
updated: 2026-04-05
tags: [claude-code, memory]
status: deprecated
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# Claude Code Auto Memory 完整指南

> 内容已迁移至 [[wiki/claude-code-p1.md|claude-code-p1]]。

> 基于官方文档整理：https://code.claude.com/docs/en/memory

---

## 核心定位

Auto Memory 是 Claude Code 的两大持久化系统之一：

| | CLAUDE.md | Auto Memory |
|---|---|---|
| **谁写的** | 你（人工） | Claude（自动） |
| **内容** | 指令和规则 | 学习和经验 |
| **作用域** | 全局 / 项目 / 组织 | 每个 git 仓库 |
| **加载方式** | 每次会话完整加载 | 每次会话加载前 200 行或 25KB |
| **用途** | 编码规范、工作流、架构 | 构建命令、调试经验、你发现的偏好 |

**简单理解：CLAUDE.md 是你给 Claude 的规章制度，MEMORY.md 是 Claude 给自己记的笔记本。**

---

## 存储位置

```
~/.claude/projects/<project>/memory/
├── MEMORY.md            # 索引入口，每次会话加载
├── debugging.md         # 主题文件，按需加载
├── api-conventions.md   # 主题文件，按需加载
└── ...                  # Claude 创建的其他主题文件
```

- `<project>` 路径由 git 仓库根目录推导
- 同一仓库的所有 worktree 和子目录共享一个 memory 目录
- 非 git 项目使用项目根目录的绝对路径
- Auto Memory 是**机器本地**的，不会跨设备同步

### 自定义存储位置

在 user 或 local settings 中设置：

```json
{
  "autoMemoryDirectory": "~/my-custom-memory-dir"
}
```

> 注意：此设置**不能**在项目 settings（`.claude/settings.json`）中配置，防止共享项目重定向写入路径。

---

## 加载规则

### MEMORY.md（索引文件）

| 规则 | 说明 |
|------|------|
| **加载时机** | 每次会话开始时 |
| **加载限制** | 前 **200 行** 或 **25KB**（以先达到者为准） |
| **超限处理** | 超出部分**不会**被加载 |
| **加载频率** | 仅会话开始一次，不会每轮重新加载 |
| **Compaction 后** | 不会重新从磁盘读取（与 CLAUDE.md 不同） |

### 主题文件（debugging.md 等）

| 规则 | 说明 |
|------|------|
| **加载时机** | **不**在启动时加载 |
| **读取方式** | Claude 使用 Read 工具按需读取 |
| **触发条件** | Claude 判断对话需要时主动读取 |

---

## Compaction 行为对比

这是两个系统最关键的区别：

| 场景 | CLAUDE.md | MEMORY.md |
|------|-----------|-----------|
| **会话开始** | 完整加载 | 前 200 行/25KB |
| **对话过长后** | 上下文压缩，但**会从磁盘重新读取并注入** | 上下文压缩，**不会重新注入** |
| **`/compact` 后** | 从磁盘重新加载，完整恢复 | 已压缩的内容丢失 |

**结论：CLAUDE.md 的持久性强于 MEMORY.md。** 如果你有一条规则必须在长对话中始终生效，放在 CLAUDE.md 里更可靠。

---

## 自动保存机制

### 什么时候 Claude 会保存

Claude **不是**每次会话都保存。它根据"这条信息在未来的对话中是否有用"自主判断。

典型触发场景：
- 你说"记住这个"或"以后都这样做"
- 你纠正了 Claude 的做法（"别这样做，要那样做"）
- Claude 了解到你的角色或偏好

### 保存格式

纯 Markdown 文件，你可以随时编辑或删除。

### 管理方式

- 会话中运行 `/memory` 浏览和管理
- 直接编辑 `~/.claude/projects/<project>/memory/` 下的文件
- MEMORY.md 充当目录索引，Claude 用它追踪"什么存在哪里"

---

## 开关控制

Auto Memory **默认开启**（需要 Claude Code v2.1.59+）。

三种关闭方式：

```json
// 方式 1：项目 settings
{ "autoMemoryEnabled": false }

// 方式 2：环境变量
CLAUDE_CODE_DISABLE_AUTO_MEMORY=1

// 方式 3：会话中运行 /memory，切换开关
```

---

## MEMORY.md 索引设计要点

### 限制

索引超过 200 行或 25KB 后，超出部分**永远不会被加载**。这意味着 Claude 在新会话中根本不知道那些条目存在。

### 最佳实践

1. **索引保持精简** — 每条一行，控制在 200 行以内
2. **详细信息放主题文件** — Claude 会按需用 Read 工具读取
3. **索引格式**：`- [标题](文件名) — 一句话摘要`
4. **定期清理** — 过时的记忆直接删除

### 不该存什么

以下内容不需要存入 Memory（因为可以通过其他方式获取）：

| 不要存 | 原因 |
|--------|------|
| 代码模式/架构 | 直接读代码就知道 |
| Git 历史 | `git log` 是权威来源 |
| 调试方案 | 修复已经在代码和 commit 里了 |
| 已在 CLAUDE.md 中的规则 | 重复浪费上下文空间 |
| 通用编程知识 | Claude 本身就知道 |

### 适合存什么

| 适合存 | 原因 |
|--------|------|
| 你纠正过 Claude 的事 | 防止下次犯同样的错 |
| 你的沟通偏好 | "中文"、"原理优先"、"不用 emoji" |
| 项目中不明显的关键上下文 | "这个 API key 下周过期"、"移动团队在用这个分支" |
| 外部资源位置 | "pipeline bug 在 Linear INGEST 项目" |

---

## 持久化优先级总结

从"最可靠"到"最不可靠"排序：

```
1. Hooks（UserPromptSubmit）     → 每轮都执行，最可靠
2. CLAUDE.md                     → compaction 后会重新加载
3. .claude/rules/（无条件）       → 同 CLAUDE.md
4. .claude/rules/（paths: 限定） → 按需加载，只处理匹配文件时触发
5. MEMORY.md（前 200 行）        → 仅会话开始加载，不重载
6. MEMORY.md 主题文件            → Claude 按需读取
7. 对话中的口头约定               → compaction 后可能丢失
```

---

## 实用命令

| 命令 | 作用 |
|------|------|
| `/memory` | 查看所有已加载的 CLAUDE.md、rules 和 auto memory |
| "记住这个" | 让 Claude 保存到 auto memory |
| "加到 CLAUDE.md" | 让 Claude 写入 CLAUDE.md 而非 auto memory |

---

## 官方文档

- 完整文档：https://code.claude.com/docs/en/memory
- Hooks 文档：https://code.claude.com/docs/en/hooks
