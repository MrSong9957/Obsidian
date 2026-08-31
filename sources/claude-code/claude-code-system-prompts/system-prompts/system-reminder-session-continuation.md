---
name: system-reminder-session-continuation
description: "Notification that session continues from another machine"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-session-continuation.md
---
<!--
name: 'System Reminder: Session continuation'
description: Notification that session continues from another machine
ccVersion: 2.1.18
variables:
  - GET_CWD_FN
-->
This session is being continued from another machine. Application state may have changed. The updated working directory is ${GET_CWD_FN()}
