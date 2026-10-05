---
name: Claude Code 子代理架构分析
description: 基于 Claude Code 源码的子代理架构分析——预设角色体系、工具策略与模型选择，作为 mi-code 子代理系统的对标参考
type: concept
updated: 2026-10-04
tags: [claude-code, subagent, agent-architecture, mi-code]
source: self
---

# Claude Code 子代理架构分析

> 基于 Claude Code 源码分析，对标 mi-code 子代理系统的参考文档。

## 预设角色体系

CC 有 **6 个内置角色 + 1 个合成角色（fork）**，同时支持用户自定义角色。

| 角色 | agentType | 工具策略 | model | 特殊 |
|------|-----------|---------|-------|------|
| general-purpose | `general-purpose` | `['*']` 全部 | 继承 | 默认角色 |
| Explore | `Explore` | 黑名单：禁 Agent/Edit/Write/ExitPlanMode | haiku(外部)/inherit(内部) | omitClaudeMd, 省略 gitStatus |
| Plan | `Plan` | 继承 Explore | inherit | omitClaudeMd |
| verification | `verification` | 同 Explore | inherit | background=true |
| claude-code-guide | `claude-code-guide` | 白名单：Glob/Grep/Read/WebFetch/WebSearch | haiku | permissionMode=dontAsk |
| statusline-setup | `statusline-setup` | 白名单：Read/Edit | sonnet | — |
| fork | `fork` | `['*']` | inherit | 继承父进程 system prompt (cache friendly) |

### 角色定义结构（BaseAgentDefinition）

```ts
type BaseAgentDefinition = {
  agentType: string           // 角色唯一标识
  whenToUse: string           // 主 agent 选择此角色的描述
  tools?: string[]            // 白名单，['*'] = 全部
  disallowedTools?: string[]  // 黑名单
  model?: string              // 'inherit' | 'haiku' | 'sonnet' | 'opus'
  omitClaudeMd?: boolean      // 只读角色省略 CLAUDE.md 省 token
  maxTurns?: number           // 最大轮数限制
  background?: boolean        // 是否始终后台运行
  permissionMode?: PermissionMode
  getSystemPrompt: (params) => string  // 动态生成 system prompt
}
```

## 工具过滤：两层机制

### 第一层：全局黑名单（所有子代理）

```ts
ALL_AGENT_DISALLOWED_TOOLS = new Set([
  TASK_OUTPUT, EXIT_PLAN_MODE, ENTER_PLAN_MODE,
  AGENT_TOOL_NAME,  // 非内部用户移除（防止递归）
  ASK_USER_QUESTION, TASK_STOP,
])
```

### 第二层：角色级黑名单

```ts
// Explore 角色
disallowedTools: [AGENT, EXIT_PLAN_MODE, FILE_EDIT, FILE_WRITE, NOTEBOOK_EDIT]
```

> **关键设计差异**：CC 用**黑名单**（disallowedTools），新工具自动可见除非显式禁用。mi-code 用**白名单**（tools 数组），新工具不可见除非显式加入。黑名单更灵活但安全性稍低。

## System Prompt 构造

```
子代理 system prompt = getSystemPrompt()   // 角色自己的 prompt
                     + 环境 notes           // 绝对路径提示、emoji 禁令
                     + envInfo              // CWD、平台、模型名
```

- CLAUDE.md 通过 **userContext** 注入（不是 system prompt），Explore/Plan 可省略（omitClaudeMd）
- 自定义 prompt 作为 **user message** 发送，不替换 system prompt
- fork 子代理继承父进程的完整 system prompt（字节相同，利于 prompt cache）

## 递归保护

三层防护：
1. **外部用户**：AGENT_TOOL_NAME 在全局黑名单 → 子代理看不到 spawn 工具
2. **异步代理**：ASYNC_AGENT_ALLOWED_TOOLS 不含 AGENT_TOOL_NAME
3. **fork**：`isInForkChild()` 检测消息中的 fork 标记 → 硬抛错

> 无显式深度数字限制，但每层受 maxTurns 约束。

## mi-code 对标差距

| 维度 | CC | mi-code | 差距等级 |
|------|-----|---------|---------|
| 预设角色数 | 7 | 3 (explore/plan/general) | 低 — 核心角色已有 |
| 工具过滤 | 黑名单(disallowedTools) | 白名单(tools) | 中 — 白名单更安全但需维护 |
| system prompt | 动态 getSystemPrompt + envInfo | 静态字符串 | 中 — 缺环境信息 |
| CLAUDE.md 省略 | omitClaudeMd (Explore/Plan) | 无 | 低 — token 优化 |
| model 分配 | per-role (haiku/sonnet/inherit) | 全用 small model | 中 — Explore 应用 cheaper model |
| fork 模式 | 继承父 prompt (cache 友好) | 有 forkMode 但未充分使用 | 低 |
| maxTurns | per-role 定义 | 统一 explore=25, 其他=15 | 低 |
| 递归保护 | 全局黑名单移除 Agent 工具 | 白名单不含 spawn_agent | ✅ 一致 |

## AUTO-0025 worktree 验证结果

日期：2026-07-22

| 验证项 | 结果 |
|--------|------|
| tsc 类型检查 | ✅ exit 0 |
| 全量测试 | 1807 passed, 4 failed (预先存在的 layout/bootstrap, 与本改动无关) |
| 核心测试聚焦(11 文件) | ✅ 144 passed |
| spawn_agent provider 修复 | ✅ 不再报 ANTHROPIC_API_KEY（走 streamingQuery 多 provider） |
| 子代理输出质量 | ✅ 只保留最后一轮 text + explore 25 步 + fallback |
| plan 模式审批 UI | ✅ ExitPlanModeOverlayV2 round 边框 + planMode 色 |
| 清屏(auto+clear) | ✅ \x1b[2J\x1b[3J\x1b[H + 重挂载 |
| PlanStore 升级 | ✅ slug 命名 + 30 天清理 + 恢复 + 可配置目录 |
| read_plan_file 工具 | ✅ 剥离 frontmatter 返回正文 |
| spinner token k 格式 | ✅ 3909 → 3.9k |

## 相关
- mi-code AUTO-0025 计划审批 UI 修订
- mi-code 源码：`src/agent/roles.ts`, `src/agent/subagent.ts`, `src/agent/tools/spawn-agent-tool.ts`
