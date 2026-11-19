---
name: Docker 容器开发方案
description: Windows 使用 Docker Desktop WSL2 开发容器，将主要开发数据放到 D 盘，并让多个项目共享 Obsidian 仓库。
type: reference
updated: 2026-11-16
tags: [Docker, WSL2, Agent, 缓存管理]
source: self
---

# Docker 容器开发方案

## 1. 目标与执行规则

Windows 保留桌面工具，Docker Desktop 使用 WSL2 后端提供 Linux 开发容器，主要开发数据放 D 盘。

Agent 默认连续执行到完成，按以下顺序选择操作方式：

1. 命令 / 配置文件；
2. 桌面自动化；
3. 用户手动兜底。

只有权限、本人验证或工具能力确实不足时，才让用户处理对应步骤。重启、关闭 Docker 或 WSL 前，先保存当前工作。

## 2. 存储规划

统一根目录：`D:\AgentLab`。

| Windows 路径 | 用途 | 容器路径 |
| --- | --- | --- |
| `D:\AgentLab\docker-data` | Docker 镜像、容器、volume | Docker 管理 |
| `D:\AgentLab\wsl-data` | WSL swap | — |
| `D:\AgentLab\projects\<project>` | 项目源码 | `/workspace` |
| `D:\AgentLab\shared\Obsidian` | 共享知识库 | `/vault` |
| `D:\AgentLab\backups` | 配置、会话等备份 | — |

已有 D/E 盘项目可以原地使用。Compose 项目名确定后不要随意修改，否则可能创建新的 volume。

## 3. Docker 与 WSL 迁移

### Docker 数据盘

目标位置：`D:\AgentLab\docker-data`。

尚未安装 Docker 时，安装参数为：

```text
--backend=wsl-2
--wsl-default-data-root=D:\AgentLab\docker-data
```

已安装 Docker 时，当前没有通用 CLI 迁移现有 Docker Desktop 数据盘，由 Agent 操作：

```text
Settings
→ Resources
→ Advanced
→ Disk image location
→ D:\AgentLab\docker-data
→ Apply / Restart
```

迁移前记录已有镜像、容器和 volume。目标目录已有数据时，先确认来源，不能直接覆盖。禁止在 Docker 运行时手工移动 `.vhdx`。

迁移后必须确认 Docker 正常启动，原镜像、容器和 volume 仍然存在，且 D 盘出现实际 Docker 数据。只修改设置中的路径不算完成。

### WSL swap

创建 `D:\AgentLab\wsl-data`，编辑 `%USERPROFILE%\.wslconfig`：

```ini
[wsl2]
swapFile=D:\\AgentLab\\wsl-data\\swap.vhdx
```

已有 `[wsl2]` 时合并配置，不重复创建。若已有 `swap=0`，表示 swap 已关闭，无须设置 swap 文件。

应用配置：

```powershell
docker desktop stop
wsl --shutdown
docker desktop start
docker info
```

若当前版本没有 `docker desktop` 命令，由 Agent 正常退出并重新启动 Docker Desktop。

验收：`docker info` 正常；swap 启用时，确认 swap 实际位于 D 盘。

## 4. 项目容器

### 配置与挂载

项目使用：

```text
<project>\.devcontainer\Dockerfile
<project>\.devcontainer\compose.yaml
```

已有文件先读取并合并，不能直接覆盖。

| 项目 | 示例或约定 |
| --- | --- |
| Compose 项目名 | `agentlab-demo` |
| 服务名 | `dev` |
| 容器名 | `agentlab-demo-dev` |
| 用户 | `dev` |
| 用户目录 | `/home/dev` |
| 工作目录 | `/workspace` |
| 源码挂载 | 项目根目录 → `/workspace` |

`compose.yaml` 位于 `.devcontainer` 时，源码可挂载为：

```yaml
source: ..
target: /workspace
```

bind mount 使用以下配置，避免路径写错时自动创建空目录：

```yaml
bind:
  create_host_path: false
```

Dockerfile 继承通用基础镜像，只增加项目实际需要的环境，并保证 `dev` 用户能读写挂载目录。

### 基础镜像与项目镜像

先构建通用 Ubuntu 基础镜像，再为项目增加开发环境：

