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

## 第六步：Windows 非 Docker 环境

当环境检测结果为非 Docker + Windows 时，使用以下方案。

### 前置：迁移已有记忆内容

**在创建任何链接之前**，检查 `$MEMORY_DIR` 是否有项目专属记忆文件：

```bash
ls "$MEMORY_DIR"
```

如果存在非 Obsidian 的内容（如项目特定的 `*.md` 文件）：
1. 将内容合并到 Obsidian wiki 的合适位置
2. 在 Obsidian MEMORY.md 中添加对应索引条目
3. 确认用户同意后删除项目记忆文件

**不要直接覆盖**，否则项目专属记忆会丢失。

### 关键发现

- **Claude Code 只读取 `memory/MEMORY.md` 作为纯文本注入系统提示，不扫描子目录。** 因此所有内容必须平铺在 memory 根目录，不能嵌套（如 `memory/obsidian/` 下的内容不会被发现）。
- **Windows 上目录 Junction 不需要管理员权限，文件 Symlink 需要。** 这是两种不同的机制。
- **跨盘符（如 C: 和 E:）不可用硬链接。**

### 方案 A：整体目录 Junction（推荐）

将整个 memory 目录替换为指向 vault 根的单个 Junction。**最简洁、无需管理员权限、双向同步。**

```bash
# 1. 清空 memory 目录（前置步骤已完成迁移）
MEMORY_DIR="<项目记忆目录>"  # 如 C:/Users/<user>/.claude/projects/E--Files-Projects-rust-my-project/memory
VAULT="<Obsidian 仓库路径>"  # 如 E:/Files/Obsidian

rm -rf "$MEMORY_DIR"

# 2. 创建整体 Junction（从 Git Bash 调用 PowerShell，注意 \$ 转义）
powershell -Command "New-Item -ItemType Junction -Path '$MEMORY_DIR' -Target '$VAULT'"
```

**注意**：从 Git Bash 调用 PowerShell 时，变量用单引号包裹避免 Bash 展开：
- ✅ `powershell -Command "... -Path '\$VAR' ..."`（Bash 转义）
- ✅ `powershell -Command "... -Path 'C:/path' ..."`（硬编码路径）
- ❌ `powershell -Command "... -Path \"$VAR\" ..."`（Bash 会吞掉 `$VAR`）

优点：一个 Junction 覆盖所有文件和目录，vault 中的 MEMORY.md、wiki/、sources/ 等全部可用。
缺点：vault 中的 .git/、.obsidian/ 等也会暴露（不影响 Claude Code 读取）。

### 方案 B：逐条链接（备选）

当不希望暴露 vault 全部内容时使用。目录用 Junction，文件用 Symlink 或复制。

```bash
MEMORY_DIR="<项目记忆目录>"
VAULT="<Obsidian 仓库路径>"

# 目录：Junction（无需管理员权限）
powershell -Command "New-Item -ItemType Junction -Path '$MEMORY_DIR/wiki' -Target '$VAULT/wiki'"
powershell -Command "New-Item -ItemType Junction -Path '$MEMORY_DIR/sources' -Target '$VAULT/sources'"

# 文件：尝试 Symlink，失败则复制
powershell -Command "New-Item -ItemType SymbolicLink -Path '$MEMORY_DIR/MEMORY.md' -Target '$VAULT/MEMORY.md'" \
  || cp "$VAULT/MEMORY.md" "$MEMORY_DIR/MEMORY.md"
```

如果 Symlink 创建失败（无管理员/未开启开发者模式），建议用户开启 Windows 开发者模式（设置 → 系统 → 开发者选项）。

### 验证

使用 Git Bash 命令验证（PowerShell 的 `Target` 属性可能显示错误盘符，不可靠）：

```bash
# 方案 A：验证整体 Junction
readlink "$MEMORY_DIR"
# 应输出 vault 路径，如 /e/Files/Obsidian/

# 通用：验证关键文件可读
head -3 "$MEMORY_DIR/MEMORY.md"
ls "$MEMORY_DIR/wiki/" | head -5
ls "$MEMORY_DIR/sources/" | head -5
```

## 输出

完成后向用户展示：

1. **链接结构图**：显示 Junction/symlink 及其目标
2. **访问验证**：确认关键文件可读
3. **下一步建议**：
   - MEMORY.md 偏好会在下次会话自动加载
   - wiki/sources 内容可通过 Claude Code 的 Read 工具按需读取
   - 在 Obsidian 侧编辑的内容会实时同步到项目（方案 A 全双向，方案 B 仅目录双向）

## 边界情况

- **vault 不含 MEMORY.md**：提示用户是否从模板创建，或仅链接已有内容
- **项目记忆目录已有内容**：先迁移到 Obsidian wiki，再创建链接（见「前置」步骤）
- **非标准项目路径**：通过 `find` 查找实际记忆目录，不硬编码路径
- **Windows WSL 环境**：宿主机路径格式可能需要转换（`C:\Users\...` → `/mnt/c/Users/...`）
- **Junction 跨盘符**：Junction 支持跨盘符（如 C: → E:），不需要同盘
- **PowerShell 编码问题**：Git Bash 调用 PowerShell 可能出现中文乱码，不影响实际执行
