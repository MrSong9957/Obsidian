---
name: 多智能体迭代框架
description: 四角色迭代：研究者→实现者→测试者→评估者 — 搭建多智能体协作系统时参考
type: concept
sources:
  - "[[sources/learn-claude-code]]"
updated: 2026-04-05
tags: [ai, multi-agent]
status: deprecated
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# 多智能体迭代框架精要

> 内容已迁移至 [[wiki/agent-arch-p1.md|agent-arch-p1]]。

**触发词：** 用户说"多智能体框架"、"生成器+评估器"、"迭代开发"等

---

## 执行约束（强制）

### 协调器禁止事项
- ❌ 禁止直接使用 Edit/Write 工具修改代码
- ❌ 禁止跳过 Agent 工具直接执行任务
- ❌ 禁止在评估通过前结束迭代

### 协调器强制行为
- ✅ 必须使用 Agent 工具启动子代理
- ✅ 必须等待子代理返回结果
- ✅ 必须基于反馈进行下一轮迭代

### Claude Code 工具映射

| 框架概念 | Claude Code 工具 |
|----------|------------------|
| `subagent(prompt)` | `Agent` 工具 |
| **研究者** | `Agent(subagent_type="Explore")` |
| **实现者** | `Agent(subagent_type="general-purpose")` |
| **测试者** | `Agent(subagent_type="general-purpose")` |
| **评估者** | `Agent(subagent_type="general-purpose")` |

**测试者 vs 评估者：**
- 测试者：主动执行测试，输出报告
- 评估者：阅读报告，决定 PASS/CONTINUE

---

## 核心原则：动态发现

框架**不硬编码**智能体/技能，而是**动态读取 CLAUDE.md 索引**：

1. 读取索引区域，发现可用资源
2. 根据任务需求匹配智能体/技能
3. 优先使用索引中的现成资源
4. 仅在索引无匹配时才自定义提示词

**索引资源示例：**
- `ECC-Agent-Usage-Instructions` - 25 个专业智能体
- `Superpowers` - 核心工作流技能
- 其他插件技能

---

## 目录结构

```
docs/agents/
├── README.md           # 框架说明
├── workflows/          # 工作流配置 (YAML)
└── prompts/            # 提示词模板 (MD)
```

---

## 工作流配置 (YAML)

**必填字段：**
```yaml
name: "工作流名称"
max_iterations: 12

agents:
  - id: generator
    role: "生成器"
    trigger: "odd"           # "odd" | "even" | [0,3,6]
    prompt_template: |
      任务: {{task_description}}
      反馈: {{last_feedback}}

  - id: evaluator
    role: "评估器"
    trigger: "even"
    criteria:
      - "功能性"
      - "代码质量"
    prompt_template: |
      标准: {{criteria}}
      改动: {{changes}}
      通过输出: <promise>PASS</promise>

completion:
  signal: "<promise>PASS</promise>"
  max_iterations_reached: "report"
```

**触发规则：**
- `odd` → 奇数轮（第 1,3,5...）
- `even` → 偶数轮（第 2,4,6...）
- `[0,3,6]` → 指定轮次

---

## 预置工作流

| 名称 | 智能体 | 场景 |
|------|--------|------|
| **researcher-implementer-tester-evaluator** | 研究者 + 实现者 + 测试者 + 评估者 | **标准四角色迭代（推荐）** |
| generator-evaluator | 生成器 + 评估器 | 简单代码 + 快速审核 |
| writer-editor | 作者 + 编辑 | 文档 + 审核 |
| coder-reviewer-tester | 程序员 + 审核者 + 测试者 | TDD（无独立评估者） |

---

## 四角色迭代流程

### 标准角色

| 角色 | 职责 | 权限 | 输出 |
|------|------|------|------|
| **研究者** | 分析需求、探索代码库、**产出验收标准** | 只读 | 分析报告 + 验收标准 |
| **实现者** | 编写/修改代码 | 可编辑 | 代码改动 |
| **测试者** | 写 .tape 脚本、跑 build、运行测试、**产出报告+截图** | 只读+运行 | 测试报告 + 截图 |
| **评估者** | 审查报告和截图（**必须多模态**）、决定结果 | 只读 | PASS/CONTINUE |

### 流程图

```
迭代 N：
  1. 研究者 → 分析需求 + 产出验收标准（只读）
  2. 实现者 → 编写代码（可编辑）
  3. 测试者 → 写 .tape 脚本、跑 build、截图、填报告
  4. 评估者 → 审查报告和截图（多模态），决定 PASS/CONTINUE
       ↑
   评估者是守门员，在最后
```

### 循环规则

当评估者返回 **CONTINUE** 时：

| 问题类型 | 循环起点 | 说明 |
|----------|----------|------|
| 代码 Bug | 实现者 | 小问题，直接修复 |
| 需求理解错误 | 研究者 | 大问题，重新分析 |
| **默认** | **实现者** | 大多数情况 |

**评估者职责强调（最严格模式）：**
- 评估者是**守门员**，在流程最后
- 必须**基于测试者的报告**判断，不能跳过测试
- **最严格原则：有任何问题都必须 CONTINUE，不能放过**
- 只有**零问题**才能输出 PASS
- CONTINUE 时必须说明问题类型，决定循环起点

---

## 产出物契约

### 测试产出物目录

```
docs/test-artifacts/
├── templates/
│   ├── acceptance-criteria.md   # 研究者产出（功能点、交互路径、UI 预期、边界情况）
│   └── report.md                # 测试者产出（构建状态、截图清单、验收检查）
├── screenshots/                 # VHS 截图存放处
├── test-{N}.tape                # 每轮迭代的 .tape 脚本
└── report-{N}.md                # 每轮迭代的测试报告
```

### 信息流

```
研究者(验收标准) → 测试者(.tape + 截图 + 报告) → 评估者(多模态判断)
```

### Git 工作流

| 阶段 | Git 操作 |
|------|----------|
| 实现者修改前 | **必须先 commit**（保存当前状态，方便回溯） |
| 评估者 PASS | **必须 commit**（放行，保存通过状态） |
| 评估者 CONTINUE | **不 commit**，直接进入下一轮迭代 |
| 全部通过后 | 最终 commit 包含完整改动 |

### 评估者多模态要求

- 评估者必须能**查看截图**（Read 工具读取 PNG）
- 判断依据：测试报告文字 + VHS 截图视觉证据
- UI 相关验收标准**必须**对照截图判断，不能仅看文字报告

### VHS 集成（测试者角色）

- 测试者使用 VHS 执行 .tape 脚本，生成终端截图
- 截图路径约定：`docs/test-artifacts/screenshots/{name}-{iteration}.png`
- 容器环境必须设置 `VHS_NO_SANDBOX=true`

---

## CLAUDE.md 索引配置

```markdown
- [Multi-Agent Framework](./docs/agents/README.md) - 多智能体迭代框架（四角色迭代：研究者→实现者→测试者→评估者）
```

---

## 使用方式

**直接描述：**
```
我需要一个生成器和评估器，迭代10次实现xxx功能
```

**引用配置：**
```
用 generator-evaluator.yaml 实现 xxx 功能
```

---

## 重建清单

创建框架时需要创建的文件：
1. `docs/agents/README.md` - 框架说明
2. `docs/agents/workflows/generator-evaluator.yaml`
3. `docs/agents/workflows/writer-editor.yaml`
4. `docs/agents/workflows/coder-reviewer-tester.yaml`
5. `docs/agents/prompts/generator.md`
6. `docs/agents/prompts/evaluator.md`
7. `docs/agents/prompts/writer.md`
8. `docs/agents/prompts/editor.md`
9. `docs/agents/prompts/coder.md`
10. `docs/agents/prompts/reviewer.md`
11. `docs/agents/prompts/tester.md`

---

## 实战验证（2026-03-27）

**任务：** 修复 ai-terminal 中 Thinking/工具调用/文件编辑不可见问题

**迭代记录：**
```
研究者 (分析 6 个根因 + 验收标准)
  → 实现者 (第一轮：启用 Thinking + 工具进度 + 中间持久化)
    → 测试者 (PASS：705/739 通过，34 失败均为预已存在)
      → 评估者 (CONTINUE：发现 tool_result 角色错误 + redacted_thinking 缺失)
        → 实现者 (第二轮：toAnthropicMessageParams 拆分 + RedactedThinkingPart)
          → 评估者 (PASS)
```

**关键发现：**
- 评估者捕获了测试者和实现者都遗漏的 **API 协议合规性问题**（`tool_result` 必须属于 `user` 角色）
- 证实了"评估者做守门员"的价值——协议层面的错误只有全局审查才能发现
- 工具循环中 `resetStreaming()` 导致 UI 空白是最核心的根因

**改进方向：**
- 测试者应补充 API 协议合规性检查（不仅看 build/test）
- 评估者应增加"历史消息回传 API"场景的验证
