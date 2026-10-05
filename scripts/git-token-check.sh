#!/bin/bash
# git-token-check.sh [仓库路径]
# Git 凭据残留检查与远程信息脱敏输出（规则依据：AGENTS.md「Git 凭据与输出脱敏」、
# sources/security/feedback_git-token-security.md）。
# 约定：全部输出（正常与错误）先经 mask 管道再出终端；被封装命令失败会传播为非零退出码；
# 脱敏不可用时抑制全部原文输出。退出码：0=干净且命令成功；1=发现凭据嵌入形态或命令失败；3=脱敏不可用。
set -o pipefail
command -v sed >/dev/null 2>&1 || { echo "[git-token-check] sed 不可用，为避免泄露原文，输出已抑制" >&2; exit 3; }
repo="${1:-.}"
mask() { sed -E 's/(ghp|gho|ghu|ghs|ghr|github_pat)_[A-Za-z0-9_-]+/\1_[REDACTED]/g; s#(https?://[^/@[:space:]]+:)[^/@[:space:]]+@#\1[REDACTED]@#g'; }
say() { printf '%s\n' "$*" | mask; }
fail=0
say "== 仓库: $repo =="
say "-- remote -v --"
out=$(git -C "$repo" remote -v 2>&1) || fail=1
printf '%s\n' "$out" | mask
say "-- credential/remote/insteadof/proxy 配置 --"
out=$(git -C "$repo" config --list --show-origin 2>&1) || fail=1
printf '%s\n' "$out" | grep -iE 'credential|remote\.|url\.|insteadof|proxy' | mask
say "-- 残留检测（凭据嵌入形态：user:pass@ 或裸 token）--"
hits=$(git -C "$repo" config --get-regexp '^remote\.' 2>/dev/null | grep -E '://[^/@[:space:]]+:[^/@[:space:]]+@|(ghp|gho)_[A-Za-z0-9]{20,}')
if [ -n "$hits" ]; then fail=1; printf '%s\n' "$hits" | mask; else say "配置检查: 干净"; fi
hits2=$(grep -rlE 'ghp_[A-Za-z0-9]{36,}' "$repo" -g '!.git' 2>/dev/null)
if [ -n "$hits2" ]; then fail=1; say "工作区含 36+ 位 token 形态（对照已知例外清单）:"; printf '%s\n' "$hits2" | mask; fi
say "-- 连通性诊断（ls-remote，失败信息中的 URL 亦被脱敏）--"
out=$(git -C "$repo" ls-remote --heads origin 2>&1) || fail=1
printf '%s\n' "$out" | mask
say "-- 结束 --"
exit $fail
