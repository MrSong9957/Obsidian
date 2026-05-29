---
name: Git 安全实践
description: Git 认证安全、token 管理、credential 配置 — 避免明文 token 暴露
type: concept
updated: 2026-04-25
tags: [git, security, credentials, docker]
---

# Git 安全实践

### Remote URL 禁止明文 token

**禁止：** 在 `git remote set-url` 中嵌入 token（如 `https://TOKEN@github.com/...`）
**原因：** 明文 token 暴露在 remote URL 中，任何 `git remote -v` 或配置查看都能看到，有泄露风险
**替代：**
1. remote URL 设为无 token 形式：`git remote set-url origin https://github.com/user/repo.git`
2. 用 credential store 持久化认证：
   ```bash
   echo "https://username:TOKEN@github.com" > ~/.git-credentials
   chmod 600 ~/.git-credentials
   git config --global credential.helper store
   ```
3. Token 从 `.env` 文件读取，不硬编码：
   ```bash
   export GITHUB_TOKEN=$(grep GITHUB_TOKEN path/to/.env | cut -d= -f2)
   ```

← [[sources/security/feedback_git-token-security.md]]
