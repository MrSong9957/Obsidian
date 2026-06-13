---
name: new-project-setup
description: 新项目初始化统一入口——Docker 容器启动、镜像配置验证、CLAUDE.md/ECC/Superpowers 补充；整合智谱MCP、首次配置（SafetyNet/StatusLine）、CLAUDE.md 模板、ECC 补强的配置参考
type: reference
updated: 2026-06-14
tags: docker, setup, claude-md, ecc, superpowers, zhipu-mcp, safety-net, configuration, "[[wiki/claude-md-base.md]]", "[[wiki/ecc-superpowers-supplement.md]]", "[[wiki/zhipu-mcp.md]]", "[[wiki/claude-code-p2.md]]"
source: https://example.com
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

#### 方式三：configure.ps1 预配置（插件和配置更新）

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

#### 各项目应用更新

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
- ❌ `docker-compose.yml` 用 `HTTP_PROXY: ${HTTP_PROXY:-}` — 宿主机环境变量优先级高于 `.env` 空值，代理仍会泄漏。**必须硬编码 `HTTP_PROXY: ""`**
- ❌ `PATH=/home/app/.bun/bin:$PATH` 写在 docker-compose.yml environment 中 — `$PATH` 在宿主机展开，Windows PATH 注入容器，覆盖 Linux PATH，导致 `bash: command not found`。**删除该行，依赖 Dockerfile 的 `ENV PATH`**
- ❌ 忘记在 apt-get 依赖中加 `unzip` — Bun 安装脚本需要 unzip，`debian:bookworm-slim` 不预装
- ❌ entrypoint.sh 以 `user: "0:0"` 运行时未设置 PATH — root 用户的 PATH 可能不完整。Dockerfile 加 `ENV PATH`，entrypoint.sh 加 `export PATH`
- ❌ Windows 编辑的 `.sh` 文件带 CRLF — Linux bash 无法识别 `\r`。Dockerfile 中用 `sed -i 's/\r$//'` 修复
- ✅ 构建成功后确认：`docker compose exec app bash` 能正常进入

### 常见运行时问题

> 以下为容器运行时（非构建期）的常见问题。

