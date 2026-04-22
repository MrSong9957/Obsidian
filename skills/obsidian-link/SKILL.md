---
name: obsidian-link
description: |
  将 Obsidian 仓库连接到当前项目的 Claude Code 记忆系统。
  当用户在新项目中想要连接 Obsidian 仓库、挂载知识库、设置记忆链接时使用此技能。
  适用于：新项目初始化、Docker 容器环境、需要跨项目共享 Obsidian 知识库的场景。
  触发短语：连接 Obsidian、挂载知识库、链接仓库、关联记忆、obsidian-link、设置 Obsidian 链接。
depends: []
---

# Obsidian Link 技能

将 Obsidian 仓库通过 Docker 挂载 + symlink 桥接到项目的 Claude Code 记忆系统，实现：
- **Docker 挂载**：自动修改 `docker-compose.yml` 将 Obsidian 仓库挂入容器
- **Symlink 桥接**：在项目记忆目录创建链接，指向 Obsidian 仓库内容
- **自动发现**：扫描仓库结构，找到含 MEMORY.md 的 vault

## 整体流程

```
用户提供宿主机 Obsidian 路径
        │
        ▼
  ┌─ 检测运行环境 ─┐
  │                 │
  Docker 容器    非 Docker
  │                 │
  ▼                 ▼
检查是否已挂载    直接用绝对路径
  │                 │
  未挂载 → 修改 docker-compose.yml
  │              提示用户重启容器
  ▼                 │
  ┌─── 发现 vault ───┐
  │  扫描 .obsidian/  │
  │  定位 MEMORY.md   │
  └────────┬─────────┘
           │
           ▼
  ┌─── 创建 symlink ──┐
  │  MEMORY.md        │
  │  wiki/            │
  │  sources/         │
  │  skills/          │
  │  templates/       │
  │  README.md        │
  └────────┬──────────┘
           │
           ▼
      验证链接可用
```

## 第一步：获取宿主机 Obsidian 路径

向用户询问 Obsidian 仓库在宿主机上的绝对路径。

如果用户没有明确给出，可以检查以下常见位置：
- `~/Obsidian`
- `~/Documents/Obsidian`
- `~/obsidian-vault`

确认路径存在且包含 `.obsidian/` 目录或 `MEMORY.md` 文件。

## 第二步：环境检测与挂载配置

### 检测是否在 Docker 容器中

```bash
# 检测方法 1：/.dockerenv 文件
[ -f /.dockerenv ] && echo "IN_DOCKER=true" || echo "IN_DOCKER=false"

# 检测方法 2：cgroup 信息
cat /proc/1/cgroup 2>/dev/null | grep -q docker && echo "IN_DOCKER=true" || echo "IN_DOCKER=false"
```

### Docker 容器环境

1. **检查是否已挂载**：查看 `docker-compose.yml` 中是否已有 Obsidian 挂载
2. **未挂载时修改配置**：

在 `docker-compose.yml` 的 `volumes` 部分添加 Obsidian 挂载：

```yaml
volumes:
  # ... 已有挂载 ...
  # Obsidian vault (knowledge base)
  - <宿主机路径>:/home/app/obsidian
```

挂载点固定为 `/home/app/obsidian`（容器内路径）。

**重要**：修改 docker-compose.yml 后，需要用户重启容器才能生效。告诉用户：

> 已更新 docker-compose.yml，需要重启容器使挂载生效：
> ```bash
> docker compose down && docker compose up -d
> ```
> 重启后再次运行此技能继续配置。

**如果用户无法立即重启**：先完成后续步骤的准备工作，等重启后再执行 symlink 创建。

3. **已挂载时继续**：跳过此步骤

### 非 Docker 环境

直接使用用户提供的绝对路径作为 Obsidian 仓库路径，无需挂载。

## 第三步：发现 Vault

容器内 Obsidian 仓库挂载在 `/home/app/obsidian`（Docker）或用户提供的路径（非 Docker）。

