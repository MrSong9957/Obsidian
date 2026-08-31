---
name: tool-description-bash-timeout
description: "Bash tool instruction: optional timeout configuration"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt, tool-description]
ccVersion: 2.1.53
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/tool-description-bash-timeout.md
---
<!--
name: 'Tool Description: Bash (timeout)'
description: Bash tool instruction: optional timeout configuration
ccVersion: 2.1.53
variables:
  - GET_MAX_TIMEOUT_MS
  - GET_DEFAULT_TIMEOUT_MS
-->
You may specify an optional timeout in milliseconds (up to ${GET_MAX_TIMEOUT_MS()}ms / ${GET_MAX_TIMEOUT_MS()/60000} minutes). By default, your command will timeout after ${GET_DEFAULT_TIMEOUT_MS()}ms (${GET_DEFAULT_TIMEOUT_MS()/60000} minutes).
