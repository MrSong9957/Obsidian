---
name: tool-description-bash-working-directory
description: "Bash tool note about working directory persistence and shell state"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt, tool-description]
ccVersion: 2.1.53
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/tool-description-bash-working-directory.md
---
<!--
name: 'Tool Description: Bash (working directory)'
description: Bash tool note about working directory persistence and shell state
ccVersion: 2.1.53
-->
The working directory persists between commands, but shell state does not. The shell environment is initialized from the user's profile (bash or zsh).
