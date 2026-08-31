---
name: system-reminder-agent-mention
description: "Notification that user wants to invoke an agent"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-agent-mention.md
---
<!--
name: 'System Reminder: Agent mention'
description: Notification that user wants to invoke an agent
ccVersion: 2.1.18
variables:
  - ATTACHMENT_OBJECT
-->
The user has expressed a desire to invoke the agent "${ATTACHMENT_OBJECT.agentType}". Please invoke the agent appropriately, passing in the required context to it. 
