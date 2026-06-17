---
name: Claude Code 配置清单详细参考
description: cc switch诊断/状态栏源码与配置/Hooks机制/Rules安装/OpenCode桥接/容器配置/缓存优化/cc switch模板/MiMoCode链接
type: reference
updated: 2026-06-17
tags: claude-code, setup, statusline, hooks, rules, caching, opencode, mimo
source: https://example.com
---

# Claude Code 配置清单详细参考

> 入口文档：[[wiki/claude-code-setup.md|Claude Code 当前配置清单]]

---

## `cc switch` 后的常见失效与修复

### 三大根因

| 根因 | 症状 | 修复 |
|------|------|------|
| `CLAUDE_SETTINGS_PATH` 指向无效文件 | 状态栏显示 Safety Net ❌ | statusLine 命令中显式覆盖 `CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json` |
| `cc switch` 重置 settings.json | env 路径/hooks 回旧值 | 按 [[sources/claude-code/cc-switch-template.json]] 修复 |
| `SAFETY_NET_PLUGIN_ROOT` 版本过时 | 插件路径不存在 | `find ~/.claude/plugins/cache -name "cc-safety-net.js"` 找正确路径 |

### 诊断流程

| 步骤 | 检查项 | 命令 | 异常修复 |
|------|--------|------|----------|
| 1 | `CLAUDE_SETTINGS_PATH` 有效 | `cat $CLAUDE_SETTINGS_PATH` | 加 `CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json` |
| 2 | `SAFETY_NET_PLUGIN_ROOT` 存在 | `ls $SAFETY_NET_PLUGIN_ROOT/dist/bin/cc-safety-net.js` | find 正确路径后更新 env |
| 3 | `CLAUDE_PLUGIN_ROOT` 无多余前缀 | `ls $CLAUDE_PLUGIN_ROOT/hooks/hooks.json` | 更新 env 路径 |
| 4 | settings.json 不含 ECC hooks | `grep -c CLAUDE_PLUGIN_ROOT ~/.claude/settings.json`（应为 0） | 删除 `${CLAUDE_PLUGIN_ROOT}` 引用 |
| 5 | Python 命令可用 | `py --version`（Windows） | Windows 必须用 `py` |

### 验证修复

```bash
echo '{"model":{"display_name":"test"},"workspace":{"current_dir":"."},"context_window":{"remaining_percentage":80},"session_id":"test"}' | py ~/.claude/scripts/statusline.py | CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json node "${SAFETY_NET_PLUGIN_ROOT}/dist/bin/cc-safety-net.js" --statusline
```

---

## 状态栏（StatusLine）详细配置

### 配置（综合版）

```json
"statusLine": {
  "type": "command",
  "command": "py ~/.claude/scripts/statusline.py | CLAUDE_SETTINGS_PATH=$HOME/.claude/settings.json node \"${SAFETY_NET_PLUGIN_ROOT}/dist/bin/cc-safety-net.js\" --statusline"
}
```

### 显示效果

```
glm-5.2 | app/project | feat/branch | ██░░░░░░░░ 24% | $6.96 0t 0f 2h41m | 🛡️ CC Safety Net ✅
 模型名(白)  目录(蓝)     Git分支(品红)   进度条(语义色)    计费(青)               安全网
```

### 配色

| 元素 | 颜色 | ANSI |
|------|------|------|
| 模型名 | 亮白 | `\033[97m` |
| 目录 | 亮蓝 | `\033[94m` |
| Git 分支 | 亮品红 | `\033[95m` |
| 计费/指标 | 亮青 | `\033[96m` |
| 进度条 | 语义（绿<50%/黄<65%/橙<80%/红≥80%） | 32/33/38;5;208/31 |

### 综合版特性

| 元素 | 来源 | 说明 |
|------|------|------|
| 模型名 | stdin `model.display_name` | 自动剥离 ANSI 残片 |
| 目录路径 | stdin `cwd` / `workspace.current_dir` | 压缩为最后 2 级 |
| Git 分支 | `git rev-parse --abbrev-ref HEAD` | 自动向上搜索 .git |
| 上下文进度条 | stdin `context_window.remaining_percentage` | 4 色阈值 |
| 当前任务 | `~/.claude/todos/*.json` | `in_progress` 的 `activeForm` |
| 成本/时长 | stdin `cost` 对象（权威） | `total_cost_usd` / `total_duration_ms` |
| 安全网 | `cc-safety-net --statusline` | 管道追加 |

### 脚本位置

- **主脚本**：`~/.claude/scripts/statusline.py`
- **完整源码**：[[sources/claude-code/statusline.py]]

### 关键实现细节

- `read_bridge()` 用 `os.environ.get("TEMP")` 而非 `tempfile.gettempdir()`（Windows 路径不一致）
- `build_context_bar()` 的 `used` 值通过 auto-compact buffer 公式计算
- 每段独立 try/except（per-segment 隔离）+ null 防御

### 已知问题

