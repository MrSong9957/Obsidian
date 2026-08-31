---
name: system-reminder-plan-file-reference
description: "Reference to an existing plan file"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-plan-file-reference.md
---
<!--
name: 'System Reminder: Plan file reference'
description: Reference to an existing plan file
ccVersion: 2.1.18
variables:
  - ATTACHMENT_OBJECT
-->
A plan file exists from plan mode at: ${ATTACHMENT_OBJECT.planFilePath}

Plan contents:

${ATTACHMENT_OBJECT.planContent}

If this plan is relevant to the current work and not already complete, continue working on it.
