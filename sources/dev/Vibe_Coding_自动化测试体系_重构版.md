---
name: Vibe Coding 自动化测试体系（重构版）
description: AI 协同编程时代的自动化测试体系——以测试作为约束 Agent 的可执行行为规格，覆盖测试设计（CTFL/CT-MBT/CT-GenAI）、TDD、单元到 Mutation Testing 全链路验证、测试架构与 Agent 测试报告
type: concept
updated: 2026-08-13
tags: vibe-coding, testing, tdd, mutation-testing, code-quality, agent
source: 原创整理
---

# Vibe Coding 时代的自动化测试体系

## 一句话知识点

**AI 时代测试的核心，不再只是“有没有测试”，而是把测试变成约束 Agent 实现的可执行行为规格，并验证这些测试本身是否真的有能力发现错误。**

---
# 1. 为什么 Vibe Coding 更依赖测试

传统开发：

```text
开发者写代码
→ 开发者阅读和理解代码
→ 测试辅助检查
```

Vibe Coding：

```text
人定义需求
→ Agent 大量生成和修改代码
→ 人很少逐行检查代码
→ 自动化测试承担主要验证工作
```

因此测试正在从：

> 代码的辅助检查工具

逐渐变成：

> **人类定义“什么才算正确”的主要接口。**

但仅仅“测试全部通过”仍然不够。

因为代码和测试可能由同一个 Agent 根据同一个错误理解生成：

```text
错误理解需求
↓
错误实现
+
验证错误实现的测试
↓
100% tests passed
```

所以真正应该问的是：

> **这些测试有没有能力发现实现中的错误？**

---
# 2. 完整测试体系要解决什么

Vibe Coding 的测试体系主要解决三个问题：

```text
1. 测什么？
   ↓
Test Design

2. 怎么验证？
   ↓
Unit / Integration / Property / E2E / Fuzz

3. 测试本身有效吗？
   ↓
Mutation Testing
```

---
# 3. 第一层：测试什么 —— Test Design

测试设计的目标不是：

> 让 Agent 随便想到几个测试。

而是：

> **从需求和行为规则中系统推导测试空间。**

首先提取行为契约：

```text
什么输入
+
什么前置条件 / 状态
↓
应该发生什么
+
不应该发生什么
```

然后再系统检查：

- 输入类别与边界
- 条件组合
- 状态与转换
- 正常、异常和恢复路径
- 持久化、并发与时序
- 外部依赖、平台与环境差异
- 高风险场景

具体怎样系统完成这一步，可以使用下面的 **CTFL → 必要时 CT-MBT → CT-GenAI** 方法。

---
# 4. 专业测试方法论：CTFL × CT-MBT × CT-GenAI

可以把 ISTQB 相关知识作为 Agent 的测试设计底座。

这里不需要把三个 CT 当成三个互相竞争的测试方法。更简单的理解是：

```text
CTFL
→ 找测试维度：应该测什么？

CT-MBT
→ 复杂系统画行为地图：系统可能怎样运行、哪些路径不能漏？

CT-GenAI
→ 让 AI 执行和辅助检查上述测试分析、设计、实现与验证
```

完整关系：

```text
需求 / Spec
↓
CTFL：系统寻找应该测试什么
↓
复杂状态 / 工作流？
├─ 否 → 直接形成测试设计
└─ 是 → CT-MBT：建立行为模型，再推导测试路径
↓
形成测试设计文档
↓
CT-GenAI：让 AI 生成、实现、审查和补强测试
```

---

## 4.1 CTFL：先系统寻找“应该测什么”

CTFL 对 Agent 最有价值的部分，是基础测试分析和测试设计思维。

### 第一步：提取行为契约

不要先根据当前代码猜测试。

优先依据：

```text
需求 / Spec
验收标准
已确认设计
Bug 描述
已有行为契约
```

回答：

```text
什么输入
+
什么前置条件 / 状态
↓
应该发生什么
+
不应该发生什么
```

把需求转成明确、可验证的行为。

---

### 第二步：按固定维度检查测试空间

#### 等价类：有哪些行为不同的类别？

把行为相同的一大类输入归为一组。

例如文件路径可以区分：

```text
合法路径
非法路径
不存在路径
普通文件
目录
symlink
```

不需要为大量本质相同的输入重复测试。

重点是：

> **每一种会导致不同系统行为的类别，都应该被考虑。**

---

#### 边界值：规则在哪里发生变化？

例如允许范围：

```text
1 ～ 100
```

不要只测试：

```text
50
```

