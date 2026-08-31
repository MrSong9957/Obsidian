---
name: system-reminder-hook-blocking-error
description: "Error from a blocking hook command"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-hook-blocking-error.md
---
<!--
name: 'System Reminder: Hook blocking error'
description: Error from a blocking hook command
ccVersion: 2.1.18
variables:
  - ATTACHMENT_OBJECT
-->
${ATTACHMENT_OBJECT.hookName} hook blocking error from command: "${ATTACHMENT_OBJECT.blockingError.command}": ${ATTACHMENT_OBJECT.blockingError.blockingError}
