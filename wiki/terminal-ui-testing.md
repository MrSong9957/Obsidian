---
name: 终端 UI 自动化测试研究
description: VHS + AI 评估器方案深入研究和多智能体协作 — 研究或实施终端 UI 测试方案时参考
type: summary
sources:
  - "[[sources/learn-claude-code]]"
updated: 2026-04-05
tags: [testing, automation]
---

# 终端 UI 自动化测试深入研究

> 相关文档：[[terminal-ui-testing.md|终端 UI 测试精要]] · [[multi-agent-framework.md|多智能体框架]] · [[ui-design.md|UI 设计]]

## Context

用户需求：
1. 自动化测试终端 UI（包括 UI 也实现自动化）
2. 能拍快照或使用无头模式，让 Claude Code 真正看到自己做出来的效果
3. 一个评估智能体负责看 UI 和找问题，一个生成智能体负责生成和根据评估修改

## 研究结论

**完全可行。** 推荐使用 **VHS + 双智能体循环** 方案。

---

## 推荐方案：VHS 可视化快照测试

### 为什么选择 VHS

| 特性 | VHS | 其他方案 |
|------|-----|---------|
| 可视化截图 | PNG 截图供 AI 分析 | node-pty 无视觉输出 |
| 声明式脚本 | .tape 文件，类似 Playwright | expect 脚本复杂 |
| CI 集成 | GitHub Action | 需要自己实现 |
| Golden File | .txt/.ascii diff 测试 | 需要额外工具 |

### VHS 核心能力

```tape
# 截图供 AI 分析
Screenshot ui-state.png

# 等待特定输出
Wait />>>/

# 隐藏 setup 命令
Hide
Type "npm install"
Show
```

---

## 双智能体循环架构

```
Generator Agent          VHS Test Runner          Evaluator Agent
     |                        |                         |
     +-- 编写/修改代码 ------->|                         |
     +-- 创建 .tape 脚本 ----->|                         |
     |                        +-- 执行 .tape             |
     |                        +-- 生成 PNG 截图 -------->|
     |                        |                         +-- 视觉分析
     |                        |                         +-- 发现问题
     |                        |                         +-- 生成反馈
     |<-------------------------------------------------+
     |                    反馈循环                        |
```

### 实现要点

1. **Generator Agent**：编写代码 + 创建 .tape 测试脚本
2. **VHS**：执行脚本，生成 PNG 截图
3. **Evaluator Agent**：多模态分析截图，发现 UI 问题，生成反馈
4. **循环**：持续迭代直到通过或达到最大次数

---

## VHS 安装

```bash
# macOS
brew install vhs

# Linux (需要 ttyd + ffmpeg)
# 参考: https://github.com/charmbracelet/vhs
```

### .tape 文件语法

```tape
# 设置终端尺寸
Set Width 1200
Set Height 600
Set FontSize 14

# 输出文件
Output demo.gif
Screenshot screenshot.png

# 执行命令
Type "npm start"
Enter

# 等待特定输出出现
Wait />>>/  # 等待提示符
Sleep 2s    # 等待 UI 渲染

# 截图
Screenshot ui-state-1.png

# 输入测试
Type "hello world"
Enter
Wait /AI:/
Sleep 3s
Screenshot response.png
```

---

## 代码实现原型

```typescript
// agent-loop.ts - 生成器/评估器循环核心

import Anthropic from '@anthropic-ai/sdk';
import { execSync } from 'child_process';
import * as fs from 'fs';

const client = new Anthropic();

interface EvaluationResult {
  passed: boolean;
  issues: string[];
  suggestions: string[];
}

// Generator Agent: 编写/修改代码
async function generateCode(task: string, feedback?: string): Promise<string> {
  const systemPrompt = `你是代码生成器。你的任务是：
${feedback ? `上次评估反馈：${feedback}` : ''}
当前任务：${task}

