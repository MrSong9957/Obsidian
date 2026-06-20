# wiki-query

Obsidian 知识库的 MCP 记忆系统中枢。让 Claude Code、MiMo-Code、Cursor、ZCode 等 AI agent 通过 MCP 协议读写 Obsidian 知识库——像调用内置工具一样自然。

**本地 stdio 传输,无云无服务器。** agent 拉起 `wiki-query.exe` 作为子进程,通过管道通信,所有数据在单机完成。

---

## 它做什么

把你的 Obsidian 知识库变成 AI agent 的**可查可写的资料库**:

- **没有它时**:agent 不认识你的知识库,要么靠 CLAUDE.md 提示手动 grep,要么瞎猜。
- **有了它**:agent 自动调用工具查知识、读文档、记新知识,你全程用自然语言,不碰命令行。

### 5 个 MCP 工具

| 工具 | 做什么 | agent 何时用 |
|------|--------|-------------|
| `wiki_search` | FTS5 全文检索(支持中文),返回匹配文档元数据列表 | 需要查找某主题/概念/技术的知识 |
| `wiki_read` | 读取指定文档完整内容 | 看了元数据想读全文 |
| `wiki_write` | 写入/更新文档(四态决策:ADD/UPDATE/NOOP/CONFLICT) | 记录新知识或更新现有文档 |
| `wiki_list` | 按标签/类型/目录结构化过滤 | 浏览某类全部文档 |
| `wiki_patch` | 按 heading 局部编辑文档某个 section(避免整篇重写) | 修改长文档某一节,而非重写全文 |

---

## ⚠️ 最重要的概念:Markdown 是权威源,SQLite 是索引

这是理解整个系统行为的关键。

```
   你在 Obsidian 编辑 Markdown(权威源,git 跟踪)
                    │
                    │  wiki-query sync(手动 或 git hook 自动)
                    ▼
   SQLite (db/wiki.db,FTS5 索引,可随时重建)
                    ▲
                    │  查询走这里
   wiki_search ─────┘
```

### 三个事实

1. **`wiki_read` 实时直读 Markdown 文件** —— 你刚在 Obsidian 改完,Claude 立刻能读到最新内容。不经过 SQLite。
2. **`wiki_search` 读 SQLite FTS5 索引** —— 索引是 Markdown 的"快照",改了 Markdown 后索引不会自动更新。
3. **`wiki_write`(AI 写)即时同步** —— AI 调用 write 写入 Markdown 后,该文件立即同步进索引,马上可搜到。

### 你修改了 Markdown 后,该怎么做?

**这取决于你想让 agent 用哪个工具看到你的改动。**

#### 情况 A:你只想让 agent `wiki_read` 读全文

**什么都不用做。** read 实时直读文件,你的改动立即可见。

#### 情况 B:你想让 agent `wiki_search` 能搜到你的改动

**需要跑一次 `wiki-query sync`** 重建索引。

两种方式:

**方式 1:手动 sync(随时)**

```bash
cd E:\Files\Obsidian
db/wiki-query/target/release/wiki-query.exe sync
```

输出类似:`Scan complete: 375 files found, 3 updated, 0 pruned`

**方式 2:git post-commit hook(自动,推荐)**

配 hook 后,每次 `git commit` 自动跑 sync:

`.git/hooks/post-commit`:

```sh
#!/bin/sh
# 在 git-lfs 钩子之后追加(若已有 lfs hook,请加在末尾)
cd "$(git rev-parse --show-toplevel)"
./db/wiki-query/target/release/wiki-query.exe sync > /dev/null 2>&1
```

这样你的工作流(改 Markdown → git commit)会自动触发索引更新,agent 立即可搜到新内容。

### 为什么不自动监听文件?

MCP server **没有文件监听**(file watcher)。原因:
- Obsidian 桌面应用自己会改文件、AI 会写文件,多源写入 + 实时监听 + 索引更新会引入复杂的一致性问题和性能开销
- 你的工作流天然有 git commit 这个"改动定稿"的时机,post-commit hook 在此刻同步最自然、最安全

> 如果你不在 Obsidian 应用里频繁编辑(只是偶尔手动改文件),手动 sync 就够。如果你重度编辑 + 频繁 commit,配 hook 最省心。

### 索引丢了/坏了怎么办?

**无所谓,重建即可。** SQLite 是 Markdown 的派生数据,随时可由 `sync` 全量重建。Markdown 文件本身(git 跟踪)是真正的权威源,永不丢。

