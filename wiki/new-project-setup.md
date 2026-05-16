---
name: new-project-setup
description: 新项目初始化统一入口——Docker 容器启动、镜像配置验证、CLAUDE.md/ECC/Superpowers 补充；整合智谱MCP、首次配置（SafetyNet/StatusLine）、CLAUDE.md 模板、ECC 补强的配置参考
metadata:
  type: reference
  updated: 2026-05-16
  tags:
    - docker
    - setup
    - claude-md
    - ecc
    - superpowers
    - zhipu-mcp
    - safety-net
    - configuration
  sources:
    - "[[wiki/claude-md-base.md]]"
    - "[[wiki/ecc-superpowers-supplement.md]]"
    - "[[wiki/zhipu-mcp.md]]"
    - "[[wiki/claude-code-p2.md]]"
---

# 新项目开发环境初始化

> 相关文档：[[wiki/claude-md-base.md|CLAUDE.md 模板]] | [[wiki/ecc-superpowers-supplement.md|ECC 补强指南]]

## 前置条件

| 条件 | 说明 |
|------|------|
| Docker | 已安装并运行 |
| 基础镜像 | `s7620605/claude-code-opencode:latest` |
| 镜像本地源码 | `D:\Files\GitHub\claude-code-docker` |
| Docker Hub | `https://hub.docker.com/r/s7620605/claude-code-opencode` |
| 凭证文件 | `D:\Files\.env`（集中管理所有 token） |
| ECC 插件 | 已安装（全局 common rules 已就绪） |
| Obsidian 仓库 | `D:\Files\Obsidian`（项目记忆和知识库） |

基础镜像预装：Claude Code / OpenCode / Codex / Node.js 20 / Python 3 / Git / GitHub CLI / 中文字体

## 基础镜像更新机制

容器是基础镜像的可读写副本，不同项目引用同一镜像互不污染。工具版本在 `docker build` 时固化到镜像中。

> **原则：始终使用方式一（docker commit）更新工具和依赖。** 方式二（docker build）是终极兜底——仅在容器已无法正常使用时才动用。即使需要新增/卸载系统依赖，也应在容器内直接操作后 commit，而非完整重建。

### 更新流程

#### 方式一：docker commit（日常更新工具，推荐）

启动临时容器（基础镜像的干净副本）→ 更新工具 → 提交为新镜像。

> **为什么用临时容器？** 项目容器内有项目文件和运行时状态，commit 会把这些也打包进去，镜像变脏。临时容器是干净环境，只有工具更新。

```powershell
# 1. 启动临时容器
docker run -d --name temp-update s7620605/claude-code-opencode:latest

# 2. 进入容器更新工具
docker exec -it -u root temp-update bash
npm update -g @anthropic-ai/claude-code opencode-ai @openai/codex
exit

# 3. 提交为新镜像（覆盖 latest 标签）
docker commit temp-update s7620605/claude-code-opencode:latest

# 4. 停止并删除临时容器（不能跳过！）
docker stop temp-update
docker rm temp-update

# 5. 各项目重建容器
cd <项目>/docker
docker compose up -d --force-recreate
```

> **不仅是工具版本更新。** 新增/卸载系统依赖也可以在容器内完成：`apt-get install/remove`、`nvm install` 等，操作后 commit 即可。无需为此触发完整重建。

#### 方式二：docker build（终极兜底，不到万不得已不要使用）

完整重建会从零构建镜像，**清除所有预配置**（Claude Code 配置、智谱 API 路由、模型映射等），重建后需要重新执行 entrypoint 或手动恢复配置。仅在容器已损坏无法正常使用时才动用此方法。

```powershell
$env:HTTP_PROXY = "http://127.0.0.1:7890"
$env:HTTPS_PROXY = "http://127.0.0.1:7890"
cd D:/Files/GitHub/claude-code-docker
docker build --no-cache -t s7620605/claude-code-opencode:latest .
```

#### 各项目应用更新

```powershell
cd <项目>/docker
docker compose up -d --force-recreate
```

> **为什么必须 `--force-recreate`？** `docker compose up -d` 在容器配置未变时会复用旧容器（基于旧镜像）。`--force-recreate` 强制销毁旧容器、基于新镜像创建新容器。

### 构建故障排查

> 以下仅适用于 `docker build`（方式二）完整重建。`docker commit`（方式一）不涉及这些网络层。

