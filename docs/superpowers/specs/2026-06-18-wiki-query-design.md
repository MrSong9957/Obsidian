# Wiki Query - Rust SQLite 查询工具设计文档

## 概述

用 Rust 实现一个 SQLite 查询工具，让 Claude Code 能快速查找 Obsidian 仓库中的文档。

**核心逻辑**：
- 传统方式：每次查询都要"翻书"（读文件），消耗大量 token
- 数据库方式：先查"目录"（元数据）→ 定位 → 再查"内容"（正文）

## 需求

| 需求 | 决策 |
|------|------|
| 查询模式 | search（返回元数据列表）、show（返回元数据+正文） |
| 搜索范围 | 只搜索元数据（name + description + tags） |
| 输出格式 | JSON |
| 数据库位置 | db/wiki.db |
| Rust 项目位置 | db/wiki-query/ |
| 同步方式 | Git Hook（post-commit） |
| 同步策略 | 首次全量扫描，后续增量同步（基于文件修改时间） |

## 架构设计

```
┌─────────────────────────────────────────────────────────┐
│                    CLI 入口 (main.rs)                    │
│  - 参数解析                                              │
│  - 命令分发                                              │
└─────────────────────────────────────────────────────────┘
                          │
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  db.rs          │ │  sync.rs        │ │  models.rs      │
│  - 建表         │ │  - 扫描文件     │ │  - Document     │
│  - 查询         │ │  - 解析 MD      │ │  - SearchResult │
│  - 插入/更新    │ │  - 增量同步     │ │  - Frontmatter  │
└─────────────────┘ └─────────────────┘ └─────────────────┘
          │               │
          ▼               ▼
┌─────────────────────────────────────────────────────────┐
│                    SQLite 数据库                         │
│  db/wiki.db                                             │
└─────────────────────────────────────────────────────────┘
```

**模块职责**：

| 模块 | 职责 |
|------|------|
| `main.rs` | CLI 入口，解析参数，分发命令 |
| `db.rs` | 数据库操作（建表、查询、插入、更新） |
| `sync.rs` | 文件同步（扫描、解析、增量更新） |
| `models.rs` | 数据结构定义（Document、SearchResult） |

## 数据库设计

```sql
CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,      -- 相对路径，如 "wiki/agent.md"
    name TEXT,                      -- frontmatter name
    description TEXT,               -- frontmatter description
    type TEXT,                      -- frontmatter type
    updated TEXT,                   -- frontmatter updated
    tags TEXT,                      -- JSON 数组字符串，如 '["agent","dev"]'
    content TEXT,                   -- 完整正文（第二个 --- 之后的内容）
    word_count INTEGER,             -- 正文字数
    modified_at TEXT,               -- 文件修改时间（增量同步用）
    content_hash TEXT               -- 文件内容 hash（可选，备用）
);

CREATE INDEX idx_name ON documents(name);
CREATE INDEX idx_type ON documents(type);
```

**字段说明**：

| 字段 | 用途 |
|------|------|
| `path` | 主键，文件相对路径 |
| `name` | 搜索字段 |
| `description` | 搜索字段 |
| `tags` | 搜索字段，JSON 数组 |
| `content` | show 命令返回 |
| `modified_at` | 增量同步判断 |

## 查询接口设计

**命令设计**：

```rust
enum Command {
    Search(String),      // 搜索元数据
    Show(String),        // 查看完整内容
    Sync,                // 同步数据库
}
```

**调用方式**：

```bash
# 搜索（返回元数据列表）
wiki-query search "agent"

# 查看（返回元数据+正文）
wiki-query show wiki/agent.md

# 同步（首次全量，后续增量）
wiki-query sync
```

**SQL 查询**：

```rust
// search：搜索元数据（当前方案，适合几千篇文档）
SELECT path, name, description, type, tags, updated
FROM documents
WHERE name LIKE '%query%'
   OR description LIKE '%query%'
   OR tags LIKE '%query%'

// show：查看完整内容
SELECT * FROM documents WHERE path = ?

// sync：插入或更新
INSERT OR REPLACE INTO documents (...) VALUES (...)
```

**性能备用方案**：
- 当前 `LIKE '%query%'` 在几千篇文档下性能极佳（<5ms）
- 若未来扩展到几万篇，可启用 FTS5 全文检索，性能将大幅提升

**输出格式**：

