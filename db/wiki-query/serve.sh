#!/usr/bin/env bash
# wiki-query MCP 启动器(Git Bash / macOS / Linux 通用)
# .mcp.json 用法: {"command": "bash", "args": ["db/wiki-query/serve.sh"], "cwd": "."}
# 从脚本自身位置解析仓库根,不依赖启动时的工作目录;二进制缺失时报错并指引 build.sh。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# 与 build.sh 相同的产物名判断: Windows 是 wiki-query.exe, Unix 是 wiki-query
BIN_NAME=""
if [[ -f "$SCRIPT_DIR/wiki-query.exe" ]]; then
  BIN_NAME="wiki-query.exe"
elif [[ -f "$SCRIPT_DIR/wiki-query" ]]; then
  BIN_NAME="wiki-query"
else
  echo "[wiki-mcp] 错误: 未找到二进制 $SCRIPT_DIR/wiki-query[.exe]" >&2
  echo "[wiki-mcp] 请先在仓库根执行: bash db/wiki-query/build.sh" >&2
  exit 1
fi

if [[ "${1:-}" == "--check" ]]; then
  echo "OK root=$ROOT bin=$SCRIPT_DIR/$BIN_NAME vault_root=${WIKI_VAULT_ROOT:-$ROOT}"
  exit 0
fi

# vault_root 优先外部 WIKI_VAULT_ROOT,否则仓库根(与 mcp_server.rs 的同名环境变量语义一致)
export WIKI_VAULT_ROOT="${WIKI_VAULT_ROOT:-$ROOT}"
exec "$SCRIPT_DIR/$BIN_NAME" serve
