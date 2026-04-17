---
name: 状态栏插件安装
description: Claude Code 状态栏内置功能配置说明 — 首次安装或配置状态栏时参考
type: summary
sources:
  - "[[sources/claude-code-system-prompts/README]]"
updated: 2026-04-05
tags: [claude-code, statusline]
status: deprecated
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# Claude Code 状态栏插件安装指南

> 内容已迁移至 [[wiki/claude-code-p2.md|claude-code-p2]]。

> 相关文档：[[safety-net-guide.md|Safety Net 安全网]] | [[Everything-Claude-Code.md|ECC 技能集]]

## 概述

StatusLine 是 Claude Code 的内置功能，无需额外安装。只需在 `~/.claude/settings.json` 中配置即可。

## 功能

在每次命令执行后显示：
- 当前模型名称
- 当前工作目录（`$HOME` 显示为 `~`）
- Git 分支名称
- 上下文使用进度条（颜色编码：绿<50%，黄<80%，红>=80%）

**示例输出：**
```
MiniMax M2.5-highspeed | sry-opencode/docs | main | ████████░░ 75%
```

## 快速开始（推荐 Python 版本）

> **强烈推荐使用 Python 版本**。Bash 版本在某些环境下可能因转义问题导致状态栏无法显示，Python 版本更稳定可靠。

### 步骤 1：创建脚本文件

保存到 `~/.claude/scripts/statusline.py`：

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
    if branch:
        parts.append(f"{GREEN}{branch}{RESET}")
    parts.append(context_info)

    print(" | ".join(parts))


if __name__ == "__main__":
    main()
```

### 步骤 2：添加执行权限

```bash
chmod +x ~/.claude/scripts/statusline.py
```

### 步骤 3：配置 settings.json

在 `~/.claude/settings.json` 中添加：

```json
{
  "statusLine": {
    "type": "command",
    "command": "python3 ~/.claude/scripts/statusline.py"
  }
}
```

### 步骤 4：重启 Claude Code

退出当前会话（`Ctrl+C`），重新运行 `claude` 即可看到状态栏。

## 可用变量

通过 stdin 传入 JSON，支持以下变量：

| 变量 | 说明 |
|------|------|
| `model.display_name` | 模型显示名称 |
| `model.name` | 模型内部名称 |
| `workspace.current_dir` | 当前工作目录 |
| `context_window.used_tokens` | 已用 token 数 |
| `context_window.limit_tokens` | token 限制 |
| `context_window.used_percentage` | 上下文使用百分比 |
| `session.id` | 会话 ID |
| `session.start_time` | 会话开始时间戳 |

## 配置类型说明

### 1. template（简洁）

适合简单需求，无需 shell：

```json
{
  "statusLine": {
    "type": "template",
    "template": "{{model.display_name}} | {{workspace.current_dir}} | {{context_window.used_percentage}}%"
  }
}
```

### 2. command（灵活）

适合自定义样式，通过命令处理 stdin JSON 数据。

## Shell 脚本示例（备选）

> **注意**：如果 Python 版本正常工作，不需要使用 Shell 版本。Shell 版本在某些环境下可能因转义问题导致无法显示。

### 简洁版

```bash
input=$(cat)
model=$(echo "$input" | jq -r '.model.display_name // empty')
cwd=$(echo "$input" | jq -r '.workspace.current_dir // empty')
used_pct=$(echo "$input" | jq -r '.context_window.used_percentage // empty')

dir_display="${cwd/#$HOME/~}"
printf "%s | %s | %s%%" "$model" "$dir_display" "$used_pct"
```

### 带颜色版

```bash
input=$(cat)
used_pct=$(echo "$input" | jq -r '.context_window.used_percentage')
model=$(echo "$input" | jq -r '.model.display_name')

if (( $(echo "$used_pct > 80" | bc -l) )); then
  color="\033[31m"  # 红色
elif (( $(echo "$used_pct > 50" | bc -l) )); then
  color="\033[33m"  # 黄色
else
  color="\033[32m"  # 绿色
fi

printf "%s\033[0m | %s%%" "$model" "$color" "$used_pct"
```

### 显示 Git 分支

```bash
input=$(cat)
model=$(echo "$input" | jq -r '.model.display_name')
cwd=$(echo "$input" | jq -r '.workspace.current_dir')

cd "$cwd" 2>/dev/null
branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
dir="${cwd/#$HOME/~}"

if [ -n "$branch" ]; then
  printf "%s | %s (%s)" "$model" "$dir" "$branch"
else
  printf "%s | %s" "$model" "$dir"
fi
```

## 高级用法

### 自定义脚本文件

```json
{
  "statusLine": {
    "type": "command",
    "command": "/path/to/your/statusline.py"
  }
}
```

### 错误处理（Python）

```python
try:
    input_data = json.load(sys.stdin)
except json.JSONDecodeError:
    print("Error: Invalid input")
    sys.exit(1)
```

### 缓存 Git 分支（性能优化）

```python
import time

CACHE_FILE = "/tmp/statusline_branch_cache"
CACHE_TTL = 30  # seconds

def get_git_branch_cached(cwd: str) -> str:
    # Check cache
    try:
        if os.path.exists(CACHE_FILE):
            mtime = os.path.getmtime(CACHE_FILE)
            if time.time() - mtime < CACHE_TTL:
                with open(CACHE_FILE) as f:
                    return f.read().strip()
    except Exception:
        pass

    branch = get_git_branch(cwd)

    # Update cache
    try:
        with open(CACHE_FILE, 'w') as f:
            f.write(branch)
    except Exception:
        pass

    return branch
```

## 性能考虑

1. **避免昂贵操作**：不要执行 `git status`，避免调用外部 API
2. **减少 subprocess**：一次处理多个字段
3. **使用缓存**：Git 分支等信息可以缓存
4. **保持轻量**：每次交互都会更新，脚本越轻响应越快

## 故障排查

| 问题 | 解决方案 |
|------|----------|
| 状态栏不显示 | 优先使用 Python 版本，避免 bash 转义问题 |
| 脚本执行失败 | 检查脚本路径和执行权限 (`chmod +x`) |
| Git 分支不显示 | 确认当前目录在 git 仓库中 |
| 颜色显示异常 | 确认终端支持 ANSI 颜色 |

## 注意事项

- 每次工具执行后自动刷新
- command 类型执行 shell，复杂命令会影响响应速度
- 命令失败时不显示状态栏

## 参考

- 官方文档：https://code.claude.com/docs/zh-CN/statusline
