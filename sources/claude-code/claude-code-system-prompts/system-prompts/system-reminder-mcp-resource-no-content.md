---
name: system-reminder-mcp-resource-no-content
description: "Shown when MCP resource has no content"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt]
ccVersion: 2.1.18
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/system-reminder-mcp-resource-no-content.md
---
<!--
name: 'System Reminder: MCP resource no content'
description: Shown when MCP resource has no content
ccVersion: 2.1.18
variables:
  - ATTACHMENT_OBJECT
-->
<mcp-resource server="${ATTACHMENT_OBJECT.server}" uri="${ATTACHMENT_OBJECT.uri}">(No content)</mcp-resource>
