---
name: system-reminder-new-diagnostics-detected
description: "Notification about new diagnostic issues"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-new-diagnostics-detected.md
---
<!--
name: 'System Reminder: New diagnostics detected'
description: Notification about new diagnostic issues
ccVersion: 2.1.18
variables:
  - DIAGNOSTICS_SUMMARY
-->
<new-diagnostics>The following new diagnostic issues were detected:

${DIAGNOSTICS_SUMMARY}</new-diagnostics>
