---
name: claude-code-config-health
description: |
  检查和审计 Claude Code 的配置文件健康状态，包括 settings.json、statusline.py、CLAUDE.md、
  项目级配置等。当用户问"配置对不对"、"检查一下配置"、"为什么配置没生效"、
  "settings 正常吗"，或需要诊断 Claude Code 行为异常时使用此技能。
  也适用于会话开始时自动验证配置一致性。
user-invocable: true
---

# Claude Code Config Health — 配置健康检查

你是一个 Claude Code 配置审计员。你的职责是快速诊断配置问题，确保所有配置文件一致且有效。

## 检查流程

### Step 1: 全局 settings.json

读取 `~/.claude/settings.json`，检查：

| 检查项 | 说明 |
|--------|------|
| JSON 语法 | 是否为合法 JSON |
| `env` 字段 | 环境变量是否完整（CLAUDE_PLUGIN_ROOT 等） |
| `hooks` 字段 | hook 配置是否正确 |
| `permissions` 字段 | 权限模式是否合理 |
| `model` 字段 | 默认模型是否正确 |

### Step 2: 本地 settings.local.json

读取 `~/.claude/settings.local.json`（如存在），检查：
- 是否与 settings.json 冲突
- 本地覆盖是否合理

### Step 3: Statusline 脚本

读取 `~/.claude/scripts/statusline.py`（如存在），检查：
- 脚本语法是否正确
- 是否引用了存在的路径
- 输出格式是否符合预期

### Step 4: 项目级配置

检查当前项目的配置文件：

| 文件 | 检查 |
|------|------|
| `CLAUDE.md` | 存在、内容完整、无过期信息 |
| `.claude/settings.json` | 项目级覆盖是否合理 |
| `.claude/CLAUDE.md` | 项目级指令是否完整 |
| `AGENTS.md` | 存在且与 CLAUDE.md 一致 |
| `AGENTS.local.md` | 本地覆盖（如有） |

### Step 5: 配置一致性

交叉检查：
- 全局 settings 中的 `env` 是否被项目级覆盖
- hooks 是否在全局和项目级都正确配置
- CLAUDE.md 中的规则是否与实际行为一致

## 输出格式

```
## Claude Code 配置健康报告

### 全局配置
| 文件 | 状态 | 问题 |
|------|------|------|
| settings.json | ✅/❌ | — |
| settings.local.json | ✅/⚠️/— | — |
| statusline.py | ✅/❌ | — |

### 项目配置
| 文件 | 状态 | 问题 |
|------|------|------|
| CLAUDE.md | ✅/❌ | — |
| .claude/settings.json | ✅/⚠️/— | — |
| AGENTS.md | ✅/❌ | — |

### 一致性检查
| 检查项 | 结果 |
|--------|------|
| env 无冲突 | ✅/❌ |
| hooks 完整 | ✅/❌ |
| 权限合理 | ✅/❌ |

**修复建议**: （如有）
```

## 常见问题

| 症状 | 可能原因 | 修复 |
|------|---------|------|
| 配置修改不生效 | 缓存或路径错误 | 检查路径，重启 Claude Code |
| hooks 不触发 | JSON 语法错误 | 用 `python -m json.tool` 验证 |
| 权限被拒 | permissions 配置过严 | 调整 defaultMode |
| statusline 不显示 | 脚本路径错误或语法错误 | 检查 settings.json 中的路径 |
| 环境变量丢失 | env 字段被覆盖 | 检查全局 vs 项目级 env |
