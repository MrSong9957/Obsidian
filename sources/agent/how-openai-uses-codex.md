---
name: how-openai-uses-codex
description: OpenAI 内部 7 类 Codex 用法与 6 条最佳实践全量中译 — 用例说明、工程师原话、示例 prompt 全保留
type: article
updated: 2026-08-30
tags: [codex, openai, coding-agent, workflow, best-practices, agents-md]
source: E:\迅雷下载\how-openai-uses-codex.pdf
---

# How OpenAI uses Codex

> 原始文件：`E:\迅雷下载\how-openai-uses-codex.pdf`（13 页，第 13 页空）
> 性质：OpenAI 官方手册，基于工程师访谈与内部使用数据
> 定位：**按用法类型组织的精简中译**，保留正文说明、工程师原话与示例 prompt
> 姊妹篇：[[sources/claude-code/how-anthropic-teams-use-claude-code|How Anthropic teams use Claude Code]]（按团队组织的案例）

## 开篇导语

Codex 已在 OpenAI 多个技术团队日常使用，例如 Security、Product Engineering、Frontend、API、Infrastructure、Performance Engineering。团队用它加速一系列工程任务：从理解复杂系统、重构大代码库，到在紧截止日期下发布新功能、处理事故。[^1]

手册来源：对 OpenAI 工程师的访谈 + 内部使用数据，整理成用例与最佳实践，说明 Codex 如何帮助团队更快交付、提高工作质量、在规模下管理复杂度。

---

## Use case 1：代码理解（Code understanding）

Codex 帮团队在上手、调试或调查事故时，快速进入代码库里不熟的部分。[^2]

他们常用 Codex 来：
- 定位某功能的核心逻辑
- 画出服务或模块之间的关系
- 追踪数据在系统里怎么流
- 揭出架构模式，或本会缺失、原本要花大量人力才能补上的文档

事故响应时，Codex 通过呈现组件之间的交互、追踪故障状态如何沿系统扩散，帮工程师快速进入新领域。

### 团队原话

- **Performance Engineer, Retrieval Systems**：「我修 bug 时，会用 **Ask Mode** 看同样的问题还可能出现在代码库哪里。」
- **Site Reliability Engineer, API Platform**：「我值班时，贴 stack trace，问 Codex auth 流程在哪。它直接跳到对的文件，我就能很快分诊。」
- **DevOps Engineer, Infrastructure Services**：「Codex 回答我跨 Terraform 和 Python 的『这事该在哪做？』类仓库问题，比 grep 快得多。」

### 示例 prompt

- 这个仓库的认证逻辑实现在哪？
- 总结请求从入口到响应怎么流过这个服务。
- 哪些模块和 `[插入模块名]` 交互，失败是怎么处理的？

---

## Use case 2：重构与迁移（Refactoring and migrations）

Codex 常用于跨多个文件或包的改动。例如工程师更新 API、改某模式的实现方式，或迁到新依赖时，Codex 让一致地应用这些改动变得容易。[^3]

在下列情况下尤其有用：
- 同一更新要落到几十个文件
- 更新需要理解结构与依赖，正则或查找替换搞不定

他们还用它做代码清理：拆开过大的模块、用现代模式替换旧模式，或为更好可测试性做准备。

### 团队原话

- **Backend Engineer, ChatGPT Web**：「Codex 把每一个 legacy `getUserById()` 换成我们的新服务模式，并开了 PR。它几分钟做完原本要几小时的事。」
- **Product Engineer, ChatGPT Enterprise**：「为了清掉上线阻塞，我让 Codex 扫出旧模式的每一个出现点，在 Markdown 里总结影响，再带着修复开 PR。」

### 示例 prompt

- 按关注点把这个文件拆成独立模块，并为每个生成测试。
- 把所有基于 callback 的数据库访问改成 async/await。

---

## Use case 3：性能优化（Performance optimization）

Codex 用来识别并处理性能瓶颈。[^4]

调优或可靠性工作中，工程师 prompt Codex 分析慢路径或内存大户——例如低效循环、冗余操作、昂贵查询——并建议更优替代，常带来效率和可靠性上的实质提升。

Codex 也用来支撑代码健康：找出仍在活跃使用的危险或过时模式。团队靠它帮减少长期技术债，并主动防回归。

### 团队原话

- **Infrastructure Engineer, API Reliability**：「我用 Codex 扫重复的昂贵 DB 调用。它很会标出 hot path，并起草我后面可以再精调的批量查询。」
- **Platform Engineer, Model Serving**：「Codex 很适合快速揪出性能问题——我花 5 分钟写 prompt，省下 30 分钟工作。」

### 示例 prompt

