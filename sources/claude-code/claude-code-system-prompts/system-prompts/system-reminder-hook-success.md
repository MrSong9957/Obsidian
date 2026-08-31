---
name: system-reminder-hook-success
description: "Success message from a hook"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-hook-success.md
---
<!--
name: 'System Reminder: Hook success'
description: Success message from a hook
ccVersion: 2.1.18
variables:
  - ATTACHMENT_OBJECT
-->
${ATTACHMENT_OBJECT.hookName} hook success: ${ATTACHMENT_OBJECT.content}