```json
// search 输出
{
  "results": [
    {
      "path": "wiki/agent.md",
      "name": "agent",
      "description": "智能体设计与协作模式",
      "type": "concept",
      "tags": ["agent", "dev"],
      "updated": "2026-06-18"
    }
  ]
}

// show 输出
{
  "path": "wiki/agent.md",
  "name": "agent",
  "description": "智能体设计与协作模式",
  "type": "concept",
  "tags": ["agent", "dev"],
  "updated": "2026-06-18",
  "content": "# Agent\n\n...",
  "word_count": 1200
}
```

## 同步逻辑设计

**扫描范围**：
- `wiki/` 目录下的所有 `.md` 文件
- `sources/` 目录下的所有 `.md` 文件

**解析流程**：

```
读取 .md 文件
      ↓
找第一个 --- 位置
      ↓
找第二个 --- 位置
      ↓
┌─────────────────────────────────────┐
│ frontmatter（两个 --- 之间）        │
│ 手动解析 YAML 格式（按行读取）      │
│ - name: xxx → name = "xxx"         │
│ - tags: [a, b] → tags = ["a","b"]  │
│ - description: xxx → description   │
└─────────────────────────────────────┘
      ↓
┌─────────────────────────────────────┐
│ 正文（第二个 --- 之后）             │
│ - 完整内容                          │
│ - 字数统计                          │
└─────────────────────────────────────┘
      ↓
存入数据库
```

**YAML 解析策略**：
- 使用手动按行解析（`starts_with("name:")` 等）
- 理由：frontmatter 格式固定，手动解析性能最高，无需额外依赖

**增量同步逻辑**：

```
扫描所有 .md 文件
      ↓
对每个文件：
├─ 查询数据库：SELECT modified_at FROM documents WHERE path = ?
├─ 获取文件修改时间
│
├─ 文件不存在于数据库 → INSERT（新增）
├─ 文件修改时间 > 数据库记录 → UPDATE（更新）
└─ 文件在数据库但不存在于文件系统 → DELETE（删除）
```

**Git Hook**：

```bash
# .git/hooks/post-commit
#!/bin/sh
cd "$(git rev-parse --show-toplevel)"
./db/wiki-query/wiki-query.exe sync
```

## 错误处理设计

**错误类型**：

| 场景 | 处理方式 |
|------|----------|
| frontmatter 解析失败 | 跳过该文件，打印警告，继续处理其他文件 |
| 数据库连接失败 | 打印错误信息，退出程序 |
| 文件读取失败 | 跳过该文件，打印警告，继续处理其他文件 |
| 路径不存在 | 打印错误信息，退出程序 |

**日志输出**：

```bash
# 同步时的输出
$ wiki-query sync
扫描目录: wiki/, sources/
找到 42 个 .md 文件
新增: 10, 更新: 2, 删除: 0, 跳过: 30
警告: sources/old/file.md frontmatter 解析失败，已跳过
同步完成
```

**容错原则**：
- 单个文件解析失败不影响整体同步
- 打印警告但不中断程序
- 尽可能处理更多文件

## 文件结构

```
E:\Files\Obsidian\
├── db/                            ← 数据库相关文件
│   ├── wiki.db                    ← SQLite 数据库
│   └── wiki-query/                ← Rust 项目
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs            # CLI 入口
│           ├── db.rs              # 数据库操作
│           ├── models.rs          # 数据结构
│           └── sync.rs            # 同步逻辑
└── .git/
    └── hooks/
        └── post-commit            # Git Hook
```

## 依赖

| crate | 用途 |
|-------|------|
| rusqlite | SQLite 操作（bundled 特性，无需系统 SQLite） |
| serde + serde_json | JSON 序列化 |
| chrono | 时间处理（解析、比较 modified_at 和 updated） |

## 复杂度评估

| 阶段 | 工作量 |
|------|--------|
| 项目结构 | 0.5h |
| 数据库模块 | 1h |
| 同步模块 | 2h |
| 查询模块 | 1h |
| CLI 入口 | 0.5h |
| Git Hook | 0.5h |
| **总计** | **5.5h** |

## 风险

| 风险 | 可能性 | 缓解措施 |
|------|--------|----------|
| frontmatter 解析失败 | 中 | 容错处理，跳过格式错误的文件 |
| 文件量大时同步慢 | 低 | 增量同步（基于修改时间） |
