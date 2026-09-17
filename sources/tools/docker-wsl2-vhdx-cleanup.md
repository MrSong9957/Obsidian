---
name: docker-wsl2-vhdx-cleanup
description: Docker Desktop WSL2 后端 VHDX 虚拟磁盘清理与压缩指南
type: concept
updated: 2026-06-08
tags: [docker, wsl2, disk-cleanup, vhdx, windows]
source: self
---

# Docker Desktop WSL2 VHDX 清理与压缩

## 背景

Docker Desktop 使用 WSL2 后端时，所有数据存储在一个 VHDX 虚拟磁盘文件中：

```
D:\Files\Docker\DockerDesktopWSL\disk\docker_data.vhdx
```

该文件包含：Docker 镜像层、容器（运行中和已停止的）、卷（volumes）、构建缓存（build cache）、WSL2 系统本身。

**关键特性：VHDX 是"只增不减"的稀疏磁盘。** 删除 Docker 资源后，VHDX 文件大小不会自动回落，必须手动压缩。

## 操作流程

### 第一步：查看 Docker 磁盘使用情况

```powershell
docker system df
```

输出示例：

| 类型 | 总数 | 活跃 | 大小 | 可回收 |
|------|------|------|------|--------|
| Images | 2 | 1 | 30.1GB | 2.5GB (8%) |
| Containers | 1 | 0 | 9.5GB | 9.5GB (100%) |
| Local Volumes | 6 | 1 | 2.3GB | 2.0GB (87%) |
| Build Cache | 95 | 0 | 16.7GB | 11.0GB |

### 第二步：清理 Docker 资源

```powershell
# 一键全清（停止的容器 + 未使用镜像 + 卷 + 构建缓存）
docker system prune -a --volumes
```

警告：`--volumes` 会删除未使用的卷，确认卷中无重要数据再执行。

分步清理（更安全）：

```powershell
# 只清理停止的容器、悬空镜像、构建缓存
docker system prune

# 加上未使用的卷
docker system prune --volumes

# 删除所有未使用镜像
docker system prune -a
```

### 第三步：压缩 VHDX 文件

```powershell
# 1. 关闭 Docker Desktop（右键系统托盘图标 → Quit）

# 2. 关闭 WSL
wsl --shutdown

# 3. 压缩 VHDX（管理员 PowerShell）
diskpart
# 在 diskpart 中依次输入：
select vdisk file="D:\Files\Docker\DockerDesktopWSL\disk\docker_data.vhdx"
compact vdisk
detach vdisk
exit
```

注意：`compact vdisk` 完成后会自动分离虚拟磁盘，`detach vdisk` 可能报"虚拟磁盘已经分离"，属正常现象。

## 实际效果

| 指标 | 清理前 | 清理后 |
|------|--------|--------|
| VHDX 文件大小 | 47.6 GB | 6.93 GB |
| 回收空间 | — | **40.67 GB** |

## 注意事项

- `docker system prune -a` 会删除所有未运行容器关联的镜像，下次使用需重新拉取
- VHDX 压缩必须在 Docker Desktop 和 WSL 都关闭的状态下进行
- 建议定期（每月）检查 `docker system df`，避免积累过多无用数据
- 如果 Docker Desktop 设置中启用了 "WSL 2 integration"，资源回收需在对应的 WSL 发行版中操作
