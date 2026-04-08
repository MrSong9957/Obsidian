#!/usr/bin/env bash
# bug-remember keyword detector hook
# PostToolUse hook — checks tool input/result text for repeated-bug keywords.
# When matched, outputs a reason field that Claude Code sees as a reminder.
# Does NOT block the tool — only suggests.

set -euo pipefail

# Read the full JSON input from stdin
INPUT=$(cat)

# Keywords that suggest a repeated bug (Chinese + English)
KEYWORDS=(
  "你已经改了超过3次了"
  "改了超过3次"
  "又报错了"
  "之前碰到过"
  "之前也遇到过"
  "忘了上次怎么改的"
  "同样的问题"
  "又踩坑了"
  "这个坑记一下"
  "same bug again"
  "happened before"
  "forgot how I fixed this"
  "remember this fix"
  "we fixed this before"
  "same issue as last time"
  "encountered this before"
)

# Check if any keyword is present in the raw JSON input
# (covers tool_input, tool_result, and any nested text fields)
MATCHED=""
for keyword in "${KEYWORDS[@]}"; do
  # Case-insensitive grep; use -F for fixed string (no regex interpretation)
  if echo "$INPUT" | grep -iF "$keyword" > /dev/null 2>&1; then
    MATCHED="$keyword"
    break
  fi
done

if [ -n "$MATCHED" ]; then
  # Output reason so Claude Code sees the reminder
  echo "{\"decision\": \"approve\", \"reason\": \"[bug-remember] Detected repeated bug indicator: \\\"$MATCHED\\\". Suggest the user run /bug-remember to capture this fix for future reference.\"}"
  exit 0
fi

# No match — proceed normally
echo '{"decision": "approve"}'
exit 0