```text
Ubuntu 24.04
    ↓
agentlab-base
    ↓
项目镜像
```

基础镜像只放所有项目都常用的工具：

| 类别 | 内容 |
| --- | --- |
| 系统基础 | Ubuntu、`dev` 用户、`ca-certificates` |
| 开发基础 | Python、pip / venv / uv、Git |
| Agent 工具 | `curl`、`openssh-client`、`jq`、`ripgrep`、`tar / gzip / zip / unzip`、`findutils / procps / less`、Claude Code / Codex |

Python 用作 Agent 的通用脚本环境，项目自己的 Python 依赖放在项目 `.venv` 中。

ZCode 是 Windows 桌面应用，不安装进容器，通过 Docker 或 SSH 连接开发容器。`openssh-client` 默认安装，用于 Git 和 SSH；只有确实需要通过 SSH 连接容器时，才安装 `openssh-server`。

基础镜像单独构建一次，保持 `agentlab-base:latest` 标签。更新后，重建项目镜像以继承变更。

每个项目的 Dockerfile 继承基础镜像：

```dockerfile
FROM agentlab-base:latest
```

项目按语言和用途选择所需环境：

```text
Node.js / pnpm
Rust / Cargo
Mojo
Go
JDK
.NET
build-essential
CMake / Ninja
CUDA
Playwright
数据库客户端
Docker CLI
openssh-server
Git LFS
GitHub CLI
```

判断工具是否进入基础镜像，只问：所有项目是否都常用？如果不是，就放到项目镜像。

工具和数据的归属：

- 基础镜像：Agent 通用工作环境；
- 项目镜像：项目语言、编译器和专用工具；
- volume / bind mount：配置、缓存、源码和长期数据。

### Wiki MCP

Wiki MCP 由 Rust 开发，保存在 `obsidian` 仓库，运行在开发容器中。Windows 上的 ZCode 通过 Docker 或 SSH 连接后，加载容器内的 MCP 配置，启动容器内的 MCP 可执行文件。

优先使用已编译好的稳定二进制：编译一次，将可执行文件复制进容器，由 ZCode 直接启动。普通项目使用 Wiki MCP 不需要 Rust / Cargo；基础镜像不含 Rust。

只有开发、调试 Wiki MCP，或必须在目标环境现场编译时，才在项目镜像中安装 Rust 工具链。

### 持久数据与缓存

环境变量决定程序往哪里写；volume 决定容器重建后数据是否保留。

| 数据 | 容器路径 | 规则 |
| --- | --- | --- |
| ZCode | `/home/dev/.zcode` | 持久 |
| Claude | `/home/dev/.claude` | 持久 |
| Codex | `/home/dev/.codex` | 持久 |
| 下载缓存 | `/cache` | 可清理 |
| Cargo | `/home/dev/.cargo` | 持久，不能整卷删除 |
| Node | `/workspace/node_modules` | 可重建 |
| Python | `/workspace/.venv` | 可重建 |
| Rust build | `/workspace/target` | 可重建 |
| Obsidian | `/vault` | Windows bind mount |

常用环境变量：

```text
CLAUDE_CONFIG_DIR=/home/dev/.claude
CODEX_HOME=/home/dev/.codex

npm_config_cache=/cache/npm
UV_CACHE_DIR=/cache/uv
PIP_CACHE_DIR=/cache/pip

CARGO_HOME=/home/dev/.cargo
CARGO_TARGET_DIR=/workspace/target

PLAYWRIGHT_BROWSERS_PATH=/cache/playwright
UV_LINK_MODE=copy
```

pnpm：

```text
storeDir=/cache/pnpm/store
cacheDir=/cache/pnpm/cache
```

这些变量写进项目 Compose，不写 Windows 全局环境变量。

### 共享 Obsidian

