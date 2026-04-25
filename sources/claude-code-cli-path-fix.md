# Claude Code CLI 命令找不到的修复

## 问题现象

在终端输入 `claude` 提示命令不存在，`npm` 也提示找不到。用户确定之前可以直接使用 `claude` CLI。

## 根因

Claude Code CLI 的可执行文件存储在：

```
C:\Users\<用户名>\.local\share\claude\versions\<版本号>
```

这是一个**独立的 PE 可执行文件，自带 Node.js 运行时**，不依赖系统安装的 Node.js/npm。

正常安装时，会在 `~/.local/bin/` 目录创建指向当前版本的符号链接。`~/.local/bin/` 已在 PATH 中，所以 `claude` 命令可用。

**符号链接意外丢失**（可能因清理、更新失败等原因），导致 PATH 中找不到 `claude`。

## 修复方法

在 PowerShell 中创建符号链接：

```powershell
New-Item -ItemType SymbolicLink -Path "$env:USERPROFILE\.local\bin\claude.exe" -Target "$env:USERPROFILE\.local\share\claude\versions\<版本号>"
```

验证：

```powershell
claude --version
```

## 补充说明

- 旧版本可安全删除，如 `versions` 目录下有多个版本，保留最新的即可
- 每个版本约 230-250MB
- 符号链接方式优于修改 PATH 环境变量：更新后只需重新指向新版本，无需修改系统变量

## 相关路径

| 路径 | 说明 |
|------|------|
| `~/.local/share/claude/versions/` | CLI 可执行文件存储 |
| `~/.local/bin/claude.exe` | 符号链接（应在 PATH 中） |
| `~/AppData/Local/claude-cli-nodejs/` | 自带 Node.js 运行时缓存 |
| `~/.claude/` | 配置和插件目录 |
