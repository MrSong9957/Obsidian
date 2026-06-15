---
name: claude-code-statusline-setup
description: 一句话说明
type: concept
updated: 2026-05-30
tags: [tag1, tag2]
source: https://example.com
---
# Claude Code 状态栏插件安装指南

## 概述

StatusLine 是 Claude Code 的内置功能，无需额外安装。只需在 `~/.claude/settings.json` 中配置即可。

## 功能

在每次命令执行后显示：
- 当前模型名称
- 当前工作目录（最后两级，前缀 `~`）
- Git 分支名称（非 git 仓库显示 `~` 占位符）
- 上下文使用进度条（颜色编码：绿<50%，黄<80%，红>=80%）
- Safety-net 保护状态（需额外配置）

**示例输出：**
```
mimo-v2.5-pro | ~/Projects/test | ~ | ███░░░░░░░ 30% | 🛡️ Safety Net ✅
```

## 快速开始（推荐 Python 版本）

> **强烈推荐使用 Python 版本**。Bash 版本在某些环境下可能因转义问题导致状态栏无法显示，Python 版本更稳定可靠。

### 步骤 1：创建脚本文件

> 🟢 **推荐直接用 canonical 源** [`statusline.py`](./statusline.py)（含 4 色配色、per-segment 隔离、null 防御、计费取 stdin `cost`、ANSI 残片剥离）：
> ```bash
> cp sources/claude-code/statusline.py ~/.claude/scripts/statusline.py && chmod +x ~/.claude/scripts/statusline.py
> ```
> 下方内联脚本仅为**最小参考、不再维护**，勿用于生产（避免与 canonical 源漂移）。

保存到 `~/.claude/scripts/statusline.py`：

```python
#!/usr/bin/env python3
"""Claude Code StatusLine - Python implementation"""

import json
import sys
import os
import subprocess

sys.stdout.reconfigure(encoding="utf-8")


def get_git_branch(cwd: str) -> str:
    """Get git branch name, searching parent directories if needed."""
    if not cwd:
        return ""

    check_dir = cwd
    while check_dir and check_dir not in ("/", "\\"):
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
        parent = os.path.dirname(check_dir)
        if parent == check_dir:
            break
        check_dir = parent
    return ""


def get_progress_bar(percentage: int, length: int = 10) -> str:
    """Generate a progress bar."""
    filled = min(max(int(percentage * length / 100), 0), length)
    empty = length - filled
    return "█" * filled + "░" * empty


def main():
    try:
        input_data = json.load(sys.stdin)
    except json.JSONDecodeError:
        print("Error reading input")
        return

    model = input_data.get("model", {}).get("display_name", "")
    cwd = input_data.get("workspace", {}).get("current_dir", "")
    used_pct = input_data.get("context_window", {}).get("used_percentage")

    # Format path: only show last two directory levels, prefix with ~
    if cwd:
        parts_list = cwd.replace("\\", "/").rstrip("/").split("/")
        # Filter out empty parts (e.g. from leading /)
        parts_list = [p for p in parts_list if p]
        if len(parts_list) > 2:
            dir_display = "~/" + "/".join(parts_list[-2:])
        else:
            dir_display = "/".join(parts_list)
    else:
        dir_display = ""

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
    else:
        parts.append(f"{DIM}~{RESET}")
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
    "command": "python \"C:/Users/<username>/.claude/scripts/statusline.py\""
  }
}
```

> **Windows 用户**：使用正斜杠 `/` 或转义反斜杠 `\\`

### 步骤 3b（可选）：集成 Safety-net 状态

