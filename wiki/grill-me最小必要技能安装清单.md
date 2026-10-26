---
name: grill-me 最小必要技能安装清单
description: Matt Pocock 技能库最小必要 9 技能的跨 Agent 安装指令——目录确认、Git 克隆复制、安装验证与执行约束 — 给任意 Agent（Claude Code / Codex / ZCode 等）新环境安装 grill 系与工程系技能时使用
type: concept
updated: 2026-10-26
tags: [agent-skills, grill-me, matt-skills, installation, checklist]
source: self
---

# grill-me 最小必要技能安装清单（v2 · 共 9 个）

本文是一份可直接交给任意 Agent 执行的安装指令：把 [mattpocock/skills](https://github.com/mattpocock/skills) 中最小必要的 9 个技能安装到**当前 Agent 的用户级全局技能目录**，方便跨项目使用。先确认目录，再执行安装。

## 一、确认目录并安装

1. 识别当前执行安装的 Agent，如 Claude Code、Codex、ZCode。以实际运行的工具为准，不按模型名称判断。
2. 根据该 Agent 的实际配置、环境变量、帮助或官方文档，确认其全局技能目录。不得默认使用 `~/.claude/skills`，不得使用项目级目录。
3. 输出 Agent 名称、确认的绝对路径及确认依据。无法确认时，先询问用户，不猜测。
4. 将确认的路径设置为环境变量 `AGENT_SKILLS_DIR`，再执行安装。

**安装仅使用 Git 克隆并复制，无需 node/npx，不通过插件安装。**

若当前 Agent 为 Claude Code，且已安装整包插件，先卸载：

```bash
claude plugin uninstall mattpocock-skills
```

确认并设置 `AGENT_SKILLS_DIR` 后，执行：

```bash
set -e

: "${AGENT_SKILLS_DIR:?请先确认当前 Agent 的用户级全局技能目录，并设置此变量}"

git clone https://github.com/mattpocock/skills.git /tmp/mp-skills
mkdir -p "$AGENT_SKILLS_DIR"

for s in productivity/grilling productivity/grill-me productivity/handoff \
         engineering/grill-with-docs engineering/domain-modeling engineering/to-spec \
         engineering/tdd engineering/code-review engineering/diagnosing-bugs; do
  cp -r "/tmp/mp-skills/skills/$s" "$AGENT_SKILLS_DIR/"
  test -f "$AGENT_SKILLS_DIR/${s##*/}/SKILL.md"
  printf '%s\n' "$AGENT_SKILLS_DIR/${s##*/}"
done

rm -rf /tmp/mp-skills
```

Agent CLI 不可用时，仍通过 Git 克隆并手动复制到已确认的全局技能目录，不得固定回退到 `.claude/skills/`。

## 二、技能清单

| # | 技能 | 用途与要求 |
|---|---|---|
| 1 | **grilling** | grill 系的共同底座。缺少它，grill 系无法正常工作。 |
| 2 | **grill-me** | 逐轮提问，把模糊想法问清楚。适合没有仓库时使用，只讨论，不改代码。 |
| 3 | **grill-with-docs** | 结合代码库澄清需求；术语写入 `CONTEXT.md`，决策形成架构决策记录（ADR）。文档产出阶段依赖 `domain-modeling`。 |
| 4 | **domain-modeling** | 统一项目术语，管理术语表和架构决策记录，是 `grill-with-docs` 文档产出的必要依赖。 |
| 5 | **to-spec** | 把已讨论清楚的内容整理成规格文档。只综合，不提问。 |
| 6 | **tdd** | 每次写代码必用：先写失败测试 → 最少代码通过 → 重构。 |
| 7 | **code-review** | 写完必审，按规格和规范检查代码改动，与 `tdd` 配合使用。 |
| 8 | **diagnosing-bugs** | 遇到 bug 时先用：复现 → 最小化 → 假设 → 插桩 → 修复 → 回归测试。禁止瞎猜乱改。 |
| 9 | **handoff** | 将进度、决策、未完事项整理成交接文档。换会话或换工具前必用。 |

## 三、标准工作流

下列为技能名称，实际调用语法以当前 Agent 为准。

```text
grill-me 或 grill-with-docs   → 把需求问清楚
to-spec                      → 整理规格文档
tdd                          → 按测试先行的方式写代码
code-review                  → 按规格审查改动

遇到 bug：diagnosing-bugs
换会话或换工具前：handoff
```

## 四、安装后验证

1. 检查 9 个技能目录均包含 `SKILL.md`。
2. 按当前 Agent 的要求刷新或重启，重新加载技能。
3. 确认当前 Agent 已识别这 9 个技能。
4. 使用当前 Agent 支持的方式调用 `grill-me`，附带以下测试文本：

```text
这是一条测试指令，回复"安装成功"即可。
```

验证失败时，先核对全局目录、加载方式及 `grilling` 是否齐全。不能仅凭回复“安装成功”认定 9 个技能全部就绪。

## 五、执行约束与安装记录

- **只装上述 9 个技能**。wayfinder、prototype、codebase-design、research 等一律不装。
- 使用 grill 系技能时，不要自动开启 plan mode。
- 装完输出 Agent 名称、全局技能目录、9 个技能的实际落盘路径及验证结果。
- 追加安装日志：`logs/2026-10-02-install-mattpocock-skills.md`。注明 v2 清单变更：新增 `domain-modeling`，共 9 个技能。

## 六、有意不装项

| 技能 | 不装原因 | 补装时机 |
|---|---|---|
| **codebase-design** | 仅供 `tdd` 查阅接口设计术语，缺少它不影响主流程。 | 开始做接口或模块设计时再补。 |

## 相关概念

- [[wiki/Agent工具初始化配置|Agent 工具初始化配置]] — 新 Agent 环境初始化清单，本清单是其第 3 步
- [[wiki/AGENTS-md-base|AGENTS.md 双层架构模板]] — 同一初始化流程第 2 步的规则模板
- [[wiki/大脑模型-v4|大脑模型-v4]] — 这套技能组（Matt Skills）在大脑模型 v4 中作为工程执行层被调度
- [[wiki/Agent Skills|Agent Skills]] — Skill 的三层加载方式、触发条件与缓存成本
- [[wiki/claude-code-setup|Claude Code 当前配置清单]] — 本机 Claude Code 侧的完整配置与技能安装记录
- [[wiki/dev-practice|开发实践]] — `tdd` / `code-review` / `diagnosing-bugs` 技能对应方法论的知识入口
