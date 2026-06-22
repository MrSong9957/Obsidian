# MiMo Code 插件市场设计

## 概述

为 MiMo Code 构建一个插件市场系统，参考 Claude Code 的插件市场实现，格式完全兼容 `claude-plugins-official`。

**一句话定位**：MiMo Code 的插件市场是一个"三层快递网络"——注册表是超市货架（GitHub 仓库），CLI 是扫码购物车（Node.js），运行时是你本人拿着购物小票取货（MiMo Code 加载器）。

**核心决策**：
- 插件形态：混合体（Skill + MCP 完全自由组合）
- 注册表：GitHub 仓库 + registry.json，格式兼容 claude-plugins-official
- CLI：Node.js/TypeScript，双模式（直接命令 + 交互式 TUI）
- 作用域：用户级 + 项目级

**基线**：无（全新项目）。

**性质**：增量新建。MiMo Code 已有 MCP 客户端，插件市场只需管发现 + 安装 + 加载。

---

## 设计动机

MiMo Code 当前没有插件市场（OpenCode bridge 文档明确标注"无 plugin marketplace"）。Claude Code 通过插件市场实现了 Skills + MCP 的统一分发，用户用一条命令就能安装语言服务器、工作流技能等扩展。MiMo Code 需要同等能力。

参考 Claude Code 的做法（30,600 stars 的 `claude-plugins-official` 已验证可行），采用完全兼容的格式，最大化复用现有生态。

---

## 系统总览

```
┌─────────────────────────────────────────────────────┐
│                 MiMo Code 插件市场                    │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐      │
│  │ 注册表    │    │  CLI     │    │ 运行时    │      │
│  │(GitHub   │◄───│(Node.js  │───►│(MiMo Code│      │
│  │ 仓库)    │    │ CLI)     │    │ 加载器)   │      │
│  └──────────┘    └──────────┘    └──────────┘      │
│       │               │               │             │
│       ▼               ▼               ▼             │
│  registry.json   installed_     SKILL.md → 注入     │
│  + 插件包目录    plugins.json   mcp.json → 启动     │
│                  + ~/.mimo/      MCP 服务器          │
│                    plugins/                          │
└─────────────────────────────────────────────────────┘
```

| 组件 | 物理本质 | 职责 |
|------|---------|------|
| **注册表** | 超市货架的商品目录 | GitHub 仓库，存 registry.json + 插件目录 |
| **CLI** | 扫码购物车 | Node.js CLI，负责搜索、下载、安装、卸载、TUI |
| **运行时** | 你本人拿着购物小票取货 | MiMo Code 启动时读 installed_plugins.json，加载 Skills + 启动 MCP |

---

## 注册表格式

### 仓库结构

```
mimo-plugins-official/
├── registry.json               # 索引文件：所有插件的元数据
├── plugins/
│   ├── rust-analyzer/
│   │   ├── SKILL.md            # 技能定义（可选）
│   │   └── mcp.json            # MCP 服务器配置（可选）
│   ├── pdf-reader/
│   │   ├── SKILL.md
│   │   └── mcp.json
│   └── git-workflow/
│       └── SKILL.md            # 纯 Skill 插件
└── README.md
```

### registry.json

```json
{
  "schemaVersion": 1,
  "plugins": {
    "rust-analyzer": {
      "description": "Rust 语言服务器集成",
      "version": "0.1.0",
      "author": "mimo-community",
      "tags": ["rust", "lsp"],
      "hasMcp": true,
      "hasSkill": true
    },
    "git-workflow": {
      "description": "Git 工作流最佳实践",
      "version": "1.0.0",
      "author": "mimo-team",
      "tags": ["git", "workflow"],
      "hasMcp": false,
      "hasSkill": true
    }
  }
}
```

### 插件下载

直接从 GitHub 仓库下载插件目录，不需要打包 .tgz：

