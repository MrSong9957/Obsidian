---
name: yazi-preview-line-numbers
description: 一句话说明
type: concept
updated: 2026-05-30
tags: [tag1, tag2]
source: https://example.com
---
# Yazi 预览行号 — 方案探索与实现

## 需求

Yazi 预览文件时不显示行号，无法快速定位代码内容。需要添加行号显示，同时保持三栏面板布局不被破坏。

## 方案演进

### 方案 1：bat 每帧调用（已弃）

每次 `peek()` 调用 `bat --style=numbers --line-range=N:M`，只获取可视区域的行。

- 显示：✓ 行号+语法高亮正确
- 滚动：✗ 极度卡顿（每次滚动启动新 bat 进程，Windows 进程创建开销大）

### 方案 2：bat 全文缓存 + 每帧 parse（已弃）

首次调 bat 获取全文，缓存原始字符串；每帧滚动时 `table.concat` + `ui.Text.parse()` 重新解析 ANSI。

- 显示：✓
- 滚动：✗ 卡（ANSI 解析开销在每帧重复执行）

### 方案 3：bat 全文缓存 + 预解析 ui.Text（当前采用）

首次调 bat 获取全文，**逐行** `ui.Text.parse()` 预解析为 ui.Text 对象缓存；滚动时纯数组下标访问 + `ui.Rect` 构造。

- 显示：✓ 行号+语法高亮正确
- 滚动：~可接受（有轻微延迟，Lua↔Rust 桥接开销无法消除）

### 方案 4：ya.preview_code + 行号叠加（不可行）

用 Rust 原生 `ya.preview_code` 渲染代码区域，Lua 侧只画行号数字。

- 显示：✗ 报错 `Lua error: error converting Lua nil to string`
- 原因：`ya.preview_code` 不能从自定义预览器插件内部调用，与预览系统冲突

## 最终实现

### 插件文件

`%AppData%\yazi\config\plugins\code-ln.yazi\main.lua`

核心逻辑：
1. 首次 `peek()`：`bat --style=numbers --color=always --paging=never --wrap=never` 获取全文
2. 按行 `ui.Text.parse()` 预解析，缓存到 `cache[url]`（单文件缓存，切换文件时替换）
3. 滚动 `peek()`：数组切片 + `ui.Rect` 构造，无字符串解析
4. `seek()`：`ya.emit("peek", ...)` 触发重新渲染

### 配置注册

`%AppData%\yazi\config\yazi.toml`：
```toml
[plugin]
prepend_previewers = [
    { mime = "text/*", run = "code-ln" },
]
```

### 依赖

- `bat`（`winget install sharkdp.bat`）— 提供行号+语法高亮
- 不触碰 `[mgr]` ratio，面板布局不受影响

## 性能瓶颈分析

Lua 插件的滚动性能上限受 Lua↔Rust 桥接约束：
- 每次 `peek()` 调用跨越 Lua→Rust→Lua
- 每帧创建 ~40 个 `ui.Rect` + `ui.Text` 对象
- `ya.preview_widget()` 传递 widget 列表需要序列化

这些开销无法通过 Lua 侧优化消除，只能等 Yazi 原生支持行号。

## 方案 5：ya.preview_code pcall 混合方案（当前采用）

方案 4 的 nil 错误可以通过 `pcall` 包装解决——`ya.preview_code` 在自定义预览器中调用时可能因内部状态问题报错，pcall 捕获后回退到纯 Lua 渲染。

### 核心思路

1. `pcall(ya.preview_code, { area, file, skip })` 尝试 Rust 原生渲染
2. 成功 → 只叠加行号列（极轻量，~40 个 `ui.Line`）
3. 失败 → 回退到纯 Lua：`io.open` 读文件，手动拼接行号 + 内容

### 关键实现细节

- **区域分割**：行号占左侧 6 字符宽（`LN_W = 6`），代码区域从 `area.x + 6` 开始
- **防御性检查**：`area.w < 8` 时直接返回，防止窄终端下宽度为负
- **seek() 必须保留**：自定义预览器插件必须实现 `seek()` 才能响应鼠标滚轮——内置预览器由 Rust 处理滚动，自定义插件依赖 `seek() → ya.emit("peek")` 链条

### 滚动幅度调节

默认 `step = math.floor(job.units)` 是 1:1 映射，每次滚轮只走 1 行。乘以倍率即可加速：

```lua
local step = math.floor(job.units) * 3  -- 滚一下走 3 行
```

倍率可按手感调整（3 为推荐起步值）。

### 性能对比