请编写/修改代码，然后创建 .tape 测试脚本。`;

  const response = await client.messages.create({
    model: 'claude-sonnet-4-6-20250514',
    max_tokens: 4096,
    system: systemPrompt,
    messages: [{ role: 'user', content: task }]
  });

  return extractCodeFromResponse(response);
}

// VHS Test Runner: 执行 .tape 并生成截图
function runVHSTest(tapePath: string): string[] {
  execSync(`vhs ${tapePath}`, { stdio: 'inherit' });

  const screenshots = fs.readdirSync('.')
    .filter(f => f.endsWith('.png'))
    .sort();

  return screenshots;
}

// Evaluator Agent: 分析截图，判断 UI 是否正确
async function evaluateScreenshots(
  screenshots: string[],
  criteria: string
): Promise<EvaluationResult> {
  const imageContents = screenshots.map(path => ({
    type: 'image',
    source: {
      type: 'base64',
      media_type: 'image/png',
      data: fs.readFileSync(path, 'base64')
    }
  }));

  const response = await client.messages.create({
    model: 'claude-sonnet-4-6-20250514',
    max_tokens: 2048,
    messages: [{
      role: 'user',
      content: [
        { type: 'text', text: `评估这些终端 UI 截图。评判标准：${criteria}` },
        ...imageContents
      ]
    }]
  });

  return parseEvaluationResponse(response.content[0].text);
}

// 主循环
async function agentLoop(
  task: string,
  criteria: string,
  maxIterations: number = 5
): Promise<boolean> {
  let iteration = 0;
  let feedback: string | undefined;

  while (iteration < maxIterations) {
    console.log(`\n=== Iteration ${iteration + 1} ===`);

    // 1. Generator 编写代码
    console.log('Generator: 编写代码...');
    const { codeChanges, tapeContent } = await generateCode(task, feedback);
    applyCodeChanges(codeChanges);
    createTapeFile(tapeContent);

    // 2. VHS 运行测试
    console.log('VHS: 运行测试...');
    const screenshots = runVHSTest('test.tape');

    // 3. Evaluator 分析截图
    console.log('Evaluator: 分析截图...');
    const result = await evaluateScreenshots(screenshots, criteria);

    if (result.passed) {
      console.log('测试通过！');
      return true;
    }

    // 4. 生成反馈
    feedback = `问题：${result.issues.join('\n')}\n建议：${result.suggestions.join('\n')}`;
    console.log(`发现问题：\n${feedback}`);

    iteration++;
  }

  console.log('达到最大迭代次数');
  return false;
}
```

---

## 评估智能体的分析维度

| 维度 | 检查项 | 提示词示例 |
|------|--------|-----------|
| **布局** | 元素位置、对齐、间距 | "检查输入框是否在底部，消息区域是否在上方" |
| **颜色** | 对比度、可读性、一致性 | "检查文字和背景的对比度是否足够" |
| **文本** | 溢出、截断、重叠 | "检查是否有文字被截断或重叠" |
| **交互** | 焦点、光标、状态 | "检查光标位置是否正确" |
| **Bug** | 闪烁、残留、异常 | "检查是否有 ANSI 转义序列残留" |

---

## 实际 .tape 示例（ai-terminal）

```tape
# tests/e2e/chat-flow.tape

# 设置
Set Width 1200
Set Height 800
Set FontSize 14
Set Theme "Catppuccin Mocha"

# 输出
Output chat-flow.gif
Screenshot screenshots/01-initial.png

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
Screenshot screenshots/02-ready.png

# 发送消息
Type "Hello, how are you?"
Enter
Wait /AI:/
Sleep 3s
Screenshot screenshots/03-response.png

# 检查消息显示
Type "Tell me a joke"
Enter
Wait /joke/i
Sleep 2s
Screenshot screenshots/04-joke.png
```

---

## 关键发现

1. **简单的交互循环 + 二元奖励**就能训练出有效 Agent
2. **环境规模比算法复杂度更重要**
3. **循环行为是主要失败模式**（39%）
4. **VHS 的 Screenshot 功能让 AI 可以"看见"终端 UI**

---

## Sources

- [VHS GitHub](https://github.com/charmbracelet/vhs) - Charmbracelet 终端录制工具
- [Endless Terminals Paper](https://arxiv.org/abs/2601.16443) - Stanford + Microsoft Research
- [OpenDev Paper](https://arxiv.org/abs/2603.05344) - 终端编码 Agent 架构
- [node-pty](https://github.com/microsoft/node-pty) - 微软 PTY 库
- [ink-testing-library](https://github.com/vadimdemedes/ink-testing-library) - Ink 测试工具