如果已安装 [safety-net](https://github.com/kenryu42/claude-code-safety-net) 插件，可通过管道连接显示保护状态：

```json
{
  "statusLine": {
    "type": "command",
    "command": "python \"C:/Users/<username>/.claude/scripts/statusline.py\" | npx -y cc-safety-net --statusline"
  },
  "enabledPlugins": {
    "safety-net@cc-marketplace": true
  }
}
```

Safety-net 状态指示器：

| 状态 | 显示 |
|------|------|
| 默认模式 | 🛡️ Safety Net ✅ |
| 严格模式 | 🛡️ Safety Net 🔒 |
| 偏执模式 | 🛡️ Safety Net 👁️ |
| Worktree 模式 | 🛡️ Safety Net 🌳 |

### 步骤 4：重启 Claude Code

退出当前会话（`Ctrl+C`），重新运行 `claude` 即可看到状态栏。

## 可用变量

通过 stdin 传入 JSON（**Claude Code 2.1.x 实测字段**），支持以下变量：

| 变量 | 说明 |
|------|------|
| `model.display_name` | 模型显示名称 |
| `model.id` | 模型内部名称 |
| `cwd` | 当前工作目录（**顶层字段，权威**；综合版脚本优先读它） |
| `workspace.current_dir` | 当前工作目录（同义，备选） |
| `context_window.remaining_percentage` | 上下文剩余百分比（综合版进度条用此） |
| `context_window.used_percentage` | 上下文使用百分比 |
| `context_window.context_window_size` | 上下文窗口大小 |
| `context_window.current_usage.*` | 当前 input/output/cache token 数 |
| `cost.total_cost_usd` | 累计成本（**计费权威源**） |
| `cost.total_duration_ms` | 累计时长毫秒（**计费权威源**） |
| `cost.total_lines_added` / `total_lines_removed` | 增 / 删行数 |
| `session_id` | 会话 ID（**顶层 `session_id`**，非 `session.id`） |
| `version` / `output_style.name` / `effort.level` / `thinking.enabled` | 其它元信息 |

> ⚠️ **计费/时长优先用 `cost.*`**（Claude Code 原生、类型稳定）。ECC metrics bridge（`/tmp/ecc-metrics-<session>.json`）的值可能为字符串（如 `first_timestamp`），直接做算术会 `TypeError` 崩溃——这是综合版脚本改用 stdin `cost` 的原因。

## 配色（综合版）

| 元素 | 颜色 | ANSI |
|------|------|------|
| 模型名 | 亮白 | `\033[97m` |
| 目录路径 | 亮蓝 | `\033[94m` |
| Git 分支 | 亮品红 | `\033[95m` |
| 计费/指标 | 亮青 | `\033[96m` |
| 上下文进度条 | 语义色（绿<50% / 黄<65% / 橙<80% / 红≥80%） | `\033[32m` / `\033[33m` / `\033[38;5;208m` / `\033[31m` |
| 当前任务 | 黄 | `\033[33m` |

> 4 个主元素用亮色（9X）互异，刻意避开进度条的语义绿/黄/橙/红，避免混淆。换色改 `_c(code, t)` 的码即可（如普通蓝 `_c(34, t)`）。

## 健壮性与常见崩溃

canonical 源已做以下防御，**自写脚本务必复刻**，否则状态栏会间歇性只剩「模型名 + 🛡️」：

1. **null 容器/叶子字段**：Claude Code 在会话启动 / 上下文压缩等过渡态会发 key 存在但值为 `null` 的字段。`data.get(k, default)` 对「存在但 null」返回 `None`（**不是 default**），后续 `.get()` / 算术即崩。→ 用 `data.get(k) or {}` / `or default`，叶子值用 `isinstance` 校验。
2. **bridge 值类型不可靠**：ECC metrics bridge 的 `first_timestamp` 等可能为字符串，`str / 1000` → `TypeError`，会让计费段崩溃、单 try/except 兜底清空整行。→ **计费/时长改取 stdin `cost`**（权威、类型稳定）；读 bridge 的值一律强转（`_num()`）。
3. **per-segment 隔离**：每段（目录/Git/进度条/任务/计费）独立 try/except，一段崩只丢该段，不全行清空。
4. **状态栏只在 idle 刷新**：agent 执行期间不刷新。调试「改脚本→等→读日志」抓不到，须让用户发条消息触发 idle 再读。
5. **顶层 try/except 兜底**：无论如何输出非空行（至少模型名），避免 Python 段零输出导致 safety-net 只剩 🛡️。

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
| Git 分支显示 `~` | 正常行为，表示当前目录不是 git 仓库 |
| Safety-net 不显示 | 确认 `enabledPlugins` 已配置，且 `npx` 可用 |
| 颜色显示异常 | 确认终端支持 ANSI 颜色 |

## 注意事项

- 每次工具执行后自动刷新
- command 类型执行 shell，复杂命令会影响响应速度
- 命令失败时不显示状态栏

## 参考

- 官方文档：https://code.claude.com/docs/zh-CN/statusline
