---
name: Docker 容器开发方案
description: Windows 上把 Docker Desktop（WSL2 后端）数据盘迁移到 D 盘、devcontainer/Compose 持久卷与缓存路径规划的执行方案——Agent 优先三级执行（命令→桌面自动化→用户兜底）
type: reference
updated: "2026-09-20"
tags: [Docker, WSL2, Agent, 缓存管理]
source: self
document_id: agentlab-docker-development-guide
version: "1.2"
status: "待执行；本文不代表已在目标电脑完成配置"
audience: [Windows 用户, 本机 Agent]
platform: "Windows + Docker Desktop + Linux 容器"
backend: WSL2
execution_policy: "命令和配置文件优先，其次 Agent 桌面自动化，用户手动兜底"
base_directory: 'D:\AgentLab'
example_project_id: demo
example_project_directory: 'D:\AgentLab\projects\demo'
---

# Docker 容器开发方案

目标：统一开发环境，把主要开发数据集中到 D 盘，减少 C 盘持续增长。**Docker 数据盘的迁移目标固定为 `D:\AgentLab\docker-data`。** 桌面应用仍可能在 C 盘保留部分数据，需要单独检查。

采用 **Windows 桌面端＋Docker Desktop WSL2 后端＋Linux 开发容器**。Docker VMM 暂作为性能对比选项，不改变本教程的存储规则。

**执行顺序：命令和配置文件 → Agent 操作桌面 → 用户手动兜底。**

本教程按你常用的 ZCode、Codex 具备电脑控制能力来安排。Agent 先确认本次会话可用的终端、文件和桌面工具，按下面的顺序完成每一步：

| 优先级 | 谁做、怎么做 |
| --- | --- |
| 1．命令或配置文件 | Agent 使用当前版本支持的命令、脚本或配置文件，执行后检查实际结果。 |
| 2．桌面自动化 | 没有合适命令时，Agent 自己打开设置、填写路径、点击应用并检查结果。需要点界面，不等于需要用户做。 |
| 3．用户兜底 | 只有缺少必要信息、工具或权限确实受阻，或遇到必须本人完成的验证时，才交给用户处理那一步。 |

后文默认由 Agent 完成；**【用户兜底】只在写明的条件出现时启用。** 交接时说清“卡在哪里、为什么做不了、用户只需做什么、完成后如何判断”；接手后由 Agent 继续验证，不重复停在等待状态。

**已明确的项目、路径和操作范围直接沿用，不需要每一步重新确认。** 迁移或重启前先保存工作；若会关闭当前 Agent 所在的应用或容器，先安排独立的宿主机会话接续，无法安排时才请用户完成这一步。下文的 `demo` 只是示例，不能当成真实项目直接执行。

1. **【Agent 优先】先确定路径，检查环境。**

   从当前项目和已有说明确定实际项目目录、工具和版本；检查 D 盘可用空间、WSL2 和 Docker 是否可用，创建缺少的下列目录。缺少软件或系统功能时，优先通过官方安装命令补齐，需要界面操作时由 Agent 完成。

   **【用户兜底】**项目无法确定时提供目录；需要本人批准的管理员弹窗、BIOS 设置或无法自动完成的重启恢复，才由用户处理。

   | 固定路径 | 用途 |
   | --- | --- |
   | **`D:\AgentLab\docker-data`** | **Docker 数据盘迁移目标**，存放镜像、容器和数据卷 |
   | `D:\AgentLab\wsl-data` | WSL 交换文件 |
   | `D:\AgentLab\projects` | 新项目源码 |
   | `D:\AgentLab\desktop-data` | 桌面应用允许迁移的数据，每个应用单独建子目录 |
   | `D:\AgentLab\backups` | 配置、会话等备份，每个项目单独建子目录 |

   以下用项目代号 `demo`、源码目录 `D:\AgentLab\projects\demo` 举例。**Agent 先确定真实项目代号和源码目录，再统一替换后文的示例路径、容器名和卷名。** 已有 D/E 盘项目可以原地使用。没有 D 盘时，先让用户选定非系统盘，再统一修改，不能悄悄改回 C 盘。

   项目代号创建后不要随意更改，否则可能连接到新卷，看起来像数据丢失。

