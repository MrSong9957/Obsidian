# wiki-query

Obsidian 知识库的**纯索引 MCP 服务**。让 Claude Code、Codex、OpenCode、MiMo-Code、Cursor、ZCode 等 AI agent 通过 MCP 协议查询知识库索引——agent 用自己的文件工具读写 Markdown,wiki-query 负责 SQLite 全文检索和查重。

**本地 stdio 传输,无云无服务器。** agent 拉起 `wiki-query.exe` 作为子进程,通过管道通信,所有数据在单机完成。

---

## 它做什么

把你的 Obsidian 知识库变成 AI agent 的**可检索的记忆聚合点**:

- **没有它时**:agent 不认识你的知识库,要么靠 CLAUDE.md 提示手动 grep,要么瞎猜。
- **有了它**:agent 调 wiki-query 查索引定位知识,用文件工具读写 vault 文档,你全程用自然语言,不碰命令行。

**核心理念(v5)**:凡是 agent 自己能干的(读写文件、编辑正文、智能决策),wiki-query 绝不重复。wiki-query 只做 agent 干不了的——维护 SQLite 索引、全文检索、查重报告。

### 4 个 MCP 工具

| 工具 | 做什么 | agent 何时用 |
|------|--------|-------------|
| `wiki_search` | FTS5 全文检索(支持中文),返回 top 10 元数据 | 找知识 / 写入前查重 |
| `wiki_list` | 按标签/类型/目录结构化过滤 | 浏览某类全部文档 |
| `wiki_sync` | 同步索引 + 查重报告(scope=incremental/full) | 写完文件后;用户说"去重"时用 full |
| `wiki_get_project_rules` | 返回可注入到项目根指令文件的规则文本 | 首次接入 wiki-query 时 |

> 注:读写 vault 的 Markdown 是 agent 用自己的文件工具干的事,wiki-query 不碰正文。

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

1. **agent 用自己的文件工具读写 Markdown** —— 这是 agent 的活,wiki-query 不碰正文。改完文件后调 `wiki_sync` 让索引跟上。
2. **`wiki_search` / `wiki_list` 读 SQLite FTS5 索引** —— 索引是 Markdown 的"快照",agent 改了 Markdown 后需调 `wiki_sync` 才能搜到。
3. **`wiki_sync` 是唯一的索引入口** —— agent 写完文件调 sync,索引跟上 + 顺带拿查重报告(检测新文档是否与已有文档重复)。

### 你修改了 Markdown 后,该怎么做?

**让 agent 调 `wiki_sync`** —— agent 写完文件后调 sync,索引跟上 + 拿查重报告。这是 v5 的唯一索引入口。

两种触发 sync 的方式:

**方式 1:agent 写完文件后自动调 `wiki_sync`(MCP 工具)**

agent 在对话里写完 vault 文档后,主动调 wiki_sync。这是常态。

**方式 2:手动 sync(运维,命令行)**

```bash
cd E:\Files\Obsidian
db/wiki-query/target/release/wiki-query.exe sync
```

输出类似:`Scan complete: 375 files found, 3 updated, 0 pruned, N similar pairs (scope=incremental)`

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

重启 Claude Code(或对应 agent),用 `/mcp` 命令应看到 wiki server + 4 个工具(search/list/sync/get_project_rules)。

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
> agent(自动调 `wiki_search "OpenCode 桥接"` → 用文件工具读 `wiki/opencode-bridge.md` 全文):找到文档,回答你...

**记知识(提炼工作流):**
> 你:"把这次会话的关键决策记到知识库"
> agent(调 `wiki_search` 查重 → 用文件工具写 `wiki/xxx.md` → 调 `wiki_sync`):已记录,索引已更新

**去重:**
> 你:"去重一下知识库"
> agent(调 `wiki_sync(scope="full")`):发现 3 对相似文档,分别是...我来合并/删除

### 提炼工作流(6 步)

提炼是 wiki-query 最核心场景。**agent 用自己的文件工具读写 vault Markdown,wiki-query 只管索引。**

```
用户:"提炼一下我的记忆"
  ↓
1. agent 全量读自己记忆系统(文件工具)
2. 逐篇调 wiki_search 查重(固定流程)
3. 用文件工具写 vault Markdown(frontmatter 按 project-rules.md 约定)
4. 调 wiki_sync() 让索引跟上 + 拿查重报告
5. 删除自己记忆系统里的源文件(避免重复提炼)
6. 告知用户"已提炼 N 篇,删源 M 个"
```

