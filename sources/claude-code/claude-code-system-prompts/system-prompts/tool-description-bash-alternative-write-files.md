---
name: tool-description-bash-alternative-write-files
description: "Bash tool alternative: use Write for file writing instead of echo/cat"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt, tool-description]
ccVersion: 2.1.53
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/tool-description-bash-alternative-write-files.md
---
<!--
name: 'Tool Description: Bash (alternative — write files)'
description: Bash tool alternative: use Write for file writing instead of echo/cat
ccVersion: 2.1.53
variables:
  - WRITE_TOOL_NAME
-->
Write files: Use ${WRITE_TOOL_NAME} (NOT echo >/cat <<EOF)