还应该重点考虑：

```text
0
1
2

99
100
101
```

同样要关注：

```text
空值
零值
单元素
最大允许长度
刚好超过限制
```

原则：

> **Bug 很容易出现在规则刚好发生变化的位置。**

---

#### 决策表：有哪些条件组合？

如果行为同时由多个条件决定，例如：

```text
文件存在？
路径合法吗？
用户批准了吗？
工具执行成功了吗？
```

不要只测试几个顺手想到的组合。

应该明确：

```text
条件 A
×
条件 B
×
条件 C
→
预期结果
```

重点覆盖：

- 每种重要结果
- 重要条件的 True / False
- 容易冲突的组合
- 高风险组合

---

#### 状态转换：同一操作在不同状态下是否不同？

例如系统存在：

```text
Idle
Streaming
ToolExecuting
AwaitingApproval
Cancelled
Finished
```

需要检查：

- 每个重要状态
- 每个合法状态转换
- 重要非法转换
- 中断
- 失败
- 恢复

如果状态、转换和路径开始变复杂，就进入 **CT-MBT**。

---

#### 异常与风险：什么最容易出问题？

主动检查：

```text
非法输入
依赖失败
超时
取消
重复操作
部分成功
资源不存在
权限不足
并发 / 时序
平台差异
持久化失败
恢复路径
```

不要求机械穷举所有理论组合。

优先考虑：

> **失败后果严重 × 更可能发生 × 更容易遗漏**

的场景。

---

### CTFL 阶段产物

先形成测试设计，而不是马上写测试代码：

```text
## 行为契约

## 等价类 / 输入分类

## 边界值

## 条件组合

## 状态与转换

## 异常 / 恢复路径

## 高风险场景

## 建议测试层级
- Unit
- Integration
- Property
- E2E
- Fuzz

## 尚不确定的问题
```

CTFL 阶段最终回答：

> **应该测试什么？**

---

## 4.2 CT-MBT：复杂系统先建立行为模型

CT-MBT 是 Model-Based Testing，即基于模型的测试。

它不是替代 CTFL，而是在复杂系统中，把 CTFL 找到的状态、条件和规则进一步组织成一张**行为地图**，再从地图系统推导测试路径。

### 什么时候需要 CT-MBT？

简单功能：

```text
输入
→ 处理
→ 输出
```

通常用 CTFL 已经足够。

出现下面情况时优先考虑 CT-MBT：

- 多个状态
- 多阶段工作流
- 同一操作在不同状态下行为不同
- 重试
- 审批
- 取消
- 超时
- 恢复
- 多次 Tool Calling
- 异步事件
- 并发或复杂时序

最简单的判断标准：

> **当测试场景越来越长，已经难以确认有没有漏掉路径时，就建立模型。**

---

### 第一步：建立最小行为模型

不需要复杂 UML。

只需要描述：

```text
状态
状态转换
触发条件
预期结果
重要不变量
```

例如：

```text
Idle
 ↓ submit
Streaming
 ↓ tool_call
ToolExecuting
 ├─ success → Streaming
 ├─ failure → Streaming
 └─ cancel  → Cancelled
                  ↓ reset
                 Idle
```

---

### 第二步：给关键转换补充行为规则

例如：

```text
ToolExecuting
    ↓ cancel
Cancelled
```

同时记录：

```text
触发：
用户取消

前置状态：
ToolExecuting

结果：
进入 Cancelled

不变量：
旧 Tool Result 不得继续提交给 Provider
```

此时模型已经不只是状态图，而是：

> **系统应该怎样运行的可检查行为规格。**

---

### 第三步：从模型推导测试

至少检查：

#### 状态覆盖

```text
每个重要状态都能被测试进入吗？
```

#### 转换覆盖

```text
每条重要合法转换都测试了吗？
```

#### 非法转换

```text
不允许发生的转换是否被拒绝？
```

#### 条件分支

```text
一个转换存在多个条件时，
重要分支都覆盖了吗？
```

#### 中断与恢复

```text
每个允许取消 / 失败的位置测试了吗？

失败以后系统进入正确状态吗？

系统还能正常继续工作吗？
```

#### 关键路径

例如 Agent 系统常见：

```text
正常完成
Tool 成功
Tool 失败
Tool → Tool 连续调用
执行中取消
审批拒绝
Provider 失败
达到最大迭代次数
```

---

### CT-MBT 阶段产物

在测试设计文档中增加：

