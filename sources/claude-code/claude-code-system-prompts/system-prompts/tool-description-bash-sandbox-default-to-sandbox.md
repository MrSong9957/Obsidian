---
name: tool-description-bash-sandbox-default-to-sandbox
description: "Default to sandbox; only bypass when user asks or evidence of sandbox restriction"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt, tool-description]
ccVersion: 2.1.53
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/tool-description-bash-sandbox-default-to-sandbox.md
---
<!--
name: 'Tool Description: Bash (sandbox — default to sandbox)'
description: Default to sandbox; only bypass when user asks or evidence of sandbox restriction
ccVersion: 2.1.53
-->
You should always default to running commands within the sandbox. Do NOT attempt to set `dangerouslyDisableSandbox: true` unless:
