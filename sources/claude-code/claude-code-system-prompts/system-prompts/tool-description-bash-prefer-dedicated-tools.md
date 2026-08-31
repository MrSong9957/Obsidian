---
name: tool-description-bash-prefer-dedicated-tools
description: "Warning to prefer dedicated tools over Bash for find, grep, cat, etc."
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt, tool-description]
ccVersion: 2.1.71
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/tool-description-bash-prefer-dedicated-tools.md
---
<!--
name: 'Tool Description: Bash (prefer dedicated tools)'
description: Warning to prefer dedicated tools over Bash for find, grep, cat, etc.
ccVersion: 2.1.71
variables:
  - READ_ONLY_SEARCHING_BASH_COMMANDS
-->
IMPORTANT: Avoid using this tool to run ${READ_ONLY_SEARCHING_BASH_COMMANDS} commands, unless explicitly instructed or after you have verified that a dedicated tool cannot accomplish your task. Instead, use the appropriate dedicated tool as this will provide a much better experience for the user:
