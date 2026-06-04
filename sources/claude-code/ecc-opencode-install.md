---
name: ECC-OpenCode 集成踩坑
description: ECC v2.0.0-rc.1 安装到 OpenCode 全局配置的完整调试过程，含 agent 格式兼容性、merge 脚本 bug、bridge plugin crash
type: summary
updated: 2026-06-02
tags: [ecc, opencode, agent, format, debugging, integration]
---

# ECC v2.0.0-rc.1 → OpenCode 集成踩坑

> 结论摘要：[[wiki/opencode-bridge.md#ECC Agent 格式兼容性]]

## 背景

将 ECC v2.0.0-rc.1（npm 包 `ecc-universal`）安装到 OpenCode 1.15.13 全局配置。

**安装路径**：
- npm 全局包：`C:\Users\<user>\AppData\Roaming\npm\node_modules\ecc-universal\`
- 桥接目录：`C:\Users\<user>\.config\opencode\ecc\`（从 npm 包复制）
- 全局配置：`C:\Users\<user>\.config\opencode\opencode.jsonc`

## Bug 1：Agent model 不存在（根因）

**症状**：`opencode run --command plan` 报 `Model not found: anthropic/mimo-v2.5-pro`

**调试过程**：
1. 最初怀疑 OpenCode `subtask: true` 路径硬编码 model → 测 `--pure` 模式也复现，排除 ECC 插件
2. 测 `/init`（内置命令）正常 → 推翻"所有 slash command 都坏"
3. 发现 `--agent planner` 直接调用正常 → 定位到 sub-agent 路径
4. 查 `opencode debug config` 日志 → 发现 `providerID=anthropic modelID=mimo-v2.5-pro` 在 share-next 日志中
5. 探索 OpenCode 源码（GitHub）→ `SessionPrompt.command` 读 `agent.model` → `getModel()` 抛 ProviderModelNotFoundError
6. **最终定位**：ECC agent `.md` 文件的 frontmatter 写着 `model: anthropic/mimo-v2.5-pro`（或 `anthropic/mimo-v2.5`）

**根因**：ECC npm 包的 agent 模板用 `model: opus/sonnet`（ECC 简写），`merge-to-global.ps1` 脚本把它们映射成了 `anthropic/mimo-v2.5-pro`，但 OpenCode 注册表中不存在这个 model。

**修复**：从 ECC npm 包重新复制 agent 文件，删除 `model:` 行，让全局 `model: opencode/minimax-m3-free` 生效。

**涉及文件**：`~/.config/opencode/agents/*.md`（63 个文件）

## Bug 2：tools 数组格式不兼容

**症状**：`Expected object | undefined, got ["Read","Grep","Glob","Bash"] tools`

**根因**：ECC agent frontmatter 用 YAML 数组格式 `tools: ["Read", "Grep", "Glob"]`，OpenCode 需要对象格式 `tools: { Read: true, Grep: true }`。

**修复**：PowerShell 脚本批量转换 61 个 agent 文件的 `tools` 字段。

**脚本**：`C:\Users\<user>\AppData\Local\Temp\fix-tools-format.ps1`

## Bug 3：color 字段枚举不兼容

**症状**：`Expected "primary"|"secondary"|...|"info", got "orange" color`

**根因**：ECC agent 用 CSS 颜色名（red, green, purple, teal, orange），OpenCode 只接受特定枚举值。

**映射**：
| ECC 颜色 | OpenCode 枚举 |
|----------|--------------|
| red | error |
| green | success |
| purple | accent |
| teal | info |
| orange | warning |

**修复**：PowerShell 脚本批量修改 5 个 agent 文件。

**脚本**：`C:\Users\<user>\AppData\Local\Temp\fix-color-frontmatter.ps1`

## Bug 4：Bridge plugin undefined crash

**症状**：`TypeError: undefined is not an object (evaluating 'output.error')` at `claude-code-hooks.ts:457`

**根因**：`tool.execute.after` handler 的 `output` 参数在某些 edge case 下是 `undefined`（如工具执行异常被取消），plugin 直接访问 `output.error` 崩溃。

**修复**：第 455 行 `output` 参数加 `?`（可选），第 456 行加 `if (!output) return;` 防御。

**文件**：`C:\Users\<user>\.config\opencode\plugins\claude-code-hooks.ts`

## Bug 5：merge-to-global.ps1 复制了全部 61 个 agent

**症状**：Ctrl+Tab 出现 63 个可切换模式（只有 plan 和 build 是用户想要的）

**根因**：`merge-to-global.ps1` 把 ECC npm 包 `agents/` 目录下的全部 61 个 `.md` 文件都复制到了 `~/.config/opencode/agents/`。但 `opencode.jsonc` 只为 25 个 agent 创建了定义。其余 36 个（语言特定的 csharp-reviewer、dart-build-resolver、领域特定的 marketing-agent、seo-specialist 等）是按需使用的，不应该全局注册。

**修复**：删除全部 61 个 `.md` 文件。25 个有 opencode.jsonc 定义的 agent 通过命令继续可用。

**备份**：`~/.config/opencode/agents.bak/`（61 文件）

## Bug 6：command.plan.model 字段无效

**症状**：在 opencode.jsonc 的 `command.plan` 段加 `"model": "opencode/minimax-m3-free"` 无效

**根因**：OpenCode 的 model 解析优先级是 `cmd.model → agent.model → currentModel()`。但 agent frontmatter 的 `model:` 字段优先级更高（通过 `agents.get(cmd.agent)` 加载），覆盖了 command 级别的 model 设置。

**修复**：删除无效的 `command.plan.model` 字段，改为删除 agent frontmatter 的 `model:` 行。

## 最终修复清单

| Bug | 根因 | 修复 | 影响文件数 |
|-----|------|------|-----------|
| Agent model 不存在 | ECC 模板 model 映射错误 | 删除 model: 行 | 61 |
| tools 数组格式 | ECC YAML 数组 vs OpenCode 对象 | 转换格式 | 61 |
| color 枚举不兼容 | ECC CSS 颜色名 vs OpenCode 枚举 | 映射替换 | 5 |
| Bridge crash | output undefined 无防御 | 加 if (!output) | 1 |
| 61 agent 全局注册 | merge 脚本无差别复制 | 删除 .md 文件 | 61 |
| command.model 无效 | agent frontmatter 优先级更高 | 删除字段 | 1 |

## 参考

- ECC npm 包：`ecc-universal@2.0.0-rc.1`（2026-05-26 发布）
- OpenCode 版本：1.15.13（Bun 1.3.14 编译）
- npm root -g：`C:\Users\<user>\AppData\Roaming\npm\node_modules`
- opencode.jsonc 备份：`opencode.jsonc.pre-ecc.bak`、`opencode.jsonc.pre-c-test.bak`
