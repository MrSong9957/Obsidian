---
name: Claude Code 提示词架构
description: Claude Code 10 层治理架构、系统提示词构造、自动模式分类器、消息规范化
type: concept
updated: 2026-06-27
tags: claude-code, prompt, architecture, system-prompt, permissions
source: [[sources/claude-code/claude-code-references.md]]
---

# Claude Code 提示词架构

## 10 层治理架构

```
┌─────────────────────────────────────────────────────────┐
│  1. Tool definition                                    │
├─────────────────────────────────────────────────────────┤
│  2. Permission rules (allow/deny/ask from 5 sources)   │
├─────────────────────────────────────────────────────────┤
│  3. Permission mode (default/auto/plan)                │
├─────────────────────────────────────────────────────────┤
│  4. Auto-mode classifier (2-stage LLM)                │
├─────────────────────────────────────────────────────────┤
│  5. Dangerous pattern detection                        │
├─────────────────────────────────────────────────────────┤
│  6. Path validation                                    │
├─────────────────────────────────────────────────────────┤
│  7. System prompt injection                            │
├─────────────────────────────────────────────────────────┤
│  8. Message normalization                              │
├─────────────────────────────────────────────────────────┤
│  9. Token budget                                       │
├─────────────────────────────────────────────────────────┤
│  10. Enterprise policy                                 │
└─────────────────────────────────────────────────────────┘
```

## 系统提示词构造

在 `src/constants/prompts.ts`（914 行）：
- `getSystemPrompt()` 从 section 函数组装 string[]
- sections: intro/system/doingTasks/actions/tools/tone/efficiency/memory
- `SYSTEM_PROMPT_DYNAMIC_BOUNDARY` 分隔静态（全局缓存）和动态（会话特定）内容

## 身份前缀

在 `src/constants/system.ts`：
- DEFAULT_PREFIX: "You are Claude Code, Anthropic's official CLI for Claude."
- AGENT_SDK_CLAUDE_CODE_PRESET_PREFIX
- AGENT_SDK_PREFIX

## 有效提示词组装

在 `src/utils/systemPrompt.ts`：
- 优先级链：override > coordinator > agent > custom(--system-prompt) > default
- `appendSystemPrompt` 始终追加在末尾

## CLAUDE.md 加载

在 `src/utils/claudemd.ts`（1479 行）：
- 4 级优先级：managed(/etc/) > user(~/.claude/) > project(CLAUDE.md) > local(CLAUDE.local.md)
- 最大 40,000 字符
- 支持 `@include` 指令和 `.gitignore` 风格排除

## 自动模式分类器

在 `src/utils/permissions/yoloClassifier.ts`：
- 2 阶段：Stage 1 快速 XML（64 tokens），Stage 2 chain-of-thought（4096 tokens）
- 解析失败时 fail-closed
- 使用单独的 LLM 调用评估工具使用是否符合 CLAUDE.md 意图

## 消息规范化

在 `src/utils/messages.ts`：
- `normalizeMessagesForAPI()` 将 system-reminder 内容包装在 `<system-reminder>` 标签中
- 剥离虚拟消息、合并连续消息、规范化工具输入

## API 层

在 `src/services/api/claude.ts`（3419 行）：
- `buildSystemPromptBlocks()` 将 SystemPrompt 数组转换为 API TextBlockParam[]（带 cache_control）
- `splitSysPromptPrefix()` 在动态边界处分割，用于全局 vs 临时缓存范围

## 工具约束系统

在 `src/Tool.ts`：
- 每个工具声明 inputSchema (Zod)、checkPermissions()、isReadOnly()、isDestructive()、isConcurrencySafe()
- maxResultSizeChars、strict、toAutoClassifierInput()

## Token 预算

- MAX_TOOL_RESULT_TOKENS=100,000
- DEFAULT_MAX_RESULT_SIZE_CHARS=50,000
- 超过 maxResultSizeChars 的工具结果持久化到磁盘（带预览）
- 当工具描述 >20% 上下文窗口时，通过 ToolSearch 延迟加载