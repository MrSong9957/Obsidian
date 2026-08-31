---
name: system-reminder-invoked-skills
description: "List of skills invoked in this session"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-invoked-skills.md
---
<!--
name: 'System Reminder: Invoked skills'
description: List of skills invoked in this session
ccVersion: 2.1.18
variables:
  - FORMATTED_SKILLS_LIST
-->
The following skills were invoked in this session. Continue to follow these guidelines:

${FORMATTED_SKILLS_LIST}
