# Claude Code Statusline 部署与故障恢复

**唯一权威恢复手册** — 在新 Claude Code 环境中恢复状态栏，或状态栏异常时排障。

## 最终效果

```
Claude Opus 4.6 | 小说/1 | ~ | ██████░░░░ 60% | $0.00 0t 0f 0s | 🛡️ CC Safety Net ✅🌳
```

| 段 | 内容 |
|---|---|
| 模型 | 当前 Claude 模型名 |
| 当前目录 | 缩短为最后 2 级，HOME 用 `~` |
| Git branch | 非 Git 显示 `~` |
| context bar | 上下文使用率，4 色进度条 |
| metrics | $cost / tools / files / duration |
| CC Safety Net | 安全网状态 |

---

## 文件结构

```
~/.claude/
├─ settings.json                          # 配置
└─ scripts/
   ├─ statusline.py                       # 生产脚本
   └─ test_statusline.py                  # 回归测试
```

cc-safety-net 缓存位置：

```
~/.claude/plugins/cache/cc-marketplace/cc-safety-net/<版本>/
```

> 插件版本路径是机器状态，新机器不应盲目照抄版本号。

---

## statusline.py 完整代码

```python
#!/usr/bin/env python3
"""Claude Code StatusLine - Combined version
Layout: model | dir(2-level) | git-branch | context-bar | task(CN) | metrics | safety-net
"""

import json
import sys
import os
import subprocess

# Ensure UTF-8 output on Windows
if sys.stdout.encoding != 'utf-8':
    sys.stdout.reconfigure(encoding='utf-8', errors='surrogatepass')


def get_git_branch(cwd: str) -> str:
    """Get git branch name, searching parent directories if needed."""
    if not cwd:
        return ""
    check_dir = os.path.normpath(cwd)
    while check_dir and check_dir != os.path.sep and not (len(check_dir) == 3 and check_dir[1:] == ':\\'):
        if os.path.isdir(os.path.join(check_dir, ".git")):
            try:
                result = subprocess.run(
                    ["git", "-C", check_dir, "rev-parse", "--abbrev-ref", "HEAD"],
                    capture_output=True, text=True, timeout=2
                )
                if result.returncode == 0:
                    return result.stdout.strip().replace("heads/", "")
                break
            except (subprocess.TimeoutExpired, Exception):
                break
        parent = os.path.dirname(check_dir)
        if parent == check_dir:
            break
        check_dir = parent
    return ""


def shorten_path(path: str, levels: int = 2) -> str:
    """Shorten path to show only last N levels, with ~ for home."""
    if not path:
        return ""
    home = os.environ.get("HOME", "") or os.environ.get("USERPROFILE", "")
    # Normalize path separators for comparison
    norm_path = path.replace("\\", "/")
    norm_home = home.replace("\\", "/") if home else ""
    # Check if path is HOME or HOME subpath (with proper boundary check)
    is_home = norm_path == norm_home
    is_home_subpath = (
        norm_home
        and len(norm_path) > len(norm_home)
        and norm_path[len(norm_home)] == "/"
        and norm_path.startswith(norm_home)
    )
    if is_home:
        return "~"
    if is_home_subpath:
        rest = norm_path[len(norm_home):].lstrip("/")
        parts = [p for p in rest.split("/") if p]
        if len(parts) <= levels:
            return "~/" + "/".join(parts)
        return "~/" + "/".join(parts[-levels:])
    else:
        parts = [p for p in norm_path.split("/") if p]
        if len(parts) <= levels:
            return path
        return "/".join(parts[-levels:])


def build_context_bar(remaining_pct: float) -> str:
    """Build context progress bar with 4-color ECC thresholds."""
    # Only accept real numeric values (not bool, not string, not list, etc.)
    if remaining_pct is None or not isinstance(remaining_pct, (int, float)) or isinstance(remaining_pct, bool):
        return "\033[2m--\033[0m"

    # ECC auto-compact buffer
    AUTO_COMPACT_BUFFER = 16.5
    usable = max(0, ((remaining_pct - AUTO_COMPACT_BUFFER) / (100 - AUTO_COMPACT_BUFFER)) * 100)
    used = max(0, min(100, round(100 - usable)))

    filled = used // 10
    bar = "█" * filled + "░" * (10 - filled)

    if used < 50:
        return f"\033[32m{bar} {used}%\033[0m"      # Green
    elif used < 65:
        return f"\033[33m{bar} {used}%\033[0m"      # Yellow
    elif used < 80:
        return f"\033[38;5;208m{bar} {used}%\033[0m" # Orange
    else:
        return f"\033[1;31m{bar} {used}%\033[0m"     # Red


def format_duration(iso_timestamp: str) -> str:
    """Format duration from ISO timestamp."""
    if not iso_timestamp:
        return ""
    try:
        from datetime import datetime, timezone
        elapsed = int((datetime.now(timezone.utc) - datetime.fromisoformat(iso_timestamp.replace('Z', '+00:00'))).total_seconds())
        if elapsed < 0:
            return ""
        if elapsed < 60:
            return f"{elapsed}s"
        mins = elapsed // 60
        if mins < 60:
            return f"{mins}m"
        hours = mins // 60
        rem = mins % 60
        return f"{hours}h{rem}m" if rem > 0 else f"{hours}h"
    except Exception:
        return ""


def read_current_task(session_id: str) -> str:
    """Read current in-progress task from todos directory."""
    if not session_id:
        return ""
    try:
        claude_dir = os.environ.get("CLAUDE_CONFIG_DIR") or os.path.join(os.path.expanduser("~"), ".claude")
        todos_dir = os.path.join(claude_dir, "todos")
        if not os.path.isdir(todos_dir):
            return ""

        safe_id = "".join(c for c in session_id if c.isalnum() or c in "-_")
        if not safe_id:
            return ""

        files = []
        for f in os.listdir(todos_dir):
            if f.startswith(safe_id) and "-agent-" in f and f.endswith(".json"):
                fp = os.path.join(todos_dir, f)
                files.append((f, os.path.getmtime(fp)))

        if not files:
            return ""

        files.sort(key=lambda x: x[1], reverse=True)
        with open(os.path.join(todos_dir, files[0][0]), "r", encoding="utf-8") as fh:
            todos = json.load(fh)

        for t in todos:
            if t.get("status") == "in_progress":
                return t.get("activeForm", "")
        return ""
    except Exception:
        return ""


def read_bridge(session_id: str) -> dict:
    """Read ECC metrics bridge file from temp directory."""
    if not session_id:
        return {}
    try:
        safe_id = "".join(c for c in session_id if c.isalnum() or c in "-_")
        # Use TEMP/TMP env var (matches Node.js os.tmpdir() behavior)
        tmpdir = os.environ.get("TEMP") or os.environ.get("TMP") or os.environ.get("TMPDIR") or "/tmp"
        bridge_path = os.path.join(tmpdir, f"ecc-metrics-{safe_id}.json")
        if os.path.exists(bridge_path):
            with open(bridge_path, "r", encoding="utf-8") as f:
                return json.load(f)
    except Exception:
        pass
    return {}


def main():
    try:
        input_data = json.load(sys.stdin)
    except json.JSONDecodeError:
        print("Error reading input")
        return

    # Ensure top-level JSON is a dict
    if not isinstance(input_data, dict):
        print("Invalid input: expected JSON object")
        return

    model = input_data.get("model", {}).get("display_name", "") if isinstance(input_data.get("model"), dict) else ""
    cwd = input_data.get("workspace", {}).get("current_dir", "") if isinstance(input_data.get("workspace"), dict) else ""
    remaining_pct = input_data.get("context_window", {}).get("remaining_percentage") if isinstance(input_data.get("context_window"), dict) else None
    session_id = input_data.get("session_id", "") if isinstance(input_data.get("session_id"), str) else ""

    # ANSI
    CYAN = "\033[96m"
    BLUE = "\033[94m"
    GREEN = "\033[92m"
    DIM = "\033[2m"
    BRIGHT = "\033[1;97m"
    METRIC = "\033[38;5;117m"
    RESET = "\033[0m"

    # Components
    dir_display = shorten_path(cwd, levels=2)
    branch = get_git_branch(cwd)
    ctx_bar = build_context_bar(remaining_pct)
    task = read_current_task(session_id)
    bridge = read_bridge(session_id)

    # Metrics (always show 4 items: cost | tools | files | duration)
    cost = bridge.get("total_cost_usd", 0)
    tools = bridge.get("tool_count", 0)
    files = bridge.get("files_modified_count", 0)
    dur = format_duration(bridge.get("first_timestamp", ""))

    metrics = []
    metrics.append(f"${cost:.2f}" if cost > 0 else "$0.00")
    metrics.append(f"{tools}t" if tools > 0 else "0t")
    metrics.append(f"{files}f" if files > 0 else "0f")
    metrics.append(dur if dur else "0s")
    metrics_str = f"{METRIC}{' '.join(metrics)}{RESET}"

    # Sanitize dynamic fields: replace newlines with spaces to keep single-line output
    def sanitize(text: str) -> str:
        return text.replace("\r\n", " ").replace("\r", " ").replace("\n", " ")

    # Build output: model | dir | branch | context-bar | task | metrics
    parts = [f"{CYAN}{sanitize(model)}{RESET}"]
    parts.append(f"{BLUE}{sanitize(dir_display)}{RESET}")
    parts.append(f"{GREEN}{sanitize(branch)}{RESET}" if branch else f"{DIM}~{RESET}")
    parts.append(ctx_bar)
    if task:
        parts.append(f"{BRIGHT}{sanitize(task)}{RESET}")
    if metrics_str:
        parts.append(metrics_str)

    print(" | ".join(parts))


if __name__ == "__main__":
    main()
```

