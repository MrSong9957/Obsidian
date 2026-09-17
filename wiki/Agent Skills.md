---
name: Agent Skills：按任务加载操作说明
description: 理解 Skills 的三层加载方式、触发条件，以及如何减少无关上下文并配合提示缓存。
type: hybrid
updated: 2026-09-17
tags: [agent-skills, prompt-engineering, progressive-disclosure, context-engineering, prompt-caching]
source: self
---

你让 Agent 做 PPT，它却同时读入退款规则、编程规范和各种文档说明。这些无关内容既占用 token（模型处理文字的单位），也可能干扰当前任务。

**动态提示词，就是根据任务，把需要的说明补进上下文。** Agent Skills 把这些说明整理成可复用的任务包：包含操作步骤，也可以附带参考文档、模板和脚本。模型读说明，实际操作仍通过工具完成。[^1]

### 1. 三层加载：先看目录，再读具体方法

**渐进式披露（Progressive Disclosure）**，就是先给足够做选择的信息，需要时再补充细节。Skills 按这个思路分层组织。[Agent Skills 规范](https://agentskills.io/specification#progressive-disclosure)

| 层级       | 加载什么                    | 解决什么问题        |
| -------- | ----------------------- | ------------- |
| 第一层：目录摘要 | Skill 的名称和简短描述          | 有哪些能力？这次该用哪个？ |
| 第二层：核心流程 | 选中 Skill 的完整 `SKILL.md` | 这项任务应该怎样做？    |
| 第三层：详细资料 | 被引用的子文档、模板、脚本等          | 当前步骤的具体方法是什么？ |

通常先让模型看到目录，选中某个 Skill 后才加载正文；遇到具体问题，再读取相关子文档或执行脚本。

例如，任务是“把 PDF 论文做成 PPT”：

1. **看目录选能力**：可以组合 PDF 和 PPTX 两个 Skill，分别负责读取论文、制作幻灯片。
2. **读核心流程**：了解如何提取内容、选择制作方法。原文提到的 `markitdown`，是微软的文档转 Markdown 工具。
3. **按需要查细节**：采用 HTML 模板制作，就读 `html2pptx.md`；需要修改已有 PPTX 的内部结构，就查 `reference.md` 中的 XML 格式说明。

PPTX 内部包含描述幻灯片内容的 XML 文件，因此相关编辑流程可以是：解压文件、修改 XML、重新打包。**这是具体任务的方法，不是每次制作 PPT 都必须执行的步骤。**

### 2. `description` 要让模型知道“什么时候用”

按 Agent Skills 通用规范，每个 Skill 至少有一个 `SKILL.md`。文件开头是 **YAML frontmatter**，也就是用两条 `---` 包住的元数据；后面是 Markdown 格式的操作说明。[文件格式规范](https://agentskills.io/specification#skill-md-format)

文件开头示例：

```yaml
---
name: pptx
description: 创建或修改 .pptx 演示文稿时使用；只提取 PDF 内容、不制作 PPT 时不使用。
---
```

* `name`：Skill 的名称。
* `description`：帮助模型判断当前任务是否需要它。
* 后面的正文：写核心步骤，并指向需要时再读的详细资料。

如果描述只写“帮助处理文档”，范围太宽，普通 PDF 任务也可能误触发 PPT Skill。

**有效的描述要说清适用任务，必要时补充不适用的情况。** 这些边界直接写进 `description` 即可，不需要另造字段；同时保持简短，因为目录会占用上下文。

### 3. 谁来决定加载 Skill

常见的触发方式有两种。Claude Code 同时支持用户直接调用和模型按任务选择。[Claude Code 文档](https://code.claude.com/docs/en/skills)

| 方式   | 加载过程                                     |
| ---- | ---------------------------------------- |
| 用户指定 | 在支持斜杠命令的客户端输入 `/pptx`，客户端处理命令并加载说明。      |
| 模型选择 | 模型看过目录后，判断需要某个 Skill，再调用激活工具或文件读取工具获取说明。 |

模型自主选择时，通常需要多一次“请求工具读取、收到结果、继续处理”的往返。

**正文放进哪种消息，由客户端或框架决定。** 原文描述的 Claude Code 实现把正文作为 `user` 消息加入会话，激活工具的返回值只是启动提示；使用通用文件读取工具的实现，则可能让正文通过 `tool result`（工具返回结果）进入上下文。这些不是 Skills 格式统一规定的行为。

### 4. 为什么这种设计有利于缓存

按需加载首先省掉了不相关的内容。要进一步复用已有计算，还需要保持请求开头的内容不变：

* 目录、系统规则等稳定内容保持原样。
* 新加载的 Skill 正文和后续消息追加在后面。
* 已有的相同前缀可以复用缓存，新增加的部分仍需要计算。

这种跨请求复用叫 **Prompt Cache（提示缓存）**。它要求被复用的前缀完全一致，并满足服务商的缓存条件；仅仅“意思没变”不够。[提示缓存机制](https://platform.claude.com/docs/en/build-with-claude/prompt-caching#how-prompt-caching-works)

因此，**按需加载负责少放无关内容，缓存负责少算重复内容**。Skill 一旦读入上下文，仍会占用空间，核心流程也需要保持精简。

## 引用索引

[^1]: 用户提供原文：2.5 动态提示词与 Agent Skills、2.5.1 Skills：领域能力的可组合单元（原文未入库）；官方链接见正文。
