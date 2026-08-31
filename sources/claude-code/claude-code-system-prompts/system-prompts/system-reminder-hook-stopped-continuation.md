---
name: system-reminder-hook-stopped-continuation
description: "Message when a hook stops continuation"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-hook-stopped-continuation.md
---
<!--
name: 'System Reminder: Hook stopped continuation'
description: Message when a hook stops continuation
ccVersion: 2.1.18
variables:
  - ATTACHMENT_OBJECT
-->
${ATTACHMENT_OBJECT.hookName} hook stopped continuation: ${ATTACHMENT_OBJECT.message}
