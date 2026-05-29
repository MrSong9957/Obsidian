---
name: Token 管理
description: Token 集中管理架构、各服务 token 获取方式、Docker 环境变量注入规则
type: concept
updated: 2026-05-29
tags: [credentials, tokens, docker, management, security, mimo, provider]
sources: [[sources/tools/2026-05-15-token-management.md]], [[sources/tools/mimo-provider-debug.md]]
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

## MiMo Provider 配置

### 认证字段差异

不同 provider 使用不同的认证环境变量：

| Provider | 认证变量 | 请求头 |
|----------|----------|--------|
| Zhipu（智谱） | `ANTHROPIC_API_KEY` | `x-api-key` |
| MiMo（小米） | `ANTHROPIC_AUTH_TOKEN` | `Authorization: Bearer` |

切换时必须清除另一个变量，否则会冲突。

← [[sources/tools/mimo-provider-debug.md]] L15-25

### Token Plan 域名

MiMo Token Plan 的正确域名是 `xiaomimimo.com`（不是 `xiaomimimimo.com`）：

- ✅ `https://token-plan-cn.xiaomimimo.com/anthropic`
- ❌ `https://token-plan-cn.xiaomimimimo.com/anthropic`（多一个 mi，DNS 不解析）

**禁止：** 从搜索结果或二手文档复制 URL，必须从官方文档确认域名
**原因：** 多个技术博客都写错了域名（多一个 mi），导致 DNS 解析失败
**替代：** 以 `platform.xiaomimimo.com` 官网域名为准，Token Plan 子域名为 `token-plan-cn.xiaomimimo.com`

← [[sources/tools/mimo-provider-debug.md]] L33-46

### 模型名后缀

`[1m]`（小写 m）是启用 1M 上下文窗口的后缀，不是独立模型变体：

- `mimo-v2.5-pro[1m]` = `mimo-v2.5-pro` + 1M 上下文
- `[1M]`（大写）可能不被识别

← [[sources/tools/mimo-provider-debug.md]] L27-31

### 速查表更新

| 服务 | 变量名 | 获取地址 |
|------|--------|---------|
| MiMo Token Plan | `MIMO_API_KEY` → 切换时映射到 `ANTHROPIC_AUTH_TOKEN` | platform.xiaomimimo.com → Token Plan |

← [[sources/tools/mimo-provider-debug.md]]

### Provider 切换脚本配置

脚本位置：`/home/app/project/scripts/switch-provider.sh`

**切换命令：**
```bash
source /home/app/project/scripts/switch-provider.sh mimo   # 切到 MiMo
source /home/app/project/scripts/switch-provider.sh zhipu   # 切到智谱
```

**模型映射（已验证可用）：**

| Claude 层级 | Zhipu 模型 | MiMo 模型 |
|-------------|-----------|-----------|
| Opus | `glm-5.1` | `mimo-v2.5-pro[1m]` |
| Sonnet | `glm-5-turbo` | `mimo-v2.5-pro[1m]` |
| Haiku | `glm-5-turbo` | `mimo-v2.5[1m]` |
| Reasoning | `glm-5.1` | `mimo-v2.5-pro[1m]` |

**切换后必须重启 Claude Code 才生效。**

← [[sources/tools/mimo-provider-debug.md]]