```bash
rm db/wiki.db && wiki-query sync   # 完全重建
```

---

## 如何接入

### 第 1 步:编译(一次性)

```bash
cd E:\Files\Obsidian\db\wiki-query
cargo build --release
# 产物:db/wiki-query/target/release/wiki-query.exe
```

> release 首次编译较慢(rmcp + tokio 优化,约 30-60 秒)。日常开发可用 `cargo build`(debug 版,编译快)。

### 第 2 步:首次同步索引

```bash
cd E:\Files\Obsidian
db/wiki-query/target/release/wiki-query.exe sync
```

### 第 3 步:配置 MCP

在你的 AI agent 配置文件里加 wiki server。**关键:`cwd` 必须是 Obsidian vault 根**(含 `wiki/` `sources/` 目录),否则工具会操作错误目录。

#### Claude Code

仓库根 `.mcp.json`:

```json
{
  "mcpServers": {
    "wiki": {
      "command": "E:\\Files\\Obsidian\\db\\wiki-query\\target\\release\\wiki-query.exe",
      "args": ["serve"],
      "cwd": "E:\\Files\\Obsidian"
    }
  }
}
```

#### MiMo-Code / OpenCode

`~/.config/opencode/mimocode.jsonc`:

```jsonc
{
  "mcp": {
    "wiki": {
      "type": "local",
      "command": ["E:\\Files\\Obsidian\\db\\wiki-query\\target\\release\\wiki-query.exe", "serve"],
      "enabled": true,
      "environment": {
        "WIKI_VAULT_ROOT": "E:\\Files\\Obsidian"
      }
    }
  }
}
```

#### Cursor / ZCode

格式类似 Claude Code,`env` 里设 `WIKI_VAULT_ROOT`:

```json
{
  "mcpServers": {
    "wiki": {
      "command": "E:\\Files\\Obsidian\\db\\wiki-query\\target\\release\\wiki-query.exe",
      "args": ["serve"],
      "env": { "WIKI_VAULT_ROOT": "E:\\Files\\Obsidian" }
    }
  }
}
```

### 第 4 步:重启 agent,验证

重启 Claude Code(或对应 agent),用 `/mcp` 命令应看到 wiki server + 5 个工具。

或用 MCP Inspector 可视化测试:

```bash
npx @modelcontextprotocol/inspector E:\Files\Obsidian\db\wiki-query\target\release\wiki-query.exe serve
```

---

## 如何使用(日常)

**配好之后,正常对话即可。** agent 会自动决定何时调用 wiki 工具。

### 示例对话

**问知识:**
> 你:"我之前怎么配置 OpenCode 桥接的?"
> Claude(自动调 `wiki_search "OpenCode 桥接"` → `wiki_read`):找到 `wiki/opencode-bridge.md`,回答你...

**记知识:**
> 你:"记一下,MiMo-Code 的 memory 是 SQLite FTS5"
> Claude(自动调 `wiki_search` 查重 → `wiki_write`):已记录到 `wiki/mimo-memory.md`(ADD 新建)

**遇冲突时:**
> 你:"记一下 Agent 架构设计"
> Claude(调 `wiki_write` 返回 CONFLICT):发现已有 `wiki/agent.md`(相似主题)。是更新它,还是新建?

### 写入的四态决策

`wiki_write` 不是无脑写,会先查重:

| 决策 | 触发条件 | 行为 |
|------|---------|------|
| **ADD** | 无相似文档,path 是新的 | 新建文件 |
| **UPDATE** | path 已存在(你指定同路径=更新) | 覆盖正文,**自动保留旧 source 等扩展字段** |
| **NOOP** | 已存在同名文档(跨路径) | 跳过,不写入,避免重复 |
| **CONFLICT** | 有相似但不同名文档 | 不落盘,返回候选让你/agent 决策 |

### 三层目录权限

工具自动校验路径权限(对应 AGENTS.md):

| 目录 | 读 | 写 | 删 |
|------|----|----|-----|
| `sources/` | ✅ | ✅ | ❌ |
| `wiki/` | ✅ | ✅ | ✅(deprecated 标记) |
| `templates/` | ✅ | ❌ | ❌ |

路径含 `..`(穿越)或绝对路径一律拒绝。

---

## 从 agent 记忆提炼到知识库

