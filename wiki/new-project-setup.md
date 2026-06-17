---
name: new-project-setup
description: 新项目初始化统一入口——Docker 容器启动、镜像配置验证、首次配置、Obsidian 记忆链接、项目级配置补充；镜像更新命令与故障排查见详细参考
type: reference
updated: 2026-06-17
tags: docker, setup, claude-md, ecc, superpowers, configuration
source: [[sources/claude-code/new-project-setup-references.md]]
---

# 新项目开发环境初始化

> 相关文档：[[wiki/claude-md-base.md|CLAUDE.md 模板]] | [[wiki/claude-code.md|Claude Code 配置与工作流]]
>
> 详细参考：[[sources/claude-code/new-project-setup-references.md|新项目初始化详细参考]]（镜像更新命令清单、构建故障排查、Docker Hub Push 代理配置）

## 前置条件

| 条件 | 说明 |
|------|------|
| Docker | 已安装并运行 |
| 基础镜像 | `s7620605/claude-code-opencode:latest` |
| 镜像本地源码 | `D:\Files\GitHub\claude-code-docker` |
| Docker Hub | `https://hub.docker.com/r/s7620605/claude-code-opencode` |
| 凭证文件 | `D:\Files\.env`（集中管理所有 token） |
| 首次配置 | Claude Code 首次安装配置已完成（插件/SafetyNet/StatusLine/ECC rules）→ 详见下方「首次安装配置」章节 |
| Obsidian 仓库 | `D:\Files\Obsidian`（项目记忆和知识库） |

基础镜像预装：Claude Code / OpenCode / Codex / Node.js 20 / Python 3 / Git / GitHub CLI / 中文字体

> OpenCode 完整配置指南见 [[sources/tools/opencode.md]]

## 基础镜像更新机制

容器是基础镜像的可读写副本，不同项目引用同一镜像互不污染。工具版本在 `docker build` 时固化到镜像中。

### 双轨更新设计

基础镜像有两条独立的更新通道，互不冲突：

| | Docker Hub（GitHub Actions） | 本地（update.ps1） |
|---|---|---|
| **触发方式** | 每周一 UTC 02:00 自动 + 手动 `workflow_dispatch` | 手动运行 `update.ps1` |
| **更新范围** | 系统依赖 + npm 工具（完整重建） | 仅 npm 工具 |
| **构建方式** | `docker build` 从零构建 | 临时容器 → `docker commit` |
| **时效性** | 最长等 7 天，或手动触发 | 立即生效 |
| **适用场景** | 远程机器拉取、自动化兜底 | 本地高频使用、即时更新 |
| **镜像体积** | 干净，无层累积 | 逐层累积（不影响日常使用） |

GitHub 工作流通过 `CACHEBUST` 构建参数（`build-args: CACHEBUST=${{ github.run_id }}`）打破 Docker 层缓存，确保 `npm install -g` 每次真正拉取最新版本，不会被缓存命中跳过。本地 `update.ps1` 负责日常快速更新，无需推送到 Docker Hub。

> **原则：** 工具版本更新用**方式一**（docker commit），插件/配置更新用**方式三**（configure.ps1），两者独立运行互不影响。**方式二**（docker build）是终极兜底——仅在容器已无法正常使用时才动用。

### 预配置 vs 项目级配置

`configure.ps1` 将部分配置预烘焙到镜像中，减少每个新项目的初始化工作量：

| 预烘焙（configure.ps1 写入镜像） | 项目级（每个新项目手动完成） |
|---|---|
| Status line 脚本 | 项目级 CLAUDE.md（步骤 5.2） |
| Superpowers 插件 | 项目级 ECC rules（步骤 5.3） |
| ECC 插件 + MCP 配置 | Obsidian MEMORY.md link（步骤 4） |
| 全局 CLAUDE.md | Git 初始化（步骤 5.1） |
| settings.json（智谱路由等） | Superpowers 目录结构（步骤 5.4） |

### 更新流程

| 方式 | 用途 | 适用场景 |
|------|------|----------|
| **方式一：docker commit** | 日常更新工具版本（npm 包） | 本地高频使用、即时更新（推荐） |
| **方式二：docker build** | 终极兜底，完整重建 | 容器已损坏无法正常使用（不到万不得已不用） |
| **方式三：configure.ps1** | 更新插件、状态栏、全局 CLAUDE.md | 预烘焙配置，减少新项目初始化步骤 |