- 优化这个循环的内存效率，并解释为什么你的版本更快。
- 找出这个请求 handler 里重复的昂贵操作，并建议缓存机会。
- 建议在这个函数里更快地批量查 DB 的方法。

---

## Use case 4：提高测试覆盖（Improving test coverage）

Codex 帮工程师更快写测试——尤其是在覆盖很薄或完全没有的地方。[^5]

修 bug 或重构时，工程师常让 Codex 建议覆盖边界情况或可能失败路径的测试。对新代码，它可以根据函数签名和周边逻辑生成单元测试或集成测试。

Codex 对识别边界条件特别有用，例如空输入、最大长度，或「少见但合法」的状态——这些常在初版测试里被漏掉。

### 团队原话

- **Frontend Engineer, ChatGPT Desktop**：「我晚上把 Codex 指向低覆盖模块，早上醒来就有可运行的单元测试 PR。」
- **Backend Engineer, Payments & Billing**：「monorepo 切分支很痛苦时，我让 Codex 写测试并踢起 CI，自己继续在当前分支上干活。」

### 示例 prompt

- 为这个函数写单元测试，包含边界情况和失败路径。
- 为这个排序工具生成基于属性的测试（property-based test）。
- 扩展这个测试文件，覆盖 null 输入和非法状态等缺失场景。

---

## Use case 5：加快开发速度（Increasing development velocity）

Codex 通过加速开发周期的**开头和结尾**，帮团队更快前进。[^6]

启动新功能时，工程师用它 scaffold 样板：生成目录、模块和 API stub，快速拿到可运行代码，而不必手接每一块。

项目接近发布时，Codex 帮顶住紧截止日期，处理较小但关键的任务：分诊 bug、补上最后一公里实现空缺、生成发布脚本、telemetry 钩子或配置文件。

它还被用来把产品反馈变成起步代码。工程师常贴进用户请求或 spec，让 Codex 生成粗稿，之后再回来精修。

### 团队原话

- **Product Engineer, ChatGPT Enterprise**：「我开了一天会，仍合了 4 个 PR，因为 Codex 在后台干活。」
- **Full-Stack Engineer, Internal Tools**：「Codex 帮忙完美交付了 3–4 个本会一直烂在 backlog 里的低优先级修复，这非常赋能。」

### 示例 prompt

- 为 `POST /events` scaffold 新 API 路由，含基本校验和日志。
- 用这个模板 `[插入你的 telemetry 代码示例]` 生成 telemetry 钩子，跟踪新 onboarding 流程的成功/失败。
- 基于这份 spec 创建 stub 实现：`[插入 spec 或产品反馈]`。

---

## Use case 6：保持心流（Staying in flow）

当工程师日程被打碎、充满打断时，Codex 帮他们保持产出。[^7]

它被用来：
- 捕获未完成的工作
- 把笔记变成可运行原型
- 甩出探索性任务，之后再回来

这让暂停和恢复工作而不丢上下文变得更容易，尤其是在值班或会议很多的时候。

### 团队原话

- **Backend Engineer, ChatGPT API**：「如果我碰到一个顺手修（drive-by fix），我会丢一个 Codex 任务，而不是切换分支；有空了再 review 它的 PR。」
- **API Engineer, Infrastructure Observability**：「我例行把 Slack 线程、Datadog trace、issue 等转给 Codex，这样我能继续专注高优先级工作。」

### 示例 prompt

- 生成把这个服务重构成更小模块的计划。
- 先 stub 出重试逻辑并加 TODO——退避逻辑我稍后补。
- 总结这个文件，方便我明天接着做。

---

## Use case 7：探索与构思（Exploration and ideation）

Codex 也适合开放式工作：找替代方案，或验证设计决策。[^8]

你可以 prompt 不同的解题方式、探索不熟悉的模式，或压测假设。这有助于露出 tradeoff、扩大设计选项、锐化实现选择。

它还被用来识别相关 bug。给定已知问题或废弃方法，Codex 能在代码别处找出相似模式，更容易抓住回归或收尾清理工作。

### 团队原话

- **Product Engineer, ChatGPT Desktop**：「Codex 帮我解决冷启动问题——我贴进 spec 和文档，它要么 scaffold 代码，要么指出我忘了什么。」
- **Performance Engineer, Retrieval Systems**：「修完 bug 后我问 Codex 类似 bug 还可能藏在哪，再甩出后续任务。」

### 示例 prompt

- 如果系统改成事件驱动而不是 request/response，这事会怎么做？
- 找出所有手写 SQL 字符串、而不是用我们 query builder 的模块。
- 用更函数式的风格重写，避免可变状态和副作用。

---

## 最佳实践（Best Practices）

手册原文：Codex 在**有结构、有上下文、有迭代空间**时效果最好。以下是 OpenAI 团队为在日常工作中从它身上稳定拿到价值，正在培养的一些习惯。[^9]