```
下载 URL: https://github.com/mimo-plugins-official/raw/main/plugins/<name>/
```

CLI 用 GitHub API 或 sparse-checkout 拉取文件。用户直接在 GitHub 上浏览插件源码。

### 兼容性

格式完全兼容 `claude-plugins-official`。键名格式 `plugin@marketplace`（如 `rust-analyzer@mimo-plugins-official`）。后续如果 Claude Code 有额外字段，再补上。

---

## 插件包格式

### 目录结构

```
plugins/<plugin-name>/
├── SKILL.md            # 技能定义（可选）
├── mcp.json            # MCP 服务器配置（可选）
└── README.md           # 插件说明（可选，给人看的）
```

### SKILL.md

```markdown
---
name: rust-analyzer
description: 使用 rust-analyzer 进行 Rust 开发
---

# Rust Analyzer 集成

当用户进行 Rust 开发时，自动启用以下能力：

## 工具调用
- 使用 `hover` 查看类型信息
- 使用 `goto_definition` 跳转到定义
- 使用 `diagnostics` 获取编译错误

## 行为准则
- 修改 Rust 文件后，主动调用 diagnostics 检查
- 代码补全时优先使用 rust-analyzer 的建议
```

- frontmatter 必须有 `name` 和 `description`
- 正文是给 AI 看的指令

### mcp.json

```json
{
  "servers": {
    "rust-analyzer-lsp": {
      "command": "npx",
      "args": ["rust-analyzer-mcp-server"],
      "env": {}
    }
  }
}
```

格式与 Claude Code 的 MCP 配置一致。

### 组合规则

| SKILL.md | mcp.json | 说明 | 示例 |
|----------|----------|------|------|
| ✅ | ❌ | 纯 Skill 插件 | git-workflow |
| ❌ | ✅ | 纯 MCP 插件 | 某个工具服务器 |
| ✅ | ✅ | 混合插件 | rust-analyzer |
| ❌ | ❌ | **非法**，至少有一个 | — |

---

## CLI 命令

### 双模式

```
模式 1: 直接命令（脚本化/CI 用）
  mimo plugin install rust-analyzer --scope user
  mimo plugin list

模式 2: 交互式 TUI（日常使用）
  mimo plugin              ← 无子命令，打开 TUI
```

### 命令清单

| 命令 | 行为 |
|------|------|
| `mimo plugin` | 打开交互式 TUI |
| `mimo plugin install <name> [--scope user\|project]` | 直接安装 |
| `mimo plugin uninstall <name> [--scope user\|project]` | 直接卸载 |
| `mimo plugin list [--scope user\|project\|all]` | 列出已安装 |
| `mimo plugin marketplace add <url>` | 添加第三方源 |
| `mimo plugin marketplace remove <name>` | 移除第三方源 |
| `mimo plugin marketplace list` | 列出所有源 |

### 各命令详细行为

#### `mimo plugin install`

```
输入: mimo plugin install rust-analyzer --scope user

流程:
  1. 读取所有已配置的 marketplace（官方 + 第三方）
  2. 在 registry.json 中查找 "rust-analyzer"
  3. 下载插件目录到 ~/.mimo/plugins/rust-analyzer/
  4. 更新 ~/.mimo/plugins/installed_plugins.json
  5. 如果插件包含 mcp.json，追加到 ~/.mimo/mcp.json

输出: ✅ Installed: rust-analyzer (v0.1.0) [user scope]
```

`--scope` 默认值：`user`

`--scope project` 时：
- 安装到 `<项目根>/.mimo/plugins/rust-analyzer/`
- 更新 `<项目根>/.mimo/plugins/installed_plugins.json`
- 在 `<项目根>/.mimo/settings.json` 中记录 `enabledPlugins`

#### `mimo plugin uninstall`

