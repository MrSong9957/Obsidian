---
name: system-reminder-verify-plan-reminder
description: "Reminder to verify completed plan"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-verify-plan-reminder.md
---
<!--
name: 'System Reminder: Verify plan reminder'
description: Reminder to verify completed plan
ccVersion: 2.1.18
variables:
  - TASK_TOOL_NAME
-->
You have completed implementing the plan. Please call the "" tool directly (NOT the ${TASK_TOOL_NAME} tool or an agent) to verify that all plan items were completed correctly.