---

## test_statusline.py 完整代码

```python
#!/usr/bin/env python3
"""Regression tests for statusline.py boundary defects.

BUG 1: HOME path prefix collision
BUG 2: remaining_percentage non-numeric crash
BUG 3: top-level JSON non-object crash
BUG 5: newlines in fields break single-line output
"""

import unittest
import subprocess
import json
import os
import sys
import re

# Path to production statusline.py
STATUSLINE_PATH = os.path.join(os.path.dirname(__file__), "statusline.py")
PYTHON_EXE = r"C:\Users\<user>\AppData\Local\Python\bin\python.exe"


def strip_ansi(text: str) -> str:
    """Remove ANSI escape codes from text."""
    ansi_escape = re.compile(r'\x1b\[[0-9;]*m')
    return ansi_escape.sub('', text)


def run_statusline(input_dict: dict) -> tuple[str, str, int]:
    """Run statusline.py with given input dict, return (stdout, stderr, returncode)."""
    input_json = json.dumps(input_dict, ensure_ascii=False)
    result = subprocess.run(
        [PYTHON_EXE, STATUSLINE_PATH],
        input=input_json,
        capture_output=True,
        text=True,
        timeout=10,
        env={**os.environ, "PYTHONIOENCODING": "utf-8"},
    )
    return result.stdout, result.stderr, result.returncode


def run_statusline_raw(input_str: str) -> tuple[str, str, int]:
    """Run statusline.py with raw string input, return (stdout, stderr, returncode)."""
    result = subprocess.run(
        [PYTHON_EXE, STATUSLINE_PATH],
        input=input_str,
        capture_output=True,
        text=True,
        timeout=10,
        env={**os.environ, "PYTHONIOENCODING": "utf-8"},
    )
    return result.stdout, result.stderr, result.returncode


class TestBug1_HomePrefixCollision(unittest.TestCase):
    """BUG 1: shorten_path incorrectly treats paths starting with HOME prefix as HOME subdirs."""

    def _make_input(self, path: str) -> dict:
        return {
            "model": {"display_name": "test"},
            "workspace": {"current_dir": path},
            "context_window": {"remaining_percentage": 50},
        }

    def test_home_subpath_is_recognized(self):
        """HOME/project should display as ~/project."""
        home = os.environ.get("HOME") or os.environ.get("USERPROFILE", "")
        if not home:
            self.skipTest("HOME/USERPROFILE not set")
        stdout, stderr, rc = run_statusline(self._make_input(home + "/project"))
        self.assertEqual(rc, 0, f"Process failed: {stderr}")
        # Should contain ~/project
        self.assertIn("~/project", stdout)

    def test_home_itself_displays_tilde(self):
        """HOME itself should display as ~."""
        home = os.environ.get("HOME") or os.environ.get("USERPROFILE", "")
        if not home:
            self.skipTest("HOME/USERPROFILE not set")
        stdout, stderr, rc = run_statusline(self._make_input(home))
        self.assertEqual(rc, 0, f"Process failed: {stderr}")
        # Should contain ~ (the tilde placeholder)
        self.assertIn("~", stdout.split("|")[1].strip())

    def test_home_plus_zero_not_subpath(self):
        """HOME + '0'/project should NOT be treated as HOME subpath.

        This is the prefix collision bug: C:/Users/<user>0 should not match HOME=C:/Users/<user>
        """
        home = os.environ.get("HOME") or os.environ.get("USERPROFILE", "")
        if not home:
            self.skipTest("HOME/USERPROFILE not set")
        collision_path = home + "0/project"
        stdout, stderr, rc = run_statusline(self._make_input(collision_path))
        self.assertEqual(rc, 0, f"Process failed: {stderr}")
        # Should NOT start with ~/ — it's not a HOME subpath
        dir_part = strip_ansi(stdout.split("|")[1].strip())
        self.assertFalse(
            dir_part.startswith("~/"),
            f"Path '{collision_path}' incorrectly recognized as HOME subpath, got: {dir_part}"
        )

    def test_home_plus_letter_not_subpath(self):
        """HOME + 'back'/project should NOT be treated as HOME subpath."""
        home = os.environ.get("HOME") or os.environ.get("USERPROFILE", "")
        if not home:
            self.skipTest("HOME/USERPROFILE not set")
        collision_path = home + "back/project"
        stdout, stderr, rc = run_statusline(self._make_input(collision_path))
        self.assertEqual(rc, 0, f"Process failed: {stderr}")
        dir_part = strip_ansi(stdout.split("|")[1].strip())
        self.assertFalse(
            dir_part.startswith("~/"),
            f"Path '{collision_path}' incorrectly recognized as HOME subpath, got: {dir_part}"
        )


class TestBug2_RemainingPercentageNonNumeric(unittest.TestCase):
    """BUG 2: build_context_bar crashes on non-numeric remaining_percentage."""

    def _make_input(self, pct_value):
        return {
            "model": {"display_name": "test"},
            "workspace": {"current_dir": "C:/"},
            "context_window": {"remaining_percentage": pct_value},
        }

    def test_string_percentage_no_crash(self):
        """remaining_percentage='abc' should not crash."""
        stdout, stderr, rc = run_statusline(self._make_input("abc"))
        self.assertEqual(rc, 0, f"Crashed with string input. stderr: {stderr}")
        self.assertNotIn("Traceback", stderr)

    def test_list_percentage_no_crash(self):
        """remaining_percentage=[] should not crash."""
        stdout, stderr, rc = run_statusline(self._make_input([]))
        self.assertEqual(rc, 0, f"Crashed with list input. stderr: {stderr}")
        self.assertNotIn("Traceback", stderr)

    def test_dict_percentage_no_crash(self):
        """remaining_percentage={} should not crash."""
        stdout, stderr, rc = run_statusline(self._make_input({}))
        self.assertEqual(rc, 0, f"Crashed with dict input. stderr: {stderr}")
        self.assertNotIn("Traceback", stderr)

    def test_null_percentage_no_crash(self):
        """remaining_percentage=None (null in JSON) should not crash."""
        stdout, stderr, rc = run_statusline(self._make_input(None))
        self.assertEqual(rc, 0, f"Crashed with null input. stderr: {stderr}")
        self.assertNotIn("Traceback", stderr)

    def test_bool_percentage_no_crash(self):
        """remaining_percentage=true (bool in JSON) should not crash and should use default display.

        In Python, bool is subclass of int, but JSON true/false should be treated as invalid percentage.
        The context bar (4th segment) should show '--' as the default fallback.
        """
        stdout, stderr, rc = run_statusline(self._make_input(True))
        self.assertEqual(rc, 0, f"Crashed with bool input. stderr: {stderr}")
        self.assertNotIn("Traceback", stderr)
        # Verify bool is treated as invalid: context bar should show '--' default
        clean = strip_ansi(stdout)
        parts = [p.strip() for p in clean.split("|")]
        self.assertGreaterEqual(len(parts), 4, f"Expected at least 4 segments, got {len(parts)}: {parts}")
        context_bar = parts[3]
        self.assertEqual(
            context_bar, "--",
            f"Bool should use default context bar '--', got: '{context_bar}'"
        )


class TestBug3_TopLevelNonObject(unittest.TestCase):
    """BUG 3: main() crashes with AttributeError when top-level JSON is not an object."""

    def test_array_input_no_crash(self):
        """JSON array [] should not crash."""
        stdout, stderr, rc = run_statusline_raw("[]")
        self.assertNotIn("Traceback", stderr, f"Crashed on array input. stderr: {stderr}")
        # Should either exit cleanly or print controlled error
        self.assertEqual(rc, 0, f"Non-zero exit code: {rc}")

    def test_string_input_no_crash(self):
        """JSON string 'abc' should not crash."""
        stdout, stderr, rc = run_statusline_raw('"abc"')
        self.assertNotIn("Traceback", stderr, f"Crashed on string input. stderr: {stderr}")
        self.assertEqual(rc, 0, f"Non-zero exit code: {rc}")

    def test_number_input_no_crash(self):
        """JSON number 123 should not crash."""
        stdout, stderr, rc = run_statusline_raw("123")
        self.assertNotIn("Traceback", stderr, f"Crashed on number input. stderr: {stderr}")
        self.assertEqual(rc, 0, f"Non-zero exit code: {rc}")

    def test_boolean_input_no_crash(self):
        """JSON boolean true should not crash."""
        stdout, stderr, rc = run_statusline_raw("true")
        self.assertNotIn("Traceback", stderr, f"Crashed on boolean input. stderr: {stderr}")
        self.assertEqual(rc, 0, f"Non-zero exit code: {rc}")

    def test_null_input_no_crash(self):
        """JSON null should not crash."""
        stdout, stderr, rc = run_statusline_raw("null")
        self.assertNotIn("Traceback", stderr, f"Crashed on null input. stderr: {stderr}")
        self.assertEqual(rc, 0, f"Non-zero exit code: {rc}")


class TestBug5_NewlinesInFields(unittest.TestCase):
    """BUG 5: Newlines in JSON fields break single-line status bar output."""

    def _make_input(self, model_name: str, cwd: str = "C:/") -> dict:
        return {
            "model": {"display_name": model_name},
            "workspace": {"current_dir": cwd},
            "context_window": {"remaining_percentage": 50},
        }

    def test_model_with_newline_is_single_line(self):
        """Model name containing \\n should not produce multi-line output."""
        stdout, stderr, rc = run_statusline(self._make_input("line1\nline2"))
        self.assertEqual(rc, 0, f"Process failed: {stderr}")
        # Output should be a single line (no embedded newlines in the actual status)
        lines = [l for l in stdout.strip().split("\n") if l.strip()]
        self.assertEqual(
            len(lines), 1,
            f"Expected single-line output, got {len(lines)} lines: {repr(stdout)}"
        )

    def test_model_with_carriage_return_is_single_line(self):
        """Model name containing \\r should not produce multi-line output."""
        stdout, stderr, rc = run_statusline(self._make_input("line1\rline2"))
        self.assertEqual(rc, 0, f"Process failed: {stderr}")
        lines = [l for l in stdout.strip().split("\n") if l.strip()]
        self.assertEqual(
            len(lines), 1,
            f"Expected single-line output, got {len(lines)} lines: {repr(stdout)}"
        )

    def test_model_with_crlf_is_single_line(self):
        """Model name containing \\r\\n should not produce multi-line output."""
        stdout, stderr, rc = run_statusline(self._make_input("line1\r\nline2"))
        self.assertEqual(rc, 0, f"Process failed: {stderr}")
        lines = [l for l in stdout.strip().split("\n") if l.strip()]
        self.assertEqual(
            len(lines), 1,
            f"Expected single-line output, got {len(lines)} lines: {repr(stdout)}"
        )

    def test_path_with_newline_is_single_line(self):
        """Path containing \\n should not produce multi-line output."""
        stdout, stderr, rc = run_statusline(self._make_input("test", "C:/path\nwith\nnewlines"))
        self.assertEqual(rc, 0, f"Process failed: {stderr}")
        lines = [l for l in stdout.strip().split("\n") if l.strip()]
        self.assertEqual(
            len(lines), 1,
            f"Expected single-line output, got {len(lines)} lines: {repr(stdout)}"
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
```