```text
## 行为模型

### 状态
- ...

### 转换
A → B：触发条件 / 预期结果
B → C：触发条件 / 预期结果

### 禁止转换
- ...

### 必须覆盖的路径
1. ...
2. ...
3. ...

### 不变量
- ...
```

CT-MBT 最终回答：

> **这个复杂系统可能怎样运行，以及哪些重要路径必须被验证？**

---

## 4.3 CT-GenAI：让 AI 执行专业测试流程

CT-GenAI 对 Vibe Coding 最重要的思想是：

> **不要只让 AI 生成测试代码，而要让 AI 参与测试分析、设计、实现、审查和缺口发现。**

### 给 AI 正确的事实来源

优先依据：

```text
明确 Spec / 行为契约
↓
用户验收标准
↓
已确认设计
↓
现有测试
↓
生产代码
```

不要只根据当前实现生成测试。

否则容易变成：

```text
错误实现
↓
AI 阅读错误实现
↓
生成证明错误实现正确的测试
```

---

### 先设计，再生成

禁止：

```text
读取需求
↓
直接写 test()
```

应该：

```text
读取需求
↓
CTFL 分析
↓
必要时建立 CT-MBT 模型
↓
形成测试设计
↓
检查遗漏
↓
再生成自动化测试
```

测试代码只是测试设计的实现形式。

---

### 保持需求到测试的可追踪关系

每个重要测试都应该能够回答：

> **这个测试为什么存在？**

它至少应该对应：

```text
某条行为契约
某个边界
某个等价类
某个条件组合
某条状态转换
某个已知 Bug
某个高风险场景
```

不要生成大量无法说明价值的测试。

---

### 显式暴露未知和假设

Agent 不确定需求时，不得偷偷补全并固化成测试。

应该明确区分：

```text
已确认事实

假设

仍然未知

因此暂时无法确定的测试
```

---

### 不让生成测试的 AI 独自证明自己正确

测试生成以后还需要独立验证，例如：

```text
执行测试
↓
确认 RED → GREEN
↓
检查真实行为
↓
Property Testing
↓
Mutation Testing
↓
必要时 E2E / Fuzz
```

核心原则：

> **测试通过不能证明测试本身正确。**

---

### CT-GenAI 阶段产物

最终不只输出测试代码，还应该报告：

```text
已覆盖行为
已覆盖边界
已覆盖状态 / 转换

生成和执行了哪些测试

Property / E2E / Mutation 等加强验证结果

没有覆盖的行为
无法自动验证的部分
仍然存在的风险
```

CT-GenAI 最终回答：

> **怎样让 AI 执行专业测试流程，同时避免 AI 的错误理解污染测试？**

---

## 4.4 三者最简单的记忆

```text
CTFL
= 把需求拆成完整测试维度

CT-MBT
= 把复杂行为变成地图，
  再从地图系统推导测试路径

CT-GenAI
= 让 AI 执行上述分析、设计、实现和检查，
  同时约束 AI 自身的误解和遗漏
```

三者组合的核心不是：

> **让 AI 写更多测试。**

而是：

> **让 AI 用专业测试方法，系统证明需求中的行为是否真的被实现。**

---
# 5. TDD：保证测试与实现的顺序正确

TDD 主要解决：

> **测试什么时候写？**

核心流程：

```text
RED
↓
先写目标行为测试
↓
确认测试真的失败
↓
GREEN
↓
写最小实现
↓
确认测试通过
↓
REFACTOR
```

它的重要价值是防止：

```text
Agent 先完成实现
↓
再根据现有实现补测试
↓
测试天然验证当前实现
```

因此：

```text
Test Design
→ 决定测什么

TDD
→ 决定测试和实现如何推进
```

两者不能混为一谈。

---
# 6. 第二层：怎么验证

不同测试技术解决不同问题。

---

## Unit Test

验证：

> 单个函数、模块或组件是否符合预期。

适合：

* 纯函数
* Parser
* 配置逻辑
* 数据转换
* 独立业务规则

优点：

```text
快
稳定
定位问题容易
```

但它不能证明整个系统组合起来正确。

---

## Integration Test

验证：

> 多个真实模块组合后是否仍然正确。

例如：

```text
Agent Loop
+
Tool Registry
+
Provider
+
History
```

可以分别正确，但组合后仍可能出现：

* 调用顺序错误
* 状态没有同步
* 数据格式不一致
* 错误没有传播
* cleanup 没有执行

因此需要 Integration Test 锁住模块之间的契约。

---
# 7. Property-Based Testing：自动探索大量输入

普通测试：

```text
输入 A → 输出正确
输入 B → 输出正确
```