```
输入: mimo plugin uninstall rust-analyzer --scope user

流程:
  1. 读 installed_plugins.json 确认已安装
  2. 删除 ~/.mimo/plugins/rust-analyzer/ 目录
  3. 从 installed_plugins.json 移除记录
  4. 如果有 mcp.json 配置，从 ~/.mimo/mcp.json 移除对应条目

输出: ✅ Uninstalled: rust-analyzer [user scope]
```

#### `mimo plugin list`

```
输入: mimo plugin list --scope all

输出:
  User plugins:
    - rust-analyzer (v0.1.0)
    - git-workflow (v1.0.0)
  Project plugins (E:\Files\Projects\my-app):
    - pdf-reader (v1.2.0)
```

#### `mimo plugin marketplace add/remove/list`

```
add:    mimo plugin marketplace add https://github.com/xxx/plugins.git
remove: mimo plugin marketplace remove xxx-plugins
list:   mimo plugin marketplace list
        → official: https://github.com/mimo-plugins-official
        → xxx-plugins: https://github.com/xxx/plugins
```

第三方 marketplace 的 registry.json 格式与官方一致。CLI 合并所有 marketplace 的 registry.json 进行搜索。

### TUI 界面

```
┌─ MiMo Plugin Manager ─────────────────────────────┐
│                                                     │
│  ▶ 📦 Installed Plugins                            │
│      rust-analyzer (v0.1.0) [user]                 │
│      git-workflow (v1.0.0) [user]                  │
│      pdf-reader (v1.2.0) [project]                 │
│                                                     │
│    🔍 Browse Marketplace                            │
│    ➕ Add Marketplace                               │
│    🔄 Update All                                    │
│                                                     │
│  [↑/↓] Navigate  [Enter] Select  [q] Quit          │
└─────────────────────────────────────────────────────┘
```

**Browse Marketplace 子界面**：

```
┌─ Marketplace: mimo-plugins-official ───────────────┐
│                                                     │
│  🔎 [___________]  ← 搜索框                        │
│                                                     │
│  ▶ rust-analyzer (v0.1.0) — Rust LSP 集成          │
│    pdf-reader (v1.2.0) — PDF 文件解析               │
│    git-workflow (v1.0.0) — Git 最佳实践             │
│    ...                                              │
│                                                     │
│  [↑/↓] Navigate  [Enter] Install  [b] Back         │
└─────────────────────────────────────────────────────┘
```

技术实现：Node.js TUI 库（如 `ink` 或 `blessed`），纯文本渲染。

---

## 本地存储

### 目录结构

```
~/.mimo/                           # 用户级根目录
├── plugins/
│   ├── installed_plugins.json     # 已安装插件索引
│   ├── rust-analyzer/             # 插件目录
│   │   ├── SKILL.md
│   │   └── mcp.json
│   └── git-workflow/
│       └── SKILL.md
├── mcp.json                       # MCP 服务器配置
└── settings.json                  # MiMo Code 全局设置

<项目根>/.mimo/                    # 项目级根目录
├── plugins/
│   ├── installed_plugins.json     # 项目级索引
│   └── pdf-reader/
│       ├── SKILL.md
│       └── mcp.json
├── mcp.json                       # 项目级 MCP 配置
└── settings.json                  # 项目级设置（含 enabledPlugins）
```

### installed_plugins.json

```json
{
  "schemaVersion": 1,
  "plugins": {
    "rust-analyzer@mimo-plugins-official": {
      "version": "0.1.0",
      "scope": "user",
      "installedAt": "2026-06-22T10:30:00Z",
      "path": "rust-analyzer"
    },
    "pdf-reader@mimo-plugins-official": {
      "version": "1.2.0",
      "scope": "project",
      "installedAt": "2026-06-22T11:00:00Z",
      "path": "pdf-reader",
      "projectPath": "E:\\Files\\Projects\\my-app"
    }
  }
}
```

- 键名格式：`plugin@marketplace`（兼容 Claude Code）
- `projectPath`：仅 project scope 时有此字段

### MCP 配置合并

