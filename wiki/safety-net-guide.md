---
name: Safety Net 安全网
description: Claude Code 安全插件，拦截危险命令 — 配置安全拦截规则时使用
type: summary
sources:
  - "[[sources/claude-code-system-prompts/README]]"
updated: 2026-04-05
tags: [claude-code, safety]
---

# Safety Net 使用手册

> Claude Code 插件 - 在危险命令执行前拦截它们

**相关文档**：[[statusline-install|状态栏配置]] | [[Everything-Claude-Code|ECC 技能集]]

## 简介

Safety Net 是一个 Claude Code 安全插件，用于防止 AI 代理执行破坏性的 git 和文件系统命令。

**核心价值**：`CLAUDE.md` 中的"软"规则无法替代"硬"技术约束。本插件通过 Hook 机制在命令执行前进行语义分析，拦截危险操作。

## 安装

```bash
# 1. 添加市场
/plugin marketplace add kenryu42/cc-marketplace

# 2. 安装插件
/plugin install safety-net@cc-marketplace

# 3. 重载插件生效
/reload-plugins
```

### 配置状态栏

状态栏会在 Claude Code 界面底部显示 Safety Net 的状态信息。

**方式一：通过技能配置**

```bash
/safety-net:set-statusline
```

按提示选择包管理器（npx 或 bunx），然后选择是否与现有状态栏命令合并。

**方式二：手动配置**

编辑 `~/.claude/settings.json`：

```json
{
  "statusLine": {
    "type": "command",
    "command": "npx -y cc-safety-net --statusline"
  }
}
```

**合并现有状态栏**

如果已有其他状态栏命令，可以用管道合并：

```json
{
  "statusLine": {
    "type": "command",
    "command": "python ~/.claude/scripts/statusline.py | npx -y cc-safety-net --statusline"
  }
}
```

> **注意**：配置变更立即生效，无需重启。

### 配置 Hook（Windows 必需）

> **Windows 特有问题**：插件系统不会自动将 hooks 合并到 `settings.json`，需要手动配置。

**步骤 1：添加环境变量**

在 `~/.claude/settings.json` 的 `env` 部分添加：

```json
{
  "env": {
    "SAFETY_NET_PLUGIN_ROOT": "C:/Users/<你的用户名>/.claude/plugins/cache/cc-marketplace/safety-net/<版本号>"
  }
}
```

查找实际路径：

```bash
# 查看插件缓存目录
ls ~/.claude/plugins/cache/cc-marketplace/safety-net/
```

**步骤 2：添加 PreToolUse Hook**

在 `hooks.PreToolUse` 数组的**开头**添加（确保优先执行）：

```json
{
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
        ],
        "description": "Safety Net - Block destructive git and filesystem commands"
      },
      // ... 其他 hooks
    ]
  }
}
```

**步骤 3：重新加载配置**

```bash
/reload-plugins
```

或重启 Claude Code。

### 自动更新

```
/plugin → Marketplaces → cc-marketplace → 启用 auto-update
```

## 可用技能

| 命令 | 用途 |
|------|------|
| `/safety-net:set-custom-rules` | 配置自定义阻止规则 |
| `/safety-net:set-statusline` | 设置状态栏显示 |
| `/safety-net:verify-custom-rules` | 验证规则配置 |

## 被拦截的命令

### Git 命令

| 命令 | 原因 |
|------|------|
| `git checkout -- files` | 永久丢弃未提交的更改 |
| `git restore files` | 丢弃未提交的更改 |
| `git reset --hard` | 销毁所有未提交的更改 |
| `git clean -f` | 永久删除未跟踪的文件 |
| `git push --force / -f` | 破坏远程历史 |
| `git branch -D` | 强制删除分支（无合并检查） |
| `git stash drop/clear` | 永久删除暂存的更改 |
| `git worktree remove --force` | 强制删除工作树 |

### 文件系统命令

| 命令 | 原因 |
|------|------|
| `rm -rf` (cwd 外的路径) | 递归删除当前目录外的文件 |
| `rm -rf /` 或 `~` | 极度危险 |
| `find ... -delete` | 永久删除匹配的文件 |
| `xargs rm -rf` | 动态输入导致目标不可预测 |
| `parallel rm -rf` | 同上 |

## 允许的安全命令

| 命令 | 原因 |
|------|------|
| `git checkout -b branch` | 创建新分支 |
| `git restore --staged` | 仅取消暂存 |
| `git branch -d` | 安全删除（有合并检查） |
| `git clean -n` | 仅预览 |
| `git push --force-with-lease` | 安全的强制推送 |
| `rm -rf /tmp/...` | 临时目录 |
| `rm -rf ./...` (cwd 内) | 限制在当前目录 |

