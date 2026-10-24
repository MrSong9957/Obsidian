# shortlink 启动说明（START）

产物：`shortlink`（Rust 1.99 release 构建，单进程，SQLite 本地文件存储）。

## 构建命令（仅供参考，产物已预构建）

```bash
cd /tmp/opencode/spec-method-2026-10-24/implementer/build
CARGO_TARGET_DIR=/tmp/opencode/spec-method-2026-10-24/implementer/build/target \
  cargo build --release --offline
# 产物在 build/target/release/shortlink，已复制到本目录（artifact/shortlink）
```

依赖仅离线缓存内的 crate：rusqlite 0.40.2（bundled SQLite 3.5x）、httparse 1.10.1、
serde_json 1、libc 0.2。无 tokio，无外部网络。

## 启动命令（验收用这个）

在 artifact 目录前台启动（`exec` 形式）：

```bash
cd /tmp/opencode/spec-method-2026-10-24/implementer/artifact
PORT=0 DB_PATH=/tmp/your/path/links.db exec ./shortlink
```

- 前台进程，不 daemonize；stdout 输出一行
  `listening on http://127.0.0.1:<实际端口>` 后立即刷新，验收脚本从该行读取实际端口。
- `SIGTERM` 后干净退出（exit code 0，实测 <0.1s），已提交数据保留。

## 环境变量

| 变量 | 说明 |
|------|------|
| `PORT` | 监听端口，仅绑定 `127.0.0.1`。`PORT=0` 表示由系统分配端口，实际端口以启动行为准；`short_url` 也使用该实际端口生成。 |
| `DB_PATH` | SQLite 数据库文件路径。文件不存在时自动创建；父目录不存在时自动创建（含多级）。服务重启用同一 `DB_PATH` 即可恢复链接、点击数与过期时间。 |

## 接口摘要（详见冻结规格）

- `POST /api/links`（`Content-Type: application/json`）→ 201 / 400 / 409
- `GET /api/links/<code>` → 200 / 404
- `GET /<code>` → 302（自增点击数）/ 410 / 404

均支持 HTTP/1.1 keep-alive，响应均带 `Content-Length`，JSON 响应为
`Content-Type: application/json; charset=utf-8`。
