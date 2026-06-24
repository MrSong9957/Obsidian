# wiki-query — 让 AI 能搜你的 Obsidian 知识库

一句话：你有一个 Obsidian 知识库，AI 想查里面的知识。wiki-query 就是中间人——AI 问它，它帮你翻索引，秒回。

---

## 它解决什么问题

**没有它时**：AI 不认识你的知识库，要么靠你手动贴内容，要么瞎猜。每次新会话，之前学到的东西都忘了。

**有了它**：AI 直接搜你的知识库，找到相关文档，读给你听。更重要的是——你可以把 AI 在会话中学到的经验"提炼"到知识库，永久保存，下次会话还能用。

---

## 安装（只需一次）

### 1. 编译

打开终端，运行一键构建脚本（Git Bash / macOS / Linux 通用）：

```bash
cd <你的 vault 根目录>
bash db/wiki-query/build.sh
```

它会 `cargo build --release`，然后把可执行文件拷到固定位置 `db/wiki-query/wiki-query[.exe]`（Windows 是 `.exe`，Mac/Linux 无后缀）。

> 也可以手动编译：`cd db/wiki-query && cargo build --release`，然后手动拷贝 `target/release/wiki-query[.exe]` 到 `db/wiki-query/`。

### 2. 建索引

```bash
cd <你的 vault 根目录>
db/wiki-query/wiki-query.exe sync
```

它会扫描你知识库里所有 `.md` 文件，建一个搜索索引。以后新加了文档，再跑一次就行。

### 3. 告诉 AI 怎么找它

在你项目根目录创建 `.mcp.json`（如果没有的话），command 指向上一步的固定位置：

```json
{
  "mcpServers": {
    "wiki": {
      "command": "<你的 vault 根目录>/db/wiki-query/wiki-query.exe",
      "args": ["serve"],
      "cwd": "<你的 vault 根目录>"
    }
  }
}
```

重启 AI agent（Claude Code / Cursor / MiMo-Code 等），它就能搜你的知识库了。

---

## 5 个工具（AI 自动调，你不用记）

| 工具 | 一句话 | 你什么时候会用到 |
|------|--------|-----------------|
| `wiki_search` | 搜关键词，返回最相关的 10 篇文档 | 你问 AI 一个知识问题，它帮你翻 |
| `wiki_list` | 按标签/类型/目录列出文档清单 | 你让 AI 列出某类全部文档 |
| `wiki_sync` | 更新索引 + 检查有没有重复文档 | AI 刚写了新文档，或者你说"去重一下" |
| `wiki_get_project_rules` | 教 AI 怎么用 wiki-query | AI 第一次接入时，或你说"接入一下 wiki-query" |
| `wiki_check_scope` | 判断文件是否在 wiki 范围内 | AI 提炼记忆前，先判断要不要提炼 |

> **重点**：读写文档是 AI 自己的活，wiki-query 只管搜。就像图书馆管理员不写书，只帮你找书。

---

## 日常怎么用

**你只管用自然语言说话，AI 会自动判断该调哪个工具。** 下面是最常见的场景。

---

### 🔍 查知识

直接问就行，AI 会自动搜知识库：

```
你："我之前怎么配置 Rust 工具链的？"
AI：（自动搜知识库）找到了，你记在 wiki/rust-toolchain.md 里...

你："MCP 相关的文档有哪些？"
AI：（搜 "MCP"）找到 5 篇相关文档...

你："有没有关于安全的笔记？"
AI：（搜 "安全"）找到 wiki/security.md，内容是...
```

也可以指定搜哪里：

```
你："sources/dev/ 目录下有哪些文档？"
AI：（按目录列出）共 12 篇...

你："列出所有 type=concept 的文档"
AI：（按类型筛选）共 28 篇概念文档...
```

---

### ✍️ 记知识

让 AI 帮你把想法记下来：

```
你："把这次会话的关键决策记到知识库"
AI：（查重 → 写文档 → 更新索引）已记录到 wiki/decision-xxx.md。

你："记住：Rust 的 GNU 工具链比 MSVC 更轻量，不需要装 Visual Studio"
AI：（写入知识库）已记录。

你："把刚才调试的踩坑经验整理一下存到知识库"
AI：（整理 → 查重 → 写入）已保存。
```

---

### 🔎 查重 & 去重

