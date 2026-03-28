---
name: docker-dev-container
description: |
  创建开箱即用的 Docker 开发环境，包含 Claude Code 和 OpenCode。
  当用户请求创建 Docker 开发环境、在容器中运行 Claude Code、设置 Docker 开发容器时使用此技能。
  支持自动挂载当前项目、环境变量传递、配置持久化。
---

# Docker 开发容器技能

为项目创建隔离的 Docker 开发环境，包含 Claude Code 和 OpenCode。

## 使用场景

用户请求以下内容时触发：
- "创建 Docker 开发环境"
- "为项目配置 Docker 容器"
- "设置 Claude Code Docker 容器"
- "在容器中运行 Claude Code"

## 核心架构

**完全隔离模式：通过 .env 注入配置，容器独立运行。**

```
┌─────────────────────────────────────────────────────────┐
│ 宿主机                                                   │
│                                                          │
│  docker/.env（git 忽略，存储 API Key）                    │
│       │                                                  │
│       └── docker-compose.yml 读取并注入环境变量           │
│                                                          │
└──────────────────────┬───────────────────────────────────┘
                       │ 环境变量注入
                       ▼
┌─────────────────────────────────────────────────────────┐
│ 容器（完全隔离）                                          │
│                                                          │
│  Claude Code / OpenCode                                  │
│       │                                                  │
│       ├── 环境变量自动注入（ANTHROPIC_*）                  │
│       └── 首次运行选择 Yes 即可                           │
│                                                          │
│  挂载：项目目录（双向同步）                                │
│  隔离：~/.claude、~/.config/opencode（Docker Volume）     │
│                                                          │
└──────────────────────┬───────────────────────────────────┘
                       │
                       ▼
              https://open.bigmodel.cn/api/anthropic
```

## 智谱 AI 配置方法（三种）

> 参考：https://docs.bigmodel.cn/cn/coding-plan/tool/claude

### 方式一：自动化助手

Coding Tool Helper 是一个编码工具助手，快速将 GLM 编码套餐加载到编码工具中：

```bash
npx @z_ai/coding-helper
```

按界面提示操作即可自动完成工具安装、套餐配置、MCP 服务器管理等。

### 方式二：自动化脚本（仅支持 MacOS/Linux）

```bash
curl -O "https://cdn.bigmodel.cn/install/claude_code_env.sh" && bash ./claude_code_env.sh
```

脚本会自动配置 `~/.claude/settings.json` 和 `~/.claude.json`。

### 方式三：手动配置

编辑 `~/.claude/settings.json`：

```json
{
  "env": {
    "ANTHROPIC_AUTH_TOKEN": "your_zhipu_api_key",
    "ANTHROPIC_BASE_URL": "https://open.bigmodel.cn/api/anthropic",
    "API_TIMEOUT_MS": "3000000",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": 1
  }
}
```

同时编辑 `~/.claude.json`：

```json
{
  "hasCompletedOnboarding": true
}
```

**注意**：使用 `ANTHROPIC_AUTH_TOKEN`，不是 `ANTHROPIC_API_KEY`。

---

首次运行 `claude` 时，按提示输入 API Key。

---

## 前置条件

### 1. 获取智谱 AI API Key

