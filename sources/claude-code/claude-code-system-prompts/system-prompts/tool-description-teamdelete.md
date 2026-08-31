---
name: tool-description-teamdelete
description: "Tool description for the TeamDelete tool"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt, tool-description]
ccVersion: 2.1.33
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/tool-description-teamdelete.md
---
<!--
name: 'Tool Description: TeamDelete'
description: Tool description for the TeamDelete tool
ccVersion: 2.1.33
-->

# TeamDelete

Remove team and task directories when the swarm work is complete.

This operation:
- Removes the team directory (`~/.claude/teams/{team-name}/`)
- Removes the task directory (`~/.claude/tasks/{team-name}/`)
- Clears team context from the current session

**IMPORTANT**: TeamDelete will fail if the team still has active members. Gracefully terminate teammates first, then call TeamDelete after all teammates have shut down.

Use this when all teammates have finished their work and you want to clean up the team resources. The team name is automatically determined from the current session's team context.
