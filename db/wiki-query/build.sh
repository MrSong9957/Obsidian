#!/usr/bin/env bash
# wiki-query 一键构建脚本（Git Bash / macOS / Linux 通用）
# 用法: bash db/wiki-query/build.sh
# 做三件事: cargo build -> 拷贝 exe 到固定位置 -> 打印 MCP 配置提示
set -euo pipefail

# 切到脚本所在目录(即 wiki-query 源码目录)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "[1/3] 编译 wiki-query (release)..."
cargo build --release

# 判断产物文件名: Windows 是 wiki-query.exe, Unix 是 wiki-query
if [[ -f target/release/wiki-query.exe ]]; then
  BIN_NAME="wiki-query.exe"
elif [[ -f target/release/wiki-query ]]; then
  BIN_NAME="wiki-query"
else
  echo "错误: 编译产物未找到 (target/release/wiki-query[.exe])" >&2
  exit 1
fi

echo "[2/3] 拷贝到固定位置 db/wiki-query/$BIN_NAME ..."
cp "target/release/$BIN_NAME" "$BIN_NAME"

echo "[3/3] 完成。下一步:"
echo ""
echo "  # 建索引(在 vault 根目录执行)"
echo "  cd \"$(cd "$SCRIPT_DIR/.." && pwd)\""
echo "  ./db/wiki-query/$BIN_NAME sync"
echo ""
echo "  # MCP 配置(command 路径指向这个固定位置)"
echo "  {\"command\": \"<vault根>/db/wiki-query/$BIN_NAME\", \"args\": [\"serve\"]}"
echo ""
echo "完成。把 command 路径填进你的 .mcp.json 或 ZCode MCP 配置即可。"