从 [智谱开放平台](https://open.bigmodel.cn/) 获取 API Key。

### 2. 安装 Docker

确保已安装 Docker 和 Docker Compose。

---

## 执行步骤

### 1. 创建 docker 目录

```bash
mkdir -p docker
```

### 2. 生成 entrypoint 脚本

创建 `docker/entrypoint.sh`：

```bash
#!/bin/bash
set -e

CLAUDE_CONFIG="/home/app/.claude.json"
CLAUDE_DIR="/home/app/.claude"
BACKUP_PATTERN="/home/app/.claude/backups/.claude.json.backup.*"

# 确保 .claude 目录存在
mkdir -p "$CLAUDE_DIR/backups"

# 检查并恢复配置文件
restore_config() {
    if [ ! -f "$CLAUDE_CONFIG" ]; then
        echo "⚠️  Claude 配置文件不存在，尝试恢复..."

        # 查找最新的备份文件
        LATEST_BACKUP=$(ls -t $BACKUP_PATTERN 2>/dev/null | head -1)

        if [ -n "$LATEST_BACKUP" ] && [ -f "$LATEST_BACKUP" ]; then
            cp "$LATEST_BACKUP" "$CLAUDE_CONFIG"
            echo "✅ 已从备份恢复: $LATEST_BACKUP"
        else
            # 创建默认配置
            echo '{"hasCompletedOnboarding": true}' > "$CLAUDE_CONFIG"
            echo "✅ 已创建默认配置文件"
        fi

        chown app:app "$CLAUDE_CONFIG" 2>/dev/null || true
    fi
}

# 初始化 settings.json（如果不存在）
init_settings() {
    SETTINGS_FILE="$CLAUDE_DIR/settings.json"

    if [ ! -f "$SETTINGS_FILE" ]; then
        echo "⚠️  Claude settings.json 不存在，创建默认配置..."

        cat > "$SETTINGS_FILE" << 'EOF'
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://open.bigmodel.cn/api/anthropic",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "glm-5",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "glm-5",
    "ANTHROPIC_DEFAULT_OPUS_MODEL": "glm-5",
    "API_TIMEOUT_MS": "3000000",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1"
  }
}
EOF
        chown app:app "$SETTINGS_FILE" 2>/dev/null || true
        echo "✅ 已创建默认 settings.json"
    fi
}

# 以 app 用户身份执行初始化
if [ "$(id -u)" = "0" ]; then
    # root 用户：修复权限后初始化
    chown -R app:app /home/app/.claude /home/app/.config/opencode 2>/dev/null || true
    restore_config
    init_settings
    chown -R app:app /home/app/.claude /home/app/.config/opencode 2>/dev/null || true
else
    # 非 root 用户：直接初始化
    restore_config
    init_settings
fi

echo "🚀 开发容器已就绪"
exec "$@"
```

### 3. 生成 Dockerfile

创建 `docker/Dockerfile`：

```dockerfile
FROM debian:bookworm-slim

# 安装系统依赖
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash curl git ca-certificates sudo \
    && rm -rf /var/lib/apt/lists/*

# 安装 Node.js 20.x
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# 安装 Claude Code 和 OpenCode
RUN npm install -g @anthropic-ai/claude-code opencode-ai

# 创建非 root 用户 app（sudo 无密码）
RUN groupadd -g 1000 app && \
    useradd -r -u 1000 -g app -s /bin/bash -d /home/app app && \
    echo 'app ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers

# 设置权限
RUN mkdir -p /home/app/project /home/app/.claude/backups /home/app/.config/opencode && \
    chown -R app:app /home/app

# 复制 entrypoint 脚本
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

WORKDIR /home/app/project
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["tail", "-f", "/dev/null"]
```

### 4. 生成 docker-compose.yml

创建 `docker/docker-compose.yml`：

```yaml
services:
  app:
    env_file:
      - .env
    build:
      context: ..
      dockerfile: docker/Dockerfile
    container_name: ${PROJECT_NAME:-dev-container}
    # 初始化配置（首次启动时以 root 运行 entrypoint）
    user: "0:0"
    environment:
      # 智谱 AI 配置（从 .env 读取）
      - ANTHROPIC_BASE_URL=https://open.bigmodel.cn/api/anthropic
      - ANTHROPIC_AUTH_TOKEN=${ANTHROPIC_API_KEY}
      - ANTHROPIC_DEFAULT_HAIKU_MODEL=glm-5
      - ANTHROPIC_DEFAULT_SONNET_MODEL=glm-5
      - ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5
      - API_TIMEOUT_MS=3000000
      - CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
      # GitHub Token（从 .env 读取）
      - GITHUB_TOKEN=${GITHUB_TOKEN:-}
      # 网络代理（可选，用于访问外网）
      - HTTP_PROXY=${HTTP_PROXY:-}
      - HTTPS_PROXY=${HTTPS_PROXY:-}
      - NO_PROXY=localhost,127.0.0.1
    volumes:
      # 项目目录（双向同步）
      - ..:/home/app/project
      # 配置目录（Docker Volume，与宿主机隔离）
      - claude-config:/home/app/.claude
      - opencode-config:/home/app/.config/opencode

volumes:
  claude-config:
  opencode-config:
```

### 5. 生成 .env 文件

创建 `docker/.env`：

```bash
# ============================================================
# Docker 开发容器配置
# ============================================================
#
# .env 文件不会被 git 追踪，API Key 安全
#
# ============================================================

# 项目名称
PROJECT_NAME=dev-container

# 智谱 AI API Key（从 https://open.bigmodel.cn 获取）
ANTHROPIC_API_KEY=your-api-key-here

# GitHub Token（用于 git 插件，从 https://github.com/settings/tokens 获取）
GITHUB_TOKEN=your-github-token-here

# 网络代理（可选，留空则不使用代理）
HTTP_PROXY=
HTTPS_PROXY=
```

### 6. 生成 Claude Code 配置模板

创建 `docker/settings.json.template`：

```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://open.bigmodel.cn/api/anthropic",
    "ANTHROPIC_API_KEY": "YOUR_API_KEY_HERE",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "glm-5",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "glm-5",
    "ANTHROPIC_DEFAULT_OPUS_MODEL": "glm-5"
  }
}
```

### 7. 生成 README.md

创建 `docker/README.md`：

```markdown
# Docker 开发容器

完全隔离的开发环境，通过 .env 配置使用智谱 AI。

## 架构

```
.env (API Key) → docker-compose.yml → 容器环境变量 → Claude Code
```

配置通过 .env 注入，**与宿主机完全隔离**。

## 自动配置恢复

容器启动时自动检查并恢复 Claude 配置：

1. 检查 `~/.claude.json` 是否存在
2. 如不存在，尝试从 `~/.claude/backups/` 恢复最新备份
3. 如无备份，创建默认配置
4. 同时初始化 `~/.claude/settings.json`

## 快速开始

```bash
# 1. 编辑 .env，填入 API Key
# ANTHROPIC_API_KEY=your-api-key

# 2. 构建并启动
docker compose up -d --build

# 3. 进入容器
docker exec -it -u app dev-container bash

# 4. 首次运行 Claude Code
claude
# 提示 "Do you want to use this API key?" 选择 Yes

# 5. 测试
claude -p "hello"
```

## 隔离说明

| 目录 | 类型 | 说明 |
|------|------|------|
| /home/app/project | Bind Mount | 项目目录，与宿主机双向同步 |
| /home/app/.claude | Docker Volume | Claude 配置，完全隔离 |
| /home/app/.config/opencode | Docker Volume | OpenCode 配置，完全隔离 |

## 故障排除

```bash
# 配置文件丢失时手动恢复
docker exec -it dev-container ls /home/app/.claude/backups/
docker exec -it dev-container cp /home/app/.claude/backups/.claude.json.backup.XXX /home/app/.claude.json

# 重建容器（保留 Volume 数据）
docker compose down && docker compose up -d

# 完全重置（删除所有配置）
docker compose down -v && docker compose up -d --build
```

## 常用命令

```bash
docker compose logs -f          # 查看日志
docker compose down             # 停止
docker compose build --no-cache # 重新构建
docker volume ls                # 查看卷
```
```

### 8. 输出使用说明

```bash
# 1. 编辑 .env 文件，填入凭证
# ANTHROPIC_API_KEY=your-api-key-here
# GITHUB_TOKEN=your-github-token-here  # 可选，用于 git 插件

# 2. 构建并启动容器
cd docker
docker compose up -d --build

# 3. 进入容器
docker exec -it -u app dev-container bash

# 4. 配置 Git 凭据（如需使用 git 插件）
git config --global credential.helper store
echo "https://oauth2:${GITHUB_TOKEN}@github.com" > ~/.git-credentials
git config --global user.email "your-email@example.com"
git config --global user.name "your-name"

# 5. 首次运行 Claude Code
claude
# 提示 "Do you want to use this API key?" 选择 Yes

# 6. 验证
claude -p "hello"
```

## 优势

| 特性 | 说明 |
|------|------|
| 完全隔离 | 容器无法访问宿主机配置 |
| 双向同步 | 项目目录实时同步 |
| 配置独立 | 每个容器独立配置 |
| 简单直接 | 无需额外代理工具 |

## 验收标准

- [ ] Dockerfile 可构建
- [ ] docker-compose.yml 配置正确
- [ ] 项目目录双向同步
- [ ] 配置目录与宿主机隔离
- [ ] Claude Code 可正常使用 GLM-5
- [ ] Git 插件可正常访问 GitHub
