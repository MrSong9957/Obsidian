---
name: ECC 完整参考
description: "@affaanmustafa 的 Everything Claude Code 完整参考 — Skills/Hooks/Subagents/MCPs/Plugins/Session Memory/Token 经济学/Evals/33 个 Hooks 清单"
type: reference
updated: 2026-05-13
tags: [claude-code, skills, hooks, subagents, mcp, plugins, context-management, session-memory, token-optimization, evals, parallel-workflow]
source:
  - https://x.com/affaanmustafa/status/2012378465664745795
  - https://x.com/affaanmustafa/status/2014040193557471352
---

# ECC 完整参考

> 作者：@affaanmustafa
>
> 来源：[入门指南](https://x.com/affaanmustafa/status/2012378465664745795) | [进阶指南](https://x.com/affaanmustafa/status/2014040193557471352)
>
> 背景：从 2025 年 2 月实验性推出起即开始使用，赢得 Anthropic x Forum Ventures 黑客松冠军（用 Claude Code 完全构建项目 PMFProbe）

---

## 入门指南

### 1. Skills（技能）— 可复用的命令式工作流

#### 是什么

Skills 本质上是限定作用域的 prompt。当你需要执行某个特定工作流时，用斜杠命令快速调用，避免每次手写相同指令。

#### 存放位置

```
~/.claude/skills/              # 较大的工作流定义
  pnpm-guidelines.md           # 项目特定的编码模式
  coding-standards.md          # 语言最佳实践
  tdd-workflow/                # 多文件技能（带 README.md）
  security-review/             # 基于清单的审查技能
  codemap-updater.md           # 在检查点更新代码地图

~/.claude/commands/            # 快速执行的短命令
```

Skills 和 Commands 的区别：
- **Skills**（`~/.claude/skills/`）：更宽泛的工作流定义，可以是多文件目录
- **Commands**（`~/.claude/commands/`）：快速可执行的单次 prompt

#### 使用方式

- 直接调用：`/refactor-clean`、`/tdd`、`/e2e`、`/test-coverage`
- **链式调用**：在一条消息里串联多个命令

#### 实用案例：Codemap Updater

作者做了一个 codemap-updater 技能，让 Claude 在关键节点自动更新代码地图。代码地图帮助 Claude 快速导航项目，不需要花上下文去探索目录结构。

---

### 2. Hooks（钩子）— 基于事件的自动化

#### 是什么

Hooks 是在特定事件发生时自动触发的 shell 命令。和 Skills 不同，Hooks 不需要手动调用，它们在工具执行或生命周期事件上自动运行。

#### 六种 Hook 类型

| 类型 | 触发时机 | 典型用途 |
|------|---------|---------|
| PreToolUse | 工具执行前 | 验证输入、发送提醒 |
| PostToolUse | 工具执行后 | 自动格式化、质量检查 |
| UserPromptSubmit | 用户发送消息时 | 预处理输入 |
| Stop | Claude 结束回复时 | 最终审计 |
| PreCompact | 上下文压缩前 | 保存关键信息 |
| Notification | 权限请求时 | 自定义通知 |

#### 作者的 Hook 配置

```json
{
  "PreToolUse": [
    // 长时间命令前提醒使用 tmux
    {
      "matcher": "npm|pnpm|yarn|cargo|pytest",
      "hooks": ["tmux reminder"]
    },
    // 阻止创建不必要的 .md 文件（只允许 README 和 CLAUDE.md）
    {
      "matcher": "Write && .md file",
      "hooks": ["block unless README/CLAUDE"]
    },
    // git push 前打开编辑器审查
    {
      "matcher": "git push",
      "hooks": ["open editor for review"]
    }
  ],
  "PostToolUse": [
    // JS/TS 文件编辑后自动 Prettier 格式化
    {
      "matcher": "Edit && .ts/.tsx/.js/.jsx",
      "hooks": ["prettier --write"]
    },
    // TS 文件编辑后做类型检查
    {
      "matcher": "Edit && .ts/.tsx",
      "hooks": ["tsc --noEmit"]
    },
    // 编辑后检查是否有 console.log
    {
      "matcher": "Edit",
      "hooks": ["grep console.log warning"]
    }
  ],
  "Stop": [
    // 会话结束前审计所有修改过的文件是否有 console.log
    {
      "matcher": "*",
      "hooks": ["check modified files for console.log"]
    }
  ]
}
```

#### 小技巧

用 `hookify` 插件可以通过对话方式创建 Hooks，不用手写 JSON。运行 `/hookify` 然后描述你想要什么。

---

### 3. Subagents（子代理）— 限定范围的任务委托

#### 是什么

Subagents 是主代理（main Claude）可以委派任务的独立进程。它们在后台或前台运行，释放主代理的上下文空间。每个子代理可以被限制只使用特定的工具和权限。

#### 目录结构

```
~/.claude/agents/
  planner.md                # 拆解功能需求
  architect.md              # 系统设计决策
  tdd-guide.md              # 测试驱动开发
  code-reviewer.md          # 代码质量审查
  security-reviewer.md      # 安全漏洞分析
  build-error-resolver.md   # 构建错误修复
  e2e-runner.md             # Playwright 端到端测试
  refactor-cleaner.md       # 死代码清理
  doc-updater.md            # 文档同步
```

#### 关键原则

- 每个子代理配置允许的工具、MCPs 和权限
- 子代理可以执行技能（skills），实现自主工作
- **限制工具 = 聚焦执行**：工具越少，子代理越不容易跑偏

#### 和 Skills 的配合

子代理 + 技能 = 强大的自动化组合。一个子代理如果能执行你的部分技能，就可以被委派任务后自主运行，使用技能完成工作。

---

### 4. Rules（规则）— 持久化的编码规范

#### 是什么

`.claude/rules/` 目录下放 `.md` 文件，写 Claude 必须始终遵守的规则。两种管理方式：

1. **单文件 CLAUDE.md** — 所有规则写在一个文件（用户级或项目级）
2. **rules/ 目录** — 按关注点拆分为多个文件

#### 推荐结构

```
~/.claude/rules/
  security.md       # 安全检查（禁止硬编码密钥、验证输入）
  coding-style.md   # 编码风格（不可变性、文件组织、大小限制）
  testing.md        # 测试规范（TDD、80% 覆盖率）
  git-workflow.md   # Git 工作流（提交格式、PR 流程）
  agents.md         # 子代理委派规则
  performance.md    # 性能（模型选择、上下文管理）
  patterns.md       # API 响应格式
  hooks.md          # Hook 文档
```

#### 规则示例

- 代码库里不要 emoji
- 前端不要用紫色调
- 部署前必须测试
- 模块化代码优于超大文件
- 永远不要提交 console.log

---

### 5. MCPs（模型上下文协议）— 连接外部服务

#### 是什么

MCP 不是 API 的替代品，而是 API 的 prompt 驱动封装，让 Claude 能更灵活地获取和操作外部信息。

#### 作者的 MCP 配置

```json
{
  "github": { "command": "npx", "args": ["-y", "@modelcontextprotocol/server-github"] },
  "firecrawl": { "command": "npx", "args": ["-y", "firecrawl-mcp"] },
  "supabase": {
    "command": "npx",
    "args": ["-y", "@supabase/mcp-server-supabase@latest", "--project-ref=YOUR_REF"]
  },
  "memory": { "command": "npx", "args": ["-y", "@modelcontextprotocol/server-memory"] },
  "sequential-thinking": { "command": "npx", "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"] },
  "vercel": { "type": "http", "url": "https://mcp.vercel.com" },
  "railway": { "command": "npx", "args": ["-y", "@railway/mcp-server"] },
  "cloudflare-docs": { "type": "http", "url": "https://docs.mcp.cloudflare.com/mcp" },
  "cloudflare-workers-bindings": { "type": "http", "url": "https://bindings.mcp.cloudflare.com/mcp" },
  "cloudflare-workers-builds": { "type": "http", "url": "https://builds.mcp.cloudflare.com/mcp" },
  "cloudflare-observability": { "type": "http", "url": "https://observability.mcp.cloudflare.com/mcp" },
  "clickhouse": { "type": "http", "url": "https://mcp.clickhouse.cloud/mcp" },
  "AbletonMCP": { "command": "uvx", "args": ["ableton-mcp"] },
  "magic": { "command": "npx", "args": ["-y", "@magicuidesign/mcp@latest"] }
}
```

#### 上下文窗口管理（最关键！）

> **你的 200k 上下文窗口，开启太多 MCP 后可能只剩 70k 可用。性能严重下降。**

经验法则：
- 配置 20-30 个 MCP 没问题
- **每项目同时启用的 MCP 控制在 10 个以内**
- **活跃工具数保持在 80 个以下**

按项目禁用不用的 MCP：

```yaml
# 在 ~/.claude.json 的 projects.[path].disabledMcpServers 下配置
disabledMcpServers: [
  "playwright",
  "cloudflare-workers-builds",
  "cloudflare-workers-bindings",
  "cloudflare-observability",
  "cloudflare-docs",
  "clickhouse",
  "AbletonMCP",
  "context7",
  "magic"
]
```

#### Chrome in Claude

内置的浏览器控制插件 MCP，让 Claude 能自主操控浏览器——点击、导航、检查 UI。

---

### 6. Plugins（插件）— 打包好的工具集

#### 是什么

插件是 Skills + MCP + Hooks 的打包组合，一条命令安装，不需要手动配置。

#### 安装方式

```bash
# 添加一个插件市场
claude plugin marketplace add https://github.com/mixedbread-ai/mgrep

# 在 Claude 里运行 /plugins，找到新市场，从那里安装
```

#### LSP 插件（强烈推荐）

如果你经常在编辑器外使用 Claude Code，LSP 插件提供实时类型检查、跳转定义、智能补全，不需要打开 IDE：

```
typescript-lsp@claude-plugins-official    # TypeScript 智能补全
pyright-lsp@claude-plugins-official       # Python 类型检查
```

#### 作者启用的插件列表

```
ralph-wiggum@claude-code-plugins          # 循环自动化
frontend-design@claude-code-plugins       # UI/UX 设计模式
commit-commands@claude-code-plugins       # Git 工作流
security-guidance@claude-code-plugins     # 安全检查
pr-review-toolkit@claude-code-plugins     # PR 自动化
typescript-lsp@claude-plugins-official    # TS 智能补全
hookify@claude-plugins-official           # 对话式创建 Hook
code-simplifier@claude-plugins-official   # 代码简化
feature-dev@claude-code-plugins           # 功能开发
explanatory-output-style@claude-code-plugins  # 解释模式
code-review@claude-code-plugins           # 代码审查
context7@claude-plugins-official          # 实时文档查询
pyright-lsp@claude-plugins-official       # Python 类型检查
mgrep@Mixedbread-Grep                     # 比 grep/ripgrep 更好的搜索
```

同样注意：同时启用的插件不要太多，保持上下文窗口健康。

---

### 7. mgrep — 比 ripgrep 更好的搜索

安装后通过 `/mgrep` 技能使用，支持本地搜索和联网搜索：

```bash
mgrep "function handleSubmit"              # 本地搜索
mgrep --web "Next.js 15 app router changes" # 联网搜索
```

---

### 8. 自定义状态栏

显示：用户名、目录、Git 分支（脏标记）、上下文剩余百分比、模型、时间、待办数量。

配置命令：`/statusline`

---

### 9. 并行工作流

#### /fork — 分叉对话

把当前对话分叉，并行处理不重叠的任务，避免排队。

#### Git Worktrees — 并行 Claude 实例

```bash
git worktree add ../feature-branch feature-branch
# 在每个 worktree 里运行独立的 Claude 实例，互不干扰
```

#### tmux — 长命令监控

让 Claude 的前端/后端服务器在 tmux 会话里运行，你可以随时 detach 和 reattach 查看日志：

```bash
tmux new -s dev
# Claude 在这里跑命令，你可以 detach
tmux attach -t dev
```

---

### 10. 编辑器集成

#### Zed（作者的选择）

Rust 写的轻量编辑器，和 Claude Code 配合很好：

- **Agent Panel** — 实时追踪 Claude 编辑的文件，直接跳转
- **性能** — Rust 写的，秒开，大项目不卡
- **CMD+Shift+R** — 命令面板，快速执行自定义斜杠命令
- **资源占用低** — 不会和 Claude 抢系统资源
- **Vim 模式** — 完整的 vim 快捷键

使用技巧：
1. 分屏：一边终端跑 Claude Code，一边编辑器看文件
2. `Ctrl+G` — 快速打开 Claude 正在编辑的文件
3. 开启自动保存，确保 Claude 读到的文件是最新的
4. 用编辑器的 Git 功能审查 Claude 的改动
5. 开启文件监听，自动重新加载修改的文件

#### VS Code / Cursor

也完全可行。两种模式：
- 终端模式 + `\ide` 同步（和 LSP 插件功能有重叠）
- 扩展模式，UI 更集成

---

### 11. GitHub Actions CI/CD

可以配置 Claude 在 PR 上自动做代码审查。

---

### 12. 沙盒模式

有风险的操作用沙盒模式，Claude 在受限环境中运行不影响真实系统。

反过来的 `--dangerously-skip-permissions` 让 Claude 自由操作，但要小心，可能造成破坏。

---

### 13. 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| Ctrl+U | 删除整行（比连续按退格快） |
| ! | 快速 bash 命令前缀 |
| @ | 搜索文件 |
| / | 斜杠命令 |
| Shift+Enter | 多行输入 |
| Tab | 切换思考过程显示 |
| Esc Esc | 中断 Claude / 恢复代码 |

---

### 14. 其他实用命令

| 命令 | 功能 |
|------|------|
| /rewind | 回到之前的状态 |
| /statusline | 自定义状态栏 |
| /checkpoints | 文件级撤销点 |
| /compact | 手动触发上下文压缩 |
| /plugins | 管理插件和 MCP |
| /mcp | 查看 MCP 状态 |

---

### 15. 作者的五条核心建议

1. **别搞太复杂** — 配置像微调，不是架构。简单就好
2. **上下文窗口很珍贵** — 不用的 MCP 和插件都禁用掉
3. **能并行就并行** — 用 /fork 分叉对话，用 git worktree 并行开发
4. **自动化重复的事** — 用 Hooks 处理格式化、lint、提醒
5. **限制子代理的范围** — 给的工具越少，执行越聚焦

---

## 进阶指南

### 1. Session Memory — 跨会话上下文持久化

#### 核心模式

创建 `.tmp` 文件保存会话状态，下一个会话直接加载恢复：

- 路径：`~/.claude/sessions/YYYY-MM-DD-topic.tmp`
- 每个会话一个文件，不污染旧上下文
- 新会话启动时提供文件路径即可恢复

#### 会话文件应包含

- 哪些方法有效（附验证证据）
- 哪些方法尝试过但失败了
- 哪些方法还没尝试，还剩什么要做

#### 使用时机

- 撞上上下文限制需要继续复杂工作
- 一天结束时保存进度
- 跨天恢复工作时提供文件路径

---

### 2. Strategic Compacting — 战略性上下文压缩

#### 问题

自动压缩（auto-compact）在任意时间点触发，经常在任务中途打断，导致关键上下文丢失。

#### 方案

1. **禁用自动压缩**
2. 在逻辑节点手动 `/compact`：
   - 探索阶段完成后、执行前
   - 完成一个里程碑后、开始下一个前
3. 用计数器 Hook 提醒：

```bash
#!/bin/bash
# Strategic Compact Suggester
# 绑定到 PreToolUse 的 Edit/Write 操作
COUNTER_FILE="/tmp/claude-tool-count-$$"
THRESHOLD=${COMPACT_THRESHOLD:-50}

if [ -f "$COUNTER_FILE" ]; then
    count=$(cat "$COUNTER_FILE")
    count=$((count + 1))
    echo "$count" > "$COUNTER_FILE"
else
    echo "1" > "$COUNTER_FILE"
    count=1
fi

if [ "$count" -eq "$THRESHOLD" ]; then
    echo "[StrategicCompact] $THRESHOLD tool calls reached - consider /compact if transitioning phases" >&2
fi
```

---

### 3. Dynamic System Prompt Injection — 动态系统提示注入

#### 核心区别

| 方式 | 权重 | 加载方式 |
|------|------|----------|
| `.claude/rules/` | 低 | 通过 Read 工具加载（tool output 级别） |
| `@ file` 引用 | 低 | 通过 Read 工具加载（tool output 级别） |
| `--system-prompt` | **高** | 注入到系统提示（system prompt 级别） |

**指令层级**：System Prompt > User Messages > Tool Results

#### 实践：按场景 CLI 别名

```bash
# 日常开发
alias claude-dev='claude --system-prompt "$(cat ~/.claude/contexts/dev.md)"'
# PR 审查模式
alias claude-review='claude --system-prompt "$(cat ~/.claude/contexts/review.md)"'
# 研究/探索模式
alias claude-research='claude --system-prompt "$(cat ~/.claude/contexts/research.md)"'
```

#### 适用场景

- 需要严格行为规则的场景
- 项目特定约束
- 必须优先处理的上下文

对大多数日常任务，区别不大。但对于安全关键、行为规则严格的场景，system prompt 注入能确保 Claude 优先遵守。

---

### 4. Memory Persistence Hooks — 内存持久化 Hook 链

#### 三 Hook 协作

```
SESSION 1                    SESSION 2
─────────                    ─────────
[Start]                      [Start]
│                            │
▼                            ▼
┌──────────────┐             ┌──────────────┐
│ SessionStart │             │ SessionStart │◄── 加载上一个会话
│ Hook         │             │ Hook         │   的上下文
└──────┬───────┘             └──────┬───────┘
│                            │
▼                            ▼
[Working]                    [Working]
│                            │ (基于历史上下文)
▼                            │
┌──────────────┐             ▼
│ PreCompact   │──► 保存状态 [Continue...]
│ Hook         │   压缩前
└──────┬───────┘
│
▼
[Compacted]
│
▼
┌──────────────┐
│ Stop Hook    │──► 持久化到 ──────────►
│ (session-end)│    ~/.claude/sessions/
└──────────────┘
```

#### 配置

```json
{
  "hooks": {
    "PreCompact": [{
      "matcher": "*",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/memory-persistence/pre-compact.sh"
      }]
    }],
    "SessionStart": [{
      "matcher": "*",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/memory-persistence/session-start.sh"
      }]
    }],
    "Stop": [{
      "matcher": "*",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/memory-persistence/session-end.sh"
      }]
    }]
  }
}
```

#### 各 Hook 职责

- **PreCompact**：记录压缩事件，更新活跃会话文件的压缩时间戳
- **SessionStart**：检查最近 7 天的会话文件，通知可用的上下文和已学技能
- **Stop**：创建/更新每日会话文件，追踪开始/结束时间

#### 持续学习机制

当 Claude 在会话中发现非平凡模式（调试技巧、变通方案、项目特定模式）时，自动保存为新 skill。下次类似问题出现时自动加载。

---

### 5. Continuous Learning — 持续学习系统

#### Stop Hook 自动评估

会话结束时自动运行评估脚本，分析会话中值得提取的模式：
- 错误解决方案
- 调试技巧
- 变通方案
- 项目特定模式

保存到 `~/.claude/skills/learned/`。

```json
{
  "hooks": {
    "Stop": [{
      "matcher": "*",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/skills/continuous-learning/evaluate-session.sh"
      }]
    }]
  }
}
```

为什么用 Stop 而不是 UserPromptSubmit？UserPromptSubmit 每条消息都触发，开销大、增加延迟。Stop 只在会话结束时触发一次，轻量，且评估完整会话。

#### `/learn` 命令手动提取

不想等到会话结束？在会话中随时运行 `/learn`，立即提取刚解决的模式，草拟 skill 文件，确认后保存。

#### 会话日志格式

`~/.claude/sessions/YYYY-MM-DD-topic.tmp`，包含：
- 当前状态
- 已完成事项
- 阻碍
- 关键决策
- 下次会话的上下文

#### 其他自改进记忆模式

1. **反思模式**：会话结束后，反思代理提取什么做得好、什么失败、做了什么修正，更新记忆文件
2. **主动建议模式**：每 15 分钟审查最近交互，提议记忆更新，用户批准/拒绝

---

### 6. Token 经济学

#### 主要策略：子代理模型分层

| 模型 | 适用场景 | 价格（input/output per M tokens） |
|------|----------|----------------------------------|
| Haiku | 重复性任务、指令非常清晰、多代理 worker | $0.25/$1.25 |
| Sonnet | 90% 的编码任务（默认选择） | $3/$15 |
| Opus | 首次尝试失败、跨 5+ 文件、架构决策、安全关键代码 | $5/$25 |

**性价比分析**：Haiku vs Opus 是 5x 价差，Sonnet vs Opus 只有 1.67x。组合 Haiku（简单任务）+ Opus（复杂任务）最划算。

#### 模型选择规则

- 默认 Sonnet 做 90% 编码
- 首次失败或跨 5+ 文件 → 升级 Opus
- 重复/明确指令/多代理 worker → 降级 Haiku

在 agent 定义中指定模型：
```yaml
---
name: quick-search
description: Fast file search
tools: Glob, Grep
model: haiku
---
```

#### 基准测试方法

建立有明确目标的 repo + well-defined 计划，每个 git worktree 全用一种模型的子代理，记录任务完成情况，对比 diff 和测试通过率。

#### 工具级优化

- **mgrep** 替代 grep/ripgrep：平均减少约 50% 的 token 消耗
- **后台进程**：不需要 Claude 处理完整输出的命令放 tmux 跑，只把需要的部分复制进来
- **模块化代码库**：文件数百行而非数千行，减少反复读取的 token 浪费

#### System Prompt 瘦身（高级）

Claude Code 的系统提示占 ~18k tokens（200k 的 9%）。可通过补丁减至 ~10k，省 ~7,300 tokens（41% 静态开销）。参考 YK 的方法。

#### 死代码持续清理

用技能持续清理代码库，定期手动浏览，发现重复代码后结合 refactor skill 和 dead code skill 处理。精简的代码库 = 更便宜的 token 成本。

---

### 7. Sub-Agent 上下文问题

#### 问题

子代理只收到查询字面意思，不知道背后的目的和推理。返回的摘要经常遗漏关键细节。

类比："老板派你去开会要摘要。你回来给了回顾。十有八九他还有追问——你的摘要不含他需要的所有信息，因为你没有他的隐式上下文。"

#### 迭代检索模式

```
┌─────────────────┐
│ ORCHESTRATOR    │
│ (has context)   │
└────────┬────────┘
         │ 传递 query + objective
         ▼
┌─────────────────┐
│ SUB-AGENT       │
│ (lacks context) │
└────────┬────────┘
         │ 返回摘要
         ▼
┌─────────────────┐     ┌─────────────┐
│ EVALUATE        │─no─►│ FOLLOW-UP   │
│ Sufficient?     │     │ QUESTIONS   │
└────────┬────────┘     └──────┬──────┘
         │ yes                 │
         ▼                     │ 子代理
      [ACCEPT]                 │ 获取答案
         │                     │
         ◄─────────────────────┘
         (最多 3 轮循环)
```

#### 解决方案

1. 评估每个子代理返回值
2. 不充分时追问（最多 3 轮防止死循环）
3. **传递目标上下文，不只是查询**——帮子代理判断摘要中该包含什么

---

### 8. Sequential Phase Orchestration — 序列阶段编排

#### 五阶段流水线

```
Phase 1: RESEARCH    (Explore agent)    → research-summary.md
Phase 2: PLAN        (planner agent)    → plan.md
Phase 3: IMPLEMENT   (tdd-guide agent)  → code changes
Phase 4: REVIEW      (code-reviewer)    → review-comments.md
Phase 5: VERIFY      (build-error-resolver) → done or loop back
```

#### 关键规则

1. 每个 agent 获得一个清晰输入，产出一个清晰输出
2. 输出成为下一阶段的输入
3. 不跳过任何阶段
4. agent 之间用 `/clear` 刷新上下文
5. 中间输出存文件，不只靠记忆

---

### 9. Agent Abstraction Tierlist

#### Tier 1：直接增益（易用好）

- **Subagents** — 防止上下文腐烂 + 临时专业化，复杂度远低于多代理
- **Metaprompting** — "花 3 分钟写 prompt 替代 20 分钟任务"，提高稳定性
- **开头多问用户** — 在 plan mode 回答问题来校准方向

#### Tier 2：高门槛（难用好）

- **Long-running agents** — 需理解 15 分钟 vs 1.5 小时 vs 4 小时任务的形态和权衡
- **Parallel multi-agent** — 方差极高，只适合高度复杂或分割良好的任务
- **Role-based multi-agent** — 模型进化太快，硬编码启发式规则不值得
- **Computer use agents** — 早期范式，需要大量调试

**结论**：从 Tier 1 开始。只有掌握了基础且有真实需求时才进阶到 Tier 2。

---

### 10. MCP → Skills 替代策略

#### 问题

很多 MCP 本质上是 CLI 的封装（GitHub、Supabase、Vercel、Railway），但占用上下文窗口。

#### 方案

把 MCP 暴露的功能打包成 skills/commands，用 CLI 直接调用：

```bash
# 替代 GitHub MCP
/gh-pr 命令 → 包装 gh pr create + 偏好选项

# 替代 Supabase MCP
skill → 直接用 Supabase CLI
```

#### Lazy Loading 的影响

Boris 团队已实现 MCP lazy loading，启动时不再占满上下文。但 **token 使用和成本**仍未解决——CLI + skills 方式仍能显著节省 token，特别是重操作（数据库查询、部署）。

---

### 11. Evaluations — 评估体系

#### 两种评估模式

**Checkpoint-Based**（基于检查点）：
- 工作流中设置显式检查点
- 每个检查点验证定义的标准
- 验证失败必须修复后才能继续
- 适合：有清晰里程碑的线性工作流

**Continuous**（持续评估）：
- 每 N 分钟或重大变更后运行
- 完整测试套件、构建状态、lint
- 发现回归立即报告并停止修复
- 适合：长会话、探索性重构、维护

#### Grader 类型

| 类型 | 特点 |
|------|------|
| Code-Based | 字符串匹配、二进制测试、静态分析。快、便宜、客观，但对有效变体脆弱 |
| Model-Based | 评分标准、自然语言断言、成对比较。灵活处理细微差别，但不稳定且贵 |
| Human | 专家审查、众包判断、抽查。质量金标准，但贵且慢 |

#### 关键指标

```
pass@k: k 次尝试中至少一次成功
  k=1: 70%  k=3: 91%  k=5: 97%
  → 只要能工作就行时用

pass^k: k 次尝试全部成功
  k=1: 70%  k=3: 34%  k=5: 17%
  → 需要一致性/确定性输出时用
```

#### Eval Roadmap（来自 Anthropic 指南）

1. 尽早开始 — 20-50 个来自真实失败的简单任务
2. 把用户报告的失败转为测试用例
3. 写明确的任务 — 两个专家应能得出相同结论
4. 构建平衡的问题集 — 测试行为应该和不应该发生的情况
5. 构建健壮的 harness — 每次试验从干净环境开始
6. 评估 agent 产出的结果，不是它走的路径
7. 阅读大量试验的转录
8. 监控饱和 — 100% 通过率意味着需要更多测试

#### 基准测试工作流

```
[Same Task]
     │
     ├─── Worktree A (WITH skill)
     │         │
     │         ▼
     │    [Output A]
     │
     └─── Worktree B (WITHOUT skill)
               │
               ▼
          [Output B]

最终：git diff → 比较日志、token 用量、输出质量
```

---

### 12. 并行实例管理

#### 核心原则

**用最少可行的并行量完成最多工作。** 不要设置任意终端数量。

#### The Cascade Method

- 新任务开新标签页（向右排列）
- 从左到右扫描（最老到最新）
- 保持一致的方向流
- 同时聚焦最多 3-4 个任务

#### Two-Instance Kickoff Pattern

新项目用两个实例启动：

**Instance 1：Scaffolding Agent**
- 创建项目结构
- 设置配置（CLAUDE.md、rules、agents）
- 建立约定
- 搭建骨架

**Instance 2：Deep Research Agent**
- 连接所有服务、web search 等
- 创建详细 PRD
- 创建架构 mermaid 图
- 编译实际文档引用

#### Git Worktrees 并行

```bash
git worktree add ../project-feature-a feature-a
git worktree add ../project-feature-b feature-b
git worktree add ../project-refactor refactor-branch
# 每个 worktree 运行独立的 Claude 实例
```

#### 并行注意事项

- 确保任务范围明确，最小重叠
- 选择正交任务防止干扰
- 使用 `/rename` 命名所有聊天避免混淆
- 主聊天做代码修改，分叉做研究/问题

---

### 13. llms.txt Pattern

很多文档站在 URL 后加 `/llms.txt` 可获取 LLM 优化版本：

```
https://docs.example.com/llms.txt
```

给 Claude 干净的、LLM 优化的文档内容，比 Context7 或 Firecrawl 抓取更直接。

---

### 14. Philosophy: Build Reusable Patterns

> "Early on, I spent time building reusable workflows/patterns. Tedious to build, but this had a wild compounding effect as models and agent harnesses improved."

值得投资的模式：
- Subagents
- Skills
- Commands
- Planning patterns
- MCP tools
- Context engineering patterns

**关键洞察**：这些模式在模型升级后仍然有效，甚至可迁移到其他 agent（如 Codex）。投资模式 > 投资特定模型技巧。

---

### 参考资料

- [Anthropic: Demystifying evals for AI agents](https://anthropic.com) (Jan 2026)
- Anthropic: "Claude Code Best Practices" (Apr 2025)
- Fireworks AI: "Eval Driven Development with Claude Code" (Aug 2025)
- YK: "32 Claude Code Tips" (Dec 2025)
- Addy Osmani: "My LLM coding workflow going into 2026"
- @bcherny: Sub-Agent Context Negotiation
- @bcherny: Agent Abstractions Tierlist
- @bcherny: Compound Effects Philosophy
- RLanceMartin: Session Reflection Pattern
- @affaanmustafa: Self-Improving Memory System

---

## Hooks 完整清单

来源：`affaan-m/everything-claude-code` 仓库 `hooks/hooks.json`（2026-05-13 读取）

### 总览

- 7 个事件类型，共 33 个 hooks
- 安装为插件后，Claude Code v2.1+ 按 `hooks/hooks.json` 约定自动加载
- `plugin.json` 中**禁止声明** hooks 字段（会触发 "Duplicate hooks file detected" 错误）

### PreToolUse（8 hooks）

| # | matcher | 脚本 | 功能 |
|---|---------|------|------|
| 1 | Bash | `pre-bash-dispatcher` | Bash 命令预检调度 |
| 2 | Write | `doc-file-warning` | 文档文件写入警告 |
| 3 | Edit\|Write | `suggest-compact` | 建议手动压缩上下文 |
| 4 | * | `observe-runner` | 通用观察器 |
| 5 | Bash\|Write\|Edit\|MultiEdit | `governance-capture` | 治理捕获（操作审计） |
| 6 | Write\|Edit\|MultiEdit | `config-protection` | 配置文件保护 |
| 7 | * | `mcp-health-check` | MCP 服务健康检查 |
| 8 | Edit\|Write\|MultiEdit | `gateguard-fact-force` | 事实守卫（强制事实检查） |

### PreCompact（1 hook）

| # | matcher | 脚本 | 功能 |
|---|---------|------|------|
| 1 | * | `pre-compact` | 手动压缩策略，避免自动压缩丢失关键上下文 |

### SessionStart（1 hook）

| # | matcher | 脚本 | 功能 |
|---|---------|------|------|
| 1 | * | `session-start-bootstrap` | 会话启动引导和初始化 |

### PostToolUse（8 hooks）

| # | matcher | 脚本 | 功能 |
|---|---------|------|------|
| 1 | Bash | `post-bash-dispatcher` | Bash 命令执行后调度 |
| 2 | Edit\|Write\|MultiEdit | `quality-gate` | 质量门禁 |
| 3 | Edit\|Write\|MultiEdit | `design-quality-check` | 设计质量检查 |
| 4 | Edit\|Write\|MultiEdit | `post-edit-accumulator` | 编辑累加器（追踪变更） |
| 5 | Edit | `post-edit-console-warn` | console.log 残留警告 |
| 6 | Bash\|Write\|Edit\|MultiEdit | `governance-capture` | 治理捕获 |
| 7 | * | `session-activity-tracker` | 会话活动追踪 |
| 8 | * | `observe-runner` | 通用观察器 |

### PostToolUseFailure（1 hook）

| # | matcher | 脚本 | 功能 |
|---|---------|------|------|
| 1 | * | `mcp-health-check` | 工具调用失败时检查 MCP 健康 |

### Stop（6 hooks）

| # | matcher | 脚本 | 功能 |
|---|---------|------|------|
| 1 | * | `stop-format-typecheck` | 格式化和类型检查 |
| 2 | * | `check-console-log` | console.log 残留检查 |
| 3 | * | `session-end` | 会话结束处理 |
| 4 | * | `evaluate-session` | 会话评估和模式提取（continuous-learning） |
| 5 | * | `cost-tracker` | Token 成本追踪 |
| 6 | * | `desktop-notify` | 桌面通知 |

### SessionEnd（1 hook）

| # | matcher | 脚本 | 功能 |
|---|---------|------|------|
| 1 | * | `session-end-marker` | 会话结束标记 |

### 共享基础设施

每个 hook 都通过 `plugin-hook-bootstrap.js` 引导启动，再由 `run-with-flags.js` 传递配置标志。这两个脚本是所有 hook 的公共入口，不单独列出功能。
