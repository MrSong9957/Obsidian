---
name: 调试教训
description: 调试方法论 — 先观察再假设、日志先行 — 遇到调试问题或总结调试经验时查阅
type: concept
updated: 2026-04-12
tags: [debugging, experience]
status: deprecated
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# 调试教训

> 内容已迁移至 [[wiki/dev-practice.md|dev-practice]]。

## 2026-03-21: 菜单返回 Bug

### 问题
按 Ctrl+P 打开菜单后，按 Enter 无法返回，程序"卡死"。

### 根因
`key.name === "enter"` 在某些系统上不匹配，因为 Enter 键被报告为 `"return"`。

### 错误的调试过程
- 猜测是 stdin/raw mode/listener 问题
- 尝试了 10+ 次修复都不相关
- 花费数小时

### 正确的调试方法
1. 添加日志：`console.log('key.name=', key.name)`
2. 观察：看到 `"return"` 而非 `"enter"`
3. 修复：`key.name === "enter" || key.name === "return"`
4. 耗时：5 分钟

### 教训
**永远先观察，再假设。**
- 打印实际值，不要猜测
- 创建最小诊断脚本
- Phase 1 (观察) 比 Phase 4 (修复) 更重要

### 通用规则
```
遇到任何 Bug:
1. 第一件事：加日志，打印相关变量
2. 运行，观察输出
3. 对比期望 vs 实际
4. 只修差异点
```

## 2026-04-12: UTF-8 Surrogate 编码错误

### 问题

终端 `input()` 输入时，多次退格删除了提示符 "你>"，执行命令报错：

```
[error] 错误: 'utf-8' codec can't encode character '\udce4' in position 68: surrogates not allowed
```

### 根因

两层问题叠加：

1. **`input()` 逐字节退格**："你" 是 UTF-8 三字节 `\xE4\xBD\xA0`，终端按字节退格残留孤立字节 `\xE4`，被 Python IO 层映射为 surrogate `\udce4`
2. **Surrogate 无法 UTF-8 编码**：Anthropic SDK 内部 `json.dumps()` → `.encode('utf-8')` 时，surrogate（U+D800–U+DFFF）是 UTF-16 专用，不在合法 UTF-8 中

### 错误的调试过程

- 误判为 `read_file` 读到了含 surrogate 的文件内容
- 花时间分析 `errors="replace"` 的行为，实际上它不会产生 surrogate
- 用户补充了"按退格删除了提示符"这个关键信息后才定位到真正根因

### 修复

| 层 | 修复 | 作用 |
|----|------|------|
| 显示层 | `input()` → `prompt_toolkit.prompt()` | 按字符退格，不会拆散多字节字符 |
| 传输层 | `_sanitize_text()` 清理 surrogate | encode 失败时替换为 `?`，快速路径零开销 |

### 教训

**先确认操作上下文，再假设代码 bug。**
- 用户描述了"按退格删除了提示符"——这是最关键的线索，但一开始被忽略了
- 编码问题的来源不只是文件内容，终端交互、子进程输出、文件系统路径都可能引入不可编码字符
- `prompt_toolkit` 是 Python TUI 的标准选择，应该从 Phase 1 就使用而非 `input()`
