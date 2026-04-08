---
name: 调试教训
description: 调试方法论 — 先观察再假设、日志先行 — 遇到调试问题或总结调试经验时查阅
type: concept
updated: 2026-04-05
tags: [debugging, experience]
---

# 调试教训

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
