# Yazi 主题安装与配色配置

## 安装环境

- **平台**: Windows 11, WinGet 安装
- **安装路径**: `C:\Users\<user>\AppData\Local\Microsoft\WinGet\Packages\sxyazi.yazi_Microsoft.Winget.Source_8wekyb3d8bbwe\yazi-x86_64-pc-windows-msvc\`
- **配置目录**: `%AppData%\yazi\config\`（即 `C:\Users\<user>\AppData\Roaming\yazi\config\`）
- **Yazi 版本**: 26.1.22

## 主题系统

### 关键发现：Yazi 没有内置主题

Yazi 默认外观是源码硬编码的最小样式，不包含任何预置 flavor。所有好看的配色都是社区贡献的 flavor 包，需要手动安装。

### Flavor vs Theme

- **Flavor**: 预制主题包（由社区维护），放在 `flavors/` 目录下，用 `ya pkg` 管理
- **Theme**: 用户自定义样式，写在 `theme.toml` 中，可以覆盖 flavor 的部分设置

### 配色系统

- 颜色支持 Hex RGB（`"#484D66"`）和 17 种命名色（black, white, red, lightred, green, lightgreen, yellow, lightyellow, blue, lightblue, magenta, lightmagenta, cyan, lightcyan, gray, darkgray, reset）
- Style 对象支持 fg, bg, bold, dim, italic, underline, blink, reversed, hidden, crossed 等属性
- 可配置区域包括：app, mgr, indicator, tabs, mode, status, which, confirm, spot, notify, pick, input, cmp, tasks, help, filetype, icon

## 安装 Catppuccin Mocha

### 步骤 1：安装 flavor

```bash
ya pkg add yazi-rs/flavors:catppuccin-mocha
```

输出：
```
Upgrading package `catppuccin-mocha.yazi`
Cloning into '...\77b6b1d6e440b3af955262706bce251'...
Deploying package `catppuccin-mocha.yazi`
Done!
```

### 步骤 2：创建 theme.toml

配置目录不存在时需先创建：
```bash
mkdir -p "$APPDATA/yazi/config"
```

创建 `%AppData%\yazi\config\theme.toml`：
```toml
[flavor]
dark = "catppuccin-mocha"
```

### 步骤 3：重启 yazi

重启即可看到 Catppuccin Mocha 配色——深紫灰底色 + 柔和的彩色高亮。

## 社区 Flavor 列表

来自 `yazi-rs/flavors` 仓库和社区贡献，共 30+ 个：

| 系列 | 名称 |
|------|------|
| Catppuccin | mocha, latte, frappe, macchiato |
| Kanagawa | kanagawa, dragon, lotus |
| Gruvbox | dark, material |
| Rose Pine | rose-pine, moon, dawn |
| Flexoki | dark, light |
| Bluloco | dark, light |
| 单色主题 | dracula, tokyo-night, ayu-dark, nord, monokai, synthwave84, eldritch, neon, lain |
| 特色主题 | ashen, modus, obsidian-glow, claude-inspired, thinkpad-red-oled, everforest-medium |
| 通用 | base16（跟随终端主题色） |

## 护眼推荐

| 偏好 | 暗色 | 亮色 |
|------|------|------|
| 柔和低对比 | catppuccin-mocha | catppuccin-latte |
| 暖色调 | gruvbox-dark | flexoki-light |
| 冷色调 | tokyo-night | — |
| 跟随终端 | base16 | base16 |
