---
name: yazi
description: Yazi 安装、配置经验 — Windows 安装、主题配色、预览插件、踩坑记录
type: summary
updated: 2026-06-05
tags: cli, yazi, config, theme, windows
source: self
node_type: memory
originSessionId: 42a3e0e7-65d6-4a16-8633-30e9a0b95b7e
---


# Yazi 文件管理器

## 安装（Windows）

- WinGet 安装：`winget install sxyazi.yazi`，安装后**需重启终端**才能使用 `yazi`/`ya` 命令
- 版本 26.5.6（2026-06-05 实测）
- **Windows 依赖**：`file.exe` 必须在 PATH 中，否则报 `Cannot find 'file' to detect the file's MIME type`
  - `file.exe` 在 `E:\Program Files (x86)\Git\usr\bin\`（Git for Windows 自带）
  - 修复：将该目录加到用户 PATH
  - PowerShell：`[Environment]::SetEnvironmentVariable("Path", [Environment]::GetEnvironmentVariable("Path", "User") + ";E:\Program Files (x86)\Git\usr\bin", "User")`

## 插件安装踩坑

- `ya pkg add MrSong9957/code-ln` 可能因包缺少 LICENSE 文件而报错中断（exit code 1）
- 文件实际已下载到 `%LOCALAPPDATA%\yazi\packages\<hash>\` 目录
- 手动修复：`mkdir -p "$APPDATA/yazi/config/plugins/code-ln.yazi" && cp "$LOCALAPPDATA/yazi/packages/<hash>/main.lua" "$APPDATA/yazi/config/plugins/code-ln.yazi/"`

## 主题配色

- Yazi **没有内置主题**，默认外观是源码硬编码最小样式，需要手动安装社区 flavor ← [[sources/tools/yazi-theme-setup.md]] L13-L14
- 安装命令：`ya pkg add yazi-rs/flavors:主题名`，如 `ya pkg add yazi-rs/flavors:catppuccin-mocha` ← [[sources/tools/yazi-theme-setup.md]] L39-L42
- 在 `theme.toml` 中用 `[flavor] dark = "xxx"` 启用，支持 dark/light 分别指定 ← [[sources/tools/yazi-theme-setup.md]] L48-L52
- `theme.toml` 中可覆盖 flavor 的个别样式，不用改 flavor 文件本身 ← [[sources/tools/yazi-theme-setup.md]] L8-L10

## Windows 配置路径

- 配置目录：`%AppData%\yazi\config\`（首次使用需手动创建）← [[sources/tools/yazi-theme-setup.md]] L7,L45-L48
- Flavors 安装位置由 `ya pkg` 自动管理，用户无需关心 ← [[sources/tools/yazi-theme-setup.md]] L18

## 护眼 Flavor 推荐

- 暗色首选 **catppuccin-mocha**（深紫灰底 + 柔和彩色，低对比度）← [[sources/tools/yazi-theme-setup.md]] L58,L80
- 亮色首选 **catppuccin-latte** ← [[sources/tools/yazi-theme-setup.md]] L80
- **base16** 特殊：不自带颜色，读取终端 16 色配置，换终端主题自动跟随 ← [[sources/tools/yazi-theme-setup.md]] L67

## 预览换行

- `[preview] wrap = "yes"` 开启预览面板自动换行（yazi v26.5.6+ 已验证）← [[sources/tools/yazi-wrap-fix.md]] L1-L10
- Windows 配置必须放在 `%AppData%\yazi\config\yazi.toml`，非 `~/.config/yazi/`（WSL/Docker 容器内路径对 yazi.exe 无效）← [[sources/tools/yazi-wrap-fix.md]] L13-L18
- **禁止**用 `Set-Content -Value '多行文本'` 写 TOML，PowerShell 会拼成单行导致解析失败；用数组语法 `'[line1]', '[line2]' | Set-Content path` ← [[sources/tools/yazi-wrap-fix.md]] L24-L35

## 预览行号

- Yazi 无内置行号选项，需用自定义 Lua 预览器插件实现 ← [[sources/tools/yazi-preview-line-numbers.md]] L1-L10
- 插件路径：`%AppData%\yazi\config\plugins\code-ln.yazi\main.lua`，在 `yazi.toml` 的 `[plugin].prepend_previewers` 中用 `mime = "text/*"` 注册 ← [[sources/tools/yazi-preview-line-numbers.md]] L55-L70
- `seek()` 不可省略——自定义预览器必须实现 `seek()` 才能响应鼠标滚轮 ← [[sources/tools/yazi-preview-line-numbers.md]] L84-L87
- **滚动幅度**：`seek()` 中 `math.floor(job.units * job.area.h / 10)` 按面板高度比例滚动 ← [[sources/tools/yazi-preview-line-numbers.md]] L90-L94
- **禁止**用 `io.popen`——Windows 上会闪现控制台窗口抢夺焦点，必须用 `Command` API ← [[sources/tools/yazi-preview-line-numbers.md]] L110-L120

### 当前方案：合并单插件（方案 7，默认 bat 高亮 + `--fast` 纯文本）

- 合并 bat 版和 fast 版为单 `main.lua`，通过 `--fast` 参数切换模式 ← [[sources/tools/yazi-preview-line-numbers.md]] L160-L166
- 默认 `run = "code-ln"` → bat 高亮 + 行号；`run = "code-ln --fast"` → 纯文本 + 行号，零延迟 ← [[sources/tools/yazi-preview-line-numbers.md]] L186-L198
- bat 未安装时自动降级为纯文本 + 行号（`cache.ansi` 标志区分渲染路径）← [[sources/tools/yazi-preview-line-numbers.md]] L168-L170
- fallback 路径也缓存到 `cache.lines`，不再每帧重新读文件 ← [[sources/tools/yazi-preview-line-numbers.md]] L173-L174
- 安装：`ya pkg add MrSong9957/code-ln`（根目录结构，无子目录后缀）← [[sources/tools/yazi-preview-line-numbers.md]] L178-L183
- 已提交 PR #206 到 `yazi-rs/plugins` 官方注册表，等待 review ← [[sources/tools/yazi-preview-line-numbers.md]] L200-L204
