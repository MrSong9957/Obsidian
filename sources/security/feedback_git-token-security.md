---
name: Git Token 安全认证方式
description: 避免 git remote URL 使用明文 token——凭据存入系统安全凭据管理器（GCM），输出前脱敏，禁用 credential store
type: feedback
updated: 2026-10-19
tags: [git, security, token, credentials]
source: self
---
Git remote URL 中禁止明文 token；凭据只存系统安全凭据管理器（本机为 Git Credential Manager，`credential.helper=manager`）。**禁用 `credential.helper store`**——`~/.git-credentials` 是明文文件，任何明文转存（其他文件、环境变量落盘、命令参数）一并禁止。

**存储实现（2026-10-05 实测，容器环境）：** 容器内无 Windows 凭据管理器与 Secret Service，采用 GCM + GPG 加密存储：`git config --global credential.credentialStore gpg`，条目落盘 `~/.password-store/<账号>.gpg`（GPG 加密，公钥 `92ED0473E56AD56689CFA89A39159B4943FB852E`，已验证无明文残留）。GPG 私钥无口令（自动化解密所需），文件权限 0600——保护强度等同私钥文件本身，宿主机 Windows 侧则走 DPAPI。helper 为 wrapper `/usr/local/bin/git-credential-manager`（设置 `GPG_TTY` 后调 `gcm-real`），供无 tty 的 git 调用环境使用。

**Why:** 明文 token 暴露在 remote URL 或凭据文件中，任何 `git config` 查看或文件读取都会泄露；2026-10-05 实际发生过一次：裸跑 `git remote -v` 使 token 进入会话输出（后已迁入 GCM 并清除 URL，token 本体保留未轮换）。

**How to apply:**
1. remote URL 保持无 token 形式：`git remote set-url origin https://github.com/user/repo.git`
2. 凭据经 stdin 存入安全凭据管理器后再清 URL（值不进命令参数、不进输出）：
   `printf 'protocol=https\nhost=github.com\nusername=USER\npassword=%s\n' "$TOKEN" | git credential approve`
3. 存取验证只比对不显示：取 `git credential fill` 的 `password=` 行与原值比对长度/一致性，原始值不打印；另用 `cmdkey /list` 确认凭据管理器有条目。
4. 任何可能输出凭据的命令（`git remote -v`、config 查看、认证报错），**正常与错误输出都必须先过脱敏管道**再进入工具输出或日志：
   `sed -E 's/(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]+/\1_[REDACTED]/g; s#(https?://)[^/@[:space:]]+@#\1[REDACTED]@#g'`
5. 检查入口：运行 `/vault/scripts/git-token-check.sh [仓库路径]`（封装 remote/config 脱敏显示与凭据嵌入形态检测，正常与错误输出均过脱敏管道）；等价手工检查（两条都应无输出）：`git config --get-regexp '^remote\.' | grep -E '://[^/@[:space:]]+:[^/@[:space:]]+@|(ghp|gho)_[A-Za-z0-9]{20,}'`；`rg -l 'ghp_[A-Za-z0-9]{36,}' -g '!.git'`。已知例外（2026-10-19 审计更新）：〔2026-05-15-token-management〕原"第 38 行 40 位真实格式 token 已随历史推送"在当前可获取对象（全部提交与悬空对象）中未检出对应实体（最长 `ghp_` 串仅 9 字符）；"未检出"不等于"从未进入"（Actions 日志匿名 403 亦未验证）；该页疑似真实格式凭据（智谱/DeepSeek/Docker token 与邮箱授权码）已于 2026-10-19 从正文移除并重写历史，待轮换。
