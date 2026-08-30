#!/usr/bin/env bash
# L1 回归测试:wiki MCP 新环境可用性(R1)
# 覆盖:.mcp.json 合法性、无个人绝对路径、启动链路不依赖 CWD、缺二进制报错、
#       serve 真实路径(env 优先 + 无警告)、构建入口与 ignore 规则、文档一致性。
# 用法: bash db/wiki-query/tests/mcp-config.test.sh   (任意 CWD 可运行)
set -u

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
MCP_JSON="$REPO/.mcp.json"
SERVE_SH="$REPO/db/wiki-query/serve.sh"

# 优先 GNU/BSD 常驻路径,缺失时回退 PATH(避免裸 grep 在本机命中 ugrep 的转义差异)
if [ -x /usr/bin/grep ]; then GREP=/usr/bin/grep; else GREP=grep; fi

pass=0
fail=0

ok()   { echo "PASS: $1"; pass=$((pass + 1)); }
bad()  { echo "FAIL: $1"; fail=$((fail + 1)); }

json_parse() {
  node -e "JSON.parse(require('fs').readFileSync(process.argv[1],'utf8'))" "$1" 2>/dev/null
}

# 盘符路径扫描:先中和 URL scheme(https:// 等,避免误报),再匹配 盘符 + 反斜杠/正斜杠
scan_drive_path() {
  sed 's|://|///|g' "$1" | "$GREP" -E '[A-Za-z]:[\\/]'
}

# T1 .mcp.json 是合法 JSON
if json_parse "$MCP_JSON"; then ok "T1 .mcp.json 合法 JSON"; else bad "T1 .mcp.json 合法 JSON"; fi

# T2 .mcp.json 不含盘符绝对路径 (盘符 + 反斜杠/正斜杠形式)
scan_drive_path "$MCP_JSON" > /dev/null
rc2=$?
if [ "$rc2" -eq 1 ]; then ok "T2 .mcp.json 无盘符绝对路径"
else bad "T2 .mcp.json 无盘符绝对路径 (grep rc=$rc2)"; fi

# T3 wiki server 形状: command=bash, args 指向存在的 launcher, 带 cwd, env 不再内嵌 vault 根
if node -e '
  const c = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")).mcpServers.wiki;
  const fs = require("fs");
  const path = require("path");
  const repo = process.argv[2];
  const okShape =
    c.command === "bash" &&
    Array.isArray(c.args) && c.args.length === 1 && c.args[0] === "db/wiki-query/serve.sh" &&
    typeof c.cwd === "string" &&
    !(c.env && c.env.WIKI_VAULT_ROOT) &&
    fs.existsSync(path.join(repo, c.args[0]));
  process.exit(okShape ? 0 : 1);
' "$MCP_JSON" "$REPO" 2>/dev/null; then
  ok "T3 wiki server 配置形状与 launcher 文件存在"
else
  bad "T3 wiki server 配置形状与 launcher 文件存在"
fi

# T4 launcher 从任意 CWD 解析仓库根(--check 通过)
tmp4="$(mktemp -d)"
out4="$(cd "$tmp4" && bash "$SERVE_SH" --check 2>&1)"
rc4=$?
rm -rf "$tmp4"
if [ $rc4 -eq 0 ] && printf '%s' "$out4" | "$GREP" -q "OK" && printf '%s' "$out4" | "$GREP" -qF "$REPO"; then
  ok "T4 launcher 任意 CWD 下 --check 通过"
else
  bad "T4 launcher 任意 CWD 下 --check 通过 (rc=$rc4)"
fi

# T5 launcher 在无二进制的目录下失败并指引 build.sh
tmp5_root="$(mktemp -d)"
tmp5="$tmp5_root/x/y"
mkdir -p "$tmp5"
if cp "$SERVE_SH" "$tmp5/serve.sh"; then
  out5="$(bash "$tmp5/serve.sh" --check 2>&1)"
  rc5=$?
  if [ $rc5 -ne 0 ] && printf '%s' "$out5" | "$GREP" -q "build.sh"; then
    ok "T5 缺二进制时报错并指引 build.sh"
  else
    bad "T5 缺二进制时报错并指引 build.sh (rc=$rc5)"
  fi