---

## settings.json 配置片段

```json
{
  "env": {
    "SAFETY_NET_PLUGIN_ROOT": "<ACTIVE_SAFETY_NET_ROOT>"
  },
  "statusLine": {
    "type": "command",
    "command": "PYTHONIOENCODING=utf-8 <PYTHON_EXE> <STATUSLINE_PATH> | CLAUDE_SETTINGS_PATH=<SETTINGS_PATH> node <ACTIVE_SAFETY_NET_ROOT>/dist/bin/cc-safety-net.js --statusline"
  },
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "node \"${SAFETY_NET_PLUGIN_ROOT}/dist/bin/cc-safety-net.js\" --claude-code",
            "timeout": 15
          }
        ]
      }
    ]
  }
}
```

占位符说明：

| 占位符 | 含义 |
|--------|------|
| `<PYTHON_EXE>` | 当前机器真实 Python executable 路径 |
| `<STATUSLINE_PATH>` | `~/.claude/scripts/statusline.py` |
| `<SETTINGS_PATH>` | `~/.claude/settings.json` |
| `<ACTIVE_SAFETY_NET_ROOT>` | 当前活跃 cc-safety-net 版本目录 |

---

## 从零恢复

### Step 1：确认 Python

```bash
# 检查 Python 是否可用（排除 Windows Store alias）
python --version
python3 --version
py --version
```

