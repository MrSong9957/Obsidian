---
name: system-reminder-exited-plan-mode
description: "Notification when exiting plan mode"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.30
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-exited-plan-mode.md
---
<!--
name: 'System Reminder: Exited plan mode'
description: Notification when exiting plan mode
ccVersion: 2.1.30
variables:
  - ATTACHMENT_OBJECT
-->
## Exited Plan Mode

You have exited plan mode. You can now make edits, run tools, and take actions.${ATTACHMENT_OBJECT.planExists?` The plan file is located at ${ATTACHMENT_OBJECT.planFilePath} if you need to reference it.`:""}
