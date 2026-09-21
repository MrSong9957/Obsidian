---
name: Docker 容器开发方案
description: Windows 使用 Docker Desktop WSL2 开发容器，将主要开发数据放到 D 盘，并让多个项目共享 Obsidian 仓库。
type: reference
updated: "2026-09-21"
tags: [Docker, WSL2, Agent, 缓存管理]
source: self
---

# Docker 容器开发方案

## 1. 方案

**Windows 保留桌面工具，Docker 提供 Linux 开发环境，主要开发数据统一放 D 盘。**

架构：

```text
Windows 桌面端
    ↓
Docker Desktop + WSL2
    ↓
Linux 开发容器
```

Agent 默认连续执行到完成，顺序固定：

```text
命令 / 配置文件
→ 桌面自动化
→ 用户手动兜底
```

只有权限、本人验证或工具能力确实不足时，才让用户处理对应步骤。

执行重启、关闭 Docker 或 WSL 前，先保存当前工作。

---

## 2. 存储规划

统一根目录：

```text
D:\AgentLab
```

| Windows 路径                       | 用途                  | 容器路径         |
| -------------------------------- | ------------------- | ------------ |
| `D:\AgentLab\docker-data`        | Docker 镜像、容器、volume | Docker 管理    |
| `D:\AgentLab\wsl-data`           | WSL swap            | —            |
| `D:\AgentLab\projects\<project>` | 项目源码                | `/workspace` |
| `D:\AgentLab\shared\Obsidian`    | 共享知识库               | `/vault`     |
| `D:\AgentLab\backups`            | 配置、会话等备份            | —            |

已有 D/E 盘项目可以原地使用，不必强行迁移。

项目名确定后不要随意修改，否则 Compose 可能创建新的 volume。

---

## 3. 宿主机迁移

### Docker 数据盘

目标：

```text
D:\AgentLab\docker-data
```

#### 尚未安装 Docker

安装时直接指定：

```text
--backend=wsl-2
--wsl-default-data-root=D:\AgentLab\docker-data
```

#### 已安装 Docker

当前没有通用 CLI 用于迁移现有 Docker Desktop 数据盘，因此由 Agent 操作：

```text
Settings
→ Resources
→ Advanced
→ Disk image location
→ D:\AgentLab\docker-data
→ Apply / Restart
```

迁移前记录已有镜像、容器和 volume。

目标目录已有数据时先确认来源，不能直接覆盖。

#### 验收

必须确认：

* Docker 正常启动；
* 原镜像、容器和 volume 仍然存在；
* D 盘出现实际 Docker 数据。

**只改了设置里的路径，不算迁移完成。**

禁止在 Docker 运行时手工移动 `.vhdx`。

---

### WSL swap

创建：

```text
D:\AgentLab\wsl-data
```

编辑：

```text
%USERPROFILE%\.wslconfig
```

配置：

```ini
[wsl2]
swapFile=D:\\AgentLab\\wsl-data\\swap.vhdx
```

已有 `[wsl2]` 时直接合并，不重复创建。

如果已经设置：

```ini
swap=0
```

说明 swap 已关闭，无须设置 swap 文件。

应用配置：

```powershell
docker desktop stop
wsl --shutdown
docker desktop start
docker info
```

如果当前版本没有 `docker desktop` 命令，就由 Agent 正常退出并重新启动 Docker Desktop。

验收：

```text
docker info 正常
+ swap 实际位于 D 盘
```

---

## 4. 项目容器

项目使用：

```text
<project>\.devcontainer\Dockerfile
<project>\.devcontainer\compose.yaml
```

已有文件先读取并合并，不能直接覆盖。

基本约定：

| 项目          | 示例                   |
| ----------- | -------------------- |
| Compose 项目名 | `agentlab-demo`      |
| 服务名         | `dev`                |
| 容器名         | `agentlab-demo-dev`  |
| 用户          | `dev`                |
| 用户目录        | `/home/dev`          |
| 工作目录        | `/workspace`         |
| 源码          | 项目根目录 → `/workspace` |

`compose.yaml` 位于 `.devcontainer` 时，源码可以挂载：

```yaml
source: ..
target: /workspace
```

bind mount 使用：

```yaml
bind:
  create_host_path: false
```

避免路径写错时自动创建空目录。