```
你："去重一下知识库"
AI：（全量查重）发现 3 对相似文档：
  1. wiki/memory-system.md 和 wiki/cross-session-memory.md 相似度很高
  2. ...
  要合并吗？

你："这篇新文档和已有的重复吗？"
AI：（增量查重）和 wiki/existing.md 有重叠，建议合并。
```

---

### 📋 整理知识库

```
你："知识库里有多少篇文档？"
AI：（wiki_list）共 156 篇。

你："sources/agent/ 下有哪些文件？"
AI：（按目录列出）共 8 篇...

你："标签含 agent 的文档有哪些？"
AI：（按标签筛选）找到 12 篇...

你："最近更新的 10 篇文档是哪些？"
AI：（按时间排序）最近更新的是...
```

---

### 🧠 其他自然语言触发方式

这些说法 AI 也能理解，会自动调对应工具：

| 你说 | AI 自动调 |
|------|----------|
| "帮我找一下关于..." | `wiki_search` |
| "有没有相关文档？" | `wiki_search` |
| "知识库里有没有..." | `wiki_search` |
| "列出所有..." | `wiki_list` |
| "存下来 / 记到知识库" | 写文件 → `wiki_sync` |
| "提炼一下 / 提炼到 wiki" | 读记忆 → 查重 → 写文件 → `wiki_sync` → 删源 |
| "去重 / 有没有重复" | `wiki_sync(scope=full)` |
| "同步一下索引" | `wiki_sync` |
| "更新一下索引" | `wiki_sync` |
| "接入一下 wiki-query" | `wiki_get_project_rules` |

---

### ⚡ 记知识的完整流程

```
你："我刚学了 jieba 分词，帮我记到知识库"

AI：（自动执行以下步骤）
  1. wiki_search("jieba 分词") → 查重，看有没有已存在的文档
  2. 发现没有重复
  3. 用文件工具写 wiki/jieba-tokenizer.md（frontmatter 带 source 溯源）
  4. wiki_sync() → 索引更新 + 确认无重复
  5. "已记录到 wiki/jieba-tokenizer.md，索引已更新。"
```

---

## 🧬 提炼：把 AI 的临时记忆变成永久知识

这是 wiki-query 最核心的场景。

### 它解决什么问题

每个 AI 工具都有自己的"记忆"：Claude Code 有 `.claude/`，Cursor 有 `.cursor/rules/`，MiMo-Code 有 `.zcode/`。这些记忆是临时的——换个会话、换个项目就可能丢失。

**提炼就是把其中有长期价值的内容，搬到你的 Obsidian 知识库里永久保存。**

### 什么时候该提炼

- 一次会话里敲定了重要技术决策
- 踩了个坑，摸索出了正确的配置方法
- 调试了很久才解决的问题
- 任何你觉得"下次还会用到"的东西

### 怎么触发

直接用自然语言说：

```
你："把这次会话的关键决策记到知识库"
你："把刚才踩的坑提炼一下"
你："这次调试的经验提炼到 wiki"
你："提炼我的记忆"
```

### 提炼流程（AI 自动执行）

```
1. AI 读自己的记忆（.claude/ 或 .cursor/ 等）
2. 逐篇 wiki_search 查重（避免和已有文档重复）
3. 写入 wiki/ 目录（frontmatter 带 source 字段溯源到原始记忆）
4. wiki_sync 更新索引 + 拿查重报告
5. 删除原始记忆文件（避免下次重复提炼）
6. 告知你"已提炼 N 篇"
```

### 溯源

每篇提炼出来的文档，frontmatter 里都有 `source` 字段，指向原始记忆文件。比如：

```yaml
---
name: Rust GNU 工具链安装
source: [[.claude/memory/rust-gnu-setup.md]]
---
```

这样你随时能追溯"这篇知识是从哪来的"。

### 记忆分工：什么该放 wiki，什么留 agent

| 内容 | 放哪里 | 为什么 |
|------|--------|--------|
| 这次会话的临时偏好 | agent 自带记忆 | 会话结束就没用了 |
| 会话进度（做到哪一步） | agent 自带记忆 | 下次会话重新开始 |
| 长期技术决策 | wiki 知识库 | 换会话、换 agent 都要记得 |
| 踩坑经验 | wiki 知识库 | 下次还会遇到 |
| 配置方法 | wiki 知识库 | 跨项目复用 |

**判断标准：这次会话结束后还有用吗？有 → 提炼到 wiki。没有 → 留在 agent 记忆里。**

