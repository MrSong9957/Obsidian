---
name: Yazi 文件管理器
description: Yazi 配置经验 — 主题安装、配色系统、Windows 配置路径
type: summary
updated: 2026-05-07
tags: [cli, yazi, config, theme]
sources:
  - "[[sources/yazi-theme-setup.md]]"
---

# Yazi 文件管理器

## 主题配色

- Yazi **没有内置主题**，默认外观是源码硬编码最小样式，需要手动安装社区 flavor ← [[sources/yazi-theme-setup.md]] L13-L14
- 安装命令：`ya pkg add yazi-rs/flavors:主题名`，如 `ya pkg add yazi-rs/flavors:catppuccin-mocha` ← [[sources/yazi-theme-setup.md]] L39-L42
- 在 `theme.toml` 中用 `[flavor] dark = "xxx"` 启用，支持 dark/light 分别指定 ← [[sources/yazi-theme-setup.md]] L48-L52
- `theme.toml` 中可覆盖 flavor 的个别样式，不用改 flavor 文件本身 ← [[sources/yazi-theme-setup.md]] L8-L10

## Windows 配置路径

- 配置目录：`%AppData%\yazi\config\`（首次使用需手动创建）
- Flavors 安装位置由 `ya pkg` 自动管理，用户无需关心

## 护眼 Flavor 推荐

- 暗色首选 **catppuccin-mocha**（深紫灰底 + 柔和彩色，低对比度）
- 亮色首选 **catppuccin-latte**
- **base16** 特殊：不自带颜色，读取终端 16 色配置，换终端主题自动跟随 ← [[sources/yazi-theme-setup.md]] L67
