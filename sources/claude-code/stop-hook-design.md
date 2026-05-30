---
name: Stop Hook Memory Review 设计文档
description: V4 最终方案（关键词触发 + 后台 agent）的完整实现记录
type: concept
updated: 2026-04-18
tags: claude-code, hooks, memory-review, design
source: https://example.com
---

# V4 复盘机制 — 完整实现记录

## 目标

用户输入"复盘"关键词时，Claude 启动后台 agent 回顾对话内容，评估是否有值得持久化的知识。有价值则写入 wiki/ 和 MEMORY.md，无价值则静默停止。复盘在后台执行，不阻塞用户。

## 架构

```
用户输入"复盘" → Claude 调用 Agent 工具（run_in_background: true）
                              ↓
                后台 agent 独立执行：
                  1. Skill 工具加载 obsidian-remember 技能
                  2. 读取 MEMORY.md 去重
                  3. 写入 wiki/ 和 MEMORY.md（如有新知识）
                              ↓
                完成后通知弹出（不阻塞用户）
```

## V4 的工作原理

V1-V3 都在 settings.json 中配置 hook，由 Claude Code 框架自动触发。V4 完全不同——**没有任何自动化机制，全靠 Claude 自己判断**。

完整链路如下：

```
① 用户输入"复盘"
       ↓
② Claude 读取 MEMORY.md，发现这条指令：
   "Hooks：V4 关键词'复盘'触发后台 agent 复盘"
       ↓
③ Claude 根据这条指令，主动调用 Agent 工具（run_in_background: true）
       ↓
④ 后台 agent 启动，执行复盘任务
```

关键点：**触发器不是 hook，不是脚本，不是配置文件，而是 MEMORY.md 里的一行文字。** Claude 在每次会话开始时自动加载 MEMORY.md，所以它知道遇到"复盘"该做什么。

注意：Claude **不会**在运行时去读 sources/stop-hook-design.md 来获取提示词模板。它根据 MEMORY.md 中的指令 + 当前对话上下文，动态构造 agent prompt。本文档是给人看的实现记录，不是 Claude 的运行时配置。

## 从零配置 V4

要让 V4 方案在全新环境中工作，只需一步：

**在 MEMORY.md 中写入指令**

在 `~/.claude/projects/<project>/memory/MEMORY.md` 中添加：

```
- 复盘：用户输入"复盘"时，用 Agent 工具启动后台复盘 agent（run_in_background: true），prompt 包含：调用 obsidian-remember 技能、总结本轮对话、读 MEMORY.md 去重、有新知识则写 wiki/ 否则静默结束
```

Claude 会话启动时自动加载 MEMORY.md，看到这条指令后就知道：遇到"复盘"关键词 → 调用 Agent 工具 → 构造上述 prompt → 后台执行。

settings.json 不需要任何修改，`"hooks": {}` 保持为空。不需要写脚本、不需要安装工具。

## Agent 工具调用（核心代码）

Claude 在主对话中发起如下工具调用：

```json
Agent({
  "description": "后台复盘本轮对话",
  "run_in_background": true,
  "prompt": "你是复盘 agent。请执行以下任务：\n\n1. 使用 Skill 工具调用 obsidian-remember 技能，复盘本轮对话。\n2. 本轮对话的关键内容：\n   - <Claude 摘要本轮对话的关键发现、决策和变更>\n3. 读取 ~/.claude/projects/-home-app-project/memory/MEMORY.md 检查是否已有相关记录，避免重复。\n4. 如果有值得记录的新知识（经验教训、架构决策、调试发现），写入 wiki/ 并更新 MEMORY.md。无价值则不输出。"
})
```

**关键字段说明：**

| 字段 | 值 | 作用 |
|------|-----|------|
| `description` | `"后台复盘本轮对话"` | 标识 agent 用途，显示在通知中 |
| `run_in_background` | `true` | 不阻塞主对话，用户可立即继续输入 |
| `prompt` | 见下方模板 | 后台 agent 的完整指令 |

**prompt 模板（实际使用时 Claude 动态填充第 2 步）：**

```
你是复盘 agent。请执行以下任务：

1. 使用 Skill 工具调用 obsidian-remember 技能，复盘本轮对话。
2. 本轮对话的关键内容：
   - <由主 Claude 总结本轮对话的关键发现>
3. 读取 ~/.claude/projects/-home-app-project/memory/MEMORY.md 检查是否已有相关记录，避免重复。
4. 如果有值得记录的新知识（经验教训、架构决策、调试发现），写入 wiki/ 并更新 MEMORY.md。无价值则不输出。
```

**实际调用示例**（某次调试会话）：

```json
{
  "description": "后台复盘本轮对话",
  "run_in_background": true,
  "prompt": "你是复盘 agent。请执行以下任务：\n\n1. 使用 Skill 工具调用 obsidian-remember 技能，复盘本轮对话。\n2. 本轮对话的关键内容：\n   - Stop Hook 方案从 V3（自动触发）演进到 V4（关键词触发）\n   - V3 废弃原因：评估器过度触发（几乎每次回复都启动后台 agent）+ 评估器 LLM 不严格输出 JSON 导致显示 error\n   - V4 方案：移除 Stop hook，用户输入\"复盘\"关键词时 Claude 启动后台 agent\n   - 设计文档已更新\n   - settings.json 已清空 hooks\n3. 读取 ~/.claude/projects/-home-app-project/memory/MEMORY.md 检查是否已有相关记录，避免重复。\n4. 如果有值得记录的新知识，写入 wiki/ 并更新 MEMORY.md。无价值则不输出。"
}
```