## 高级配置

### 环境变量

```bash
# 严格模式：无法解析的命令将被阻止
export SAFETY_NET_STRICT=1

# 偏执模式：启用所有严格检查
export SAFETY_NET_PARANOID=1

# 单独启用偏执检查
export SAFETY_NET_PARANOID_RM=1           # 阻止 cwd 内的 rm -rf
export SAFETY_NET_PARANOID_INTERPRETERS=1 # 阻止解释器单行命令
```

### 偏执模式行为

- **rm**: 阻止非临时目录的 `rm -rf`，即使在当前目录内
- **解释器**: 阻止 `python -c`、`node -e`、`ruby -e`、`perl -e` 等单行命令

### 自定义规则

配置文件位置：

| 范围 | 路径 | 优先级 |
|------|------|--------|
| 用户级 | `~/.cc-safety-net/config.json` | 低 |
| 项目级 | `.safety-net.json` | 高（覆盖用户级） |

示例配置：

```json
{
  "$schema": "https://raw.githubusercontent.com/kenryu42/claude-code-safety-net/main/assets/cc-safety-net.schema.json",
  "version": 1,
  "rules": [
    {
      "name": "block-git-add-all",
      "command": "git",
      "subcommand": "add",
      "block_args": ["-A", "--all", "."],
      "reason": "Use 'git add <specific-files>' instead."
    },
    {
      "name": "block-npm-global",
      "command": "npm",
      "subcommand": "install",
      "block_args": ["-g", "--global"],
      "reason": "Use npx or local install."
    }
  ]
}
```

## CLI 命令

```bash
# 诊断检查
npx -y cc-safety-net doctor

# 解释命令分析过程
npx -y cc-safety-net explain "git push --force"

# 验证配置
npx -y cc-safety-net --verify-config

# 查看自定义规则文档
npx -y cc-safety-net --custom-rules-doc

# 状态栏显示
npx -y cc-safety-net --statusline
```

## 特性

### Shell 包装检测

递归分析包装在 shell 中的命令：

```bash
bash -c 'git reset --hard'    # 被拦截
sh -lc 'rm -rf /'             # 被拦截
```

### 解释器单行命令检测

检测隐藏在解释器中的破坏性命令：

```bash
python -c 'import os; os.system("rm -rf /")'  # 被拦截
```

### 敏感信息脱敏

拦截消息自动脱敏敏感数据（令牌、密码、API 密钥）。

### 审计日志

所有被拦截的命令记录到 `~/.cc-safety-net/logs/<session_id>.jsonl`：

```json
{"ts": "2025-01-15T10:30:00Z", "command": "git reset --hard", "reason": "...", "cwd": "/path/to/project"}
```

## 被拦截时的处理

当命令被拦截时，你会看到：

```
BLOCKED by Safety Net

Reason: git checkout -- discards uncommitted changes permanently. Use 'git stash' first.

Command: git checkout -- src/main.py

If this operation is truly needed, ask the user for explicit permission and have them run the command manually.
```

**解决方案**：如果确实需要执行该操作，请用户手动运行命令。

## 测试

### 使用 explain 命令分析

```bash
# 分析危险命令（不实际执行）
npx -y cc-safety-net explain "git reset --hard HEAD"
npx -y cc-safety-net explain "rm -rf /"
npx -y cc-safety-net explain "git checkout -- src/main.py"
```

### 实际拦截测试

```bash
# 测试拦截 - 以下命令会被 Safety Net 阻止
git checkout -- README.md  # BLOCKED: 丢弃未提交的更改
git reset --hard HEAD      # BLOCKED: 销毁未提交的更改
rm -rf /                   # BLOCKED: 根目录删除

# 测试允许 - 以下命令会被放行
git checkout -b test-branch      # ALLOWED: 创建新分支
git branch -d merged-branch      # ALLOWED: 安全删除已合并分支
rm -rf ./node_modules            # ALLOWED: 当前目录内（非 home 目录）
```

### 预期输出

当命令被拦截时，你会看到：

```
BLOCKED by Safety Net

Reason: git checkout -- discards uncommitted changes permanently. Use 'git stash' first.

Command: git checkout -- test.txt

If this operation is truly needed, ask the user for explicit permission and have them run the command manually.
```

## 相关链接

- 仓库：https://github.com/kenryu42/claude-code-safety-net
- 许可证：MIT
