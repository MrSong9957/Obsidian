---
name: ECC 快速指南
description: ECC 插件安装方式、Hooks 配置档位、持续学习/会话记忆/代理注册/模板库/隐藏组件、故障排除
type: reference
updated: 2026-10-20
tags: ecc, hooks, rules, agents, templates, session-memory, continuous-learning, installation, troubleshooting
source: https://github.com/affaan-m/everything-claude-code
---

# 快速开始

## 第一步：安装插件

使用此仓库的最简单方法 - 作为 Claude Code 插件安装：
```
# 将此仓库添加为市场
/plugin marketplace add https://github.com/affaan-m/ECC

# 安装插件
/plugin install ecc@ecc
```

或直接添加到你的 `~/.claude/settings.json`：
```
{
  "extraKnownMarketplaces": {
    "ecc": {
      "source": {
        "source": "github",
        "repo": "affaan-m/everything-claude-code"
      }
    }
  },
  "enabledPlugins": {
    "ecc@ecc": true
  }
}
```

这让你可以立即访问所有命令、代理、技能和钩子。

## 第二步：仅在需要时安装规则

WARNING: 重要提示： Claude Code 插件无法自动分发 rules。

如果你已经通过 /plugin install 安装了 ECC，不要再运行 ./install.sh --profile full、.\install.ps1 --profile full 或 npx ecc-install --profile full。插件已经会自动加载 ECC 的技能、命令和 hooks；此时再执行完整安装，会把同一批内容再次复制到用户目录，导致技能重复以及运行时行为重复。

对于插件安装路径，请只手动复制你需要的 rules/ 目录 或 使用命令安装：
```
/ecc:project-init 在项目目录中安装与技术栈匹配的规则，仅安装规则
```
**实测**：该命令会在项目目录中安装所需的全部规则，包括 `rules/Common` 和 `自动识别匹配的规则`。

---

# HOOKS 配置

> **详细文档**：[[sources/claude-code/ecc-references|ECC Hooks 完整参考]] — 所有 Hook 作用、ECC_HOOK_PROFILE 档位对照

通过 `ECC_HOOK_PROFILE` 配置控制的 Hooks 三个粗粒度配置档位如下：

* **`minimal`**：仅启用最基础配置，不包含全局运行时挂钩，适用于低上下文或本地模型。
* **`standard`（默认）**：平衡性能与审查力度，确保核心挂钩和上下文监控生效。
* **`strict`**：启用最严格的安全审查与只读沙盒校验，常用于高安全性生产环境。

配置命令：
在全局配置中将 `ECC_HOOK_PROFILE` 设置为 standard

---

# Continuous Learning — 持续学习

Stop hook 自动评估会话并保存模式到 `~/.claude/skills/learned/`。手动触发用 `/learn`。

适合提取的模式：错误解决方案、调试技巧、变通方案、项目特定模式。

---

# Session Memory — 跨会话上下文

SessionStart + PreCompact + Stop 三个 hook 协作管理会话状态文件（`~/.claude/sessions/YYYY-MM-DD-topic.tmp`）。

- **保存**：撞上下文限制前、复杂任务阶段完成后、一天结束
- **恢复**：跨天继续、上下文压缩后
- **内容**：有效方法（附证据）、失败尝试、剩余待做

```
/save-session    # 保存
/resume-session  # 恢复
```

---

# Agents — 全局代理注册

ECC 提供 63 个语言/框架专用代理，通过 `Agent` 工具的 `subagent_type` 参数调用。

Agents 路径：`~/.claude/plugins/cache/ecc/ecc/2.0.0/agents`

---

# CLAUDE.md 模板库

位于 `~/.claude/plugins/marketplaces/ecc/docs/zh-CN/examples/`

| 层级 | 使用 ECC 模板 | 说明 |
|------|---------------|------|
| **用户级** | 否 | 使用 [[wiki/AGENTS-md-base.md\|Karpathy 行为准则]] |
| **项目级** | 参考 | 选最接近的模板按需裁剪 |

6 个项目级模板：`CLAUDE.md`（通用）、`django-api-CLAUDE.md`、`go-microservice-CLAUDE.md`、`laravel-api-CLAUDE.md`、`rust-api-CLAUDE.md`、`saas-nextjs-CLAUDE.md`

---

# 高价值隐藏组件

## 1. 项目技术栈自动检测

**文件**：`config/project-stack-mappings.json`

`/project-init` 读取此文件，自动检测技术栈并推荐 skills、rules、commands。覆盖 18 种技术栈，每种定义了指示文件、推荐 rules/skills、构建命令、权限白名单/黑名单。

## 2. Plan-PRD 模式

**文件**：`docs/PLAN-PRD-PATTERN.md`

流程：`/plan-prd` → `/plan` → TDD → `/pr`，每阶段产出 `.md` 文件。

## 3. 上下文模式文件

**文件**：`contexts/dev.md`、`contexts/research.md`、`contexts/review.md`

| 模式 | 行为 |
|------|------|
| 开发 | 先写代码后解释 |
| 研究 | 先理解后行动 |
| 审查 | 按严重性分级 |

## 4. 状态栏配置

**文件**：`examples/statusline.json`

布局：`模型名 | 当前任务 | $成本 工具数 文件数 时长 | 目录 | 上下文进度条`

进度条颜色：<50% 绿 → <65% 黄 → <80% 橙 → ≥80% 红闪

## 5. 安全扫描

`/security-scan` SKILL 技能，作用：扫描密钥泄露、注入漏洞、风险依赖，输出修复补丁

## 6. 多模型协同编排系统 (`Multi-Model Orchestration`)

* **核心文件**：`skills/autonomous-loops/`、`skills/dmux-workflows/`
* **对应命令**：`/multi-plan`、`/multi-execute`、`/multi-backend`
* **隐藏价值**：突破单一模型限制。结合 tmux 实现分布式并行任务，支持高层模型负责全局规划，派生低成本模型并行编写前后端代码，最终通过 DAG 编排流完成总装与联动。

## 7. 商业级资产引擎 (`Investor & Brand Engine`)

* **核心文件**：`skills/article-writing/`、`skills/brand-voice/`、`skills/investor-materials/`
* **隐藏价值**：定位商业交付。内置去 AI 腔调的品牌声量提取器，能反向生成高度拟真的文案；配备完整投融资材料流，可直接输出视口适配的 HTML 幻灯片、财务草案及募资触达信。

## 8. 音视频 AI 敏捷工作流 (`Media Workflow`)

* **核心文件**：`skills/videodb/`、`skills/fal-ai-media/`、`skills/video-editing/`
* **隐藏价值**：音视频处理嵌入开发流。封装 FFmpeg 和 Remotion 驱动逻辑，支持自动化剪辑，并可联动 fal-ai 生成营销视频或技术原理解释动画，闭环全栈交付。

---

# 12. 故障排除指南

**文件**：`docs/TROUBLESHOOTING.md`

| 问题 | 说明 |
|------|------|
| Hook Error 误标 | 正常运行但显示红色错误 |
| 提前压缩 | 上下文未满时意外触发压缩 |
| MCP 认证失效 | 认证过期不提示 |
| Hook 不热重载 | 修改后需重启 |
| 529 过载 | 服务器过载时重试策略 |