agent(Claude Code / MiMo / Cursor 等)通常有自己的项目记忆——瞬时偏好、会话进度、临时笔记。这些默认不进 Obsidian 知识库。但其中**长期有价值**的部分(技术决策、配置方法、踩坑经验)值得提炼沉淀。

**触发方式**:用户主动说一句话,例如:

> "把当前项目记忆提炼到知识库"  
> "这个会话里的踩坑记到 wiki 里"

**不会自动触发**——避免把噪声(临时偏好、未定型的想法)灌进知识库。提炼的判断在 agent(LLM),不在 wiki-query。

### agent 执行流程

```
1. agent 读自己的项目记忆(agent 知道存储位置)
2. agent 用 LLM 挑出长期有价值内容(技术决策 / 配置方法 / 踩坑)
3. agent 调 wiki_write 写入,frontmatter 带 source 溯源
```

第 3 步用现成的 `wiki_write`,wiki-query **不需要新工具**。提炼的智能在 agent 端,wiki-query 只负责接收 + 四态决策查重。

### 提炼 vs 原样记录

| 场景 | 用哪个工具 | frontmatter |
|------|-----------|-------------|
| 把一段原始资料归档到 `sources/` | `wiki_write` | type=reference,source 指向原始来源 |
| 把 agent 记忆里的**要点**总结成 wiki 页 | `wiki_write`(agent 先提炼) | type=concept/summary,source 指向 agent 记忆出处 |
| 修改长文档某一节(如更新配置示例) | `wiki_patch` | — |

### 分工原则(详见 AGENTS.md「记忆系统分工」)

- **瞬时偏好 / 会话进度** → agent 自带记忆(临时)
- **长期技术知识 / 需溯源的决策** → wiki-query(永久 + 溯源)

---

## 工作流总结

### 你编辑知识库(人工)

```
Obsidian 里编辑 Markdown → (可选)git commit → (可选)hook 自动 sync
```

- 只想 read:无需任何操作
- 想 search 搜到:commit(触发 hook)或手动 `wiki-query sync`

### AI 读写知识库(自动)

```
Claude 对话 → 自动调 wiki 工具 → read 实时 / write 后即时索引 / search 走 SQLite
```

- AI 写:`write` 内部自动同步索引,立即可搜
- AI 读:直接读文件,永远最新

### 两个数据流

| 操作方 | read 看到什么 | search 看到什么 |
|--------|--------------|----------------|
| 你改 Markdown 后 | ✅ 立即最新(直读文件) | ❌ 陈旧(需 sync) |
| AI write 后 | ✅ 立即最新(直读文件) | ✅ 立即最新(write 内部 sync) |

---

## 故障排除

| 症状 | 原因 | 解决 |
|------|------|------|
| `wiki_search` 永远空 | 未 sync 或 cwd 错 | 跑 `wiki-query sync`;确认 cwd=vault 根 |
| 改了 Markdown 但 search 搜不到 | 索引未更新 | 跑 `sync`(或配 hook) |
| `wiki_write` 报 PolicyViolation | 路径在 templates/ 或含 `..` | path 必须在 sources/ wiki/ 下 |
| `wiki_write` 报 InvalidFrontmatter | 字段缺失或 type 非法 | type 须为白名单值(concept/reference/summary/...) |
| 启动警告"未找到 wiki/ 或 sources/" | cwd 不是 vault 根 | 设 `WIKI_VAULT_ROOT` 或确保 cwd 正确 |
| 中文搜不到 | sync 未跑 / 分词 | 跑 sync;FTS5 用 jieba 细分,单字也能命中 |
| 多 agent 同时访问报错 | 并发锁 | 已用 WAL + busy_timeout 5s,通常自动重试 |

---

## 架构一览

```
AI agent (Claude Code/MiMo/Cursor/ZCode)
    │ MCP stdio(本地子进程,无云无服务器)
    ▼
wiki-query serve (Rust 二进制)
    │ 调用 core 层(单一真相源)
    ▼
┌─────────────────────────────────┐
│  core: read / search / write / list / patch  │
│  + frontmatter 校验 + policy 权限     │
│  + write 四态决策                     │
└─────────────────────────────────┘
    │                       │
    ▼ read/write            ▼ search/sync
Markdown (权威源)        SQLite (FTS5 派生索引)
sources/ wiki/           db/wiki.db (可重建)
templates/               
```

详细设计见 `docs/superpowers/specs/2026-06-19-wiki-memory-system-design.md`。
