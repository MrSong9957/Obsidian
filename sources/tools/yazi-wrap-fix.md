---
name: Yazi 文本换行配置调试
description: yazi 预览不换行问题的排查过程 — 从错误路径到 PowerShell 写入陷阱
date: 2026-05-09
---

# Yazi 文本换行配置调试

## 问题
yazi 预览面板长文本不自动换行，超出终端宽度的内容被截断。

## 排查过程

1. 查阅 yazi 文档（context7），确认 `[preview] wrap = "yes"` 是原生支持选项
2. 在 Linux 容器创建 `~/.config/yazi/yazi.toml`，写入 `wrap = "yes"`
3. 用户反馈未生效
4. 请求用户运行 `yazi --version`，确认版本为 **Yazi 26.5.6**（完全支持 wrap）
5. 用户终端路径为 PowerShell，缓存目录 `C:\Users\<user>\AppData\Local\Temp\yazi-0`
6. 发现根因：配置写在了 Linux 容器的 `~/.config/yazi/`，而 yazi.exe 在 Windows 上读取 `%AppData%\yazi\config\`

## PowerShell 写入陷阱

用 `Set-Content -Value '...'` 写多行内容时，PowerShell 会把所有内容拼成一行：

```powershell
# 错误 — 全部内容变成单行，TOML 解析失败
Set-Content -Path "$env:AppData\yazi\config\yazi.toml" -Value '[preview]
wrap = "yes"
tab_size = 4'
```

报错：`TOML parse error at line 1, column 117 — unexpected key or value, expected newline`

正确写法是数组语法：

```powershell
'[preview]', 'wrap = "yes"', 'tab_size = 4' | Set-Content "$env:AppData\yazi\config\yazi.toml"
```

## 最终方案

```toml
[preview]
wrap = "yes"
tab_size = 4
```

配置路径：`%AppData%\yazi\config\yazi.toml`
