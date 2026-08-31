---
name: tool-description-bash-sleep-use-check-commands
description: "Bash tool instruction: use check commands rather than sleeping when polling"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt, tool-description]
ccVersion: 2.1.53
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/tool-description-bash-sleep-use-check-commands.md
---
<!--
name: 'Tool Description: Bash (sleep — use check commands)'
description: Bash tool instruction: use check commands rather than sleeping when polling
ccVersion: 2.1.53
-->
If you must poll an external process, use a check command (e.g. `gh run view`) rather than sleeping first.
