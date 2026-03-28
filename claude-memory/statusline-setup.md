# Claude Code 状态栏安装指南

## 步骤 1：创建脚本

创建目录并写入 `~/.claude/scripts/statusline.py`：

```python
#!/usr/bin/env python3
"""Claude Code StatusLine - Python implementation"""

import json
import sys
import os
import subprocess


def get_git_branch(cwd: str) -> str:
    """Get git branch name, searching parent directories if needed."""
    if not cwd:
        return ""

    check_dir = cwd
    while check_dir and check_dir != "/":
        if os.path.isdir(os.path.join(check_dir, ".git")):
            try:
                result = subprocess.run(
                    ["git", "-C", check_dir, "rev-parse", "--abbrev-ref", "HEAD"],
                    capture_output=True,
                    text=True,
                    timeout=2
                )
                if result.returncode == 0:
                    return result.stdout.strip().replace("heads/", "")
                break
            except (subprocess.TimeoutExpired, Exception):
                break
        check_dir = os.path.dirname(check_dir)
    return ""


def get_progress_bar(percentage: int, length: int = 10) -> str:
    """Generate a progress bar."""
    filled = min(max(int(percentage * length / 100), 0), length)
    empty = length - filled
    return "█" * filled + "░" * empty


def shorten_path(path: str, levels: int = 2) -> str:
    """Shorten path to show only last N levels, with ~ for home."""
    if not path:
        return ""

    home = os.environ.get("HOME", "")
    if home and path.startswith(home):
        rest = path[len(home):].lstrip(os.sep)
        if not rest:  # Exactly home
            return "~"
        parts = rest.split(os.sep)
        if len(parts) <= levels:
            return "~/" + "/".join(parts)
        return "/".join(parts[-levels:])
    else:
        parts = path.split(os.sep)
        # Filter empty parts (e.g., from leading /)
        parts = [p for p in parts if p]
        if len(parts) <= levels:
            return path
        return "/".join(parts[-levels:])


def main():
    try:
        input_data = json.load(sys.stdin)
    except json.JSONDecodeError:
        print("Error reading input")
        return

    model = input_data.get("model", {}).get("display_name", "")
    cwd = input_data.get("workspace", {}).get("current_dir", "")
    used_pct = input_data.get("context_window", {}).get("used_percentage")

    # Show only last 2 levels of path
    dir_display = shorten_path(cwd, levels=2)

    # Get git branch
    branch = get_git_branch(cwd)

    # ANSI color codes
    CYAN = "\033[96m"
    BLUE = "\033[94m"
    GREEN = "\033[92m"
    YELLOW = "\033[93m"
    RED = "\033[91m"
    DIM = "\033[2m"
    RESET = "\033[0m"

    # Build context info with progress bar
    if used_pct is not None and isinstance(used_pct, (int, float)):
        if used_pct < 50:
            bar_color = GREEN
        elif used_pct < 80:
            bar_color = YELLOW
        else:
            bar_color = RED
        bar = get_progress_bar(int(used_pct))
        context_info = f"{bar_color}{bar}{RESET} {int(used_pct)}%"
    else:
        context_info = f"{DIM}--{RESET}"

    # Build output
    parts = [f"{CYAN}{model}{RESET}", f"{BLUE}{dir_display}{RESET}"]
    parts.append(f"{GREEN}{branch or '~'}{RESET}")
    parts.append(context_info)

    print(" | ".join(parts))


if __name__ == "__main__":
    main()
```

设置执行权限：`chmod +x ~/.claude/scripts/statusline.py`

## 步骤 2：安装 Safety Net 插件

在 Claude Code 中执行：
1. `/plugin marketplace add kenryu42/cc-marketplace`
2. `/plugin install safety-net@cc-marketplace`

## 步骤 3：配置 settings.json

修改 `~/.claude/settings.json`：

```json
{
  "skipDangerousModePermissionPrompt": true,
  "statusLine": {
    "type": "command",
    "command": "python3 ~/.claude/scripts/statusline.py | npx -y cc-safety-net --statusline"
  },
  "enabledPlugins": {
    "safety-net@cc-marketplace": true
  }
}
```

注意：`skipDangerousModePermissionPrompt` 和其他已有配置项需保留。

## 步骤 4：重启 Claude Code

退出当前会话（Ctrl+C），重新运行 `claude`。

## 验证

```bash
echo '{"model":{"display_name":"Test"},"workspace":{"current_dir":"/home/app/project"},"context_window":{"used_percentage":45}}' | python3 ~/.claude/scripts/statusline.py
```

预期输出：`Test Model | ~/project | ████░░░░░░ 45%`（带颜色）

## 经验

- Python 版比 bash 内联更稳定（避免转义问题）
- 脚本通过 stdin 接收 JSON（可用变量：model、workspace、context_window）
- 进度条颜色：<50% 绿、<80% 黄、>=80% 红
- Git 分支仅在 git 仓库内显示，非 git 目录用 `~` 占位