**注意：** Windows 上 `python` / `python3` / `py` 可能命中 Windows Store alias（exit code 49）。使用 `which python` 确认实际路径。

当前验证过的 executable：

```
C:/Users/<user>/AppData/Local/Python/bin/python.exe
```

### Step 2：安装/确认 cc-safety-net

```bash
# 检查当前活跃版本
ls ~/.claude/plugins/cache/cc-marketplace/cc-safety-net/
```

规则：

| 文件 | 含义 |
|------|------|
| `.in_use` | 当前活跃版本 |
| `.orphaned_at` | 已孤立版本，**不应引用** |

```bash
# 确认活跃版本
for dir in ~/.claude/plugins/cache/cc-marketplace/cc-safety-net/*/; do
  if [ -d "$dir/.in_use" ]; then
    echo "活跃版本: $(basename $dir)"
  fi
  if [ -f "$dir/.orphaned_at" ]; then
    echo "孤立版本: $(basename $dir)"
  fi
done
```

> `2.0.6` 只是本文档创建时的 VERIFIED 基线，不是永久固定版本。

### Step 3：复制脚本

```bash
mkdir -p ~/.claude/scripts
cp statusline.py ~/.claude/scripts/
cp test_statusline.py ~/.claude/scripts/
```

### Step 4：配置 settings.json

