---
name: AGENTS-md
description: 全局行为准则与项目开发规范的双层 AGENTS.md 模板；默认使用 Codex 目录，兼容 Claude Code、ZCode、MiMo Code 等 Agent 的规则入口
type: concept
updated: 2026-10-20
tags: [agent, agents-md, claude-md, template, karpathy, setup]
source: https://learn.chatgpt.com/docs/agent-configuration/agents-md
---

# AGENTS.md 双层架构模板

> 来源：
> - 全局层：[andrej-karpathy-skills](https://github.com/forrestchang/andrej-karpathy-skills) — Karpathy 四条行为准则
> - 项目层：自建

规则正文统一保存在 `AGENTS.md`。Codex、ZCode、MiMo Code 等 Agent 工具原生支持该文件；`CLAUDE.md` 仅用于适配 Claude Code 的加载机制，不维护另一套规则正文。[^codex][^zcode][^mimo]

`CLAUDE.md` 的全部内容只有一行：[^claude]

```markdown
@AGENTS.md
```

全局的 `~/.claude/CLAUDE.md` 导入同目录的 `~/.claude/AGENTS.md`，项目级的 `./CLAUDE.md` 导入同目录的 `./AGENTS.md`。相对导入路径以所在的 `CLAUDE.md` 为基准；调查与文档依据见 [[wiki/agents-md-claude-md|AGENTS.md 与 CLAUDE.md 配置关系]]。[^claude]

---

# 第一层：全局 AGENTS.md

> 默认示例位置：`~/.codex/AGENTS.md`（Codex 用户级，跨项目生效）。设置了 `CODEX_HOME` 时使用其目录内的 `AGENTS.md`。[^codex]

全局层放**行为准则 + 个人偏好**，不放项目特定配置。下方 Karpathy 四条准则供各 Agent 复用；项目层为自建规范。[^karpathy]

### 不同 Agent 的全局规则入口

| Agent 工具 | 规则正文位置 | 加载方式 |
|---|---|---|
| Codex（默认） | `~/.codex/AGENTS.md`，或 `$CODEX_HOME/AGENTS.md` | 原生读取；同目录非空 `AGENTS.override.md` 优先于 `AGENTS.md`。[^codex] |
| Claude Code | `~/.claude/AGENTS.md` | 适配文件 `~/.claude/CLAUDE.md` 仅含一行 `@AGENTS.md`。[^claude] |
| ZCode | `~/.zcode/AGENTS.md` | 原生读取用户全局与当前 Workspace 的 `AGENTS.md`；不展开 `@import` / `@include`。[^zcode] |
| MiMo Code | `<实际全局配置目录>/AGENTS.md`；Linux 通常为 `~/.config/mimocode/AGENTS.md` | 原生读取；也可在用户配置的 `instructions` 数组中显式指定共享规则文件的绝对路径。[^mimo] |

`~` 表示运行该 Agent 的用户主目录。规则目录与程序安装目录是两回事；其他支持 `AGENTS.md` 的工具，按其实际用户配置目录放置相同规则正文。

MiMo Code 的 Windows / XDG 路径应以当前版本实际解析的配置目录为准；设置 `MIMOCODE_HOME` 时使用 `<MIMOCODE_HOME>/config/AGENTS.md`，设置 `MIMOCODE_CONFIG_DIR` 时优先检查该目录下的 `AGENTS.md`。不要把项目级 `.mimocode/` 当成用户全局目录。[^mimo]

同时使用多个工具时，将下方 `AGENTS.md` 正文放入各工具的原生入口，或使用其明确支持的共享文件配置。`CLAUDE.md` 的单行适配仅用于 Claude Code。[^claude][^zcode][^mimo]

### 全局规则正文

````markdown
# AGENTS.md

---

## Karpathy 编码准则

减少常见编码错误，与项目指令配合使用。这些准则偏向谨慎；小任务可适当简化流程。

### 1. 编码前思考

**不擅自猜测，把疑问和取舍说清楚。**

- 修改前，先读相关实现和测试。
- 说清假设；有疑问先问清，再实现。
- 有多种理解时，列出差别，不擅自选。
- 有更简单的办法，主动提出；必要时指出当前方案的问题。

### 2. 极简主义

**用最少的代码解决问题，不为猜测中的需求做准备。**

- 不添加需求之外的功能、灵活性或配置项。
- 只用一次的代码，不必包装成通用接口或框架。
- 不处理不可能发生的错误。
- 能用 50 行解决，就别写 200 行。

**检验标准：** 资深工程师会觉得这过于复杂吗？如果是，简化。

### 3. 精准修改

**只改任务需要的内容，只清理自己造成的问题。**

- 先查看已有的未提交改动，不覆盖或回滚用户及其他 Agent 的工作。
- 不顺手修改无关代码、注释或格式，不重构没坏的代码。
- 遵循项目现有风格。
- 清理本次修改造成的无用导入、变量和函数。
- 发现原有且与任务无关的废弃代码，只提醒，不删除。

**检验标准：** 每一处修改，都能说清与当前任务的关系。

### 4. 目标驱动执行

**定义成功标准。循环验证直到达成。**

- 添加校验：先写无效输入的测试，再让它通过。
- 修复 bug：先写能复现问题的测试，再修到通过。
- 重构代码：确认修改前后测试都通过。

对于多步骤任务，说明一个简短的计划：
```
1. [步骤] → 验证: [检查]
2. [步骤] → 验证: [检查]
3. [步骤] → 验证: [检查]
```
同一办法反复失败时，先补证据、查原因，再继续修改。
完成条件要具体，不能只写“让它工作”。

````

---

# 第二层：项目级 AGENTS.md

> 文件位置：`./AGENTS.md`（项目根目录，项目特有）

项目层放**开发规范 + 项目配置**。每个项目独立定制。

```markdown
# AGENTS.md

## 1. 项目原则

- 中文交流。
- 优先复用项目已有代码、工具、API、状态管理和成熟方案，只实现无法复用的最小部分。

## 2. 日志

Agent 按需自主收集证据，记录到 `logs/*.md`。

只保留关键设计依据、测试要点、失败原因和验证结果。最终报告引用同一份证据，不重复抄写。

## 3. 测试实现规范

**验证真实行为，不绑定内部写法。**

- 测试彼此独立，不依赖执行顺序，不污染真实环境。
- 涉及用户目录、配置、缓存或文件时，使用独立临时目录，覆盖相关路径环境变量，结束后清理；禁止读写真实用户环境。
- 随机数、系统时钟、时区/locale 和依赖版本必须可控：固定随机种子，注入可替换时钟，显式设定时区，用 lockfile 锁定依赖版本；禁止依赖真实随机源、真实当前时间或未锁定的依赖版本。
- 稳定文本（输出、屏幕渲染、序列化结果等）优先用快照/黄金主文件比对。失败时按期望与实际的差异定位修复；不得为让测试通过而直接接受新快照。
- 不为覆盖率或形式完整增加低价值测试。
- 先确认相关测试的原有状态，再验证本次改动；涉及公共逻辑时扩大范围。
- 完成规定检查后，不重复运行没有新增信息的检查。

### 单元与集成测试

- 单元测试：检查输入输出、状态变化、边界和异常。
- 集成测试：检查模块间的数据流、接口调用、状态同步和外部交互。
- 优先真实代码和组件。Mock（测试替身）仅用于系统边界，不替代本次要验证的核心逻辑、状态变化或业务判断。
- Bug 修复必须在能稳定复现问题的最低层补回归测试。

### 回归测试（一条命令自动运行）

- 单元、集成与回归用例统一由仓库根目录的一条命令自动运行：非交互，无需人工确认或手测，以退出码和标准输出报告结论。
- 默认无参数运行全部回归用例；支持按名称过滤单个用例排查，但过滤结果不能作为交付验证依据。
- 回归用例遵守上述测试规则，并按第 5 节离线运行、响应固定可复现，不依赖真实密钥。
- 纳入项目已有的其他确定性检查，如架构依赖规则、契约测试、文档示例执行、内存/未定义行为检查；项目没有的不强加。
- Bug 修复、行为变更和接口变化必须补入回归集合。
- 全量通过只是必要条件：关键新增或回归用例仍须按第 4 节证明能发现错误。
- 具体命令与工作目录填入第 7 节「全量验证」行。

### 端到端测试（E2E）

E2E 覆盖关键用户流程，不替代低层测试。主要检查用户最终看到的结果，日志仅辅助定位和交叉验证。

| 场景 | 验证方式 |
|---|---|
| Web | 启动真实项目，用无头浏览器（如 Playwright）模拟操作，检查页面结果和相关数据。 |
| 普通 CLI | 启动真实进程，检查标准输出、错误输出、退出码和文件变化。 |
| 交互式 CLI / TUI | 使用伪终端模拟输入和按键，检查渲染后的屏幕、光标和状态。 |

Windows 交互测试使用 ConPTY，优先用 `node-pty` 启动真实进程、`@xterm/headless` 解析 ANSI 输出并检查屏幕状态。

涉及终端缩放、滚动历史等宿主终端行为时，增加目标终端验证；模拟屏幕通过仅为部分证据。

不得仅因缺少 `tmux`、`script` 等工具就让用户手测。确实无法自动验证时，报告验证缺口。

## 4. 测试质量

按“准备输入 → 执行动作 → 检查结果”组织测试。

- 输入有代表性、可复现，覆盖关键边界。
- 按需检查结果、状态和副作用，如文件是否改动。
- 证明目标路径确实执行，防止测试空跑。
- 关键新增或回归测试须证明能发现错误：保持断言不变，通过修复前后对照或临时破坏被测行为，确认错误时失败、恢复正确行为后通过。
- 需求未变时，不得靠删除测试、放宽断言或绕过检查过关。

高风险模块按需做变异测试（故意改坏代码，检查测试能否发现）。与本任务有关却未被发现的错误必须调查；不改变实际行为的改动可说明后排除，不追求 100% 的分数。

## 5. 外部依赖

API、网络服务按需使用 VCR（录制并回放响应）或等价方案。

- 测试可离线运行，不依赖真实密钥。
- 响应固定、可复现。
- 录制的请求和响应必须去掉密钥、令牌等敏感信息。

## 6. 静态质量

按第 7 节已核实的命令执行 typecheck（类型检查）和 lint（代码检查）。

- 对应检查应通过，不引入未使用代码、未处理异步调用等明显问题。
- 涉及对外接口、Schema 或持久化数据时，按项目适用运行兼容性检查（如 `buf breaking`、`api-extractor`、`cargo semver-checks`），防止破坏性变更；不适用的项目在第 7 节注明原因。
- 区分原有失败与本次新增问题；检查失败或未执行时，不得报告全部通过。

## 7. 项目信息

在实际项目中核实后填写，不猜测命令；不适用的项目注明原因。

- 技术栈、运行时、包管理器：[待填写]
- 核心入口与模块边界：[待填写]
- 必须保持的行为，如接口、配置和已有数据兼容性：[待填写]

| 用途 | 命令与工作目录 |
|---|---|
| 启动 / 构建 | [待填写] |
| 单元 / 集成测试 | [待填写] |
| 回归测试（一条命令全量运行） | [待填写] |
| E2E | [待填写] |
| typecheck / lint | [待填写] |
| 全量验证 | [待填写：一条命令自动运行全部回归测试的命令及适用条件] |

## 8. 参考资料

- Obsidian 本地仓库：`D:\AgentLab\shared\Obsidian`。记忆文档和可沉淀的知识通过 wiki MCP 保存到此处。
- Obsidian GitHub 仓库：[MrSong9957/Obsidian](https://github.com/MrSong9957/Obsidian)，保存个人知识、常用技能和 Agent 代理。若缺失 SKILL 技能和 Agent 代理时，按需下载到 Agent 工具全局目录使用。
- @`project-lessons.md`：项目经验/决策沉淀，按需加载；Agent 执行时，将与本项目直接相关、对后续执行或决策有复用价值的信息（如已确认决策、问题根因与解决方法）统一记录于此，**不得与「7. 项目信息」重复，禁止直接写入 AGENTS.md**，以避免重复踩坑、减少决策偏差。

```

## 相关概念

- [[wiki/agents-md-claude-md|AGENTS.md 与 CLAUDE.md 配置关系]] — Claude Code 单行适配的文档依据与加载机制
- [[wiki/Agent工具初始化配置|Agent 工具初始化配置]] — 引用本模板的两步初始化清单，本模板为其第 1 步
- [[wiki/grill-me最小必要技能安装清单|grill-me 最小必要技能安装清单]] — 同一初始化流程第 2 步安装的技能清单

## 引用索引

[^codex]: https://learn.chatgpt.com/docs/agent-configuration/agents-md — Codex 的全局目录、`CODEX_HOME` 与覆盖文件优先级。
[^claude]: https://code.claude.com/docs/zh-CN/memory — Claude Code 用户指令位置与相对导入路径解析。
[^zcode]: https://zcode.z.ai/cn/docs/agents#项目指令文件 — ZCode 的用户全局 / Workspace 指令路径与导入限制。
[^mimo]: https://github.com/XiaomiMiMo/MiMo-Code/blob/main/packages/cli/src/session/instruction.ts — `globalFiles()` 与 `instructions` 文件加载；https://github.com/XiaomiMiMo/MiMo-Code/blob/main/packages/shared/src/global.ts — XDG 与 `MIMOCODE_HOME/config` 路径解析；https://github.com/XiaomiMiMo/MiMo-Code#configuration — 各平台配置目录。
[^karpathy]: https://github.com/multica-ai/andrej-karpathy-skills — 全局正文所依据的四条编码准则。