### 1. 先从 Ask Mode 开始（Start with Ask Mode）

大改动时，先用 **Ask Mode**（只分析、不直接改代码）prompt Codex 要一份实现计划；再切到 **Code Mode**（真正改代码）时，这份计划成为后续 prompt 的输入。

两步流程让 Codex 更「贴地」，有助于避免输出出错。

Codex 在**边界清楚的任务**上表现最好：大约你或一个同事**一小时能做完**，或**几百行代码**能实现完。随着模型变强，它能接的任务规模预计会变大。

### 2. 迭代改进 Codex 的开发环境

配好启动脚本、环境变量和网络访问，能显著降低 Codex 的错误率。

跑任务时，留意那些可以在 Codex 环境配置里修掉的构建错误。这可能要几轮迭代，但长期会带来显著效率收益。

### 3. 把 prompt 写成 GitHub Issue

当 prompt 镜像你在 PR 或 Issue 里描述改动的方式时，Codex 响应更好。也就是说，相关时要包含：
- 文件路径
- 组件名
- diff
- 文档片段

用「按 `[模块 X]` 里同样的方式实现」这类模式 prompt，能改善结果。

### 4. 把 Codex 任务队列当轻量 backlog

可以开任务去捕获：跑题想法、半成品工作、顺手修。

**没有压力**要一次生成完整 PR。Codex 很适合当暂存区（staging area），你回到专注状态时再回来处理。

### 5. 用 AGENTS.md 提供持久上下文

维护一份 `AGENTS.md`，帮 Codex 跨 prompt 在你的仓库里更有效地工作。

这类文件通常包含：
- 命名约定
- 业务逻辑
- 已知怪癖（quirks）
- 仅从代码推不出来的依赖

手册原文指向 docs，说明如何组织 AGENTS.md（本 PDF 未展开结构细节）。

### 6. 用 Best-of-N 提升输出

**Best-of-N** 功能让你对同一任务同时生成多份响应，以便快速探索多种解法并挑最好的。

对更复杂的任务，你可以 review 若干次迭代，并把不同响应里的部分**拼装**起来，得到更强的结果。

---

## 展望（Looking Ahead）

Codex 仍在 research preview，但已经在真实改变 OpenAI 的构建方式：帮他们更快前进、写出更好代码，并接下原本永远不会被排上优先级的工作。[^10]

他们对前路的潜力感到兴奋——随着模型变好、Codex 更深地融入工作流，期待解锁更有力量的软件开发方式。会继续分享路上学到的东西。

---

## 术语表

| 术语 | 大白话 |
|------|--------|
| Codex | OpenAI 的编程 Agent：能读仓库、改代码、开 PR |
| Ask Mode | 只分析和回答，不直接改代码的模式 |
| Code Mode | 真正动手改代码/开 PR 的模式 |
| PR (Pull Request) | 把改动提交进主仓库的审查单元 |
| CI | 持续集成：自动跑测试/构建的流水线 |
| stack trace | 报错时打印的调用栈，能看出错在哪一层 |
| hot path | 频繁执行、很费时间的代码路径 |
| scaffold | 先生成目录/接口/样板，把骨架搭起来 |
| tech debt | 技术债：图快留下的、以后要还的烂摊子 |
| monorepo | 把很多项目塞进一个超大仓库 |
| drive-by fix | 顺手修，不是原计划里的事 |
| Best-of-N | 同一题让模型答 N 次，挑最好的或拼装 |
| AGENTS.md | 写给编码 Agent 看的仓库说明书 |
| telemetry | 遥测：上报成功/失败等运行指标的钩子 |
| property-based test | 基于属性的测试：对任意输入验证不变量 |
| query builder | 用来拼 SQL 的封装工具，避免手写字符串 |

## 引用索引

页码对应 `E:\迅雷下载\how-openai-uses-codex.pdf`。

[^1]: 原 PDF P3 — Introduction：多团队日常使用与手册来源
[^2]: 原 PDF P4 — Code understanding：说明、原话、示例 prompt
[^3]: 原 PDF P5 — Refactoring and migrations：说明、原话、示例 prompt
[^4]: 原 PDF P6 — Performance optimization：说明、原话、示例 prompt
[^5]: 原 PDF P7 — Improving test coverage：说明、原话、示例 prompt
[^6]: 原 PDF P8 — Increasing development velocity：说明、原话、示例 prompt
[^7]: 原 PDF P9 — Staying in flow：说明、原话、示例 prompt
[^8]: 原 PDF P10 — Exploration and ideation：说明、原话、示例 prompt
[^9]: 原 PDF P11-12 — 六条 Best Practices 全文
[^10]: 原 PDF P12 — Looking Ahead