---

## 🔧 首次接入：让 AI 学会用 wiki-query

AI 第一次接触 wiki-query 时，不知道怎么用。你需要让它"学"一下。

### 怎么触发

```
你："接入一下 wiki-query"
你："把 wiki-query 的使用规则注入到项目里"
```

AI 会自动调 `wiki_get_project_rules`，拿到一份使用说明书，然后：

1. 检查当前项目根的 `CLAUDE.md`（或 `AGENTS.md`）里有没有 `<!-- wiki-query-rules -->` 标记
2. 没有 → 把规则追加到文件末尾
3. 有 → 跳过（避免重复）

### 注入后 AI 就知道

- 什么时候该调 `wiki_search`（找知识时）
- 什么时候该调 `wiki_sync`（写完文件后）
- 什么时候该调 `wiki_sync(scope=full)`（去重时）
- frontmatter 怎么写（name/description/type/updated/tags/source）
- 提炼流程的 6 个步骤

### 不同 AI 工具的注入目标

| AI 工具 | 注入到 | 命令 |
|---------|--------|------|
| Claude Code | `CLAUDE.md` | `wiki_get_project_rules(agentType="claude")` |
| Cursor | `AGENTS.md` | `wiki_get_project_rules()` |
| MiMo-Code | `AGENTS.md` | `wiki_get_project_rules()` |
| Codex | `AGENTS.md` | `wiki_get_project_rules()` |
| OpenCode | `AGENTS.md` | `wiki_get_project_rules()` |

> 注入后规则存在你项目的 Markdown 文件里，改规则直接改那个文件就行，不用重编译。

---

## 索引更新

**核心原则：Markdown 是正本，索引是副本。索引丢了无所谓，重建就行。**

### 什么时候索引会过时？

- 你在 Obsidian 里手动编辑了文档 → 索引不会自动更新
- AI 写了新文档 → 它会自动调 `wiki_sync` 更新索引

### 怎么更新索引？

**方法 1：手动跑（偶尔编辑时用）**

```bash
cd E:\Files\Obsidian
db/wiki-query/target/release/wiki-query.exe sync
```

**方法 2：配 git hook 自动跑（推荐，重度编辑时用）**

在 `.git/hooks/post-commit` 里加：

```sh
#!/bin/sh
cd "$(git rev-parse --show-toplevel)"
./db/wiki-query/target/release/wiki-query.exe sync > /dev/null 2>&1
```

以后每次 `git commit`，索引自动更新。

### 索引坏了怎么办？

删掉重建：

```bash
rm db/wiki.db
cd E:\Files\Obsidian
db/wiki-query/target/release/wiki-query.exe sync
```

---

## 故障排除

| 问题 | 原因 | 解决 |
|------|------|------|
| AI 说搜不到东西 | 索引是空的 | 跑一次 `wiki-query sync` |
| AI 写了文档但搜不到 | 忘了更新索引 | 让 AI 调 `wiki_sync` |
| 启动时警告"找不到 wiki/ 目录" | cwd 设错了 | 确认 `.mcp.json` 里的 `cwd` 指向知识库根目录 |
| 中文搜不到 | 分词问题 | 跑 sync，系统会用中文分词重建索引 |

---

## 它怎么工作（技术细节）

```
你说话 → AI 想查知识 → 调 wiki_search → 翻 SQLite 索引 → 返回结果 → AI 回答你
                                    ↑
                           索引由 wiki_query sync 维护
                           来源是你知识库里的 .md 文件
```

- **传输方式**：本地 stdio（AI 和 wiki-query 通过管道对话，不走网络）
- **存储**：SQLite 数据库（`db/wiki.db`），可随时从 Markdown 重建
- **分词**：用 jieba 做中文分词，搜"记忆"能命中"记忆系统"
- **查重**：用候选文档的 name 做 FTS5 查询，在全库中找 BM25 分数 ≤ -2.0 的相似文档

---

## 配套 AI Agent 支持

| Agent | 配置文件 | 状态 |
|-------|---------|------|
| Claude Code | `.mcp.json` | ✅ |
| MiMo-Code | `~/.config/opencode/mimocode.jsonc` | ✅ |
| Cursor | `.cursor/mcp.json` | ✅ |
| ZCode | `.zcode/mcp.json` | ✅ |
| OpenCode | `~/.config/opencode/config.json` | ✅ |