### 查重报告

`wiki_sync` 返回的查重报告(scope=incremental 查新文档,scope=full 查全库):
- 检测新入库文档与已有文档的 bm25 相似对
- **只提示不自动合并**——合并(保留哪篇、怎么融合)是 agent 的智能决策
- agent 若写入前忘了 wiki_search,sync 的报告能兜底

### 首次接入:规则注入

agent 在**别的项目**干活时,首次接 wiki-query 要知道"怎么用"。调一次:

```
wiki_get_project_rules(agentType="claude")
  → 返回规则文本(来自 vault 内 templates/project-rules.md)
  → agent 检查当前项目根的 CLAUDE.md/AGENTS.md 是否已含
    <!-- wiki-query-rules --> 标记
  → 未含则追加规则文本(带标记),已含则跳过
```

规则文本含:frontmatter 约定 + 6 步提炼流程 + 删源原则 + 查重指引。改规则直接改 `templates/project-rules.md`,不用重编译。

---

## 工作流总结

### 你编辑知识库(人工,Obsidian)

```
Obsidian 里编辑 Markdown → (可选)git commit → (可选)hook 自动 sync
```

- agent 想搜到:需 sync(或配 git post-commit hook 自动触发)

### agent 读写知识库(自动)

```
agent 对话 → wiki_search 定位 → 文件工具读写 Markdown → wiki_sync 同步索引
```

- agent 读写 vault:用 agent 自己的文件工具,wiki-query 不碰正文
- 索引同步:agent 写完调 `wiki_sync`,SQLite 索引跟上 + 查重报告

### 两个数据流

| 操作方 | search 看到什么 | 何时最新 |
|--------|----------------|---------|
| 你改 Markdown 后 | ❌ 陈旧(需 sync) | sync 后 |
| agent 写完后调 sync | ✅ 最新(sync 内部更新) | 立即 |

---

## 故障排除

| 症状 | 原因 | 解决 |
|------|------|------|
| `wiki_search` 永远空 | 未 sync 或 cwd 错 | 跑 `wiki-query sync`;确认 cwd=vault 根 |
| agent 写了文件但 search 搜不到 | 忘了调 wiki_sync | 让 agent 写完调 `wiki_sync` |
| `wiki_get_project_rules` 返回警告 | 未找到 templates/project-rules.md | 确认 vault 根有该文件 |
| 启动警告"未找到 wiki/ 或 sources/" | cwd 不是 vault 根 | 设 `WIKI_VAULT_ROOT` 或确保 cwd 正确 |
| 启动警告"未找到 wiki/ 或 sources/" | cwd 不是 vault 根 | 设 `WIKI_VAULT_ROOT` 或确保 cwd 正确 |
| 中文搜不到 | sync 未跑 / 分词 | 跑 sync;FTS5 用 jieba 细分,单字也能命中 |
| 多 agent 同时访问报错 | 并发锁 | 已用 WAL + busy_timeout 5s,通常自动重试 |

---

## 架构一览

```
AI agent (Claude Code/Codex/OpenCode/MiMo/Cursor/ZCode)
    │ MCP stdio(本地子进程,无云无服务器)
    │
    │ agent 用自己的文件工具读写 vault Markdown(权威源)
    │ agent 只在需要「查索引/同步索引/拿规则」时调 wiki-query
    ▼
wiki-query serve (Rust 二进制,纯索引服务)
    │
    ▼
┌──────────────────────────────────────┐
│  core: search / list / find_similar  │
│  sync: run_with_scope(查重)           │
│  + frontmatter 解析(sync 提取字段)    │
│  + policy 权限(sync 校验路径)         │
└──────────────────────────────────────┘
    │                       │
    ▼ search/list/sync      ▼ sync 写索引
SQLite (FTS5 派生索引)      Markdown (权威源)
db/wiki.db (可随时重建)     sources/ wiki/ templates/
```

**职责边界:agent 读写字数正文 + 智能决策;wiki-query 只管 SQLite 索引。**

详细设计见 `docs/superpowers/specs/2026-06-20-wiki-mcp-v5-index-only.md`。
