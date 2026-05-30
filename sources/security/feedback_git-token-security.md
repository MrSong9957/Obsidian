---
name: Git Token 安全认证方式
description: 避免在 git remote URL 中使用明文 token，改用 credential store
type: feedback
updated: 2026-05-30
tags: [tag1, tag2]
source: https://example.com
---
Git remote URL 中禁止明文 token，改用 credential store 方案。

**Why:** 明文 token 暴露在 remote URL 中，任何 git config 查看都能看到，且有泄露风险。

**How to apply:**
1. remote URL 设为无 token 的形式：`git remote set-url origin https://github.com/user/repo.git`
2. 用 credential store 持久化认证：`echo "https://username:TOKEN@github.com" > ~/.git-credentials && chmod 600 ~/.git-credentials && git config --global credential.helper store`
3. Token 源从 `.env` 文件读取，不硬编码：`$(grep GITHUB_TOKEN path/to/.env | cut -d= -f2)`
