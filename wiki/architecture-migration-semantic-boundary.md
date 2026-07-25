---
name: 架构迁移语义边界准则
description: 从"renderer 控制视觉"迁移到"message store 控制语义"时,必须区分视觉控制指令与真实数据,否则旧时代的视觉指令会污染新时代的语义数据
type: concept
updated: 2026-07-23
tags: [architecture, migration, semantic-boundary, tui, message-store, renderer, anti-pattern]
source: [[sources/projects/mi-code/2026-07-23-message-store-semantic-boundary]]
---

# 架构迁移语义边界准则

## 核心准则

> **当系统从"渲染层控制视觉"迁移到"数据层控制语义"时,必须严格区分:哪些是旧时代的视觉控制指令,哪些是新时代的真实数据。视觉控制指令绝不能进入语义数据层。**

## 适用场景

任何"渲染模型"变更的架构迁移,典型如:
- ANSI 直接写 stdout → React/Ink 组件树
- 自定义 renderer diff 系统 → message store + 声明式渲染
- 命令式 UI(手动定位/画空行)→ 响应式 UI(数据驱动)

## 两种模型的根本差异

### 旧模型:Renderer 控制视觉

```
应用逻辑 → renderer 写屏指令(包括间距、空行、样式标记)→ stdout
```

- **空字符串 = 视觉间距**(物理空行)
- **role/type = 渲染样式标记**(决定怎么画)
- 间距、位置、视觉占位都由 renderer 直接控制

### 新模型:Message Store 控制语义

```
应用逻辑 → message 对象(语义化)→ store → renderer 只展示
```

- **每条 message = 真实内容**(有业务语义)
- **空字符串 = 真实空内容**(会被渲染成空白,污染视觉)
- **role/type = 语义类型**(决定数据归属,不是渲染样式)
- 间距、位置由渲染层声明式处理(如 CSS margin、组件 layout)

## 迁移期的核心陷阱

**把旧 renderer 的视觉控制指令误当成 message 数据**,这是迁移期最大的风险。两类典型表现:

### 陷阱 1:视觉间距被当数据

旧时代 `printMessage('', 'system')` 产空行控间距 → 新时代空字符串进入 store → 渲染成真实空白 → 异常大片留白。

**判别:** 这个数据有业务语义吗?空字符串"什么都没说",它只是想让 renderer 留个空 —— 那它是视觉指令,不该进 store。

### 陷阱 2:样式标记被当数据类型

旧时代 role='thinking_summary' 只是告诉 renderer "用灰色画这行" → 新时代 role 经 mapRole 映射,可能丢失或合并 → 破坏 message 生命周期(被错误续接/合并)。

**判别:** 这个 role/标记是给 renderer 看的样式提示,还是数据的真实语义分类?如果是样式,它不该决定 message 的归属和合并行为。

## 修复策略:边界层兼容

新旧架构共存时,**在新旧边界层做兼容处理**,而不是动旧的内部逻辑或新的渲染层。

```
[旧系统内部]                [边界层兼容]              [新系统内部]
block-pipeline          pipeline-adapter          message-store + <Static>
(产视觉指令)    →    (识别并翻译/拦截)     →    (只接收语义数据)
```

原则:
1. **旧系统内部不动** —— 它的状态机、gap 逻辑是自洽的,改它会引入新 bug
2. **新系统渲染层不动** —— 它只该处理语义数据,不该知道"空行是间距"这种视觉概念
3. **边界层负责翻译** —— 识别旧视觉指令,拦截或转换为合法的语义数据

## 判别清单(遇到疑似 bug 时)

当新架构出现"内容消失"或"异常空白"时,检查:

- [ ] 数据是否含空字符串行?(可能来自旧间距指令)
- [ ] role/type 是否被映射函数降级?(可能丢失语义)
- [ ] 数据是否被"续接/合并"进错误对象?(旧合并规则碰上新数据)
- [ ] 渲染层是否对某些数据做特殊处理?(把视觉指令当数据)

如果命中,根因大概率是**视觉控制指令混入了语义数据层**。

## 具体案例

详见 [[sources/projects/mi-code/2026-07-23-message-store-semantic-boundary]] —— mi-code 从 ANSI renderer 迁移到 message-store + Ink `<Static>` 时,两个同源 bug(thinking summary 消失 + 异常空白)的完整定位过程。

## 相关
- [[mi-code-tui-experience]] —— TUI 渲染架构演进
- [[terminal-inline-renderer-principles]] —— 终端渲染不变原则
