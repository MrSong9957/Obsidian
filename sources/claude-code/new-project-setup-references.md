---
name: 新项目初始化详细参考
description: 基础镜像更新的完整命令清单、构建故障排查、运行时问题、Docker Hub Push 代理配置
type: reference
updated: 2026-06-17
tags: docker, setup, image-maintenance, troubleshooting
source: https://hub.docker.com/r/s7620605/claude-code-opencode
---

# 新项目初始化详细参考

> 入口文档：[[wiki/new-project-setup.md|新项目开发环境初始化]]

## 基础镜像更新方式

### 方式一：docker commit（日常更新工具，推荐）

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

### 方式二：docker build（终极兜底，不到万不得已不要使用）

完整重建会从零构建镜像，**清除所有预配置**（Claude Code 配置、智谱 API 路由、模型映射等），重建后需要重新执行 entrypoint 或手动恢复配置。仅在容器已损坏无法正常使用时才动用此方法。

```powershell
$env:HTTP_PROXY = "http://127.0.0.1:7890"
$env:HTTPS_PROXY = "http://127.0.0.1:7890"
cd D:/Files/GitHub/claude-code-docker
docker build --no-cache -t s7620605/claude-code-opencode:latest .
```

### 方式三：configure.ps1 预配置（插件和配置更新）

将插件、状态栏、全局 CLAUDE.md 等配置预烘焙到镜像中，减少每个新项目的初始化步骤。

```powershell
# 交互模式：手动安装插件 + 自动扫描 + commit
D:/Files/GitHub/claude-code-docker/configure.ps1

# 仅更新全局 CLAUDE.md（跳过手动步骤）
D:/Files/GitHub/claude-code-docker/configure.ps1 -SkipManual
```

**流程：** 启动临时容器（`--env-file` 提供 API 访问）→ 用户在容器内手动安装插件/状态栏 → 恢复 settings.json 的 `env` 块（Claude Code 运行时会覆盖该文件，丢失智谱路由配置）→ 复制全局 CLAUDE.md → 双重敏感内容扫描 → commit 为新镜像。

**安全机制：**
- `--env-file` 注入的环境变量仅存在于运行时，`docker commit` 不捕获
- 脚本内含双重扫描：关键字模式匹配（`api_key`、`secret` 等）+ `.env` 实际值的精确匹配
- 发现敏感内容时暂停，需用户确认后才继续 commit

**与 update.ps1 的关系：** `update.ps1` 更新工具版本（npm 包），`configure.ps1` 更新插件和配置，两者独立运行互不影响。

> **注意：** configure.ps1 的变更写入 `/home/app/.claude`（命名卷），各项目应用更新时必须用 `down -v` 重建卷，`--force-recreate` 不会更新卷内数据。

### 各项目应用更新

```powershell
cd <项目>/docker

# 工具版本更新（方式一）：工具安装在系统路径，不在命名卷中
docker compose up -d --force-recreate

# 插件/配置更新（方式三）：配置在命名卷中，必须删除卷才能从新镜像重新初始化
docker compose up -d --force-recreate
docker compose down -v
docker compose up -d
```

> `--force-recreate` 重建容器但**保留命名卷数据**，新镜像中 `.claude` 的变更不会生效。`down -v` 连同命名卷一起删除，新容器首次启动时从镜像重新初始化 `/home/app/.claude`。项目级配置（`CLAUDE.md`、`.claude/rules`）在项目目录的 bind mount 中，不受 `-v` 影响。

---

## 构建故障排查

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
- ❌ `docker-compose.yml` 用 `HTTP_PROXY: ${HTTP_PROXY:-}` — 宿主机环境变量优先级高于 `.env` 空值，代理仍会泄漏。**必须硬编码 `HTTP_PROXY: ""`**
- ❌ `PATH=/home/app/.bun/bin:$PATH` 写在 docker-compose.yml environment 中 — `$PATH` 在宿主机展开，Windows PATH 注入容器，覆盖 Linux PATH，导致 `bash: command not found`。**删除该行，依赖 Dockerfile 的 `ENV PATH`**
- ❌ 忘记在 apt-get 依赖中加 `unzip` — Bun 安装脚本需要 unzip，`debian:bookworm-slim` 不预装
- ❌ entrypoint.sh 以 `user: "0:0"` 运行时未设置 PATH — root 用户的 PATH 可能不完整。Dockerfile 加 `ENV PATH`，entrypoint.sh 加 `export PATH`
- ❌ Windows 编辑的 `.sh` 文件带 CRLF — Linux bash 无法识别 `\r`。Dockerfile 中用 `sed -i 's/\r$//'` 修复
- ✅ 构建成功后确认：`docker compose exec app bash` 能正常进入

---

## 常见运行时问题

> 以下为容器运行时（非构建期）的常见问题。

- **`claude: command not found`** — bash 命令哈希缓存过期，执行 `hash -r` 清除缓存后重试
- **`Unable to connect to Anthropic services`** — 临时容器缺少 `--env-file`，必须通过 `--env-file D:\Files\.env` 提供 API key
- **`docker push` EOF 错误** — 国内网络推送到 Docker Hub 时的代理配置问题，见下方 [[#Docker Hub Push 代理配置]]

---

## Docker Hub Push 代理配置

> 从国内通过 FlClash 代理推送 Docker 镜像到 Docker Hub 的正确配置。

**正确配置（无需修改 Docker Desktop）：**

| 设置项 | 值 | 说明 |
|--------|-----|------|
| Docker Desktop Proxy | System proxy（默认） | 不需要改动 |
| Containers proxy | Same as host proxy（默认） | 不需要改动 |
| Docker Desktop Manual proxy | **不要修改** | 默认即可 |

如果默认节点 push 时出现 EOF，在 FlClash 中切换代理节点后重试 `docker push`。

**已验证的失败方式：**

| 方式 | 结果 | 原因 |
|------|------|------|
| Manual proxy + HTTP 代理 | EOF | FlClash HTTP 代理与 Docker Hub TLS 握手不兼容 |
| Manual proxy + SOCKS5 | Connection refused | FlClash SOCKS5 端口 7891 未开放 |
| `$env:HTTPS_PROXY` 环境变量 | 无效 | `docker push` 是 daemon 行为，不读 shell 环境变量（与 `docker build` 不同） |