else
  bad "T5 缺二进制时报错并指引 build.sh (setup: cp 失败)"
fi
rm -rf "$tmp5_root"

# T6 构建入口存在,且 .gitignore 忽略两种平台产物名
# (用 -f 而非 -x:Windows 上提交的文件模式为 100644,Unix checkout 后无 exec 位,
#  而文档用法是 `bash build.sh`,不需要执行位)
if [ -f "$REPO/db/wiki-query/build.sh" ] \
   && git -C "$REPO" check-ignore -q db/wiki-query/wiki-query.exe \
   && git -C "$REPO" check-ignore -q db/wiki-query/wiki-query; then
  ok "T6 build.sh 存在且 .gitignore 忽略两种产物名"
else
  bad "T6 build.sh 存在且 .gitignore 忽略两种产物名"
fi

# T7a wiki MCP 配置/使用文档集合不含盘符绝对路径(防开发者个人路径回归)
t7a_ok=1
for f in db/README.md AGENTS.md CLAUDE.md docs/mcp-integration.md db/wiki-query/README.md; do
  if [ ! -f "$REPO/$f" ]; then bad "T7a $f 文件不存在"; t7a_ok=0; break; fi
  if scan_drive_path "$REPO/$f" > /dev/null; then t7a_ok=0; fi
done
if [ "$t7a_ok" = "1" ]; then ok "T7a 文档无盘符绝对路径"; else bad "T7a 文档无盘符绝对路径"; fi

# T7b AGENTS/CLAUDE 双写新 clone 构建说明
t7b_ok=1
for f in AGENTS.md CLAUDE.md; do
  if [ ! -f "$REPO/$f" ]; then bad "T7b $f 文件不存在"; t7b_ok=0; break; fi
  "$GREP" -q 'bash db/wiki-query/build.sh' "$REPO/$f" || t7b_ok=0
done
if [ "$t7b_ok" = "1" ]; then ok "T7b AGENTS/CLAUDE 双写构建说明"; else bad "T7b AGENTS/CLAUDE 双写构建说明"; fi

# T8a serve 真实路径:任意 CWD 经 launcher 启动,默认 vault root 正确(不出现 vault 警告)。
# 依赖二进制在读到 stdin EOF 后即退出的契约(与 mcp_server.rs 的 serve 循环一致)。
tmp8="$(mktemp -d)"
out8="$(cd "$tmp8" && bash "$SERVE_SH" </dev/null 2>&1)"
rc8=$?
rm -rf "$tmp8"
if [ $rc8 -ne 0 ] && ! printf '%s' "$out8" | "$GREP" -q "警告"; then
  ok "T8a serve 真实路径启动且默认 vault root 正确"
else
  bad "T8a serve 真实路径启动且默认 vault root 正确 (rc=$rc8)"
fi

# T8b serve 真实路径:外部 WIKI_VAULT_ROOT 优先于仓库根(错误路径必须触发警告)
tmp8b="$(mktemp -d)"
out8b="$(cd "$tmp8b" && WIKI_VAULT_ROOT=/nonexistent-xyz-for-test bash "$SERVE_SH" </dev/null 2>&1)"
rc8b=$?
rm -rf "$tmp8b"
if [ $rc8b -ne 0 ] && printf '%s' "$out8b" | "$GREP" -q "警告" && printf '%s' "$out8b" | "$GREP" -q "/nonexistent-xyz-for-test"; then
  ok "T8b 外部 WIKI_VAULT_ROOT 优先生效"
else
  bad "T8b 外部 WIKI_VAULT_ROOT 优先生效 (rc=$rc8b)"
fi

echo "----"
echo "passed=$pass failed=$fail"
[ "$fail" -eq 0 ]
