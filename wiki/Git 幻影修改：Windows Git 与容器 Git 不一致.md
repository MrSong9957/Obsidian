---
name: Git 幻影修改：Windows Git 与容器 Git 不一致
description: 同一仓库在 Windows Git 与容器/WSL Git 下结论不一致的幻影 modified——行尾（autocrlf）与 filemode 两类幻影的形成机制、判断方法与根治原则
type: guide
updated: 2026-10-07
tags: [git, windows, wsl, autocrlf, filemode, vscode, line-endings]
source: self
---

# Git 幻影修改：Windows Git 与容器 Git 不一致

**问题模式**：一个仓库被两侧 git 共用（Windows 目录 bind/挂载进 WSL 或容器，典型如 9p/drvfs），VS Code 用 Windows git、终端/容器里用 Linux git，两侧对"哪些文件被修改"结论不一致——一侧干净，另一侧显示一批 M，而文件内容（除行尾外）没有任何变化。

---

## 问题模式（为什么会发生）

幻影来自三因素叠加，缺一通常不出现：

| # | 因素 | 后果 |
|---|------|------|
| 1 | **同一 `.git` 目录被两个系统共用** | index 缓存的 stat 元数据（inode、设备号、时间戳）由一侧写入，另一侧读取时必然失配，被迫对所有文件重新计算 diff——幻影的前提 |
| 2 | **行尾规则不一致**（内容类幻影） | Windows git 安装默认 `core.autocrlf=true`，Linux 侧等效 `false`；仓库若没有 `.gitattributes`，CRLF 入库的文件在其中一侧永远显示 modified |
| 3 | **filemode 在挂载文件系统上不可靠**（元数据类幻影） | `core.filemode=true` 时 9p/drvfs 呈现的权限位不稳定，产生与内容无关的 mode 幻影，名单里常混入从未改动过的文件 |

---

## 判断方法（先确认是幻影，再动手）

- **命令行 git 是权威**：两个环境各跑 `git status --porcelain`，数量对不上就是幻影信号；不要只信 GUI 面板。
- **看 diff 内容**：幻影文件的 diff 要么为空，要么整篇全红全绿且新旧文字逐字相同（纯行尾）。
- **行尾专项验证**：`git diff --cached --ignore-cr-at-eol --stat`——忽略行尾后 diff 归空，即证明"只是行尾"。
- **听 warning**：`warning: LF will be replaced by CRLF the next time Git touches it` 是 `autocrlf=true` 的标志语，直接指认行尾类幻影。
- **对文件名单**：幻影名单里常出现从未改动过的文件（对照提交记录即可识别）。

---

## 解决原则（根治，而不是每次丢弃）

1. **行尾规则由仓库裁决**，不依赖各机器的全局配置：入库 `.gitattributes`（最小可用 `* text=auto`），一次性 `git add --renormalize .` 把存量文件规范化并**单独提交**。此后任何一侧 git 看到的都一致。
2. **挂载文件系统上关掉 filemode 比较**：`git config core.filemode false`（仓库级）。mode 跟踪在 9p/drvfs 上没有可靠意义。
3. **只 renormalize，不 rewrite 工作区**：`--renormalize` 只改 git 的入库内容，不重写磁盘文件；绝不要对幻影文件用 checkout/restore/discard"修复"——那会按错误的行尾规则真实重写文件，制造行尾 churn。
4. **规范化提交与业务改动分离**：单独一个 `chore` 提交，便于回溯与 review。
5. **替代思路**：直接让编辑器用容器内 git（VS Code Dev Containers / Remote 打开挂载侧），从架构上消除"两侧 git 共用一个 .git"。

---

## 实例（2026-10-07，demo 仓库）

- 现象：VS Code 显示 8 个 M，容器内同一仓库 0 个。构成：3 个 CRLF 行尾幻影（`docs/Agent开发核心宗旨.md`、规格 001/002）+ 5 个 mode 幻影（含从未改动的 `main.rs`、`Dockerfile`）。
- 修复：`git config core.filemode false` → 新增 `.gitattributes`（`* text=auto`）→ `git add --renormalize .` → 单独提交 `46d3fb2 chore: normalize git line endings`。
- 验证：`--ignore-cr-at-eol` 归空（纯行尾证明）；VS Code 刷新后 8 个 M 全部消失；同项目另一仓库（/vault Obsidian）的真实修改未受影响。
- 详细日志：`AgentLab/projects/demo/logs/2026-10-07-git-phantom-changes-fix.md`
