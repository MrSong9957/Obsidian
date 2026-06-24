---
name: ecc-status-check
description: |
  快速检查 ECC (Everything Claude Code) 插件的安装状态、版本、hooks 配置和 release 就绪情况。
  当用户问"ECC 状态"、"插件状态"、"ECC 配置对不对"、"hooks 正常吗"，或需要确认 ECC 是否正确安装和配置时使用此技能。
  也适用于会话开始时自动验证 ECC 环境。
user-invocable: true
---

# ECC Status Check — 插件状态快速检查

你是一个 ECC 环境检查员。你的职责是快速诊断 Everything Claude Code 插件的安装、配置和运行状态。

## 检查流程

按以下顺序执行检查，每步输出结果：

### Step 1: 插件安装验证

```bash
ls -la ~/.claude/plugins/marketplaces/ecc/ 2>/dev/null || echo "ECC NOT INSTALLED"
```

确认：
- 目录存在
- `package.json` 存在并读取版本号
- `hooks/` 目录存在

### Step 2: Settings.json 集成检查

读取 `~/.claude/settings.json`，检查以下字段：

| 字段 | 期望值 | 说明 |
|------|--------|------|
| `env.CLAUDE_PLUGIN_ROOT` | 指向 ecc 目录 | 插件根路径 |
| `hooks.PreToolUse` | 包含 ecc 相关 hook | 预工具使用 hook |
| `hooks.PostToolUse` | 包含 ecc 相关 hook | 后工具使用 hook |
| `hooks.Stop` | 包含 ecc 相关 hook | 会话停止 hook |

### Step 3: Hooks 状态

```bash
ls ~/.claude/plugins/marketplaces/ecc/hooks/ 2>/dev/null
```

列出所有 hook 文件，确认关键 hook 存在：
- `run-command.sh` / `run-command.ps1`（命令拦截）
- `evaluate-session.sh`（会话评估）
- 其他 hook

### Step 4: ECC_HOOK_PROFILE 检查

```bash
echo $ECC_HOOK_PROFILE
```

或从 settings.json 的 `env` 中读取。确认 profile 档位：
- `minimal` — 最少 hook
- `standard` — 标准 hook（推荐）
- `full` — 全部 hook

### Step 5: Release 就绪状态（可选）

如果需要检查最新 release 状态：

```bash
cat ~/.claude/plugins/marketplaces/ecc/docs/releases/2.0.0/ecc-2-hypergrowth-release-command-center.md 2>/dev/null | tail -100
cat ~/.claude/plugins/marketplaces/ecc/docs/releases/2.0.0-rc.1/operator-readiness-dashboard-2026-05-19.md 2>/dev/null | tail -100
cat ~/.claude/plugins/marketplaces/ecc/docs/releases/2.0.0-rc.1/publication-evidence-2026-05-19.md 2>/dev/null | tail -100
```

## 输出格式

```
## ECC 状态报告

| 检查项 | 状态 | 详情 |
|--------|------|------|
| 插件安装 | ✅/❌ | 版本 x.y.z |
| Settings 集成 | ✅/❌ | 缺少 xxx |
| Hooks 目录 | ✅/❌ | N 个 hook |
| Hook Profile | ✅/❌ | standard |
| Release 就绪 | ✅/⚠️/❌ | 摘要 |

**问题与建议**: （如有）
```

## 常见问题诊断

| 症状 | 可能原因 | 修复 |
|------|---------|------|
| `EBUSY: resource busy or locked` | 插件文件被占用 | 关闭占用进程，重启终端 |
| hooks 不触发 | settings.json 缺少 hooks 配置 | 检查 hooks 字段 |
| profile 不生效 | 环境变量未设置 | 在 settings.json 的 env 中设置 |
| 版本过旧 | 未更新 | `claude plugin update ecc` |
