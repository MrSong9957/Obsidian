---
name: Token 管理
description: Token 集中管理架构、各服务 token 获取方式、Docker 环境变量注入规则
type: concept
updated: 2026-05-15
tags: [credentials, tokens, docker, management, security]
sources: [[sources/tools/2026-05-15-token-management.md]]
---

# Token 管理

## 集中管理架构

所有 token 统一存放在 `D:\Files\.env`，一处修改全局生效。

- Docker 项目：`env_file: [D:\Files\.env, .env]`，全局管 token，项目级 `.env` 管项目配置（代理地址等）
- Claude Code 宿主机：`~/.claude/settings.json` 中 `env.ANTHROPIC_AUTH_TOKEN` 独立管理
- Docker Hub：`docker login` 交互式使用，不存 .env

← [[sources/tools/2026-05-15-token-management.md]]

## Token 获取方式速查

| 服务 | 变量名 | 获取地址 |
|------|--------|---------|
| Zhipu AI | `ANTHROPIC_API_KEY` / `ANTHROPIC_AUTH_TOKEN` | open.bigmodel.cn → API Keys |
| DeepSeek | `DEEPSEEK_API_KEY` | platform.deepseek.com → API Keys |
| GitHub | `GITHUB_PERSONAL_ACCESS_TOKEN` / `GITHUB_TOKEN` | github.com/settings/tokens |
| QQ 邮箱 | `QQ_MAIL_APP_PASSWORD` | mail.qq.com → 设置 → 账户 → 生成授权码 |
| Gmail | `GMAIL_APP_PASSWORD` | myaccount.google.com/apppasswords |
| 163 邮箱 | `MAIL163_APP_PASSWORD` | mail.163.com → 设置 → 授权密码 |
| Outlook | `OUTLOOK_APP_PASSWORD` | account.microsoft.com/security → 应用密码 |
| Docker Hub | 交互式 `docker login` | hub.docker.com → Settings → Security |

← [[sources/tools/2026-05-15-token-management.md]]

## 关键规则

### env_file 不做 ${VAR} 替换

docker-compose 的 `env_file:` 只注入容器环境，不参与 `${VAR}` 替换。所以同一个 token 需要写入两个变量名：

```bash
ANTHROPIC_API_KEY=xxx           # 原始名
ANTHROPIC_AUTH_TOKEN=xxx        # 容器内映射名（Claude Code 读取的变量名）
```

### 代理地址差异

- 宿主机：`127.0.0.1:7890`
- Docker 容器内：`host.docker.internal:7890`

代理配置保留在各项目的 `.env`，不放全局。

### Token 更新流程

1. 修改 `D:\Files\.env`
2. 重启容器 `docker compose up -d`
3. Claude Code 宿主机的 token 需单独更新 `~/.claude/settings.json`
4. Git remote URL 中的 token 不会自动同步，需手动 `git remote set-url`（推荐改用 credential store，详见 [[wiki/git-security.md|Git 安全实践]]）

← [[sources/tools/2026-05-15-token-management.md]]