| 指标 | bat 缓存方案（方案 3） | pcall 混合方案（方案 5） |
|------|----------------------|------------------------|
| 代码渲染 | bat 子进程 + ANSI 解析 | Rust 原生（成功时）/ 纯文本（回退时） |
| 行号渲染 | bat 内置 | 独立行号列叠加 |
| 桥接次数 | 4 次（seek→emit→peek→widget） | 3 次（peek→preview_code→widget） |
| 首次渲染 | 需启动 bat 进程 | Rust 直接处理 |

## 方案 6：Command API 替代 io.popen（当前采用，2026-05-09）

方案 3 的 `io.popen` 实现存在致命问题：**鼠标全部失效**（无法点击、无法拖动面板、无法滚动）。

### 根因

`io.popen` 是 Lua 标准库同步 API。在 Windows 上，`io.popen` 创建子进程时会**短暂闪现控制台窗口**，抢夺终端焦点。这导致：
- 鼠标点击事件被吞（焦点不在 Yazi 窗口）
- 面板分隔线无法拖动调整
- 鼠标滚轮事件丢失

源码层面：Yazi 的 `peek` 运行在 `spawn_blocking` 线程上，`io.popen` 本身不阻塞主线程，但 Windows 控制台窗口的焦点抢占发生在 OS 层面，与线程模型无关。

### 修复

将 `io.popen` 替换为 Yazi 内置 `Command` API（基于 `tokio::process::Command`）：

```lua
-- 旧（导致焦点抢占）：
local handle = io.popen(string.format('bat ... -- "%s" 2>NUL', url))

-- 新（async，无控制台窗口）：
local output, err = Command("bat"):arg({
    "--style=numbers", "--color=always", "--paging=never", "--wrap=never",
    "--", path,
}):output()
if output and output.stdout ~= "" then
    for line in output.stdout:gmatch("[^\r\n]+") do
        cache.lines[#cache.lines + 1] = line
    end
end
```

关键改动：
- `io.popen` → `Command("bat"):arg({...}):output()` — Yazi async API，不创建控制台窗口
- `tostring(job.file.url)` → `tostring(job.file.path)` — 得到文件系统路径而非 URL 格式
- seek 步进改为 `math.floor(job.units * job.area.h / 10)` — 与内置 code.lua 一致

### 渲染路径

仍使用**可见区切片 + ui.Text.parse**（方案 3 的无 scroll 版本）：
1. 缓存 bat 原始字符串行（不预解析）
2. 每次 peek 取 `table.concat(lines, "\n", skip+1, end_i)` 拼接可视区
3. `ui.Text.parse(visible_raw):area(area)` 解析 ~40 行 ANSI → 渲染

### 效果

- 显示：✓ 行号 + 语法高亮
- 滚动：~可接受（轻微卡顿，Lua↔Rust 桥接开销）
- 鼠标：✓ 点击/拖动/滚轮全部正常
- 面板：✓ 分隔线可拖动调整

## 方案 7：合并 bat/fast 为单插件 + PR 官方注册表（2026-05-09）

此前本地仓库 `MrSong9957/code-ln.yazi` 有两个独立目录 `bat.yazi/` 和 `fast.yazi/`，用户需分别安装。为提交到官方插件注册表 `yazi-rs/plugins`，合并为单插件。

### 合并设计

- 单 `main.lua`，通过 `job.args[1] == "--fast"` 切换模式
- 默认（无参数）→ `Command("bat")` 路径 + `ui.Text.parse` 渲染 ANSI
- `--fast` → 跳过 bat，纯 `io.open` + `ui.Line`/`ui.Span` 构造
- `cache.ansi` 标志区分渲染路径
- bat 失败时自动降级为纯 Lua（fallback 逻辑保留）

### 修复：fallback 缓存

原 bat 版的 fallback 路径每次 `seek` 都重新 `io.open` 读文件。合并版将 fallback 结果也缓存到 `cache.lines`，滚动时只做渲染、不做 I/O。

### 仓库结构调整

本地仓库最终结构（main.lua 在根目录）：
```
code-ln.yazi/
  main.lua
  README.md
  README.zh-CN.md
```

安装命令简化：`ya pkg add MrSong9957/code-ln`（不再需要 `:bat` 或 `:fast` 后缀）。

用法：
```toml
# 默认（bat 高亮）
[[plugin.prepend_previewers]]
mime = "text/*"
run = "code-ln"

# 快速（纯文本，零延迟）
[[plugin.prepend_previewers]]
mime = "text/*"
run = "code-ln --fast"
```

### PR 提交

- Fork `yazi-rs/plugins` → 分支 `add-code-ln` → PR #206
- PR 中 `code-ln.yazi/` 作为子目录（符合官方注册表规范），本地仓库则直接根目录
- 等待 yazi-rs/plugins 维护者 review