- **`claude: command not found`** — bash 命令哈希缓存过期，执行 `hash -r` 清除缓存后重试
- **`Unable to connect to Anthropic services`** — 临时容器缺少 `--env-file`，必须通过 `--env-file D:\Files\.env` 提供 API key
- **`docker push` EOF 错误** — 国内网络推送到 Docker Hub 时的代理配置问题，见下方 [[#Docker Hub Push 代理配置]]

### Docker Hub Push 代理配置

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
| **ECC Rules** | 全局 `~/.claude/rules/ecc/common` + 项目级语言 rules 手动复制 | — | [[wiki/ecc-superpowers-supplement.md#2-rules-语言编码规范\|ECC Rules §2]] |
| **智谱 MCP** | 4 个 MCP 服务器（视觉/搜索/网页/仓库） | — | [[wiki/zhipu-mcp.md\|智谱 MCP]] |

### 手动配置步骤

以下展开表格中**未预烘焙**的配置项的具体操作。StatusLine 已由 `configure.ps1` 自动配置，详见下方 [[#StatusLine 配置详情]]。

#### 1. 官方插件市场 + 插件

```
/plugin
```

→ Marketplaces → + Add Marketplace → `https://github.com/anthropics/claude-plugins-official`

从官方市场安装以下 5 个插件：

- **superpowers** — 技能调度框架
- **frontend-design** — 前端设计辅助
- **learning-output-style** — 交互式学习输出风格
- **skill-creator** — 技能创建工具
- **context7** — 实时文档查询

> [!NOTE]
> 如果同时需要 **GitHub** 插件，也从官方市场安装。ECC 的 GitHub MCP 工具与官方 GitHub 插件功能不同，可并存。

#### 2. Safety Net

```bash
/plugin marketplace add kenryu42/cc-marketplace
/plugin install safety-net@cc-marketplace
```

安装后在 `settings.json` 添加全局 Hook，用于敏感内容扫描。完整 Hook 配置已在 [[#完整 settings.json 示例]] 中包含，复制该示例即可。

#### 3. ECC

```bash
/plugin marketplace add https://github.com/affaan-m/everything-claude-code
/plugin install ecc@ecc
```

安装后复制规则文件：

**全局规则**（所有项目共享）：

```bash
mkdir -p ~/.claude/rules/ecc
cp -r ~/.claude/plugins/marketplaces/ecc/rules/common ~/.claude/rules/ecc/
```

**项目规则**：每个项目按需安装，参见步骤 5.3。

> [!NOTE]
> 全局 CLAUDE.md 已由 `configure.ps1` 自动复制，项目级 CLAUDE.md 见步骤 5.2。

#### 4. 智谱 MCP

不推荐在基础镜像中安装。API KEY 会写入 `settings.json`，有泄露风险。如有需要，在具体项目中手动安装，参见 [[wiki/zhipu-mcp.md|智谱 MCP]]。

---

### StatusLine 配置详情

StatusLine 是 Claude Code 内置功能，通过 `settings.json` 的 `statusLine` 字段配置。当前配置：Python 脚本输出模型/目录/分支/上下文进度条，管道接入 Safety Net 显示安全网状态。

#### statusLine 字段

```json
"statusLine": {
  "type": "command",
  "command": "python3 ~/.claude/scripts/statusline.py | npx cc-safety-net --statusline"
}
```

管道结构：`statusline.py`（模型/目录/分支/上下文）→ `cc-safety-net --statusline`（附加安全网状态）。

#### statusline.py 脚本

路径：`~/.claude/scripts/statusline.py`。stdin 读取 JSON，字段：`model.display_name` / `workspace.current_dir` / `context_window.used_percentage`。

```python
#!/usr/bin/env python3
"""Claude Code StatusLine"""

import json
import sys
import os
import subprocess

sys.stdout.reconfigure(encoding="utf-8")


def get_git_branch(cwd: str) -> str:
    if not cwd:
        return ""

    check_dir = cwd
    while check_dir and check_dir not in ("/", "\\"):
        if os.path.isdir(os.path.join(check_dir, ".git")):
            try:
                result = subprocess.run(
                    ["git", "-C", check_dir, "rev-parse", "--abbrev-ref", "HEAD"],
                    capture_output=True,
                    text=True,
                    timeout=2
                )
                if result.returncode == 0:
                    return result.stdout.strip().replace("heads/", "")
                break
            except (subprocess.TimeoutExpired, Exception):
                break
        parent = os.path.dirname(check_dir)
        if parent == check_dir:
            break
        check_dir = parent
    return ""


def get_progress_bar(percentage: int, length: int = 10) -> str:
    filled = min(max(int(percentage * length / 100), 0), length)
    empty = length - filled
    return "█" * filled + "░" * empty


def main():
    try:
        input_data = json.load(sys.stdin)
    except json.JSONDecodeError:
        print("Error reading input")
        return

    model = input_data.get("model", {}).get("display_name", "")
    cwd = input_data.get("workspace", {}).get("current_dir", "")
    used_pct = input_data.get("context_window", {}).get("used_percentage")

    # Show last two directory levels with ~/ prefix
    if cwd:
        parts_list = cwd.replace("\\", "/").rstrip("/").split("/")
        parts_list = [p for p in parts_list if p]
        if len(parts_list) > 2:
            dir_display = "~/" + "/".join(parts_list[-2:])
        else:
            dir_display = "/".join(parts_list)
    else:
        dir_display = ""

    branch = get_git_branch(cwd)

    CYAN = "\033[96m"
    BLUE = "\033[94m"
    GREEN = "\033[92m"
    YELLOW = "\033[93m"
    RED = "\033[91m"
    DIM = "\033[2m"
    RESET = "\033[0m"

    # Context progress bar: green <50%, yellow <80%, red >=80%
    if used_pct is not None and isinstance(used_pct, (int, float)):
        if used_pct < 50:
            bar_color = GREEN
        elif used_pct < 80:
            bar_color = YELLOW
        else:
            bar_color = RED
        bar = get_progress_bar(int(used_pct))
        context_info = f"{bar_color}{bar}{RESET} {int(used_pct)}%"
    else:
        context_info = f"{DIM}--{RESET}"

    parts = [f"{CYAN}{model}{RESET}", f"{BLUE}{dir_display}{RESET}"]
    if branch:
        parts.append(f"{GREEN}{branch}{RESET}")
    parts.append(context_info)

    print(" | ".join(parts))


if __name__ == "__main__":
    main()
```

示例输出：`GLM-5.1 | ~/my-project | main | ████████░░ 75%`

#### 完整 settings.json 示例

statusLine + hooks + enabledPlugins 关键结构（`env` 和 `mcpServers` 因环境而异，此处省略）。Safety Net Hook（拦截 `rm -rf`、`git push --force` 等危险命令）包含在 `hooks` 字段中。

```json
{
  "statusLine": {
    "type": "command",
    "command": "python3 ~/.claude/scripts/statusline.py | npx cc-safety-net --statusline"
  },
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "npx cc-safety-net --claude-code"
          }
        ]
      }
    ]
  },  // Safety Net: 拦截危险 Bash 命令。v0.9.0+ 使用 --claude-code（从 stdin 读 JSON）
  "enabledPlugins": {
    "frontend-design@claude-plugins-official": true,
    "superpowers@claude-plugins-official": true,
    "context7@claude-plugins-official": true,
    "github@claude-plugins-official": true,
    "skill-creator@claude-plugins-official": true,
    "learning-output-style@claude-plugins-official": true,
    "safety-net@cc-marketplace": true,
    "ecc@ecc": true
  },
  "extraKnownMarketplaces": {
    "cc-marketplace": {
      "source": {
        "source": "github",
        "repo": "kenryu42/cc-marketplace"
      }
    }
  }
}
```

#### 备注

StatusLine 已由 `configure.ps1` 预烘焙到基础镜像（脚本 + settings.json 配置）。以上配置结构仅供参考，新容器无需手动操作。仅在新环境或脚本更新时需手动安装。

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