Property Testing：

```text
定义一个始终应该成立的性质
↓
自动生成大量不同输入
↓
寻找反例
```

例如：

```text
对于任意合法输入：

decode(encode(x)) == x
```

测试工具可以自动尝试：

```text
空值
极短输入
极长输入
Unicode
特殊字符
随机组合
边界数据
……
```

发现失败后，还可以尝试把复杂失败输入缩小成最简单的复现案例。

特别适合：

* Parser
* 路径处理
* 配置解析
* 序列化
* Tool 参数
* 状态机
* 数据结构

Rust 常用：

```text
proptest
```

---
# 8. E2E：验证用户真正使用的系统

Unit Test 证明：

```text
某个组件正确
```

Integration Test 证明：

```text
几个模块组合正确
```

但用户真正使用的是完整系统：

```text
键盘输入
↓
终端
↓
TUI
↓
Agent Loop
↓
Provider
↓
Tool
↓
状态更新
↓
最终输出
```

所以 CLI / Agent 产品应该保留真实用户路径测试，例如：

```text
PTY / ConPTY
真实 subprocess
真实 filesystem
真实配置文件
真实 cancel / interrupt
```

否则可能出现：

```text
所有 Unit Tests PASS
所有 Integration Tests PASS

但真实终端无法使用
```

E2E 的作用就是验证：

> **整个系统最终是否真的像用户看到的那样工作。**

---
# 9. Fuzz Testing：寻找无法提前想到的异常输入

Fuzz Testing 的目标是：

```text
自动制造大量异常 / 随机输入
↓
持续攻击程序
↓
寻找 crash、panic、越界、不变量破坏
```

特别适合：

* Parser
* 文件格式
* 协议解析
* 网络输入
* Tool 参数
* 外部不可信数据

Rust 常见工具：

```text
cargo-fuzz
```

Property Testing 和 Fuzz Testing 的区别可以简单理解为：

```text
Property Testing
→ 某个规则在大量输入下始终成立吗？

Fuzz Testing
→ 有没有什么奇怪输入能把程序搞坏？
```

---
# 10. 第三层：Mutation Testing —— 测试测试本身

这是 Vibe Coding 中非常关键的一层。

因为即使：

```text
1000 tests passed
```

仍然无法证明这些测试真的有用。

Mutation Testing 会故意破坏生产代码。

例如：

```text
>      → >=
true   → false
+      → -
删除一个条件
删除一次函数调用
修改返回值
```

然后重新运行测试。

如果测试失败：

```text
mutation killed
```

说明测试成功发现了错误。

如果测试仍然通过：

```text
mutation survived
```

说明：

```text
这个代码变化没有被测试发现
```

可能存在真实测试缺口。

因此：

```text
Coverage
→ 这段代码有没有被执行？

Mutation Testing
→ 这段代码如果错了，测试能不能发现？
```

后者更接近 Vibe Coding 真正关心的问题。

Rust 可以使用：

```text
cargo-mutants
```

---
# 11. 推荐的 Vibe Coding 测试架构

完整体系可以理解为：

```text
Spec / Requirements
        ↓
Behavior Contract
        ↓
Test Design
        ↓
TDD
        ↓
┌────────────────────────────┐
│ Unit                       │
│ Integration                │
│ Property                   │
│ E2E                        │
│ 必要时 Fuzz                 │
└────────────────────────────┘
        ↓
Mutation Testing
        ↓
发现测试缺口
        ↓
补充测试 / 修复实现
        ↓
Final Verification
```

各层负责的问题：

| 层级           | 解决的问题          |
| ------------ | -------------- |
| Test Design  | 有没有漏掉应该测试的行为？  |
| TDD          | 测试与实现是否按 RED → GREEN 推进？ |
| Unit         | 单个组件正确吗？       |
| Integration  | 模块组合正确吗？       |
| Property     | 大量未知输入下规则仍成立吗？ |
| E2E          | 用户真实使用路径正确吗？   |
| Fuzz         | 有没有意料之外的危险输入？  |
| Mutation     | 测试本身真的能发现错误吗？  |
| Verification | 最终结果有没有真实执行证据？ |

---
# 12. Testing Skill 与现有 Skill 的职责分工

不应该把所有测试知识都塞进 TDD Skill。

更合理的是：

```text
comprehensive-testing
        ↓
决定“测什么”

test-driven-development
        ↓
决定“测试和实现怎么推进”

verification-before-completion
        ↓
决定“如何证明最终真的通过”
```

---

