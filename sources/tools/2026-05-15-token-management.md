---
name: Token 获取与管理
description: 各平台 API token 获取方式与集中管理方案 — 智谱/DeepSeek/GitHub/QQ/Gmail/163/Outlook/Docker Hub 的 key 申请流程与 D:\Files\.env 统一管理架构
type: reference
updated: 2026-05-15
tags: token, credentials, env, security, api-key
source: self
---
# Token 获取与管理

## 2026-05-15 建立集中管理

所有 token 统一存放在 `D:\Files\.env`，各项目 docker-compose.yml 通过 `env_file: D:\Files\.env` 引用。不再使用用户环境变量或散落的 .env 文件管理 token。

## 各 Token 获取方式

### Zhipu AI (GLM) — `ANTHROPIC_API_KEY` / `ANTHROPIC_AUTH_TOKEN`

1. 访问 https://open.bigmodel.cn
2. 登录 → 控制台 → API Keys
3. 创建新 key 或复制现有 key
4. 格式示例：`<32位hex>.<16位字母数字>`

Claude Code 宿主机配置：`~/.claude/settings.json` 中 `env.ANTHROPIC_AUTH_TOKEN` 直接硬编码（不读 .env）。容器内通过 `D:\Files\.env` 注入。

### DeepSeek — `DEEPSEEK_API_KEY`

1. 访问 https://platform.deepseek.com
2. 登录 → API Keys
3. 创建新 key
4. 格式示例：`sk-<32位hex>`

### GitHub — `GITHUB_PERSONAL_ACCESS_TOKEN` / `GITHUB_TOKEN`

1. 访问 https://github.com/settings/tokens
2. Generate new token (classic) 或 Fine-grained token
3. 勾选所需权限（一般需要 repo, read:org）
4. 格式示例：`[REDACTED]`

⚠️ token 只在创建时显示一次，务必保存。

### QQ 邮箱 — `QQ_MAIL_APP_PASSWORD`

1. 访问 https://mail.qq.com → 设置 → 账户
2. 找到 POP3/SMTP/IMAP 栏
3. 开启 IMAP/SMTP 服务（需手机验证）
4. 点击"生成授权码"
5. 格式示例：`<16位小写字母>`（16 位小写字母）

⚠️ 授权码生成后无法再次查看，只能重新生成。

### Gmail — `GMAIL_APP_PASSWORD`

1. 开启两步验证：https://myaccount.google.com/security
2. 生成应用专用密码：https://myaccount.google.com/apppasswords
3. 选择"邮件"→ 生成
4. 格式示例：`<16位小写字母>`（16 位小写字母）

⚠️ Google 于 2024 年逐步移除应用专用密码，改用 OAuth 2.0。如果无法生成，可能需要用 OAuth 方式。

### 163 邮箱 — `MAIL163_APP_PASSWORD`

1. 访问 https://mail.163.com → 设置 → POP3/SMTP/IMAP
2. 开启 IMAP/SMTP
3. 设置客户端授权密码（需手机验证）
4. 格式示例：`<16位混合大小写>`

### Outlook — `OUTLOOK_APP_PASSWORD`

1. 访问 https://account.microsoft.com/security
2. 高级安全选项 → 应用密码
3. 创建新的应用密码
4. 格式示例：`<16位小写字母>`

### Docker Hub — Access Token（非环境变量）

1. 访问 https://hub.docker.com → Settings → Security
2. New Access Token
3. 终端登录：`docker login -u s7620605`，输入 token 作为密码
4. 格式示例：`dckr_pat_<token>`

Docker Hub token 不适合放 .env（通过 `docker login` 交互式使用）。

## 管理架构

```
D:\Files\.env（唯一 token 来源）
    │
    ├── docker-compose.yml
    │   env_file: D:\Files\.env     ← 容器注入 token
    │   env_file: .env              ← 项目专用配置（代理、路径）
    │
    └── Claude Code settings.json   ← 独立管理（ANTHROPIC_AUTH_TOKEN 硬编码）
```

### .env 文件格式

```bash
# === AI API Keys ===
ANTHROPIC_API_KEY=xxx
ANTHROPIC_AUTH_TOKEN=xxx    # 同上，容器内变量名映射
DEEPSEEK_API_KEY=xxx

# === 邮箱凭证 ===
QQ_MAIL_USER=xxx@qq.com
QQ_MAIL_APP_PASSWORD=xxx
# ... 其他邮箱

# === GitHub ===
GITHUB_PERSONAL_ACCESS_TOKEN=xxx
GITHUB_TOKEN=xxx    # 同上，容器内变量名映射
```

### 注意事项

- 同一个 token 可能需要两个变量名（如 ANTHROPIC_API_KEY 和 ANTHROPIC_AUTH_TOKEN），因为 docker-compose 的 `env_file` 不做 `${VAR}` 替换，需要直接注入最终变量名
- Docker 容器内的代理地址用 `host.docker.internal`，与宿主机的 `127.0.0.1` 不同，所以代理配置保留在项目级 `.env`
- 修改 `D:\Files\.env` 后需要重启容器生效

## 2026-05-15 Git Remote Token 过期事件

**现象：** `git push` 报 `fatal: could not read Password for 'https://ghp_***...@github.com': No such device or address`

**排查：**
1. 先怀疑容器 `.env` 未加载，检查发现容器内 `ANTHROPIC_AUTH_TOKEN` 与 `D:\Files\.env` 一致 → `env_file` 机制正常
2. 报错信息中的 token `ghp_***...` 与 `.env` 中的 `ghp_***...` 不同 → git remote URL 里嵌了旧 token
3. `git remote -v` 确认：`origin https://ghp_***...@github.com/MrSong9957/email-assistant.git`

**修复：**
```bash
git remote set-url origin "https://ghp_***...@github.com/MrSong9957/email-assistant.git"
git push origin master  # 成功
```

**根因：** Git remote URL 中嵌入的 token 与 `.env` 文件中的 token 是**独立管理**的。更新 `.env` 不会自动更新 remote URL。这与 `wiki/git-security.md` 中"禁止明文 token 嵌入 remote URL"的规则一致——正确做法是用 credential store。