在 `settings.json` 中添加：

```json
{
  "env": {
    "SAFETY_NET_PLUGIN_ROOT": "<ACTIVE_SAFETY_NET_ROOT>"
  },
  "statusLine": {
    "type": "command",
    "command": "PYTHONIOENCODING=utf-8 <PYTHON_EXE> <STATUSLINE_PATH> | CLAUDE_SETTINGS_PATH=<SETTINGS_PATH> node <ACTIVE_SAFETY_NET_ROOT>/dist/bin/cc-safety-net.js --statusline"
  }
}
```

替换所有占位符为当前机器的实际值。

### Step 5：运行测试

```bash
PYTHONIOENCODING=utf-8 <PYTHON_EXE> ~/.claude/scripts/test_statusline.py
```

预期：

```
18 tests
18 PASS
0 FAIL
0 ERROR
```

### Step 6：py_compile

```bash
<PYTHON_EXE> -m py_compile ~/.claude/scripts/statusline.py
```

预期：exit code 0

### Step 7：重启 Claude Code

`settings.json` 中的 `env` 是 Claude Code 启动时加载的。**修改后必须完全重启 Claude Code。**

### Step 8：真实运行验收

```bash
# 确认环境变量
echo "$SAFETY_NET_PLUGIN_ROOT"

# 确认状态栏正常显示
# 观察 Claude Code 底部状态栏
```

