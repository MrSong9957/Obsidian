---
name: Obsidian（详细）
description: Obsidian CLI完整参考/快捷键/自动化脚本 — 需要完整命令或配置细节时参考
updated: 2026-04-17
tags: [obsidian, cli, reference]
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# Obsidian CLI 完整参考

> Obsidian 1.12+ 内置命令行工具，Early Access 阶段，需 Catalyst Insider 许可证

## 前置条件与安装

1. Obsidian 1.12+ 桌面版**正在运行**
2. Catalyst Insider 许可证（$25 一次性）
3. Settings -> General -> Register CLI
4. 添加到系统 PATH：

| 平台 | 方式 |
|------|------|
| macOS | 自动写入 `~/.zprofile` |
| Windows | 通过 Obsidian.com redirector |
| Linux | 创建 `/usr/local/bin/obsidian` 符号链接 |

```bash
obsidian version          # 验证安装
obsidian                  # 无参数启动 TUI 交互模式
```

## 命令分类速查

### 日常笔记与搜索

```bash
obsidian daily                              # 打开今日日记
obsidian daily:append content="- [ ] 买菜"   # 向日记追加内容
obsidian search query="会议记录"              # 全库搜索
obsidian read                               # 读取当前文件
```

### 任务与标签

```bash
obsidian tasks daily          # 列出日记中的任务
obsidian tags counts          # 标签频率统计
obsidian tags list            # 列出所有标签
```

### 文件操作与模板

```bash
obsidian create name="旅行" template=Travel   # 从模板创建笔记
obsidian files sort=modified limit=5          # 最近修改的 5 个文件
obsidian files sort=modified limit=5 --copy   # 复制文件路径到剪贴板
obsidian diff file=README from=1 to=3         # 比较文件版本差异
obsidian links                               # 显示当前文件的链接
obsidian unresolved                          # 检查未解析的链接
```

### 书签与属性

```bash
obsidian bookmarks list       # 列出书签
obsidian bookmarks add        # 添加书签
obsidian properties           # 查看当前文件属性
```

### 开发者工具

```bash
obsidian devtools                                # 打开 DevTools
obsidian plugin:reload my-plugin                 # 重载插件
obsidian eval "app.vault.getFiles().length"      # 执行 JS 表达式
obsidian dev:errors                              # 查看 JS 错误日志
obsidian dev:screenshot file=shot.png            # 截图保存
obsidian dev:css selector=".workspace"           # 检查 CSS
obsidian dev:dom selector=".nav-files"           # 查询 DOM
```

### 同步

```bash
obsidian sync                     # 手动触发同步
obsidian sync --headless          # 无 GUI 运行同步（服务器/自动化场景）
obsidian sync status              # 查看同步状态
```

### 主题与插件管理

```bash
obsidian themes list              # 列出已安装主题
obsidian plugins list             # 列出已安装插件
```

## TUI 交互模式

运行 `obsidian`（无参数）进入 TUI。

### 键盘快捷键

**导航**

| 快捷键 | 功能 |
|--------|------|
| <- / Ctrl+B | 光标左移 |
| -> / Ctrl+F | 光标右移 |
| Ctrl+A | 移到行首 |
| Ctrl+E | 移到行尾 |
| Alt+B | 前移一词 |
| Alt+F | 后移一词 |

**编辑**

| 快捷键 | 功能 |
|--------|------|
| Ctrl+U | 清除光标前内容 |
| Ctrl+K | 清除光标后内容 |
| Ctrl+W / Alt+Backspace | 删除前一词 |
| Tab | 接受补全 |
| Shift+Tab | 关闭补全 |

**通用**

| 快捷键 | 功能 |
|--------|------|
| Up / Ctrl+P | 上一条历史 |
| Down / Ctrl+N | 下一条历史 |
| Ctrl+R | 搜索历史 |
| Escape | 取消 |
| Ctrl+L | 清屏 |
| Ctrl+C / Ctrl+D | 退出 |

**文件浏览器（TUI 内）**

| 快捷键 | 功能 |
|--------|------|
| Up/Down | 选择文件 |
| Enter | 打开文件 |
| / | 搜索 |
| n | 新建 |
| d | 删除 |
| r | 重命名 |
| q | 退出 |

## 自动化示例

```bash
# JSON 输出（脚本友好）
obsidian search query="status::active" vault="Notes" format=json

# 管道组合
obsidian files sort=modified limit=5 --copy | xargs -I{} cat "{}"

# 批量操作
for tag in $(obsidian tags list | tail -n +2); do
  echo "Tag: $tag"
done

# 无头同步（crontab / CI）
obsidian sync --headless --vault="Work"
```

## 命令分类总览

| 分类 | 命令 |
|------|------|
| daily | 日常笔记创建与追加 |
| search | 全库全文搜索 |
| read | 读取文件内容 |
| create | 从模板创建笔记 |
| tasks | 任务管理 |
| tags | 标签统计与列表 |
| files | 文件列表与排序 |
| links | 链接关系查看 |
| bookmarks | 书签管理 |
| properties | 文件属性（YAML frontmatter） |
| diff | 文件版本对比 |
| devtools | 开发者工具 |
| plugin | 插件管理 |
| eval | JS 表达式执行 |
| dev:* | 开发调试（errors/screenshot/css/dom） |
| sync | 同步控制 |
| themes | 主题管理 |
| templates | 模板管理 |
| vault | 仓库操作 |
| version | 版本信息 |