> 完整命令清单、各项目应用更新（`--force-recreate` vs `down -v`）、configure.ps1 的安全机制见 [[sources/claude-code/new-project-setup-references.md|详细参考 §基础镜像更新方式]]。

### 构建与运行时问题

- **`docker build` 网络层故障**（FROM/apt/curl/npm/运行时代理）— 终端设临时代理 + Dockerfile 内换源，9 条踩坑清单见 [[sources/claude-code/new-project-setup-references.md#构建故障排查|详细参考 §构建故障排查]]
- **`docker push` EOF**（国内推 Docker Hub）— 详见 [[sources/claude-code/new-project-setup-references.md#Docker Hub Push 代理配置|详细参考 §Push 代理配置]]
- **`claude: command not found`** — bash 命令哈希缓存过期，执行 `hash -r` 清除缓存后重试
- **`Unable to connect to Anthropic services`** — 临时容器缺少 `--env-file`，必须通过 `--env-file D:\Files\.env` 提供 API key

---

## Claude Code 首次安装配置（一次性）

> 以下配置在基础镜像中只需完成一次。`configure.ps1` 已预烘焙部分配置（见下表 ✓ 标记），其余需手动完成。
>
> **详细步骤：** [[sources/claude-code/claude-code-first-time-setup.md|首次安装配置完整指南]]

| 配置项 | 操作方式 | 预烘焙 | 详细文档 |
|--------|---------|--------|---------|
| **插件市场** | `/plugin` → 添加 `anthropics/claude-plugins-official` | — | [[sources/claude-code/claude-code-first-time-setup.md#1-安装官方插件市场\|首次配置 §1]] |
| **官方插件**（6 个） | `/plugin` → 从官方市场安装：GitHub / context7 / skill-creator / learning-output-style / superpowers / frontend-design | — | [[sources/claude-code/claude-code-first-time-setup.md#2-安装官方插件\|首次配置 §2]] |
| **Safety Net** | `/plugin marketplace add kenryu42/cc-marketplace` → 安装 → 配置 `settings.json` hook | — | [[sources/claude-code/claude-code-first-time-setup.md#3-安全网插件safety-net\|首次配置 §3]] |
| **StatusLine** | Python 脚本 + 管道接入 Safety Net | ✓ | [[sources/claude-code/claude-code-first-time-setup.md#4-状态栏插件statusline\|首次配置 §4]] |
| **ECC Rules** | 全局 `~/.claude/rules/ecc/common` + 项目级语言 rules 手动复制 | — | — |
| **智谱 MCP** | 4 个 MCP 服务器（视觉/搜索/网页/仓库） | — | [[wiki/zhipu-mcp.md\|智谱 MCP]] |

> [!NOTE]
> 未预烘焙项（插件市场、官方插件、Safety Net、ECC 全局 rules）的具体操作命令见 [[sources/claude-code/claude-code-first-time-setup.md|首次安装配置完整指南]]。智谱 MCP 因 API KEY 泄露风险不推荐烘焙到镜像，按项目手动安装（[[wiki/zhipu-mcp.md|智谱 MCP]]）。

### StatusLine 配置

StatusLine 已由 `configure.ps1` 预烘焙到基础镜像（脚本 + settings.json 配置），**新容器无需手动操作**。脚本配置与完整 settings.json 示例详见 [[sources/claude-code/claude-code-statusline-setup.md|状态栏配置]]（canonical 源 [[sources/claude-code/statusline.py|statusline.py]]）。

**验证：** 配置完成后运行 `/plugin`（确认 7+ 插件）+ `/mcp`（确认 4 个智谱 MCP Connected）+ 执行 bash 命令（StatusLine 显示模型/目录/分支/上下文进度条）+ 执行危险命令（Safety Net 拦截）。

---

## 流程概览

```
首次安装配置（一次性）→ 创建 docker/docker-compose.yml → 启动容器 → 验证基础镜像配置 → Obsidian 记忆链接 → 补充缺失项（Git/CLAUDE.md/ECC rules/目录结构）
```

---

## 步骤 1：创建 docker/docker-compose.yml [宿主机]

```yaml
services:
  app:
    image: s7620605/claude-code-opencode:latest
    build:
      context: D:/Files/GitHub/claude-code-docker
    container_name: <项目名>
    user: "0:0"
    extra_hosts:
      - "host.docker.internal:host-gateway"
    environment:
      - HTTP_PROXY=http://host.docker.internal:7890
      - HTTPS_PROXY=http://host.docker.internal:7890
      - NO_PROXY=localhost,127.0.0.1,open.bigmodel.cn,.bigmodel.cn
    volumes:
      - ..:/home/app/project
      - D:/Files/Obsidian:/home/app/obsidian
      - config:/home/app/.claude
    env_file:
      - D:/Files/.env            # 全局 token（必须同时包含 ANTHROPIC_API_KEY 和 ANTHROPIC_AUTH_TOKEN）
      - ../.env                   # 项目级配置（代理地址等，此文件必须存在，空文件即可）
    stdin_open: true
    tty: true

volumes:
  config:
```

**字段说明：**

| 字段 | 用途 |
|------|------|
| `image` | 优先使用本地镜像，本地无则从 Docker Hub 拉取 |
| `build.context` | 本地源码构建兜底（两级均不可用时，用 `/docker-dev-container` 技能重建并推送 Hub） |
| `user: "0:0"` | 以 root 运行，entrypoint 自动处理权限 |
| `extra_hosts` | 确保 `host.docker.internal` 可解析（Docker Desktop 部分版本不自动注入） |
| `environment` | 代理配置 + `NO_PROXY` 排除国内域名（`open.bigmodel.cn`）避免直连请求绕代理 |
| `volumes: ..` | 项目根目录（`..` 从 docker/ 子目录向上）挂载到工作目录 |
| `volumes: config` | Claude 配置持久化（跨容器重建保留） |
| `volumes: obsidian` | Obsidian 知识库挂载，项目记忆通过 symlink 桥接到此仓库 |
| `env_file` | 两层：全局 `D:/Files/.env` 管 token + 项目级 `.env` 管代理等配置。项目级 `.env` **必须存在**（docker-compose 不支持可选 env_file，空文件即可）。全局文件**必须**同时包含 `ANTHROPIC_API_KEY` 和 `ANTHROPIC_AUTH_TOKEN`（Claude Code 读取后者），参见 [[wiki/token-management.md\|凭证管理]] |

---

## 步骤 2：启动容器 [宿主机]

```bash
cd docker
docker compose up -d
docker compose exec app bash
```

---

## 步骤 3：验证基础镜像已有配置 [容器内]

容器启动后 `entrypoint.sh` 自动完成初始化，需逐一验证。

### 3.1 Claude Code 配置

| 检查项 | 命令 | 预期 |
|--------|------|------|
| 配置文件存在 | `cat ~/.claude.json` | `{"hasCompletedOnboarding": true}` |
| settings.json 存在 | `cat ~/.claude/settings.json` | 包含智谱 API 配置 |
| API 地址 | `grep ANTHROPIC_BASE_URL ~/.claude/settings.json` | `open.bigmodel.cn/api/anthropic` |
| 模型配置 | `grep ANTHROPIC_DEFAULT ~/.claude/settings.json` | haiku/sonnet→glm-5-turbo, opus→glm-5.1 |
| Claude 版本 | `claude --version` | 最新版（由基础镜像提供） |

### 3.2 CLI 工具

| 检查项 | 命令 |
|--------|------|
| Node.js 20 | `node --version` |
| Python 3 | `python --version` |
| Git | `git --version` |
| GitHub CLI | `gh --version` |

### 3.3 系统环境

| 检查项 | 命令 | 预期 |
|--------|------|------|
| 工作用户 | `id app` | uid=1000(app) |
| 工作目录 | `pwd` | `/home/app/project` |
| 中文字体 | `fc-list :lang=zh \| head -1` | Noto CJK 字体 |
| sudo 免密 | `sudo -l` | NOPASSWD: ALL |

### 3.4 凭证验证

```bash
echo $ANTHROPIC_API_KEY | head -c 10     # 确认原始变量已加载
echo $ANTHROPIC_AUTH_TOKEN | head -c 10  # 确认 Claude Code 读取的变量已加载
```

**常见问题：** `ANTHROPIC_AUTH_TOKEN` 为空——`D:\Files\.env` 中缺少此变量。参见 [[wiki/token-management.md|凭证管理]] 中的"env_file 不做 ${VAR} 替换"规则，两个变量必须同时写入。

### 3.5 API 连通性

```bash
claude --version  # 能启动即连通
```

---

## 步骤 4：Obsidian 记忆链接 [容器内]

使用 `obsidian-link` 技能，将项目记忆目录 symlink 到 Obsidian 仓库。

### 4.1 验证挂载

```bash
ls /home/app/obsidian/MEMORY.md  # 确认 Obsidian 仓库已挂载
```

### 4.2 执行 obsidian-link 技能

在 Claude Code 中调用：

```
/obsidian-link
```

技能自动完成：
1. 检测运行环境（Docker 容器内）
2. 扫描 `/home/app/obsidian` 找到含 MEMORY.md 的 vault
3. 在项目记忆目录创建 symlink：
   - `MEMORY.md` → `/home/app/obsidian/MEMORY.md`
   - `wiki/` → `/home/app/obsidian/wiki/`
   - `sources/` → `/home/app/obsidian/sources/`
   - `skills/` → `/home/app/obsidian/skills/`
   - `templates/` → `/home/app/obsidian/templates/`
   - 以及 vault 中存在的其他目录/文件
4. 验证所有 symlink 可读

完成后，Obsidian 侧编辑的内容实时同步到项目，Claude Code 直接读写 MEMORY.md。

---

## 步骤 5：补充缺失配置 [容器内]

基础镜像已通过 `configure.ps1` 预烘焙全局 CLAUDE.md、插件（Superpowers/ECC）、状态栏脚本等配置（对照表见上方）。以下为**项目级配置**，每个新项目需单独完成。

### 5.1 Git 初始化

```bash
git init
git config user.name "<用户名>"
git config user.email "<邮箱>"
```

### 5.2 项目级 CLAUDE.md

按 [[wiki/claude-md-base.md|CLAUDE.md 双层架构模板]] 第二层创建（含 Superpowers 七阶段工作流、技能库触发表、设计原则与铁律、ECC 补强模块、项目信息）。核心内容从该模板复制，仅「项目信息」（技术栈/架构/文件约定/项目描述）按项目实际填写。

### 5.3 项目级 ECC rules

```bash
mkdir -p .claude/rules/ecc
cp -r ~/.claude/plugins/marketplaces/ecc/rules/common .claude/rules/ecc/
# + 按技术栈安装对应语言 rules
```

**18 个可用语言规则集：**

angular, arkts, cpp, csharp, dart, fsharp, golang, java, kotlin, perl, php, python, ruby, rust, swift, typescript, web

每种语言含 5 个维度：coding-style / security / testing / patterns / hooks

### 5.4 Superpowers 目录结构

```bash
mkdir -p docs/superpowers/specs
mkdir -p docs/superpowers/plans
```

- `specs/`：设计文档，命名 `YYYY-MM-DD--<name>.md`
- `plans/`：任务计划，命名 `YYYY-MM-DD-<name>.md`

### 5.5 .gitignore

```gitignore
# Superpowers
.worktrees/

# Claude Code
.claude/
```

---

## 按技术栈定制

| 技术栈 | ECC rules | 备注 |
|--------|-----------|------|
| Python | common + python | |
| TypeScript | common + typescript | |
| Rust | common + rust | |
| Flutter | common + dart | |
| 全栈 JS | common + typescript + web | |

---

## 验证清单

全部完成后逐项确认：

- [ ] `docker compose ps` 容器运行中
- [ ] 步骤 3 所有检查项通过（基础镜像配置完好）
- [ ] `ls ~/.claude/` 包含插件目录（superpowers、ecc）
- [ ] `cat ~/.claude/CLAUDE.md` 包含全局指令
- [ ] 状态栏脚本可用
- [ ] `git status` 显示干净初始仓库
- [ ] `CLAUDE.md` 包含 Superpowers 七阶段工作流
- [ ] `.claude/rules/ecc/` 包含 common + 对应语言 rules
- [ ] `docs/superpowers/specs/` 和 `plans/` 目录存在
- [ ] `/home/app/obsidian/MEMORY.md` 可读（Obsidian 仓库已挂载）
- [ ] 项目记忆目录 symlink 指向 Obsidian 仓库内容（`obsidian-link` 技能已完成）
- [ ] `claude` 能正常连接 API 并响应