插件包含 `mcp.json` 时，CLI 将其服务器条目追加到对应作用域的 `mcp.json`。卸载时自动移除。

---

## 运行时加载

```
MiMo Code 启动
  │
  ▼
Step 1: 扫描已安装插件
  → 读 ~/.mimo/plugins/installed_plugins.json（用户级）
  → 读 <项目>/.mimo/plugins/installed_plugins.json（项目级）
  → 合并去重（项目级优先于用户级）
  │
  ▼
Step 2: 加载 Skills
  → 对每个已安装插件，检查是否有 SKILL.md
  → 有则读取 frontmatter（name, description）+ 正文
  → 注入到 system prompt 的 Skills 区块
  → 同时扫描 ~/.mimo/skills/ 和 .mimo/skills/（用户自定义 Skills）
  │
  ▼
Step 3: 启动 MCP 服务器
  → 读 ~/.mimo/mcp.json（用户级）
  → 读 <项目>/.mimo/mcp.json（项目级）
  → 对每个 server 条目，spawn 子进程
  → 建立 JSON-RPC 通信通道
  │
  ▼
Step 4: AI 行为
  → system prompt 已包含所有 SKILL.md 的内容
  → AI 知道有哪些 MCP 工具可用
  → 用户提问时，AI 根据 SKILL.md 的指引调用对应工具
```

### 版本冲突

项目级优先于用户级。冲突时日志提示：

```
⚠️ Version conflict: rust-analyzer (user: v0.3.1, project: v0.2.0). Using project version.
```

### 插件禁用

项目级 `settings.json` 中可禁用特定插件：

```json
{
  "disabledPlugins": ["rust-analyzer@mimo-plugins-official"]
}
```

被禁用的插件不加载 Skills 和 MCP，文件仍保留在磁盘。

---

## 错误处理

| 场景 | 处理方式 |
|------|---------|
| 网络不可用 | 提示 "无法连接到注册表，请检查网络"，不 crash |
| 插件不存在 | 提示 "插件 xxx 未在注册表中找到"，列出相似名称建议 |
| 版本冲突 | 项目级优先，日志提示冲突详情 |
| MCP 服务器启动失败 | 重试 3 次，超过则禁用该插件的 MCP 并提示用户 |
| SKILL.md 格式错误 | 跳过该插件的 Skill 加载，日志警告 |
| installed_plugins.json 损坏 | 重建：扫描 plugins/ 目录，根据 SKILL.md frontmatter 恢复 |
| 磁盘空间不足 | 安装前检查，不足则提示 |

---

## 测试策略

| 测试类型 | 覆盖内容 |
|---------|---------|
| **单元测试** | registry.json 解析、版本比较、路径处理、MCP 配置合并 |
| **集成测试** | 完整 install/uninstall 流程（mock 网络）、TUI 交互 |
| **E2E 测试** | 真实 GitHub 仓库 → 安装 → MiMo Code 加载 → MCP 服务器可用 |

---

## 不在本轮范围

- 插件更新（`mimo plugin update`）— 后续迭代
- 插件搜索（`mimo plugin search <keyword>`）— 后续迭代
- 插件评分/评论系统 — 不做
- 插件签名/安全审计 — 不做
- 向量语义检索插件 — 不做

---

## 复杂度估计

| 工作 | 估计 |
|------|------|
| 注册表仓库搭建（registry.json + 示例插件） | 1h |
| CLI 工程初始化（TypeScript + 构建配置） | 0.5h |
| CLI 核心命令（install/uninstall/list） | 2h |
| CLI marketplace 命令（add/remove/list） | 1h |
| CLI TUI 界面 | 2h |
| 运行时加载器（Skills 注入 + MCP 启动） | 2h |
| 错误处理 + 边界场景 | 1h |
| 测试（单元 + 集成） | 1.5h |
| 文档 | 0.5h |
| **总计** | **~11.5h** |