## comprehensive-testing

负责测试分析和测试设计：

- 根据 Spec / 行为契约做 CTFL 分析
- 复杂状态或工作流时建立 CT-MBT 模型
- 推导测试场景和高风险路径
- 选择 Unit / Integration / Property / E2E / Fuzz 等测试层级
- 检查测试缺口
- 明确假设、未知和剩余风险

它回答：

> **应该测试什么？**

---

## test-driven-development

负责：

```text
RED
→ GREEN
→ REFACTOR
```

它回答：

> **测试和生产代码应该怎样实现？**

---

## verification-before-completion

负责：

```text
执行真实验证命令
↓
检查完整结果
↓
确认 failures / exit code
↓
再宣布完成
```

它回答：

> **有什么证据证明任务真的完成？**

---

## Agent 最小执行规则

1. **优先根据 Spec 和行为契约设计测试，不要只根据现有代码反推。**
2. **先做测试分析，再写测试代码。**
3. **至少检查等价类、边界、条件组合、状态、异常和风险。**
4. **复杂状态或工作流考虑 CT-MBT；简单功能不要强行建模。**
5. **每个重要测试都应能追溯到需求、边界、状态、风险或已知 Bug。**
6. **不得把 Agent 自己的猜测偷偷固化成测试；假设和未知必须显式报告。**
7. **测试优先验证真实行为，而不是只验证 mock 或实现细节。**
8. **测试通过后仍需检查测试有效性；重要功能按风险考虑 Property、E2E、Fuzz 和 Mutation。**
9. **最终报告已证明的行为、加强验证结果和仍未验证的风险，不能只报告 PASS 数量。**

---
# 13. Agent 的测试报告也需要升级

低价值报告：

```text
127 tests passed.
```

它只能证明：

> 现有 127 个测试没有失败。

不能证明：

* 测试设计完整
* 边界被覆盖
* 状态转换被覆盖
* 测试能够发现错误
* 真实环境能够运行

更好的测试报告应该包含：

```text
测试设计覆盖：

- 正常路径
- 边界值
- 非法输入
- 错误路径
- 状态转换
- cancellation
- persistence
- tool failure

执行结果：

Unit / Integration:
127 PASS

Property:
10,000 generated cases PASS

E2E:
12 PASS

Mutation:
83 killed
3 survived

Survivors:
- 2 equivalent mutations
- 1 real test gap

处理：
已补 regression test。

仍未验证：
- Windows Terminal resize race
- 真实网络断线恢复
```

因此测试报告应该从：

```text
有多少测试通过
```

升级为：

```text
哪些行为已经被证明
+
测试本身有多强
+
还有哪些风险没有验证
```

---
# 14. 更适合 Vibe Coding 的开发流程

把前面的设计、实现和验证串起来：

```text
用户提出需求
↓
Brain 明确 Spec / 行为契约 / 验收标准
↓
comprehensive-testing
├─ CTFL：寻找测试维度
└─ 复杂流程时 CT-MBT：建立行为模型并推导路径
↓
形成测试设计
↓
CT-GenAI：由 AI 辅助生成、审查和补强测试
↓
Coding Agent 按 TDD 实现
↓
Unit / Integration
↓
按风险增加 Property / E2E / Fuzz
↓
Mutation Testing 检查测试有效性
↓
发现缺口 → 补测试 / 修实现
↓
Final Verification
↓
报告已覆盖行为 + 剩余风险
```

这里最重要的变化是：

> **不再由 Coding Agent 写完代码以后顺手补几个测试。**

而是：

```text
先定义正确性
↓
系统设计怎样验证正确性
↓
再让 Agent 实现代码
↓
最后反过来验证测试本身是否足够强
```

---
# 最终理解

传统关系：

```text
代码
↓
测试是代码的附属品
```

Vibe Coding 更适合：

```text
Spec
↓
Executable Tests
↓
Agent Implementation
```

其中：

```text
Spec
→ 定义什么叫正确

Executable Tests
→ 自动判断是否正确

Implementation
→ Agent 可以持续修改、重构甚至重新生成
```

因此长期真正应该稳定的是：

> **行为契约和能够验证这些行为的测试体系。**

而不是某一版具体代码。

## 最核心的三个认识

1. **测试通过 ≠ 实现正确，测试本身也可能存在错误和遗漏。**
2. **Mutation Testing 可以反过来验证测试是否真的具有发现错误的能力。**
3. **Vibe Coding 应逐渐形成 `Spec → Executable Tests → Agent Implementation` 的开发关系。**
