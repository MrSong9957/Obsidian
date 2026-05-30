---
name: Docker 容器内安装 Python 版本
description: 容器网络受限环境下安装 Python 版本的经验路线图
type: feedback
updated: 2026-05-14
tags: docker, python, networking, experience
source: https://example.com
---

# Docker 容器内安装 Python 版本

## 核心结论

1. 容器无法直连 GitHub/Debian 官方源（SSL 握手失败 + 502），必须走国内镜像
2. 有效路线：**miniconda（清华镜像）→ conda（清华频道 + `--override-channels`）→ 重建 venv → pip（清华 PyPI）**
3. conda 装完 Python 后即可删除 miniconda，只保留二进制

## 环境约束

容器通过 `host.docker.internal:7890` 代理出网，但 TLS 对西方主机不可靠。

| 目标 | 状态 | 说明 |
|------|------|------|
| `deb.debian.org` | 502 | apt-get 不可用 |
| `github.com` / `astral.sh` | SSL eof | releases/安装脚本均失败 |
| `repo.anaconda.com` | SSL eof | conda 默认频道不可用 |
| `pypi.tuna.tsinghua.edu.cn` | 通 | pip 依赖安装 |
| `mirrors.tuna.tsinghua.edu.cn` | 通 | miniconda + conda 包 |

> **注意**：容器只设了大写 `HTTP_PROXY`，curl 等工具需要小写 `https_proxy` 才走代理。

## 失败路线

| # | 方法 | 失败原因 |
|---|------|----------|
| 1 | apt-get build-essential | 502 Bad Gateway，编译工具装不了 |
| 2 | GitHub releases（indygreg 预编译包） | SSL_ERROR_SYSCALL，加代理也失败 |
| 3 | Python 源码编译 | 源码下到了（npmmirror），但无 GCC |
| 4 | uv python install | 底层从 GitHub 下载，同样 SSL 失败 |

> **关键认知**：uv/pyenv 等工具底层都从 GitHub 下载二进制，GitHub 不通则全废。先 `curl -v https://github.com` 测网络再选路线。

## 成功路线：miniconda（清华镜像）

```bash
# 1. 下载 + 安装 miniconda
curl -fsSL -o /tmp/miniconda.sh --proxy http://host.docker.internal:7890 \
  "https://mirrors.tuna.tsinghua.edu.cn/anaconda/miniconda/Miniconda3-latest-Linux-x86_64.sh"
bash /tmp/miniconda.sh -b -p ~/.local/miniconda3

# 2. 用清华频道创建 Python 3.13（必须 --override-channels）
~/.local/miniconda3/bin/conda create -y -n py313 python=3.13 \
  --override-channels -c https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/main/

# 3. 重建 venv + 装依赖
~/.local/miniconda3/envs/py313/bin/python -m venv .venv
pip install anthropic pydantic -i https://pypi.tuna.tsinghua.edu.cn/simple

# 4. 清理 miniconda
rm -rf ~/.local/miniconda3
```

> `--override-channels` 阻止 conda 回退到被墙的 `repo.anaconda.com`。

## 目录权限不匹配

容器镜像构建时由 `root` 创建的目录，运行时以 `app` 用户执行会导致写入失败。

**表现：** `EACCES: permission denied, mkdir '/home/app/project/.claude/worktrees'`（EnterWorktree 被拒绝）

**原因：** `.claude/` 目录 owner 为 `root`（755），`app` 用户无写权限。

**修复：** `sudo chown -R app:app /home/app/project/.claude/`

**预防：** Dockerfile 中 `COPY` 后加 `RUN chown -R app:app /path`，或在 entrypoint 中用 `gosu` 降权前先 chown。

## 决策树

```
需要安装 Python 版本？
├── apt 可用？→ deadsnakes PPA 或源码编译
├── GitHub 可达？→ uv python install 或 indygreg 预编译包
├── 仅国内镜像通？→ miniconda（清华）+ --override-channels（本路线）
└── 全不通？→ 宿主机 docker cp 二进制进来
```
