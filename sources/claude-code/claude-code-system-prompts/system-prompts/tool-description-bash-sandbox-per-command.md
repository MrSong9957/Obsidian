---
name: tool-description-bash-sandbox-per-command
description: "Treat each command individually; default to sandbox for future commands"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt, tool-description]
ccVersion: 2.1.53
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/tool-description-bash-sandbox-per-command.md
---
<!--
name: 'Tool Description: Bash (sandbox — per-command)'
description: Treat each command individually; default to sandbox for future commands
ccVersion: 2.1.53
-->
Treat each command you execute with `dangerouslyDisableSandbox: true` individually. Even if you have recently run a command with this setting, you should default to running future commands within the sandbox.