**核心原则：终端设临时代理 + Dockerfile 内用国内源 + 不动 Docker Desktop 代理设置。**

Docker 构建涉及多个网络层，每层解法不同：

| 层级 | 网络需求 | 解法 |
|------|---------|------|
| `FROM` 拉基础镜像 | `docker.1ms.run` 国内镜像站 | 不需要代理，**不要配 Docker Desktop 代理** |
| `apt-get` 系统包 | `mirrors.aliyun.com` | Dockerfile 内 `sed` 换源 |
| `curl` 下载 gh CLI / nodesource | GitHub / NodeSource | 终端 `$env:HTTPS_PROXY` 自动传递给 buildkit |
| `npm install` | `registry.npmmirror.com` | Dockerfile 内换源 |
| 运行时代理 | 容器内 `HTTP_PROXY` | `docker-compose.yml` 的 `environment` |

**常见错误（踩过的坑）：**

- ❌ 在 Docker Desktop → Settings → Proxies 填 `host.docker.internal:7890` — 会把 `docker.1ms.run` 也路由到代理，代理拒绝局域网连接，`FROM` 行直接失败
- ❌ 在 Dockerfile 加 `ENV HTTP_PROXY=${HTTP_PROXY}` — 同上，影响镜像拉取
- ❌ 试图开启 FlClash Allow LAN — 不应该为修复构建而改代理软件配置
- ✅ 构建成功后确认：`docker images s7620605/claude-code-opencode`

## 流程概览

```
创建 docker/docker-compose.yml → 启动容器 → 验证基础镜像配置 → Obsidian 记忆链接 → 补充缺失项（Git/CLAUDE.md/ECC rules/目录结构）
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

基础镜像**不包含**以下项目级配置，需手动完成。

### 5.1 Git 初始化

```bash
git init
git config user.name "<用户名>"
git config user.email "<邮箱>"
```

### 5.2 项目级 CLAUDE.md

按 [[wiki/claude-md-base.md|CLAUDE.md 双层架构模板]] 第二层创建，包含：

- **Superpowers 七阶段工作流**：头脑风暴 → Git Worktree → 编写计划 → 子代理驱动开发 → TDD → 代码审查 → 完成分支
- **技能库**：按阶段分类的技能触发表
- **设计原则 + 铁律**：TDD 铁律 / 调试铁律 / 验证铁律
- **ECC 补强模块**：Strategic Compacting / Continuous Learning / Session Memory / Token 经济学（参见 [[wiki/ecc-superpowers-supplement.md|ECC 补强指南]]）
- **项目信息**：技术栈 / 架构 / 文件约定 / 项目描述 ← 按项目实际情况填写

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

## 相关配置参考

新项目初始化涉及的完整配置细节：

| 配置项 | 文档 | 说明 |
|--------|------|------|
| 智谱 MCP | [[wiki/zhipu-mcp.md\|智谱 MCP]] | Web reader 限制和替代方案 |
| 首次配置 | [[wiki/claude-code-p2.md\|首次配置]] | 插件/SafetyNet/StatusLine 安装步骤 |
| CLAUDE.md 模板 | [[wiki/claude-md-base.md\|CLAUDE.md 模板]] | 双层架构（karpathy + superpowers） |
| Safety Net 状态栏 | [[wiki/claude-code-p2.md\|首次配置]] | 管道接入、Windows 环境变量 |
| ECC 补强 | [[wiki/ecc-superpowers-supplement.md\|ECC 补强]] | 8 项功能配置和使用规则 |

---

## 验证清单

全部完成后逐项确认：

- [ ] `docker compose ps` 容器运行中
- [ ] 步骤 3 所有检查项通过（基础镜像配置完好）
- [ ] `git status` 显示干净初始仓库
- [ ] `CLAUDE.md` 包含 Superpowers 七阶段工作流
- [ ] `.claude/rules/ecc/` 包含 common + 对应语言 rules
- [ ] `docs/superpowers/specs/` 和 `plans/` 目录存在
- [ ] `/home/app/obsidian/MEMORY.md` 可读（Obsidian 仓库已挂载）
- [ ] 项目记忆目录 symlink 指向 Obsidian 仓库内容（`obsidian-link` 技能已完成）
- [ ] `claude` 能正常连接 API 并响应
