---
name: 终端 UI 自动化测试
description: VHS + AI 评估器方案，融入四角色框架 — 实施终端 UI 测试方案时参考
type: summary
sources:
  - "[[sources/learn-claude-code]]"
updated: 2026-04-05
tags: [testing, automation]
---

# 终端 UI 自动化测试方案

**触发词：** "终端测试"、"TUI 测试"、"VHS"、"终端截图"

---

## 核心方案：VHS + AI 评估器

### VHS 工具

**用途：** 终端录制 + PNG 截图，让 AI 可以"看见"终端 UI

**安装：** `brew install vhs`（需要 ttyd + ffmpeg）

**关键命令：**
```tape
Screenshot ui.png    # 截图供 AI 分析
Wait /pattern/       # 等待特定输出
Type "command"       # 模拟输入
```

### 容器环境关键配置

**必须设置：** `VHS_NO_SANDBOX=true`

VHS 默认使用沙箱模式，在 Docker/CI 容器中会失败。设置此环境变量可绕过沙箱限制。

### 融入四角色框架

VHS 不再是独立的双智能体循环，而是嵌入四角色框架的**测试者**角色：

```
研究者(验收标准) → 实现者(代码) → 测试者(.tape + 截图 + 报告) → 评估者(多模态判断)
                                       ↑                              │
                                       └──────── CONTINUE ────────────┘
```

**信息流：**
1. **研究者** → 产出验收标准（功能点、交互路径、UI 预期、边界情况）
2. **测试者** → 根据验收标准写 .tape 脚本、跑 build、截图、填报告
3. **评估者** → 阅读报告 + 查看截图（多模态），输出 PASS/CONTINUE

### 产出物目录

```
docs/test-artifacts/
├── templates/                   # 模板
│   ├── acceptance-criteria.md   # 研究者产出
│   └── report.md                # 测试者产出
├── screenshots/                 # VHS 截图
├── test-{N}.tape                # 每轮 .tape 脚本
└── report-{N}.md                # 每轮测试报告
```

### 代码实现原型（agent-loop.ts 核心逻辑）

```typescript
// 生成器/评估器循环 — 四角色框架中测试者+评估者的核心

interface EvaluationResult {
  passed: boolean;
  issues: string[];
  suggestions: string[];
}

async function agentLoop(task: string, criteria: string, maxIterations = 5) {
  let feedback: string | undefined;
  for (let i = 0; i < maxIterations; i++) {
    // 1. Generator 编写代码 + .tape 脚本
    const { codeChanges, tapeContent } = await generateCode(task, feedback);
    applyCodeChanges(codeChanges);
    createTapeFile(tapeContent);

    // 2. VHS 运行测试 → 生成 PNG 截图
    const screenshots = runVHSTest('test.tape');

    // 3. Evaluator 多模态分析截图
    const result = await evaluateScreenshots(screenshots, criteria);
    if (result.passed) return true;

    feedback = `问题：${result.issues.join('\n')}\n建议：${result.suggestions.join('\n')}`;
  }
  return false;
}

// Evaluator 核心调用：将截图以 base64 发送给多模态模型
async function evaluateScreenshots(screenshots: string[], criteria: string) {
  const images = screenshots.map(p => ({
    type: 'image', source: {
      type: 'base64', media_type: 'image/png',
      data: fs.readFileSync(p, 'base64')
    }
  }));
  const response = await client.messages.create({
    model: 'claude-sonnet-4-6-20250514', max_tokens: 2048,
    messages: [{ role: 'user', content: [
      { type: 'text', text: `评估这些终端 UI 截图。评判标准：${criteria}` },
      ...images
    ]}]
  });
  return parseEvaluationResponse(response.content[0].text);
}
```

### 实际 .tape 示例（ai-terminal chat-flow）

```tape
# tests/e2e/chat-flow.tape
Set Width 1200
Set Height 800
Set FontSize 14
Set Theme "Catppuccin Mocha"

Output chat-flow.gif

# 启动应用
Hide
Type "cd /home/app/project/ai-terminal && npm run build"
Enter
Sleep 2s
Show

Type "npm start"
Enter
Wait />>>/
Sleep 1s
Screenshot screenshots/01-ready.png

# 发送消息并验证响应
Type "Hello, how are you?"
Enter
Wait /AI:/
Sleep 3s
Screenshot screenshots/02-response.png

Type "Tell me a joke"
Enter
Wait /joke/i
Sleep 2s
Screenshot screenshots/03-joke.png
```

### 详细评估维度

| 维度 | 检查项 | 提示词示例 |
|------|--------|-----------|
| 布局 | 元素位置、对齐、间距 | "检查输入框是否在底部，消息区域是否在上方" |
| 颜色 | 对比度、可读性、一致性 | "检查文字和背景的对比度是否足够" |
| 文本 | 溢出、截断、重叠 | "检查是否有文字被截断或重叠" |
| 交互 | 焦点、光标、状态 | "检查光标位置是否正确" |
| Bug | 闪烁、残留、异常 | "检查是否有 ANSI 转义序列残留" |

---

## 三层测试金字塔

```
         ┌─────────┐
         │ E2E     │  ← VHS + Evaluator（5%）
       ┌─┴─────────┴─┐
       │ Integration  │  ← Hook 集成测试（15%）
     ┌─┴──────────────┴─┐
     │    Unit Tests    │  ← ink-testing-library（80%）
     └──────────────────┘
```

---

## Sources

- [VHS](https://github.com/charmbracelet/vhs) - charmbracelet 终端录制工具
- [Endless Terminals](https://arxiv.org/abs/2601.16443) - Stanford + Microsoft Research
- [node-pty](https://github.com/microsoft/node-pty) - Microsoft PTY 库
- [ink-testing-library](https://github.com/vadimdemedes/ink-testing-library) - Ink 组件测试
