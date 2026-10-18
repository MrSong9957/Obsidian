---
name: Claude Code Statusline 部署与故障恢复
description: 状态栏运维手册——在新 Claude Code 环境恢复状态栏或排障：canonical 源为准的部署结构、Linux 容器实测流程、边界矩阵验证与快速排障（2026-10-18 起不集成 Safety-net）
type: guide
updated: 2026-10-18
tags: [claude-code, statusline, ops]
source: self
---

# Claude Code Statusline 部署与故障恢复

**唯一权威恢复手册** — 在新 Claude Code 环境中恢复状态栏，或状态栏异常时排障。

> **2026-10-18 三项重大变更**：
> 1. **不再集成 cc-safety-net**（决策依据见下）；状态栏依赖仅 `python3 + git`，无 node/npm。
> 2. **代码以 canonical 源 `sources/claude-code/statusline.py` 为唯一事实源**，本手册不再内嵌完整代码副本——两处副本漂移曾造成两个真实缺陷（见「漂移教训」）。
> 3. Linux 容器（WSL2 devcontainer）部署全流程实测通过。

## 最终效果

（Linux 容器 /workspace 实测，2026-10-18）

```
glm-5.3[1M] | /workspace | main | ███████░░░ 72% | $0.01 0t 0f 5m
```

| 段 | 内容 |
|---|---|
| 模型 | 当前模型 display_name（剥 ANSI 残片、折叠换行） |
| 当前目录 | 缩短为最后 2 级 |
| Git branch | 非 Git 仓库隐藏该段（按进程 cwd / CLAUDE_WORKING_DIR 探测） |
| context bar | 上下文使用率，4 色进度条（绿<50/黄<65/橙<80/红≥80，含 16.5% auto-compact 缓冲） |
| 当前任务 | todos 中 in_progress 项的 activeForm（存在才显示） |
| metrics | $成本 / 工具数 / 文件数 / 时长（成本与时长取 stdin `cost.*` 权威源；工具/文件数读 ECC bridge，缺省 0t/0f） |

## 为什么移除 Safety-net（2026-10-18 调研结论）

对照 Claude Code 2.1.302 官方文档（sandboxing / permissions / hooks / permission-modes / checkpointing）：

- **原生已覆盖其主职责**：`permissions.deny` 全模式生效（含 bypassPermissions）、拆复合命令并穿透 `$(...)`/子 shell；auto 模式分类器（v2.1.283 起为交互会话默认起始模式）内置 force push、`git reset --hard`、`rm -rf "$VAR"` 等阻断清单；critical-path rm 硬编码保险丝（任何 allow 规则与 hook allow 均不可豁免）；可选 OS 级沙箱（WSL2 需补装 bubblewrap+socat）。
- **原生仍有缺口**（rewind 不覆盖 Bash 改动、deny 是文本匹配可被等价写法绕过），但 cc-safety-net 自身 fail-open 且非沙箱，仅部分补上。
- **部署环境无 node/npx**：集成需先装 Node 18+；且旧方案管道 `| npx cc-safety-net --statusline` 任一环失败会导致**整条状态栏消失**。
- 需要硬门禁时走原生路线：少量 `permissions.deny` 规则＋自写 PreToolUse hook（`onFailure: "block"`）＋可选沙箱。

## 文件结构

```
~/.claude/
├─ settings.json          # statusLine 配置（合并进现有文件，勿整文件覆盖）
└─ scripts/
   └─ statusline.py       # 生产脚本（canonical 的逐字节副本）
```

canonical 源：vault 内 `sources/claude-code/statusline.py`。**修改只改 canonical，再 cp 到各机器**；勿就地改副本（会与源头漂移）。

## 从零恢复（Linux 容器实测流程，2026-10-18）

### Step 1：确认依赖

```bash
python3 --version   # 3.x 即可；脚本纯标准库，零 pip 依赖
git --version       # 仅分支段需要；缺失/超时(3s)只隐藏该段，不影响其余
```

不需要 node/npx。

### Step 2：部署脚本

```bash
mkdir -p ~/.claude/scripts
cp <vault>/sources/claude-code/statusline.py ~/.claude/scripts/statusline.py
chmod +x ~/.claude/scripts/statusline.py
python3 -m py_compile ~/.claude/scripts/statusline.py   # 预期：无输出，exit 0
```

> canonical **无 shebang**——必须以 `python3 <绝对路径>` 调用，不能直接执行。

### Step 3：配置 settings.json（先备份，再合并）

```bash
cp ~/.claude/settings.json ~/.claude/backups/settings.json.bak-$(date +%Y%m%d-%H%M%S)
```

```json
{
  "statusLine": {
    "type": "command",
    "command": "python3 /home/<user>/.claude/scripts/statusline.py"
  }
}
```

用绝对路径；保留 settings.json 既有字段（env/theme/model 等）。Windows 用 `python`＋正斜杠路径，命令前加 `PYTHONIOENCODING=utf-8 `。

### Step 4：边界矩阵验证

```bash
S=~/.claude/scripts/statusline.py
printf '%s' '{"model":{"display_name":"t"},"cwd":"/tmp","context_window":{"remaining_percentage":50}}' | python3 "$S"
```

