---
name: system-reminder-usd-budget
description: "Current USD budget statistics"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-usd-budget.md
---
<!--
name: 'System Reminder: USD budget'
description: Current USD budget statistics
ccVersion: 2.1.18
variables:
  - ATTACHMENT_OBJECT
-->
USD budget: $${ATTACHMENT_OBJECT.used}/$${ATTACHMENT_OBJECT.total}; $${ATTACHMENT_OBJECT.remaining} remaining
