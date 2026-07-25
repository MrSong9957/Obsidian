---
name: Message Store 语义边界——两个同源 bug 的调试故事
description: mi-code 从 ANSI renderer 迁移到 message-store + Ink <Static> 时,旧 renderer 的视觉控制指令被误当 message 数据,引发 thinking summary 消失与异常空白两个同源 bug 的定位与修复
type: debugging-story
updated: 2026-07-23
tags: [mi-code, tui, message-store, ink, static, renderer-migration, ansi, architecture-boundary]
---

# Message Store 语义边界——两个同源 bug 的调试故事

## 背景

mi-code 的 TUI 渲染架构经历了三次演进:
1. **Ink/React**(已废弃)
2. **自研差量渲染引擎**(ANSI 直接写 stdout)
3. **message-store + Ink `<Static>`**(当前架构,见 `wiki/mi-code-tui-experience.md` 对比)

第三次迁移的核心变化:**消息从"renderer 直接画的视觉内容"变成了"store 里的语义化 message 对象"**。`<Static>` 只负责把 finalized message 渲染到 scrollback。

这次迁移留了两颗"地雷"——它们的共同根因是:**旧 renderer 时代的"视觉控制指令"被误当成 message 数据**。本文记录这两个 bug 的定位过程。

## Bug 1:Thinking summary 消失

### 现象
模型 thinking 结束后,本应显示的 `Thought for Ns (ctrl+o to expand)` 摘要行肉眼不可见,屏幕只看到一片空白。

### 定位过程(三边界点插桩)

数据流有三层组件边界:
```
LLM event
  → index.ts (thinking_end emit)
  → block-pipeline.ts (thinking_end 分支 → print(summary, 'thinking_summary'))
  → pipeline-adapter.ts (printMessage → mapRole → appendLine)
  → messages-store.ts (appendLine 续接/新建)
  → InlineAppV2.tsx (<Static> items)
```

在三个边界点加旁路诊断日志(写独立 jsonl,不干扰 TUI),用 **node-pty 创真伪终端 + xterm headless 还原最终屏幕**验证,得到决定性 trace:

```
seq 74  pipeline.emit thinking_end
seq 75  store.removeStreamingThinking (移除流式 thinking)
seq 76  adapter.printMessage
        roleIn = "thinking_summary"
        roleMapped = "system"                    ← mapRole 把 role 改写
        lastUuid = "msg-3", lastRole = "system"  ← 末条是 system 空行消息
seq 77  store.appendLine
        decision = "append-to"                   ← 续接!不是新建
        targetUuid = "msg-3"                     ← 续接进空行消息
```

### 根因

`thinking_summary` 经 `mapRole` 降级为 `system`,再经 `appendLine` 的续接逻辑(`last.role===role && last.finalized && kind===undefined`),被**续接进 thinking_start 时 `openModelBlock()` 输出的 system 空行分隔符消息**。summary 沉到该空白块的第 2 行,`firstLinePreview` 为空,肉眼不可见。

### 修复(commit 158cc5b)

`pipeline-adapter.ts` 的 `printMessage` 识别 `role === 'thinking_summary'`,走 `appendMessage` 创建独立消息(新 uuid),绕开 `mapRole→'system'→appendLine` 续接路径。

```ts
if (role === 'thinking_summary') {
  this.store.getState().appendMessage('system', [line]);
  return;
}
```

## Bug 2:异常空白

### 现象
Bug 1 修复后,summary 可见,但 summary 与 assistant 回复之间出现大片多余空白。

### 定位(测试环境采集 messages 结构)

模拟 `thinking_start → thinking_delta → thinking_end → assistant_text` 完整生命周期,dump messages:

```
[msg-0] system  lines:[""]                         ← 空行①
[msg-1] system  lines:["Thought for 1s...", ""]    ← summary + 空行②
[msg-2] assistant lines:["● 8"]
```

空字符串行总数 2,全部来自 `printMessage('', 'system')`。两个空行源:
- `block-pipeline.ts:368` `openModelBlock()` —— 首个模型块强制空行
- `block-pipeline.ts:377` `ensureGap()` —— 块间分隔

调查 6 个调用点,**全部是纯布局占位(B 类),无一个是真实内容(A 类)**。

### 根因

这套 `ensureGap`/`openModelBlock` 是 **ANSI renderer 时代的遗留**——那时直接写 stdout,块间间距只能靠"写一个物理空行"。在 message-based store + `<Static>` 模型下,空字符串行被渲染成真实空白行(因为 `MessageLine` 对每行 `content + '\n'` 渲染,空 content 就是纯空行)。

### 修复(commit 3c07f3a)

在 `pipeline-adapter.ts` 的 `printMessage` 边界层拦截空 system 行:

```ts
if (text === '' && role === 'system') {
  return;
}
```

范围严格限定,不影响非空 system 文本、thinking_summary、其他 role。block-pipeline 的 `hasContent`/`assistantGapApplied` 状态机不动(拦截在 adapter 边界,block-pipeline 不感知空行是否进 store)。

## 共同根因(复盘价值点)

两个 bug 本质同源,都是**架构迁移边界的语义混淆**:

```
旧系统: Renderer 控制视觉(空行=视觉间距,role=渲染样式标记)
新系统: Message Store 控制语义(每条 message=真实内容,role=语义类型)
```

迁移过程中最大的风险是:**把旧 renderer 的"视觉控制指令"误当成 message 数据**。

- Bug 1:`thinking_summary` 这个 role 是旧 renderer 时代的"渲染样式标记",被 `mapRole` 降级后破坏了 message 生命周期
- Bug 2:空字符串是旧 renderer 时代的"视觉占位",污染了 message 数据

最终修复方向都是:**让进入 message store 的数据保持真实语义**。

## 关键方法论沉淀

### 1. 多组件系统调试:边界插桩法
当 bug 涉及多层组件(index → pipeline → adapter → store → render),**不要猜**,在每层边界加旁路日志,一次运行拿到完整数据流,再按决策树定位首次断点。

### 2. Agent 驱动 TTY 验证:node-pty + xterm headless
Agent 无法肉眼观察终端,但可以用:
- `node-pty`(cmd.exe 包装,ConPTY,`isTTY=true` 走真实代码路径)
- `@xterm/headless`(解析 ANSI 流还原最终屏幕缓冲)

这样 Agent 能自主完成"真实 TTY 验证",不依赖人工。

### 3. 最小修复原则:边界层兼容
旧/新架构共存时,**在新旧边界层做兼容处理**,而不是动旧的状态机或新的渲染层。本次两个修复都在 `pipeline-adapter`(旧输出模型 → 新 message 模型的边界),block-pipeline 的 gap 逻辑和 Ink 渲染层都不动。

## 相关文档
- [[mi-code-tui-experience]] —— TUI 架构前两阶段(Ink→自研引擎)
- [[mimocode-ui-rendering]] —— 旧渲染管道(diff 系统)
- [[streaming-architecture]] —— 流式架构与 EventBus

## 提交记录
```
3c07f3a fix(tui): drop empty system separator lines from message store
0f98aa8 test(tui): guard thinking summary standalone-message contract
158cc5b fix(tui): render thinking summary as standalone message
```

*分支:codex/auto-0025 | 定位日期:2026-07-23*
