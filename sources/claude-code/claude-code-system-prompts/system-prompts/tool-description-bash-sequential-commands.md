---
name: tool-description-bash-sequential-commands
description: "Bash tool instruction: chain dependent commands with &&"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt, tool-description]
ccVersion: 2.1.53
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/tool-description-bash-sequential-commands.md
---
<!--
name: 'Tool Description: Bash (sequential commands)'
description: Bash tool instruction: chain dependent commands with &&
ccVersion: 2.1.53
variables:
  - BASH_TOOL_NAME
-->
If the commands depend on each other and must run sequentially, use a single ${BASH_TOOL_NAME} call with '&&' to chain them together.
