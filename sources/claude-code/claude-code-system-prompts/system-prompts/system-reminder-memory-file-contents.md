---
name: system-reminder-memory-file-contents
description: "Contents of a memory file by path"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.79
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-memory-file-contents.md
---
<!--
name: 'System Reminder: Memory file contents'
description: Contents of a memory file by path
ccVersion: 2.1.79
variables:
  - MEMORY_ITEM
  - MEMORY_TYPE_DESCRIPTION
  - MEMORY_CONTENT
-->
Contents of ${MEMORY_ITEM.path}${MEMORY_TYPE_DESCRIPTION}:

${MEMORY_CONTENT}
