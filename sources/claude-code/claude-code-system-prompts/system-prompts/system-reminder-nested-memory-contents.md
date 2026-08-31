---
name: system-reminder-nested-memory-contents
description: "Contents of a nested memory file"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-nested-memory-contents.md
---
<!--
name: 'System Reminder: Nested memory contents'
description: Contents of a nested memory file
ccVersion: 2.1.18
variables:
  - ATTACHMENT_OBJECT
-->
Contents of ${ATTACHMENT_OBJECT.content.path}:

${ATTACHMENT_OBJECT.content.content}
