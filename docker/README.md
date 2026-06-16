# MiMo-Code Docker 开发容器

完全隔离的开发环境，支持智谱 AI 和小米 MiMo 双模型切换。

## 架构

```
.env (API Key + 模型配置) → docker-compose.yml → 容器环境变量 → Claude Code / OpenCode
```

配置通过 `.env` 注入，**与宿主机完全隔离**。

## 快速开始

```bash
# 1. 复制并编辑 .env 文件
cp .env.example .env
# 编辑 .env，填入 API Key 和 GitHub Token

# 2. 构建并启动容器
docker compose up -d --build

# 3. 进入容器
docker exec -it -u app mimocode-dev bash

# 4. 首次运行 Claude Code
claude
# 提示 "Do you want to use this API key?" 选择 Yes

# 5. 测试
claude -p "hello"
```

## 模型切换

所有模型配置在 `.env` 中，切换时只需注释/取消注释对应行：

### 智谱 AI（默认）

```env
ANTHROPIC_BASE_URL=https://open.bigmodel.cn/api/anthropic
ANTHROPIC_API_KEY=your-zhipu-api-key
ANTHROPIC_DEFAULT_HAIKU_MODEL=glm-5-turbo
ANTHROPIC_DEFAULT_SONNET_MODEL=glm-5-turbo
ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5.2
ANTHROPIC_MODEL=glm-5.2
ANTHROPIC_REASONING_MODEL=glm-5.2
```

### 小米 MiMo

```env
ANTHROPIC_BASE_URL=https://token-plan-cn.xiaomimimo.com/anthropic
ANTHROPIC_API_KEY=your-xiaomi-api-key
ANTHROPIC_DEFAULT_HAIKU_MODEL=mimo-v2.5
ANTHROPIC_DEFAULT_SONNET_MODEL=mimo-v2.5
ANTHROPIC_DEFAULT_OPUS_MODEL=mimo-v2.5-pro
ANTHROPIC_MODEL=mimo-v2.5-pro
ANTHROPIC_REASONING_MODEL=mimo-v2.5-pro
```

切换后执行 `docker compose up -d` 重新加载。

## Git 凭据配置

容器内使用 Git 推送到 GitHub 时，需配置凭据：

```bash
# 进入容器
docker exec -it -u app mimocode-dev bash

# 配置 Git 凭据存储
git config --global credential.helper store
echo "https://oauth2:${GITHUB_TOKEN}@github.com" > ~/.git-credentials

# 配置 Git 用户信息
git config --global user.email "your-email@example.com"
git config --global user.name "your-name"
```

## 隔离说明

| 目录 | 类型 | 说明 |
|------|------|------|
| /home/app/project | Bind Mount | 项目目录，与宿主机双向同步 |
| /home/app/obsidian | Bind Mount (只读) | Obsidian 仓库 |
| /root | Docker Volume (ai-home) | 所有 AI CLI（Claude/OpenCode/codex/...）配置 + bun，完全隔离 |

## 自动配置恢复

容器启动时自动检查并恢复 Claude 配置：

1. 检查 `~/.claude.json` 是否存在
2. 如不存在，尝试从 `~/.claude/backups/` 恢复最新备份
3. 如无备份，创建默认配置
4. 同时初始化 `~/.claude/settings.json`

## 常用命令

```bash
# 查看日志
docker compose logs -f

# 停止容器
docker compose down

# 重建容器（保留 Volume 数据）
docker compose down && docker compose up -d

# 完全重置（删除所有配置）
docker compose down -v && docker compose up -d --build

# 查看卷
docker volume ls
```

## 故障排除

### 构建失败

#### 代理泄漏（exit code 100）

`apt-get update` 报 `Could not connect to 127.0.0.1:7890`：

- **根因**：宿主机代理通过 `${HTTP_PROXY:-}` 注入构建容器
- **解法**：docker-compose.yml 中构建参数硬编码空值：
  ```yaml
  args:
    HTTP_PROXY: ""
    HTTPS_PROXY: ""
  ```
- **教训**：`${VAR:-}` 不是安全的"默认为空"——宿主机环境变量优先级更高

#### unzip 缺失

Bun 安装报 `error: unzip is required to install bun`：

- **根因**：`debian:bookworm-slim` 不预装 unzip
- **解法**：apt-get 依赖列表加 `unzip`

#### CRLF 行尾

entrypoint.sh 执行报语法错误：

- **根因**：Windows 编辑器保存的 `.sh` 文件带 CRLF（`\r\n`）
- **解法**：Dockerfile 中 `sed -i 's/\r$//'`

### 运行时失败

#### bash / mkdir: command not found

`docker compose exec app bash` 报找不到命令：

- **根因 1**：docker-compose.yml 中 `- PATH=/home/app/.bun/bin:$PATH`，`$PATH` 在宿主机展开，Windows PATH 注入容器覆盖 Linux PATH
- **解法 1**：删除 docker-compose.yml 中的 `- PATH=...` 行，依赖 Dockerfile 的 `ENV PATH`

- **根因 2**：entrypoint.sh 以 `user: "0:0"` 运行时 PATH 不完整
- **解法 2**：Dockerfile 加 `ENV PATH`，entrypoint.sh 加 `export PATH`

### 配置文件丢失

`/root` 通过 `ai-home` volume 持久化，`docker compose down` 不会丢配置。只有 `docker compose down -v`（删除 volume）才会清空，届时重新跑 `claude` 走一遍 onboarding 即可（API key 从 `.env` 环境变量自动注入，无需重填）。

### 排查顺序

遇到构建/运行失败时，按此顺序排查：

1. **代理** — `docker inspect` 检查 HTTP_PROXY 是否指向不可达地址
2. **PATH** — `docker inspect mimocode-dev --format '{{range .Config.Env}}{{println .}}{{end}}' | findstr PATH` 检查是否被宿主机污染
3. **依赖** — 缺包通常有明确错误信息
4. **行尾** — Windows 环境下 `.sh` 文件优先检查 CRLF

## 验收标准

- [ ] Dockerfile 可构建
- [ ] docker-compose.yml 配置正确
- [ ] 项目目录双向同步
- [ ] Obsidian 仓库可读取
- [ ] Claude Code 可正常使用
- [ ] OpenCode 可正常使用
- [ ] GitHub CLI 可正常使用
- [ ] Bun 可正常使用