扫描仓库结构，找到包含 `MEMORY.md` 的 vault 目录：

```bash
# 优先查找含 MEMORY.md 的目录
find /home/app/obsidian -maxdepth 2 -name "MEMORY.md" -type f 2>/dev/null

# 同时检查 .obsidian/ 目录（标准 Obsidian vault 标志）
find /home/app/obsidian -maxdepth 2 -name ".obsidian" -type d 2>/dev/null
```

如果找到多个 vault，让用户选择。如果未找到任何 vault，在仓库根目录创建基本结构。

确定 vault 路径后，记录它用于后续 symlink 创建。下文称此路径为 `$VAULT_PATH`。

## 第四步：创建 Symlink

项目记忆目录固定为：
```
<项目根>/.claude/projects/-home-app-project/memory/
```

实际路径可通过以下方式获取：
```bash
# 获取当前项目记忆目录
MEMORY_DIR=$(dirname "$(find /home/app/.claude/projects -name "memory" -type d 2>/dev/null | head -1)")/memory
```

需要创建的 symlink：

| Symlink | 目标 | 说明 |
|---------|------|------|
| `MEMORY.md` | `$VAULT_PATH/MEMORY.md` | 核心记忆索引 |
| `README.md` | `$VAULT_PATH/README.md` | 知识库导航 |
| `wiki/` | `$VAULT_PATH/wiki/` | Wiki 知识文档 |
| `sources/` | `$VAULT_PATH/sources/` | 原始资料 |
| `skills/` | `$VAULT_PATH/skills/` | 技能目录 |
| `templates/` | `$VAULT_PATH/templates/` | 模板 |

**只链接 vault 中实际存在的目录和文件**，不创建不存在的链接。

创建命令（使用绝对路径）：
```bash
cd "$MEMORY_DIR"
ln -sf "$VAULT_PATH/MEMORY.md" MEMORY.md
ln -sf "$VAULT_PATH/README.md" README.md
ln -sf "$VAULT_PATH/wiki" wiki
# ... 只创建实际存在的 ...
```

如果 `$MEMORY_DIR` 中已有同名文件（非 symlink），先备份再创建：
```bash
[ -f MEMORY.md ] && [ ! -L MEMORY.md ] && mv MEMORY.md MEMORY.md.bak
ln -sf "$VAULT_PATH/MEMORY.md" MEMORY.md
```

## 第五步：验证

逐项验证每个 symlink 是否可用：

```bash
cd "$MEMORY_DIR"
echo "=== 验证 symlink ==="
for link in $(find . -maxdepth 1 -type l); do
    target=$(readlink -f "$link")
    if [ -e "$target" ]; then
        echo "  ✓ $link → $target"
    else
        echo "  ✗ $link → $target (BROKEN)"
    fi
done

echo ""
echo "=== 验证内容可读 ==="
cat MEMORY.md | head -3
ls wiki/ | head -5
```

如果有断裂的链接，报告给用户并尝试修复。

## 输出

完成后向用户展示：

1. **链接结构图**：显示所有创建的 symlink 及其目标
2. **访问验证**：确认每个链接都可读取
3. **下一步建议**：
   - MEMORY.md 偏好会在下次会话自动加载
   - wiki/sources 内容可通过 Claude Code 的 Read 工具按需读取
   - 在 Obsidian 侧编辑的内容会实时同步到项目

## 边界情况

- **vault 不含 MEMORY.md**：提示用户是否从模板创建，或仅链接已有内容
- **项目记忆目录已有内容**：备份旧文件到 `.bak`，然后创建 symlink
- **非标准项目路径**：通过 `find` 查找实际记忆目录，不硬编码路径
- **Windows WSL 环境**：宿主机路径格式可能需要转换（`C:\Users\...` → `/mnt/c/Users/...`）
- **符号链接权限**：如果创建失败，检查文件系统是否支持 symlink（某些 Windows 环境需要管理员权限）