预期：单行、rc=0。再补三类边界——`{"model":null}`（null 容器）、`{}`（空对象）、`[]`（顶层数组）——均须 rc=0、无 Traceback、保持单行。完整矩阵见文末。

### Step 5：重启 Claude Code

statusLine 在启动时加载，**必须完全重启**。

### Step 6：真实运行验收

底部状态栏出现五段（无 todos 任务时四段）即为通过。注意状态栏**只在 idle 刷新**，调试改脚本后须发条消息触发。

## 漂移教训（2026-10-18）

本手册曾内嵌 statusline.py 与 test_statusline.py 完整副本，与 canonical 源漂移造成两个真实缺陷（均已在 canonical 修复并实测）：

1. **BUG 5 回归**：wiki 副本 2026-08 已修「字段含换行 → 状态栏多行」，canonical 缺此修复（实测模型名含 `\n` 输出两行）。修复：canonical 新增 `one_line()` 折叠 `\r\n`，应用于 model/dir/task 三个动态段；修复前后对照实测（多行 → 单行）。
2. **todos 数组格式失效**：Claude Code todos 文件是 JSON **数组**，canonical 按单对象 `.get()` 读取 → 任务段永不显示。修复：兼容数组/裸对象、多文件按 mtime 最新优先、坏 JSON 文件跳过不崩。

结论：**单一事实源 = canonical 文件**；本手册只保留部署/排障知识，不再留代码副本。旧内嵌代码与 Windows 版 test_statusline.py 见 git 历史（2026-10-18 之前版本）。

## 故障速查表

| 症状 | 优先检查 |
|---|---|
| 整个 statusline 消失 | 把 settings.json 里的 command 原样手跑：`printf '%s' '{...}' \| <command>`，看 rc 与 stderr |
| 状态栏不刷新 | 只在 idle 刷新——发条消息触发后再观察 |
| 修改脚本/配置后仍旧行为 | Claude Code 是否完全重启 |
| Git 分支段不显示 | 脚本按**进程 cwd**（或 `CLAUDE_WORKING_DIR`）找 .git，非 stdin 的 cwd；确认启动目录是仓库 |
| 非 Git 没有占位符 | canonical 设计：无分支即隐藏该段（旧 wiki 版显示 `~`，行为已变更） |
| context 显示 0% | stdin 无合法 `remaining_percentage` 时按 100% 剩余处理（显示 0%）——检查字段名与类型 |
| 状态栏出现多行 | 副本须 ≥2026-10-18 canonical（`one_line()`）；`diff` 副本与 canonical |
| 中文乱码 | Linux：stdout 已强制 UTF-8；Windows：命令前加 `PYTHONIOENCODING=utf-8 ` |
| 改了 canonical 没生效 | 安装副本是独立文件，须重新 `cp` 同步 |

## 已验证边界矩阵（2026-10-18，Linux 容器实测）

| 类别 | 边界 | 结果 |
|---|---|---|
| JSON | null 容器字段（model/workspace/context_window/cost/session_id） | ✅ 逐段降级，不崩 |
| JSON | 空对象 `{}` | ✅ unknown + 0% 条 + $0.00 |
| JSON | 顶层数组/字符串 | ✅ fallback 输出「statusline」，单行、rc=0 |
| JSON | remaining_percentage 为字符串 | ✅ 按 100% 剩余，不崩 |
| 终端 | 模型名/路径/activeForm 含 `\n` `\r` | ✅ 折叠为空格，保持单行 |
| todos | 数组格式（真实格式） | ✅ 显示 in_progress 的 activeForm |
| todos | 多文件并存 | ✅ mtime 最新优先 |
| todos | 坏 JSON 文件 | ✅ 跳过该文件不崩 |
| todos | 无 in_progress | ✅ 隐藏任务段 |
| Git | 真实仓库（/workspace） | ✅ 显示 main |
| 计费 | cost.total_cost_usd / total_duration_ms | ✅ `$x.xx`＋时长格式化 |

Windows 时代历史边界（逻辑等价或已随 canonical 重写不再适用）：中文/日文/Emoji 路径、HOME 前缀碰撞（canonical 不再做 `~` 替换，问题不复存在）、detached HEAD、worktree、git 超时保护、bool remaining（现统一按非法值处理）。

## 历史：历轮修复要点（保留备查）

- 中文乱码 → PYTHONIOENCODING=utf-8 + stdout reconfigure
- HOME 前缀碰撞（`C:/Users/<user>` vs `<user>0`）→ 路径边界检查
- bool 被当 int（`remaining=true` → 100%）→ isinstance 排除
- 顶层非对象 JSON 崩溃 → isinstance 检查
- 换行破坏单行 → sanitize（2026-10-18 canonical 以 `one_line()` 重实现）
- Safety-net 版本漂移（2.0.3→2.0.6）→ 随 Safety-net 移除（2026-10-18）不再相关

## 参考

- 官方文档：https://code.claude.com/docs/en/statusline
- 安装指南：`sources/claude-code/claude-code-statusline-setup.md`
- Safety-net 移除调研（2026-10-18）：原生 permissions/hooks/sandbox/permission-modes/checkpointing 对照结论