1. Windows `python` 命令返回 exit code 49，必须用 `py`
2. stdin 叶子字段 null 导致间歇 render error，需做 null 防御
3. ECC bridge `first_timestamp` 为字符串致计费段崩溃，改取 stdin `cost`

### 备选状态栏

```json
"statusLine": {
  "type": "command",
  "command": "py -3 \"C:/Users/<user>/.claude/scripts/statusline-simple.py\" | npx -y cc-safety-net --statusline"
}
```

---

## Hooks 系统详细配置

### settings.json 中的 Hooks（2 个）

| 匹配器 | 脚本 | 功能 |
|--------|------|------|
| `Bash` | `cc-safety-net.js --claude-code` | Safety Net 危险命令拦截 |
| `Bash` | `npx block-no-verify@1.1.2` | 阻止未验证的 git 操作 |

### ECC 插件注入的 Hooks

ECC v2.0.0 自动注册 28 个 hooks（PreToolUse/PostToolUse/Stop 等），**无需在 settings.json 维护**。

### ECC Hooks 开关机制

| 变量 | 作用 | 默认值 |
|------|------|--------|
| `ECC_HOOK_PROFILE` | 档位：`minimal`/`standard`/`strict` | `standard` |
| `ECC_DISABLED_HOOKS` | 细粒度黑名单 | 无 |

#### 当前禁用的 hooks

| Hook ID | 禁用理由 |
|---------|----------|
| `stop:desktop-notify` | WSL/容器无桌面 daemon |
| `pre:edit-write:gateguard-fact-force` | 每个新文件首次编辑都被拦 |
| `pre:observe:continuous-learning` | 每次工具调用都跑，开销最重 |
| `post:observe:continuous-learning` | 同上 |
| `stop:evaluate-session` | 配合观察器关闭 |

#### 其他开关

| 变量 | 作用 |
|------|------|
| `ECC_GATEGUARD=off` | 单独关 GateGuard |
| `ECC_GOVERNANCE_CAPTURE=1` | 反向：治理捕获默认关 |

---

## Rules — 语言编码规范详细配置

### 目录结构

```
rules/
├── common/          # 通用准则（必须安装）
├── typescript/ / angular/ / python/ / golang/ / web/
├── swift/ / php/ / ruby/ / arkts/
```

### 安装方式

```bash
# 一键安装（推荐）
./install.sh typescript python

# 手动安装
mkdir -p ~/.claude/rules/ecc
cp -r rules/common ~/.claude/rules/ecc/
cp -r rules/typescript ~/.claude/rules/ecc/
```

> **重要**：禁止 `/*` 扁平化复制（会覆盖同名文件）。

ECC 源目录提供 18 个规则集，每种语言含 5 个维度：coding-style、security、testing、patterns、hooks。

---

## OpenCode 桥接详细配置

- 配置位置：`~/.opencode/config.json`
- 共享：环境变量、插件路径
- Hooks 需单独配置，不自动继承
- 详见 [[wiki/opencode-bridge.md|OpenCode 桥接文档]]

---

## 容器 / WSL 一步到位配置

```bash
# 1. 状态栏脚本
cp sources/claude-code/statusline.py ~/.claude/scripts/statusline.py && chmod +x ~/.claude/scripts/statusline.py

# 2. settings.json（classifier 禁 agent 直写 → stage + 用户 !cp）
cp sources/claude-code/container-settings.json /tmp/cc-target-settings.json
!cp /tmp/cc-target-settings.json ~/.claude/settings.json

# 3. 重载
/reload-plugins
```

关键差异：容器用 `python3`（非 `py`），settings.json 写入需 stage 到 `/tmp` 后用户手动 cp。

---

## 缓存优化（Prompt Caching）

### 根因

Claude Code 的 `x-anthropic-billing-header` 中 `cch` 哈希值每次对话不同，导致第三方 API 缓存失效。

### 解决方案

```json
"CLAUDE_CODE_ATTRIBUTION_HEADER": "0"
```

### 社区方案

```bash
npm install -g claude-code-cache-fix@1.7.1
```

---

## cc switch 通用配置模板

完整模板：[[sources/claude-code/cc-switch-template.json]]

| 字段 | 规则 |
|------|------|
| `env.CLAUDE_PLUGIN_ROOT` | 指向实际 ECC 版本目录 |
| `env.SAFETY_NET_PLUGIN_ROOT` | 指向实际 Safety Net 版本目录 |
| `hooks` | 只保留 Safety Net + block-no-verify |
| `statusLine.command` | 必须用 `py`，必须加 `CLAUDE_SETTINGS_PATH` 覆盖 |

---

## MiMoCode Obsidian 链接方法

| 项目 | Claude Code | MiMoCode |
|------|-------------|----------|
| 记忆位置 | `.claude/projects/<slug>/memory/` | `~/.local/share/mimocode/memory/projects/<pid>/` |
| 项目标识 | 路径派生的 slug | UUID |

```bash
# 创建链接（Windows: Junction）
powershell -Command "New-Item -ItemType Junction -Path '$PROJECT_MEMORY_DIR' -Target 'E:\Files\Obsidian'"
```

创建前务必备份原有记忆目录。
