---
name: system-reminder-file-opened-in-ide
description: "Notification that user opened a file in IDE"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-file-opened-in-ide.md
---
<!--
name: 'System Reminder: File opened in IDE'
description: Notification that user opened a file in IDE
ccVersion: 2.1.18
variables:
  - ATTACHMENT_OBJECT
-->
The user opened the file ${ATTACHMENT_OBJECT.filename} in the IDE. This may or may not be related to the current task.