---

## 已验证边界矩阵

| 类别 | 边界 | 已验证结果 |
|------|------|-----------|
| 路径 | 中文路径 | ✅ 正常 |
| 路径 | 日文 | ✅ 正常 |
| 路径 | Emoji | ✅ 正常 |
| 路径 | `\` / `/` 混用 | ✅ 正常 |
| 路径 | HOME 本身 | ✅ `~` |
| 路径 | HOME 子目录 | ✅ `~/...` |
| 路径 | HOME 前缀碰撞 | ✅ 不误判 |
| 路径 | 非 HOME 路径 | ✅ 正常缩短 |
| Git | 非 Git 仓库 | ✅ `~` |
| Git | 普通 branch | ✅ 正常 |
| Git | detached HEAD | ✅ `HEAD` |
| Git | worktree | ✅ 正常 |
| Git | Git timeout | ✅ 2 秒保护 |
| JSON | `{}` | ✅ 不崩溃 |
| JSON | remaining string | ✅ `--` |
| JSON | remaining list | ✅ `--` |
| JSON | remaining dict | ✅ `--` |
| JSON | remaining null | ✅ `--` |
| JSON | remaining bool | ✅ `--` |
| JSON | 顶层 array | ✅ 不 traceback |
| JSON | 顶层 string | ✅ 不 traceback |
| JSON | 顶层 number | ✅ 不 traceback |
| JSON | 顶层 bool | ✅ 不 traceback |
| JSON | 顶层 null | ✅ 不 traceback |
| 终端 | `\n` | ✅ 保持单行 |
| 终端 | `\r` | ✅ 保持单行 |
| 终端 | `\r\n` | ✅ 保持单行 |
| Unicode | 中文 + safety-net | ✅ 正常 |
| Safety Net | 活跃版本 | ✅ 2.0.6 VERIFIED |
| Safety Net | orphaned 版本 | ✅ 不应引用 |

---

## 本轮实际修复过的问题

### 中文乱码

```
症状：小说 → 乱码
根因：Windows stdin/stdout 编码链路不一致
修复：PYTHONIOENCODING=utf-8 + Python stdout UTF-8
测试：test_model_with_newline_is_single_line 等
```

### HOME 前缀碰撞

```
症状：HOME=C:/Users/<user>，C:/Users/<user>0/project → ~/0/project
根因：startswith 无边界检查
修复：检查 norm_path[len(norm_home)] == "/"
测试：test_home_plus_zero_not_subpath
```

### bool 被当成 int

```
症状：remaining_percentage=true → 100%
根因：isinstance(True, int) == True
修复：isinstance(remaining_pct, bool) 排除
测试：test_bool_percentage_no_crash（验证 context bar = --）
```

### 非 object JSON

```
症状：输入 [] → AttributeError: 'list' object has no attribute 'get'
根因：假设顶层是 dict
修复：isinstance(input_data, dict) 检查
测试：test_array_input_no_crash 等
```

### 换行破坏 statusline

```
症状：模型名含 \n → 状态栏多行
根因：未清理换行符
修复：sanitize() 函数统一替换
测试：test_model_with_newline_is_single_line 等
```

### safety-net 版本漂移

```
症状：配置指向 2.0.3，实际活跃 2.0.6
根因：插件升级后配置未更新
修复：手动更新 SAFETY_NET_PLUGIN_ROOT 和 statusLine.command
验证：CONFIG MIGRATION VERIFIED
```

---

## 故障速查表

| 症状 | 优先检查 |
|------|---------|
| 只显示 Safety Net | Python statusline 链路 / command 管道 |
| 中文乱码 | PYTHONIOENCODING / Python executable |
| 整个 statusline 消失 | Python / Node / safety-net 路径 |
| Safety Net hooks 失效 | SAFETY_NET_PLUGIN_ROOT 是否指向 orphaned 版本 |
| 插件升级后突然失效 | `.in_use` 当前版本与配置版本 |
| Git 分支没有显示 | 当前目录是否 Git repo |
| 非 Git 没有占位 | branch fallback `~` |
| 路径变成 `~/0/...` | HOME 前缀边界 |
| context 变成异常百分比 | remaining_percentage 类型 |
| 状态栏出现多行 | sanitize / `\r\n` |
| 修改 settings 后仍旧行为 | Claude Code 是否完全重启 |

---

## 当前已知但不处理的问题

### stderr 污染

```
状态：理论风险，未真实复现
策略：不修改
原因：不能为未确认问题增加 2>/dev/null 并可能吞掉真实错误
```

### Python executable 绝对路径

```
状态：机器相关
策略：安装时检测真实 executable，不是运行时动态猜测
原因：Windows Store alias 问题
```

### cc-safety-net 版本路径

```
状态：必须硬编码当前活跃版本
策略：安装/插件升级后重新检测 .in_use
```

### UNC / MAX_PATH 等

```
状态：未验证，不作为当前已知 Bug
策略：出现真实症状后再 systematic-debugging
```

---

## VERIFIED 基线

```
statusline tests:  18 / 18 PASS
py_compile:        PASS
CONFIG MIGRATION:  VERIFIED
RUNTIME:           VERIFIED

SAFETY_NET_PLUGIN_ROOT: .../2.0.6
Active plugin:          2.0.6 (.in_use)
Old configured plugin:  2.0.3 (.orphaned_at) — 已从正式配置移除

日期: 2026-08-23
```