Dockerfile 基于通用基础镜像构建（见下文），只安装项目实际需要的语言和依赖，并保证 `dev` 用户能读写挂载目录。

### 基础镜像与项目环境

**不要让每个项目从零搭环境。先做一个通用 Ubuntu 基础镜像，项目再在它上面增加自己的开发环境。**

结构：

```text
Ubuntu 24.04
    ↓
agentlab-base
    ↓
项目镜像
```

#### 基础镜像

基础镜像只放**所有项目都常用的工具**：

```text
系统基础
├─ Ubuntu
├─ dev 用户
└─ ca-certificates

开发基础
├─ Python
├─ pip / venv / uv
└─ Git

Agent 常用工具
├─ curl
├─ openssh-client
├─ jq
├─ ripgrep
├─ tar / gzip / zip / unzip
├─ findutils / procps / less
│
└─ Claude Code / Codex
```

Python 作为 Agent 的通用脚本环境保留，但项目自己的 Python 依赖仍放进项目 `.venv`。

ZCode **不安装进容器**。它是 Windows 桌面应用，通过 Docker 或 SSH 连接开发容器。

`openssh-client` 默认安装，用于 Git 和 SSH。只有确实需要通过 SSH 连接容器时，才安装 `openssh-server`。

基础镜像单独构建一次，保持 `agentlab-base:latest` 标签；更新基础镜像后，重建项目镜像即可继承。

#### 项目镜像

每个项目继承基础镜像：

```dockerfile
FROM agentlab-base:latest
```

然后只增加本项目真正需要的环境，按语言和用途从下列内容中选择：

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

#### 原则

判断一个工具是否应该进入基础镜像，只问一个问题：

> 大多数项目是否都会用到？

如果不是，就放到项目镜像。

最终保持：

```text
基础镜像 = Agent 通用工作环境
项目镜像 = 项目语言、编译器和专用工具
volume / bind mount = 配置、缓存、源码和长期数据
```

这样公共环境只维护一次，同时不会把基础镜像堆成一个包含所有语言和工具的巨大镜像。

---

### Wiki MCP 的运行方式

**Wiki MCP 由 Rust 开发，保存在 `obsidian` 仓库中，运行在开发容器里，不在 Windows 上。** ZCode 在 Windows 通过 Docker 或 SSH 连接容器后，加载的是容器内的 MCP 配置，启动的是容器内的 MCP 可执行文件。

Wiki MCP 优先使用已编译好的稳定二进制：编译一次，把可执行文件复制进容器，ZCode 直接启动。普通项目因此不需要安装 Rust / Cargo；基础镜像同样不含 Rust，Rust 按需放进项目镜像。

只有两种情况才在容器里安装 Rust：正在开发或调试这个 MCP；或它必须在目标环境现场编译。

> **使用 Wiki MCP，只需要可执行文件；开发 MCP，才需要 Rust 工具链。**

---

### 持久数据和缓存

核心原则：

**环境变量决定程序往哪里写；volume 决定容器重建后数据还在不在。**

| 数据         | 容器路径                      | 规则                 |
| ---------- | ------------------------- | ------------------ |
| ZCode      | `/home/dev/.zcode`        | 持久                 |
| Claude     | `/home/dev/.claude`       | 持久                 |
| Codex      | `/home/dev/.codex`        | 持久                 |
| 下载缓存       | `/cache`                  | 可清理                |
| Cargo      | `/home/dev/.cargo`        | 持久，不能整卷删除          |
| Node       | `/workspace/node_modules` | 可重建                |
| Python     | `/workspace/.venv`        | 可重建                |
| Rust build | `/workspace/target`       | 可重建                |
| Obsidian   | `/vault`                  | Windows bind mount |

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

---

### 共享 Obsidian

仓库：

```text
https://github.com/MrSong9957/Obsidian
```

Windows 只保留一份：

```text
D:\AgentLab\shared\Obsidian
```

所有容器统一访问：

```text
/vault
```

Compose：

```yaml
- type: bind
  source: 'D:/AgentLab/shared/Obsidian'
  target: /vault
  read_only: false
  bind:
    create_host_path: false
```

规则：

* 多个项目都可以读写 `/vault`；
* 按 Obsidian 仓库已有规则管理；
* 同一文件避免多个 Agent 同时修改；
* Git pull、commit、push 由明确的 Agent 统一处理；
* 长期知识、经验和文档进 Obsidian；
* 构建缓存、索引和原始运行数据留在开发环境。

