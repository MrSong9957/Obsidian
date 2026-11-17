---
name: Rust 环境搭建与工具链
description: Rust 安装四场景（Windows、GNU 免 VS、WSL2 容器、macOS）、rustup 镜像加速、自定义安装目录、Cargo 与容器编译踩坑 — 装环境或排查工具链问题时参考
type: reference
updated: 2026-11-17
tags: [rust, installation, rustup, toolchain, cargo, docker, mirror, troubleshooting]
source: "[[sources/dev/rust]]"
---

# Rust 环境搭建与工具链

**核心：装 Rust 用 rustup；先备好 C 链接器；下载慢就先设 USTC 镜像变量再装。**

本页是入口。完整命令、实测数据和踩坑记录见 [[sources/dev/rust|sources/dev/rust.md]]。

## 选哪条路线

| 场景 | 路线 | 关键点 |
|---|---|---|
| Windows 常规 | rustup-init.exe + MSVC | 官方推荐；需 Visual Studio Build Tools 的「C++ 桌面开发」工作负载 |
| Windows 免 Visual Studio | rustup-init.exe 自定义安装 + GNU 工具链 | host triple 手输 `x86_64-pc-windows-gnu`；缺链接器时 winget 装 WinLibs |
| WSL2 / Docker 容器 | 官方脚本装到用户目录 | 不要用 root 装；下载中断清 `~/.rustup/downloads/` 后重试 |
| macOS | Xcode CLT + 官方脚本 | 系统自带 Clang，无需额外装编译器 |

## 三条通用经验

1. **下载慢，先上镜像。** Windows 直连工具链约 50 KB/s；先设 `RUSTUP_DIST_SERVER` 和 `RUSTUP_UPDATE_ROOT` 到 USTC 镜像再装，实测约 3.7 MB/s。两个变量只影响当次会话。安装中断后重跑 `rustup-init.exe -y` 自动续装，无需清理目录。
2. **自定义目录，两个变量都要设。** `RUSTUP_HOME`（工具链与元数据，单工具链 1–2 GB）和 `CARGO_HOME`（cargo 本体与缓存）都先 `setx` 写入用户环境变量，新开终端确认后再跑安装器。只设一个会把工具链和缓存拆到两处。
3. **链接器报错，先查 `cc` 指向。** 容器里 `linker 'cc' not found` 是缺 gcc；`gnu_get_libc_version` 未定义是新 Rust 要配新 glibc；`cc` 指向 musl-gcc 时，把符号链接改回系统 gcc。详见详文第 8 节。

## 内容地图（sources/dev/rust.md 章节）

| 章节 | 内容 |
|---|---|
| 1 安装 | 四场景完整步骤、镜像加速、自定义目录 |
| 2 工具链管理 | rustup list / update / default |
| 3–6 Cargo | Cargo 职责、常用命令、项目结构、常用 crate |
| 7 核心概念 | 所有权、借用、生命周期、枚举与 match、宏、栈与堆、二次释放 |
| 8 容器编译踩坑 | glibc 版本、cc 符号链接、musl 目标 |
| 9 查文档六法 | 编辑器悬停、cargo doc、docs.rs、故意报错、type_name_of_val、rustup doc |

## 相关页面

- 学习路线与工具取舍：[[wiki/Rust学习完整路线|《Rust学习完整路线》]]（其第 0 步「装环境」落到本页执行）
- 语言概念与教材导读：[[wiki/Rust 程序设计语言/Rust 入门：核心价值、开发工具与学习方法|《Rust 入门：核心价值、开发工具与学习方法》]]
- 容器内工具链何时装、Cargo 目录持久规则：[[wiki/Docker容器开发方案|《Docker容器开发方案》]]
