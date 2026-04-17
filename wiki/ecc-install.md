---
name: ECC 安装指南
description: everything-claude-code 安装步骤 — 首次安装或更新 ECC 时参考
type: summary
sources:
  - "[[sources/claude-code-system-prompts/README]]"
updated: 2026-04-05
tags: [claude-code, installation]
status: deprecated
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# ECC (everything-claude-code) 安装指南

> 内容已迁移至 [[wiki/claude-code-p2.md|claude-code-p2]]。

> 返回 [[Everything-Claude-Code.md|ECC 技能集总览]]

## 环境要求
- Claude Code 最新版本
- Node.js 18+ (用于执行 hook 脚本)

---

## 安装步骤

### 步骤 1：添加插件市场

```bash
/plugin marketplace add affaan-m/everything-claude-code
```

### 步骤 2：安装插件

```bash
/plugin install everything-claude-code@everything-claude-code
```

### 步骤 3：配置 settings.json

将以下配置添加到 `~/.claude/settings.json`：

```json
{
  "extraKnownMarketplaces": {
    "everything-claude-code": {
      "source": {
        "source": "directory",
        "path": "C:/Users/<你的用户名>/.claude/plugins/marketplaces/everything-claude-code"
      }
    }
  },
  "enabledPlugins": {
    "everything-claude-code@everything-claude-code": true
  }
}
```

> **实际经验**：官方文档用 `"source": "github"`，但如果 GitHub 无法访问会下载失败。建议用 `"source": "directory"` 指向本地已下载的插件目录。

#### 3.1 插件目录结构说明

Claude Code 插件有两个关键目录：

| 目录 | 作用 |
|------|------|
| `marketplaces/` | 插件源码存放（source） |
| `cache/` | 运行时加载（runtime） |

**如果 hooks 不生效**，可能需要手动将插件复制到 cache 目录：
```bash
# 从 marketplaces 复制到 cache（保留隐藏目录）
cp -r ~/.claude/plugins/marketplaces/everything-claude-code ~/.claude/plugins/cache/
```

### 步骤 4：手动安装 Rules

> ⚠️ Claude Code 插件系统不支持 rules 通过插件分发，需要手动安装。

#### 4.1 准备工作：备份现有规则

如果你已有自定义规则，先备份：

```bash
# 备份现有规则（重要！）
cp -r ~/.claude/rules ~/.claude/rules.backup
```

#### 4.2 安装 ECC 规则

```bash
# 克隆仓库
git clone https://github.com/affaan-m/everything-claude-code.git

# 查看 ECC 提供的规则
ls everything-claude-code/rules/common/
```

#### 4.3 合并规则（重要）

**⚠️ 不要直接覆盖！** 现有规则可能包含你的个人偏好和工作流。

**推荐做法**：
```bash
# 方法1：只复制不存在的文件（保留现有规则）
for f in everything-claude-code/rules/common/*; do
  if [ ! -f "~/.claude/rules/common/$(basename $f)" ]; then
    cp "$f" ~/.claude/rules/common/
  fi
done

# 方法2：手动选择需要的规则文件复制
# 只复制你确实需要的，避免覆盖已有配置
```

**实际经验**：
- 用户通常已有 `agents.md`（Agent 编排）、`coding-style.md`（代码风格）、`performance.md`（性能优化）等自定义规则
- ECC 的规则可能与你的现有规则冲突或重复
- 建议先查看 ECC 规则内容，再决定是否需要

#### 4.4 项目级规则（可选）

```bash
# 项目级规则（仅当前项目生效）
mkdir -p .claude/rules
# 只复制你需要的规则文件到这个项目
cp everything-claude-code/rules/common/<需要的文件> .claude/rules/
```

### 步骤 5：配置 Hooks（重要）

> ⚠️ **警告**：不要直接覆盖用户的现有 hooks！必须保留用户已有的配置。

插件的 hooks 定义在仓库的 `hooks/hooks.json`，需要**追加**到现有 hooks 后面，而不是替换。

#### 5.1 合并 Hooks 配置

**正确做法**：将 ECC 的每个 hook 类型**追加**到用户现有 hooks 的末尾。

```json
{
  "hooks": {
    "PreToolUse": [
      /* 用户的现有 hooks（保留） */
      {"matcher": "...", "hooks": [...]},

      /* ECC hooks（追加到末尾） */
      {"matcher": "...", "hooks": [...]}
    ],
    "PreCompact": [
      /* 用户现有 + ECC */
    ],
    ...
  }
}
```

**错误做法**：直接用 ECC hooks 替换用户的 hooks。

> 具体配置请参考 ECC 仓库的 `hooks/hooks.json` 文件内容。

#### 5.2 配置环境变量（Windows 必需）

> ⚠️ **Windows 特有问题**：`${CLAUDE_PLUGIN_ROOT}` 在 Windows 下不会自动解析，必须显式配置。

在 `settings.json` 中添加 `env` 部分：

```json
{
  "env": {
    "CLAUDE_PLUGIN_ROOT": "C:/Users/<你的用户名>/.claude/plugins/cache/everything-claude-code/everything-claude-code/<版本号>"
  }
}
```

**如何查找实际路径**：
```bash
# 查看插件缓存目录
ls -la ~/.claude/plugins/cache/everything-claude-code/
```

---

## 验证安装

运行以下命令检查 hooks 是否正常工作：

```bash
/compact
```

**预期输出**：
- ✅ 成功：`completed successfully`
- ❌ 失败：`failed: node: internal/modules/cjs/loader`

---

## 常见问题

### Q1: /compact 报错 "Cannot find module"

**原因**：Windows 下 `${CLAUDE_PLUGIN_ROOT}` 未解析。

**解决**：参考「步骤 5.2」配置环境变量。

### Q2: Hooks 不生效

**原因**：hooks 配置未合并到 settings.json。

**解决**：手动将 `hooks/hooks.json` 内容**追加**到 settings.json 的 `hooks` 字段，保留用户的现有 hooks。

### Q3: 用户的 hooks 被覆盖了

**原因**：直接用 ECC hooks 替换了用户的 hooks。

**解决**：将用户的现有 hooks 保留，只将 ECC hooks 追加到末尾。参考「步骤 5.1」。

### Q4: 命令不可用

**原因**：rules 未正确安装。

**解决**：参考「步骤 4」安装 rules。

### Q5: 我的自定义规则被覆盖了

**原因**：直接用 `cp -r` 复制 ECC 规则，覆盖了现有文件。

**解决**：
```bash
# 如果有备份，从备份恢复
cp -r ~/.claude/rules.backup/* ~/.claude/rules/common/

# 然后用步骤 4.3 的方法合并（只复制不存在的文件）
```

**预防**：以后安装任何规则前，先执行 `cp -r ~/.claude/rules ~/.claude/rules.backup`

---

## 快速检查清单

- [ ] 插件已安装 (`/plugin list`)
- [ ] settings.json 包含 `enabledPlugins`
- [ ] settings.json 包含 `hooks` 配置
- [ ] **用户的现有 hooks 已被保留**（不是被覆盖）
- [ ] settings.json 包含 `env.CLAUDE_PLUGIN_ROOT`（Windows）
- [ ] rules 已复制到 `~/.claude/rules/`
- [ ] **现有规则已备份**（安装前执行了 `cp -r ~/.claude/rules ~/.claude/rules.backup`）
- [ ] 规则已合并（非直接覆盖）
- [ ] `/compact` 执行成功