仓库：[MrSong9957/Obsidian](https://github.com/MrSong9957/Obsidian)。

Windows 只保留一份 `D:\AgentLab\shared\Obsidian`，所有容器统一通过 `/vault` 读写：

```yaml
- type: bind
  source: 'D:/AgentLab/shared/Obsidian'
  target: /vault
  read_only: false
  bind:
    create_host_path: false
```

按 Obsidian 仓库已有规则管理，并遵守：

- 避免多个 Agent 同时修改同一文件；
- Git pull、commit、push 由明确的 Agent 统一处理；
- 长期知识、经验和文档放进 Obsidian；
- 构建缓存、索引和原始运行数据留在开发环境。

### 构建与挂载踩坑教训

继承自早期 MiMo-Code 容器实践（2026-11-16 从 `docker/` 目录归档，原始细节见 git 历史）：

- **构建参数代理泄漏**：compose 中 `args: HTTP_PROXY: "${HTTP_PROXY:-}"` 不是安全的"默认为空"——宿主机环境变量优先级更高，会让构建走不可达代理。要禁用代理就硬编码空值。
- **Windows CRLF 行尾**：宿主机编辑过的 `.sh` 复制进镜像前必须 `sed -i 's/\r$//'`，否则 entrypoint 报语法错误。
- **PATH 污染**：compose `environment` 里写 `PATH=...:$PATH` 会在宿主机展开，把 Windows PATH 注入容器导致 `command not found`。容器 PATH 一律在 Dockerfile `ENV` 中定死，不从 compose 注入。
- **slim 镜像缺包**：`*-slim` 基础镜像不预装 `unzip` 等工具，Bun 等官方安装脚本会失败；安装前按脚本依赖核对 apt 包列表。
- **基础镜像无 xz**：`agentlab-base`（继承 ubuntu:24.04）不预装 `xz-utils`，`tar -xJ` 解压 `.tar.xz` 会失败（实测 2026-11-18 装 Node 时踩到）。官方二进制发布一般同时有 `.tar.gz`，优先用 tar.gz；确需 xz 再 `apt-get install xz-utils`。

## 5. Agent 接入

### 启动项目

```powershell
Set-Location 'D:\AgentLab\projects\demo'

docker compose -f .devcontainer/compose.yaml config --quiet
docker compose -f .devcontainer/compose.yaml up -d --build
```

配置检查失败时先修复，不能继续启动。

### ZCode

Docker Desktop 使用 WSL2 后端时，连接类型仍选 `Docker`。

打开容器 `agentlab-demo-dev`，目录选 `/workspace`。

项目 Skill / MCP：

```text
/workspace/.zcode/skills/
/workspace/.zcode/config.json
```

用户级 MCP：

```text
/home/dev/.zcode/cli/config.json
```

Windows 路径必须转换成 Linux 容器路径。不要把 Windows 整份 `.zcode` 或设备相关凭据直接复制到容器。

### Claude / Codex

在容器中启动：

```powershell
docker compose -f .devcontainer/compose.yaml exec -u dev -w /workspace dev claude
```

```powershell
docker compose -f .devcontainer/compose.yaml exec -u dev -w /workspace dev codex
```

需要扫码、验证码或本人授权时，才交给用户。

### SSH

仅在桌面工具确实需要 SSH 时配置：

- 只绑定 `127.0.0.1`；
- 不同项目使用不同端口；
- 交付主机、端口、用户名和密钥位置。

连接后检查当前用户、`/workspace`、Agent 配置目录和缓存目录，不能只证明“连上了”。

## 6. 验收与维护

### 实际验收

必须验证实际结果，不能只检查配置。

检查挂载：

```powershell
docker inspect agentlab-demo-dev --format '{{json .Mounts}}'
```

| 项目 | 必须确认 |
| --- | --- |
| Docker | 主要数据实际增长在 D 盘 |
| 源码 | `/workspace` 指向正确项目 |
| Agent | 能真实执行任务 |
| 配置 | 实际写入对应持久卷 |
| 缓存 | 实际写入指定目录 |
| Obsidian | `/vault` 可读写 |
| 构建 | 至少完成一次真实安装、构建或测试 |
| 重建 | 配置、会话和长期数据仍然存在 |
| C 盘 | 剩余增长可以定位到具体目录 |

执行重建测试：

```powershell
docker compose -f .devcontainer/compose.yaml up -d --build --force-recreate
```

重建后再次检查 Agent、项目、配置和 `/vault`。

### 备份与清理

| 数据类型 | 处理方式 |
| --- | --- |
| 长期数据 | 先备份，默认保留 |
| 可重建数据 | 验收后可清理 |
| 不确定数据 | 先确认用途，不删除 |

备份放在 `D:\AgentLab\backups\<project>\`。Docker volume 使用 Docker 方式导出备份。

通常可清理：

- npm / pnpm / pip / uv 缓存；
- `node_modules`、`.venv`、Rust `target`；
- Docker build cache；
- 已确认无用的 Agent 运行输出。

禁止：

- 把 `docker compose down -v` 当普通重建；
- 全局清理未知 volume；
- 删除未确认用途的 Agent 配置、会话或 Cargo 数据。

Docker 数据迁到 D 盘后，C 盘旧缓存不会自动消失。删除虚拟磁盘内部数据，也不代表 `.vhdx` 会立即缩小。

处理顺序：

```text
迁移并验收
→ 定位 C 盘剩余占用
→ 判断用途
→ 清理允许删除的数据
→ 必要时再处理虚拟磁盘缩容
```

### 虚拟磁盘缩容

`docker_data.vhdx` 只增不减：删除镜像、容器、卷、构建缓存后，空间只在虚拟磁盘内部标记为可复用，文件体积不变。判断依据：`docker system df` 的存活数据远小于 `.vhdx` 体积（如差距超过 2~3 倍）时执行缩容。实测 191 GB → 4.2 GB，压缩约 90 秒。

```powershell
# 1. Docker 内部先清空，缩容才有空间可还
docker builder prune -a -f     # 构建缓存全清，下次构建自动重建
docker images                   # 只保留 agentlab-base:latest 和 ubuntu:24.04，其余 docker rmi

# 2. 停止 Docker Desktop 与 WSL
docker desktop stop
wsl --shutdown
```

diskpart 脚本（如 `compact.txt`）：

```text
select vdisk file="D:\AgentLab\docker-data\DockerDesktopWSL\disk\docker_data.vhdx"
attach vdisk readonly
compact vdisk
detach vdisk
```

本机没有 Hyper-V 模块（`Optimize-VHD` 不可用），用 diskpart 压缩，需要管理员权限。当前 shell 非管理员时，先写 runner.ps1（内容为 `diskpart /s "<compact.txt 路径>" > "<日志路径>" 2>&1`），再提权运行：

```powershell
Start-Process powershell -Verb RunAs -Wait -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','<runner.ps1 路径>'
```

执行时确认 UAC 弹窗。`wsl --shutdown` 会停止所有 WSL 发行版，先保存工作（第 1 节规则）。

```powershell
# 3. 重启并验收
docker desktop start
docker images                   # agentlab-base、ubuntu:24.04 完好
```

### 清理基准

全量清理并缩容后，`D:\AgentLab` 的正常基线约 6.3 GB，以下内容必须保留：

| 路径 | 占用 | 内容 |
| --- | --- | --- |
| `docker-data\...\disk\docker_data.vhdx` | ~4.3 GB | 存活镜像 `agentlab-base:latest`、`ubuntu:24.04` 与 overlay2 元数据 |
| `wsl-data\swap.vhdx` | ~1.9 GB | WSL swap（第 3 节配置） |
| `docker-data\...\main\ext4.vhdx` | ~96 MB | Docker Desktop 系统发行版盘 |
| `shared\Obsidian` | ~136 MB | 共享知识库（git 仓库） |
| `base-image\Dockerfile` | — | 基础镜像定义源文件 |
| `projects\`、`backups\` | — | 项目与备份目录 |

注意：`docker system df` 中镜像的 RECLAIMABLE 只表示"没有容器在用"，保留镜像显示可回收是正常的，不作为清理依据。清理对象只看：构建缓存、无标签镜像、确认废弃的项目镜像与卷。

## 7. 完成标准

Agent 最终报告必须包含：

- 实际项目路径、Docker 数据位置；
- 修改过的配置、容器和 volume；
- Agent 连接方式、`/vault` 是否可读写；
- 构建或测试结果、容器重建结果；
- C / D 盘实际数据位置；
- 未执行项目及原因。

仅生成配置文件不算完成，必须同时满足：

```text
能开发
+ 能重建
+ 长期数据不丢
+ Obsidian 可共享
+ 大量开发数据主要落到 D 盘
```