2. **【Agent 优先】迁移 Docker 数据盘。**

   **Agent：**记录当前存储位置和已有镜像、容器、数据卷，检查目标目录。目标目录已有 Docker 数据时，先核对来源，不直接覆盖。

   **尚未安装 Docker 时：**Agent 按[官方安装说明](https://docs.docker.com/desktop/setup/install/windows-install/)使用安装参数 `--backend=wsl-2`、`--wsl-default-data-root=D:\AgentLab\docker-data`，直接指定数据位置。使用实际安装包路径；这些是安装参数，不是已有数据的迁移命令。

   **已经安装时：**本次核对的[官方 CLI](https://docs.docker.com/reference/cli/docker/desktop/)没有列出数据盘迁移命令。Agent 保存工作、正常停止开发任务，再用电脑控制工具操作 Docker Desktop：

   **Settings → Resources → Advanced → Disk image location（磁盘镜像位置）→ 选择 `D:\AgentLab\docker-data` → 应用并重启。**

   这里选的是**文件夹**，不需要自行填写 `.vhdx` 文件名。Docker 可能在里面自动创建子目录。[官方说明](https://docs.docker.com/desktop/features/wsl/)

   **Agent 验证：**重新打开设置，确认位置已保存；检查 D 盘目标下的实际磁盘文件，确认 Docker 恢复运行，原来的镜像、容器和数据卷仍可见。只改了路径文字，不能算迁移完成。

   **【用户兜底】**Agent 确实无法控制该设置页或处理必要授权时，用户按上面的同一路径完成设置；其余准备和验证仍由 Agent 负责。

   **不要把 `projects`、`desktop-data` 或 `backups` 填进这个设置，也不要在 Docker 运行时手工搬动虚拟磁盘文件。** 找不到该设置时，先核对版本和后端，不猜替代路径。

3. **【Agent 优先】把 WSL 交换文件也指定到 D 盘。**

   **Agent：**创建 `D:\AgentLab\wsl-data`，编辑当前 Windows 用户的配置文件：

   ```text
   %USERPROFILE%\.wslconfig
   ```

   这是文件位置的写法；在 PowerShell 中使用 `$env:USERPROFILE` 获取当前用户目录。在已有的 `[wsl2]` 段中添加或修改以下内容；没有文件时再新建：

   ```ini
   [wsl2]
   swapFile=D:\\AgentLab\\wsl-data\\swap.vhdx
   ```

   保留其他设置，不重复添加 `[wsl2]` 段。如果原本已设置 `swap=0`，保持禁用即可，无须修改交换文件位置。

   Agent 保存工作、正常停止相关任务，先用 `docker desktop --help` 确认命令可用，再在 **Windows PowerShell** 逐条执行；上一条成功后再继续：

   ```powershell
   docker desktop stop
   wsl --shutdown
   docker desktop start
   docker info
   ```

   `docker info` 能返回服务端信息，才算 Docker 恢复运行；同时核对交换文件的位置。当前版本没有 `docker desktop` 命令时，Agent 用桌面工具正常退出、启动 Docker Desktop。[Docker Desktop CLI 说明](https://docs.docker.com/desktop/features/desktop-cli/)

   **【用户兜底】**命令和桌面操作都受阻，或有 Agent 无法代为保存的工作时，用户只处理受阻部分。

   `wsl --shutdown` 会停止所有 WSL 环境。小配置文件仍在用户目录，大交换文件才放到 D 盘。[Microsoft 说明](https://learn.microsoft.com/en-us/windows/wsl/wsl-config)

4. **【Agent】在项目里生成容器配置。**

   示例文件位置固定为：

   ```text
   D:\AgentLab\projects\demo\.devcontainer\Dockerfile
   D:\AgentLab\projects\demo\.devcontainer\compose.yaml
   ```

   已有这些文件时先读取、合并，不直接覆盖。Agent 根据项目实际语言、版本和使用的工具生成完整配置；下面的表格是配置要求，不是可以直接运行的配置文件。

   | 配置项 | 示例固定值 |
   | --- | --- |
   | Compose 项目名 | `agentlab-demo` |
   | 服务名 | `dev` |
   | 容器名 | `agentlab-demo-dev` |
   | 开发用户及用户目录 | `dev`、`/home/dev` |
   | 容器工作目录 | `/workspace` |
   | 镜像构建目录 | `.devcontainer` |
   | 源码挂载 | 项目根目录 → `/workspace` |

   `compose.yaml` 位于 `.devcontainer` 内，因此源码挂载使用 `source: ..`。使用长格式挂载，并设置 `bind.create_host_path: false`，避免路径写错时自动创建空目录。[Compose 说明](https://docs.docker.com/reference/compose-file/services/)

   Dockerfile 安装项目需要的工具和 CLI，并确保所有挂载目录对 `dev` 可写。容器启动后必须持续运行，ZCode 才能连接。

5. **【Agent】同时配置保存路径和持久卷。**

   **环境变量告诉程序往哪里写；持久卷让数据在重建容器后保留。两项都要配置。**

   在 Compose 中按下表填写数据卷的 `name` 和挂载目标。未使用的工具不必创建对应卷。

   | 数据卷名称 | 容器内挂载位置 | 管理规则 |
   | --- | --- | --- |
   | `agentlab-demo-zcode` | `/home/dev/.zcode` | 保留配置和所需会话，输出记录另清 |
   | `agentlab-demo-claude` | `/home/dev/.claude` | 保留 |
   | `agentlab-demo-codex` | `/home/dev/.codex` | 保留 |
   | `agentlab-demo-cache` | `/cache` | 下载缓存，可清理 |
   | `agentlab-demo-cargo` | `/home/dev/.cargo` | 含缓存、配置和命令，不能整卷当缓存删除 |
   | `agentlab-demo-node` | `/workspace/node_modules` | 可重建 |
   | `agentlab-demo-venv` | `/workspace/.venv` | 可重建 |
   | `agentlab-demo-target` | `/workspace/target` | 可重建 |

   这些卷由 Compose 首次启动时创建，实际位于 **D 盘 Docker 虚拟磁盘内部**，不是 Windows 普通文件夹。[数据卷说明](https://docs.docker.com/reference/compose-file/volumes/)

   将实际使用的变量写入 `compose.yaml` 的 `services.dev.environment`，不要写进 Windows 全局环境变量：

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

   pnpm 按安装版本配置：`storeDir=/cache/pnpm/store`、`cacheDir=/cache/pnpm/cache`。`UV_LINK_MODE=copy` 用于处理缓存与虚拟环境位于不同卷的情况，可能增加复制占用。[uv 说明](https://docs.astral.sh/uv/guides/integration/docker/)

   Claude Code 必须同时设置上述目录和变量，才能避免遗漏目录外的配置文件。[官方说明](https://code.claude.com/docs/en/devcontainer)

6. **【Agent 优先】启动容器并配置连接。**

   **Agent：**在 **Windows PowerShell** 进入实际项目根目录，检查配置并启动：

   ```powershell
   Set-Location 'D:\AgentLab\projects\demo'
   docker compose -f .devcontainer/compose.yaml config --quiet
   docker compose -f .devcontainer/compose.yaml up -d --build
   ```

   第一条 Compose 命令检查配置；检查失败就先修配置，不继续启动。

   | 使用方式 | Agent 默认完成 | 用户兜底条件 |
   | --- | --- | --- |
   | ZCode 桌面端 | 准备容器，自动操作：项目 → ＋ → 远程连接 → Docker → 选择 `agentlab-demo-dev` → 打开 `/workspace` | 界面工具无法完成时，用户只代做该连接操作 |
   | CLI | 检查安装、启动交互终端，复用可用登录状态或打开登录流程 | 需要本人扫码、验证码或授权，或 Agent 无法操作交互终端 |
   | Claude、Codex 桌面端的 SSH 连接 | 配置容器 SSH，在支持的桌面版本中自动添加连接，验证可用，并交付主机、端口、用户名和密钥位置 | 自动配置与桌面操作均受阻，或需要本人验证 |

   Docker Desktop 使用 WSL2 后端时，ZCode 的连接类型仍选择 **Docker**。

   CLI 启动命令由 Agent 在 **Windows PowerShell、项目根目录** 执行。没有交互终端工具时，先尝试桌面终端；两种方式都无法完成，才交给用户：

   ```powershell
   docker compose -f .devcontainer/compose.yaml exec -u dev -w /workspace dev claude
   docker compose -f .devcontainer/compose.yaml exec -u dev -w /workspace dev codex
   ```

   **SSH 仅在使用该连接方式时配置。** Agent 必须给出可用的连接信息，不能只写“通过 SSH 连接”就算完成。端口只绑定本机 `127.0.0.1`，不同项目避免端口冲突。

   SSH 登录不一定继承 Compose 的全部环境变量。Agent 从桌面工具发起一次检查任务，核对实际使用的用户、项目和保存路径。

7. **【Agent 优先】补齐 ZCode 的技能和工具。**

   **Agent：**配置所需技能和 MCP，转换路径、安装容器内依赖，并检查能否运行。

   | 内容 | 容器路径 |
   | --- | --- |
   | 项目技能 | `/workspace/.zcode/skills/` |
   | 项目 MCP 配置 | `/workspace/.zcode/config.json` |
   | 用户级 MCP 配置 | `/home/dev/.zcode/cli/config.json` |

   优先由 Agent 写入配置并安装依赖；需要界面中的 Sync 时，也由 Agent 点击并验证。ZCode 的官方 Sync 目前只明确支持 SSH、WSL。Docker 直连时单独配置；需要 Sync，可以改用 SSH 连接同一个 Docker 容器。[官方说明](https://zcode.z.ai/cn/docs/remote-development)

   Windows 路径要改成容器路径，Windows 专用程序要换成 Linux 版本。不要把 Windows 的整份 `.zcode` 和设备加密凭据直接覆盖进容器。

   **桌面数据位置：**Agent 先记录并检查现有位置；需要调整时，优先用当前版本支持的配置方式，否则自动操作应用设置。`D:\AgentLab\desktop-data\zcode` 只接收桌面端明确支持迁移的数据，不能假定该设置覆盖全部 ZCode 数据。

   **【用户兜底】**需要本人完成 MCP 授权，或 Agent 确实无法操作对应设置时，用户只接手这一项。

8. **【Agent 优先】先验收，再清理旧数据。**

   **Agent：**在 Windows PowerShell 查看实际挂载，对照第 4、5 步确认源码路径、卷名和容器目标一致：

   ```powershell
   docker inspect agentlab-demo-dev --format '{{json .Mounts}}'
   ```

   | 检查 | 谁执行 | 通过标准 |
   | --- | --- | --- |
   | 桌面工具连接 | Agent 自动发起真实任务并检查 | 使用 `dev`，项目位于 `/workspace` |
   | 实际写入位置 | Agent | 新会话和下载缓存写入指定目录，不只是变量显示正确 |
   | 一次安装、构建、测试 | Agent | 正常完成 |
   | 重建容器 | Agent 重建，检查文件并打开桌面会话 | 源码、所需配置和会话仍能使用 |
   | 磁盘变化 | Agent | 大量开发数据增长发生在 D 盘；C 盘残留增长能定位到具体目录 |

   Agent 先保存工作、正常结束相关任务，再在项目根目录执行重建：

   ```powershell
   docker compose -f .devcontainer/compose.yaml up -d --build --force-recreate
   ```

   日常清理也按分工执行：

   | 谁做 | 操作 |
   | --- | --- |
   | Agent | 沿用已明确的会话保留规则；保存工作、正常退出相关应用，再执行清理。 |
   | Agent | 先备份配置和会话到 `D:\AgentLab\backups\demo\`，文件名包含日期和时间；备份数据卷时通过 Docker 导出。停止相关任务后，按第 5 步的规则清理已选定的缓存和编译输出。 |
   | Agent | 配置 Compose 日志轮转，可设每文件 10 MB、保留 3 个；构建缓存可设 20 GB 回收目标。优先修改受支持的配置文件，需要在 Docker Desktop 界面应用时也由 Agent 操作。这两项不限制应用自己写入的文件。 |
   | Agent | ZCode 完全退出、没有后台命令后，才清理对应环境的 `cli/exec`：Windows 为 `%USERPROFILE%\.zcode\cli\exec`，容器为 `/home/dev/.zcode/cli/exec`。旧命令输出将无法回看；其他会话按保留规则，优先通过应用管理界面处理。[FAQ](https://zcode.z.ai/cn/docs/qa) |
   | 用户兜底 | 保留范围不明确时作出选择；有 Agent 无法代为保存的工作，或操作确实受阻时，处理该项。 |

   **不用 `down -v` 或全局数据卷清理来重建环境。** 删除容器不会删除这里配置的持久卷，镜像和构建缓存也不会一起消失。

   迁移不会自动清掉原来留在 C 盘的缓存，删除内部文件也不保证虚拟磁盘立即缩小。Agent 在验收后列出剩余目录和占用，再按已明确的清理范围处理；磁盘缩容单独安排。

   **Agent 最后交付：**实际路径、修改的文件、连接方法、验收结果，以及确实需要用户接手的步骤和原因。没执行的项目写“未执行”，只生成配置文件不能写成“已部署”。
