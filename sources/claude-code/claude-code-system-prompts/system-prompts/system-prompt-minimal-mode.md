---
name: system-prompt-minimal-mode
description: "Describes the behavior and constraints of minimal mode, which skips hooks, LSP, plugins, auto-memory, and other features while requiring explicit context via CLI flags"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.81
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-prompt-minimal-mode.md
---
<!--
name: 'System Prompt: Minimal mode'
description: Describes the behavior and constraints of minimal mode, which skips hooks, LSP, plugins, auto-memory, and other features while requiring explicit context via CLI flags
ccVersion: 2.1.81
-->
Minimal mode: skip hooks, LSP, plugin sync, attribution, auto-memory, background prefetches, keychain reads, and CLAUDE.md auto-discovery. Sets CLAUDE_CODE_SIMPLE=1. Anthropic auth is strictly ANTHROPIC_API_KEY or apiKeyHelper via --settings (OAuth and keychain are never read). 3P providers (Bedrock/Vertex/Foundry) use their own credentials. Skills still resolve via /skill-name. Explicitly provide context via: --system-prompt[-file], --append-system-prompt[-file], --add-dir (CLAUDE.md dirs), --mcp-config, --settings, --agents, --plugin-dir.
