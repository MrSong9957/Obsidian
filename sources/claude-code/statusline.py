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
def white(t):   return _c(97, t)    # bright white  — model name
def blue(t):    return _c(94, t)    # bright blue   — directory
def magenta(t): return _c(95, t)    # bright magenta — git branch
def cyan(t):    return _c(96, t)    # bright cyan   — billing/metrics

# ============================================================
# Git branch detection
# ============================================================
def get_git_branch() -> str:
    try:
        cwd = os.environ.get("CLAUDE_WORKING_DIR", ".")
        # First try: git branch --show-current (works for normal repos)
        result = subprocess.run(
            ["git", "branch", "--show-current"],
            capture_output=True, text=True, timeout=3, cwd=cwd,
        )
        if result.returncode == 0 and result.stdout.strip():
            return result.stdout.strip()
        # Fallback: git symbolic-ref HEAD (works for empty repos with no commits)
        result = subprocess.run(
            ["git", "symbolic-ref", "--short", "HEAD"],
            capture_output=True, text=True, timeout=3, cwd=cwd,
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

    def _num(v, default=0.0):
        """Coerce stdin/bridge values safely — they can arrive as strings."""
        try:
            return float(v)
        except (TypeError, ValueError):
            return default

    def seg(fn):
        """Run one segment builder; skip on failure so a single bad stdin
        field can't blank the whole line or starve the safety-net pipe."""
        try:
            v = fn()
            if v:
                parts.append(v)
        except Exception:
            pass

    try:
        # Extract fields from stdin (null-defended: Claude Code emits null
        # container/leaf fields during startup & compaction transitions).
        model = data.get("model") or {}
        workspace = data.get("workspace") or {}
        context_window = data.get("context_window") or {}
        cost_data = data.get("cost") or {}
        session_id = data.get("session_id") or ""
        cwd = data.get("cwd") or workspace.get("current_dir") or os.getcwd()
        # display_name may carry a stray ANSI artifact (e.g. "glm-5.2[1m]");
        # strip real escapes and their literal bracket form.
        model_name = re.sub(r"(\x1b)?\[[0-9;]*m\]?", "",
                            model.get("display_name") or model.get("id") or "unknown") or "unknown"

        parts = [white(model_name)]

        # Directory (blue)
        seg(lambda: blue(shorten_path(cwd)))

        # Git branch (magenta)
        def _git():
            b = get_git_branch()
            return magenta(b) if b else None
        seg(_git)

        # Context bar (semantic 4-color: green/yellow/orange/red)
        def _bar():
            rpct = context_window.get("remaining_percentage")
            if not isinstance(rpct, (int, float)):
                rpct = 100
            usable = max(0, ((rpct - AUTO_COMPACT_BUFFER) / (100 - AUTO_COMPACT_BUFFER)) * 100)
            used = max(0, min(100, round(100 - usable)))
            return build_context_bar(used)
        seg(_bar)

        # Current task (yellow, only if present)
        def _task():
            t = read_current_task(session_id)
            return yellow(t) if t else None
        seg(_task)

        # Metrics (cyan): cost + duration from Claude Code's own `cost` object
        # (authoritative); tool/file counts from the ECC bridge, best-effort.
        def _metrics():
            cost = _num(cost_data.get("total_cost_usd"))
            elapsed = _num(cost_data.get("total_duration_ms")) / 1000
            bridge = read_bridge(session_id)
            tool_count = int(_num(bridge.get("tool_count")))
            files_modified = int(_num(bridge.get("files_modified_count")))
            m = []
            m.append(f"${cost:.2f}" if cost > 0 else "$0.00")
            m.append(f"{tool_count}t" if tool_count > 0 else "0t")
            m.append(f"{files_modified}f" if files_modified > 0 else "0f")
            m.append(format_duration(elapsed) if elapsed > 0 else "0s")
            return cyan(" ".join(m))
        seg(_metrics)

        # Output (cc-safety-net will append safety indicator via pipe)
        print(" | ".join(parts))
    except Exception:
        # Ultimate fallback: never emit zero stdout.
        try:
            print(white(model_name))
        except Exception:
            print(white("statusline"))

if __name__ == "__main__":
    main()
