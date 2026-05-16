---
name: Safety Net Hook 参数修复 + ECC 规则全量安装
description: 排查 cc-safety-net --hook 报错 "Unknown option: --hook" 的调试过程，以及 ECC 规则的全局+项目安装实践
type: debugging-story
updated: 2026-05-13
tags: [claude-code, cc-safety-net, hook, ecc, rules, debugging]
---

# Safety Net Hook 参数修复 + ECC 规则全量安装

## 问题现象

每次 Claude Code 执行 Bash 命令时，PreToolUse hook 报错：

```
PreToolUse:Bash hook error — Failed with non-blocking status code: Unknown option: --hook
```

命令照常执行（non-blocking），但每次都打印错误信息。

## 排查过程

### 1. 定位 hook 配置

读取 `~/.claude/settings.json`，找到 PreToolUse hook：

```json
{
  "matcher": "Bash",
  "hooks": [
    {
      "type": "command",
      "command": "npx cc-safety-net --hook \"$TOOL_INPUT\""
    }
  ]
}
```

### 2. 检查 cc-safety-net CLI

运行 `npx cc-safety-net --help`，发现 v0.9.0 的命令列表中根本没有 `--hook` 选项。正确的 Claude Code hook 模式参数是：

```
--claude-code    (或 -cc)    Run as Claude Code PreToolUse hook (reads JSON from stdin)
```

### 3. 根因

`--hook` 是旧版参数或错误配置。cc-safety-net v0.9.0 中：
- 没有 `--hook` 选项
- Claude Code hook 模式用 `--claude-code`
- 输入通过 stdin 传递，不需要 `$TOOL_INPUT` 环境变量

### 4. 修复

```json
"command": "npx cc-safety-net --claude-code"
```

## ECC 规则安装

### 发现

ECC 插件的 `hooks/hooks.json` 会被插件系统自动加载（31 个 hooks），但 `rules/` 不会自动分发，需要手动复制。

ECC 源目录有 18 个规则集：common + 17 种语言（angular, arkts, cpp, csharp, dart, fsharp, golang, java, kotlin, perl, php, python, ruby, rust, swift, typescript, web）。

### 安装策略

**全局（~/.claude/rules/ecc/）**：只安装 `common` 规则集（语言无关的通用规范）。Claude Code 会加载 `~/.claude/rules/` 下所有 `.md` 到每个会话 context，无按项目过滤，因此全局只保留通用的 common 以避免浪费 context。

**项目级（project/.claude/rules/ecc/）**：只安装项目实际使用的技术栈规则。当前项目是 Python，安装 common + python。

### 安装命令

```bash
# 全局安装所有规则
cp -r ~/.claude/plugins/marketplaces/ecc/rules/* ~/.claude/rules/ecc/

# 项目级只安装必要的
mkdir -p project/.claude/rules/ecc
cp -r ~/.claude/plugins/marketplaces/ecc/rules/common project/.claude/rules/ecc/
cp -r ~/.claude/plugins/marketplaces/ecc/rules/python project/.claude/rules/ecc/
```

### 验证

```bash
# 验证全局安装完整
diff <(find ~/.claude/plugins/marketplaces/ecc/rules/ -name "*.md" -not -path "*/README.md" -not -path "*/zh/*" | sed 's|.*/rules/||' | sort) <(find ~/.claude/rules/ecc/ -name "*.md" | sed 's|.*/ecc/||' | sort)
# 无输出 = 完全一致
```
