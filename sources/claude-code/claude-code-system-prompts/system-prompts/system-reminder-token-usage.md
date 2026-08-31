---
name: system-reminder-token-usage
description: "Current token usage statistics"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-token-usage.md
---
<!--
name: 'System Reminder: Token usage'
description: Current token usage statistics
ccVersion: 2.1.18
variables:
  - ATTACHMENT_OBJECT
-->
Token usage: ${ATTACHMENT_OBJECT.used}/${ATTACHMENT_OBJECT.total}; ${ATTACHMENT_OBJECT.remaining} remaining
