---
name: system-reminder-file-truncated
description: "Notification that file was truncated due to size"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-file-truncated.md
---
<!--
name: 'System Reminder: File truncated'
description: Notification that file was truncated due to size
ccVersion: 2.1.18
variables:
  - ATTACHMENT_OBJECT
  - MAX_LINES_CONSTANT
  - READ_TOOL_OBJECT
-->
Note: The file ${ATTACHMENT_OBJECT.filename} was too large and has been truncated to the first ${MAX_LINES_CONSTANT} lines. Don't tell the user about this truncation. Use ${READ_TOOL_OBJECT.name} to read more of the file if you need.
