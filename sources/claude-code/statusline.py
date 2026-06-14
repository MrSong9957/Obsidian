import sys
import json
import os
import re
import subprocess
import tempfile
from pathlib import Path

# ============================================================
# Context bar thresholds (ECC standard)
# ============================================================
AUTO_COMPACT_BUFFER = 16.5  # Reserve for auto-compact overhead

# ============================================================
# ANSI color helpers
# ============================================================
def _c(code: int, text: str) -> str:
    return f"\033[{code}m{text}\033[0m"

def green(t):  return _c(32, t)
def yellow(t): return _c(33, t)
def red(t):    return _c(31, t)
def orange(t): return _c("38;5;208", t)
def dim(t):    return _c(2, t)
def bold(t):   return _c(1, t)

# ============================================================
# Git branch detection
# ============================================================
def get_git_branch() -> str:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            capture_output=True, text=True, timeout=3,
            cwd=os.environ.get("CLAUDE_WORKING_DIR", "."),
        )
        if result.returncode == 0:
            return result.stdout.strip()
    except Exception:
        pass
    return ""

# ============================================================
# Path shortening
# ============================================================
def shorten_path(path: str, n: int = 2) -> str:
    parts = Path(path).parts
    if len(parts) <= n:
        return path
    return os.sep.join(parts[-n:])

# ============================================================
# Context bar (4-color: green < 50%, yellow < 65%, orange < 80%, red >= 80%)
# ============================================================
def build_context_bar(context_used_pct: float, width: int = 10) -> str:
    if context_used_pct < 0:
        return dim("N/A")
    used = max(0, min(100, round(context_used_pct)))
    filled = round(used / 100 * width)
    empty = width - filled
    bar = "█" * filled + "░" * empty

    if used < 50:
        colored_bar = green(bar)
    elif used < 65:
        colored_bar = yellow(bar)
    elif used < 80:
        colored_bar = orange(bar)
    else:
        colored_bar = red(bar)

    return f"{colored_bar} {used}%"

# ============================================================
# Duration formatting
# ============================================================
def format_duration(seconds: float) -> str:
    if seconds < 60:
        return f"{int(seconds)}s"
    minutes = int(seconds // 60)
    if minutes < 60:
        return f"{minutes}m"
    hours = minutes // 60
    mins = minutes % 60
    return f"{hours}h{mins}m"

# ============================================================
# Current task from todo files
# ============================================================
def read_current_task(session_id: str) -> str:
    """Read active task from todo files."""
    if not session_id:
        return ""
    try:
        todo_dir = Path.home() / ".claude" / "todos"
        if not todo_dir.exists():
            return ""
        for f in todo_dir.glob(f"{session_id[:8]}*-agent-*.json"):
            try:
                data = json.loads(f.read_text(encoding="utf-8"))
                if data.get("status") == "in_progress" and data.get("activeForm"):
                    return data["activeForm"]
            except Exception:
                continue
    except Exception:
        pass
    return ""

# ============================================================
# ECC metrics bridge reader
# ============================================================
def read_bridge(session_id: str) -> dict:
    """Read ECC metrics bridge file from temp directory."""
    if not session_id:
        return {}
    try:
        safe_id = "".join(c for c in session_id if c.isalnum() or c in "-_")
        tmpdir = os.environ.get("TEMP") or os.environ.get("TMP") or os.environ.get("TMPDIR") or "/tmp"
        bridge_path = os.path.join(tmpdir, f"ecc-metrics-{safe_id}.json")
        if os.path.exists(bridge_path):
            with open(bridge_path, "r", encoding="utf-8") as f:
                return json.load(f)
    except Exception:
        pass
    return {}

# ============================================================
# Main: read stdin JSON from Claude Code
# ============================================================
def main():
    try:
        data = json.loads(sys.stdin.read())
    except Exception:
        print(dim("statusline: no input"))
        return

    # Extract fields from stdin
    model = data.get("model", {})
    model_name = model.get("display_name") or model.get("id", "unknown")
    session_id = data.get("session_id", "")

    # Context usage
    context_window = data.get("context_window", {})
    context_used_pct = context_window.get("used_percentage", -1)
    context_remaining_pct = context_window.get("remaining_percentage", 100)

    # Calculate real usage accounting for auto-compact buffer
    # ECC logic: remaining_pct includes buffer, so subtract it to get usable
    usable = max(0, ((context_remaining_pct - AUTO_COMPACT_BUFFER) / (100 - AUTO_COMPACT_BUFFER)) * 100)
    used = max(0, min(100, round(100 - usable)))

    # Working directory
    cwd = data.get("cwd", os.getcwd())
    short_cwd = shorten_path(cwd)

    # Git branch
    branch = get_git_branch()

    # ECC bridge metrics
    bridge = read_bridge(session_id)
    tool_count = bridge.get("tool_count", 0)
    files_modified = bridge.get("files_modified_count", 0)
    cost = bridge.get("total_cost_usd", 0)

    # Duration from bridge
    first_ts = bridge.get("first_timestamp", 0)
    elapsed = 0
    if first_ts:
        import time
        elapsed = time.time() - first_ts / 1000  # bridge stores ms

    # Current task
    task = read_current_task(session_id)

    # Build status line parts
    parts = []

    # Model
    parts.append(bold(model_name))

    # Directory
    parts.append(dim(short_cwd))

    # Git branch
    if branch:
        parts.append(dim(branch))

    # Context bar
    parts.append(build_context_bar(used))

    # Current task (if any)
    if task:
        parts.append(yellow(task))

    # Metrics: cost | tools | files | duration (always show 4 items)
    metrics = []
    metrics.append(f"${cost:.2f}" if cost > 0 else "$0.00")
    metrics.append(f"{tool_count}t" if tool_count > 0 else "0t")
    metrics.append(f"{files_modified}f" if files_modified > 0 else "0f")
    metrics.append(format_duration(elapsed) if elapsed > 0 else "0s")
    parts.append(dim(" ".join(metrics)))

    # Output (cc-safety-net will append safety indicator via pipe)
    print(" | ".join(parts))

if __name__ == "__main__":
    main()