---

## 5. Agent 接入

启动项目：

```powershell
Set-Location 'D:\AgentLab\projects\demo'

docker compose -f .devcontainer/compose.yaml config --quiet
docker compose -f .devcontainer/compose.yaml up -d --build
```

配置检查失败时先修复，不能继续启动。

## ZCode

Docker Desktop 使用 WSL2 后端时，连接类型仍选：

```text
Docker
```

打开：

```text
容器：agentlab-demo-dev
目录：/workspace
```

项目 Skill / MCP：

```text
/workspace/.zcode/skills/
/workspace/.zcode/config.json
```

用户级 MCP：

```text
/home/dev/.zcode/cli/config.json
```

Windows 路径必须转换成 Linux 容器路径。

不要把 Windows 整份 `.zcode` 或设备相关凭据直接复制到容器。

---

## Claude / Codex

直接在容器中启动：

```powershell
docker compose -f .devcontainer/compose.yaml exec -u dev -w /workspace dev claude
```

```powershell
docker compose -f .devcontainer/compose.yaml exec -u dev -w /workspace dev codex
```

需要扫码、验证码或本人授权时，才交给用户。

---

## SSH

只有桌面工具确实需要 SSH 时才配置。

要求：

* 只绑定 `127.0.0.1`；
* 不同项目使用不同端口；
* 明确交付主机、端口、用户名和密钥位置。

连接后检查真实环境：

```text
当前用户
/workspace
Agent 配置目录
缓存目录
```

不能只证明“连上了”。

---

## 6. 验收与维护

**配置正确不等于系统真的按预期工作，必须验证实际结果。**

检查挂载：

```powershell
docker inspect agentlab-demo-dev --format '{{json .Mounts}}'
```

验收标准：

| 项目       | 必须确认                |
| -------- | ------------------- |
| Docker   | 主要数据实际增长在 D 盘       |
| 源码       | `/workspace` 指向正确项目 |
| Agent    | 能真实执行任务             |
| 配置       | 实际写入对应持久卷           |
| 缓存       | 实际写入指定目录            |
| Obsidian | `/vault` 可读写        |
| 构建       | 至少完成一次真实安装、构建或测试    |
| 重建       | 配置、会话和长期数据仍然存在      |
| C 盘      | 剩余增长可以定位到具体目录       |

重建测试：

```powershell
docker compose -f .devcontainer/compose.yaml up -d --build --force-recreate
```

重建后再次检查 Agent、项目、配置和 `/vault`。

---

## 备份和清理

只分三类：

| 类型    | 处理        |
| ----- | --------- |
| 长期数据  | 先备份，默认保留  |
| 可重建数据 | 验收后可清理    |
| 不确定数据 | 先确认用途，不删除 |

备份放：

```text
D:\AgentLab\backups\<project>\
```

Docker volume 使用 Docker 方式导出备份。

可清理内容通常包括：

* npm / pnpm / pip / uv 缓存；
* `node_modules`；
* `.venv`；
* Rust `target`；
* Docker build cache；
* 已确认无用的 Agent 运行输出。

禁止：

```text
把 docker compose down -v 当普通重建
全局清理未知 volume
删除未确认用途的 Agent 配置、会话或 Cargo 数据
```

Docker 数据迁到 D 盘后，C 盘旧缓存不会自动消失。

删除虚拟磁盘内部数据，也不代表 `.vhdx` 会立即缩小。

正确顺序：

```text
迁移并验收
→ 定位 C 盘剩余占用
→ 判断用途
→ 清理允许删除的数据
→ 必要时再处理虚拟磁盘缩容
```

---

## 完成标准

Agent 最终报告：

* 实际项目路径；
* Docker 数据位置；
* 修改过的配置；
* 容器和 volume；
* Agent 连接方式；
* `/vault` 是否可读写；
* 构建或测试结果；
* 容器重建结果；
* C / D 盘实际数据位置；
* 未执行项目及原因。

**生成配置文件不等于完成。**

真正完成必须同时满足：

```text
能开发
+ 能重建
+ 长期数据不丢
+ Obsidian 可共享
+ 大量开发数据主要落到 D 盘
```