### 4. 后台 agent 执行流程

后台 agent 收到 prompt 后按以下步骤执行：

```
1. 调用 Skill 工具，加载 obsidian-remember 技能
       ↓
2. 按技能指引评估对话内容
       ↓
3. 读取 MEMORY.md，检查是否已有相关记录（去重）
       ↓
4a. 有新知识 → 写入 wiki/ 文件 → 更新 MEMORY.md 索引
4b. 无新知识 → 静默结束，不输出任何内容
       ↓
5. 完成通知弹出（用户可见结果摘要）
```

## 踩坑记录（V1-V3 结论性教训）

| 问题 | 根因 | 教训 |
|------|------|------|
| agent hook 无 Write/Skill 权限 | agent hook 只暴露 Read/Grep/Glob | 复盘需要写权限，不能用 agent hook |
| `claude -p` 子进程终端乱码 | ANSI 转义码与 TUI 冲突 | 后台任务避免直接输出到终端 |
| heredoc 长 prompt 传递失败 | shell 转义/展开问题 | 复杂 prompt 用临时文件传递 |
| settings.json 修改不立即生效 | hook 在会话启动时加载 | 改 hook 配置需重启 Claude Code |
| 评估器几乎每次都触发 | 评估器 LLM 无法准确判断"有复盘价值" | 自动判断不如用户判断可靠 |
| 评估器输出非 JSON 显示 error | glm-5.1 对 JSON 格式遵循度不足 | 避免 LLM 输出严格格式 |

## 验证方法

1. 进行一次包含新知识发现的对话（如调试一个新问题、做出架构决策）
2. 输入"复盘"
3. 观察 Claude 是否立即回复并启动后台 agent
4. 等待后台 agent 完成通知
5. 检查 wiki/ 和 MEMORY.md 是否已更新
6. 验证不触发时对话正常结束（零额外开销）

## V4.1 探索：prompt hook 强化触发（实验中）

### 问题

V4 依赖 MEMORY.md 中的指令触发复盘。当上下文窗口过长时，Claude 可能忽略该指令，导致"复盘"关键词失效。

### 方案：prompt hook 关键词检测 + 强指令注入

用 prompt hook 检测用户消息是否包含"复盘"：
- 匹配时：向 Claude 注入强指令，要求调用 Agent 工具启动后台复盘
- 不匹配时：透传用户原始消息，零干扰

这样 Claude 调用的仍然是 Agent 工具（全权限），不走受限的 agent hook。

### 约束力对比

| 触发方式 | 约束力 | 长上下文可靠性 | 权限 |
|----------|--------|----------------|------|
| MEMORY.md 指令（V4） | 弱，Claude 可忽略 | 低 | 全（Agent 工具） |
| prompt hook 注入（V4.1） | 强，框架级注入 | 高 | 全（Agent 工具） |
| agent hook（V1） | 最强，框架级自动 | 最高 | 低（只读） |

### 实验状态

待验证。验证后根据结果决定保留或删除本节。

## V5：从 SKILL 到斜杠命令（2026-04-18）

### 问题

V4 使用两个 skill 注册在 `~/.claude/skills/` 下：
- `note-obsidian/` — wrapper，description 含"记住"、"复盘"等词
- `obsidian-remember/` — vault 原版，description 也含类似词

两个 skill 的 description 都能匹配用户意图，Claude Code 的 skill 调度器可能选错或犹豫。用户说"记住这个"时，主会话可能误触发 `obsidian-remember` 而不是 `note-obsidian` wrapper。

### 方案：斜杠命令替代 wrapper skill

将触发方式从"skill description 匹配"改为"用户显式输入 `/note-obsidian`"：

1. **删除两个旧 skill 目录** — 消除 description 冲突
2. **创建 symlink** — `~/.claude/skills/obsidian-remember` → vault 原版，仅供后台 agent 通过 `Skill("obsidian-remember")` 调用
3. **创建斜杠命令** — `~/.claude/commands/note-obsidian.md`，用户显式触发

### 为什么斜杠命令更好

| 维度 | SKILL（V4） | 斜杠命令（V5） |
|------|-------------|----------------|
| 触发方式 | description 模糊匹配 | 用户显式 `/note-obsidian` |
| 冲突风险 | 两个 skill 竞争匹配 | 零冲突 |
| 误触发 | 用户说"记住"可能误触发 | 不可能误触发 |
| 后台 agent 调用 | wrapper skill 启动 agent | 命令内容直接启动 agent |

### 架构

```
用户输入 /note-obsidian
       ↓
Claude Code 加载 ~/.claude/commands/note-obsidian.md 内容
       ↓
Claude 按 prompt 指令调用 Agent（run_in_background: true）
       ↓
后台 agent：Skill('obsidian-remember') → 写入 wiki/ → 更新 MEMORY.md
```

symlink 指向 vault 原版，后台 agent 通过 `Skill("obsidian-remember")` 调用。主会话不直接匹配该 skill，因为没有 wrapper skill 竞争。

### 文件变更

```
删除：~/.claude/skills/note-obsidian/SKILL.md
删除：~/.claude/skills/obsidian-remember/SKILL.md（旧副本）
新增：~/.claude/skills/obsidian-remember → /home/app/obsidian/skills/obsidian-remember（symlink）
新增：~/.claude/commands/note-obsidian.md
```
