---
name: 前端与 UI（完整资料）
description: UI设计原则/终端UI测试/Playwright自动化/技术栈策略的完整资料，含代码示例和详细教程
updated: 2026-04-17
tags: [frontend, ui, testing, playwright, tech-stack]
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# 前端与 UI（完整资料）

> 本文档是从 wiki/frontend.md 提取的完整原始内容，包含代码示例、安装步骤、调试过程和完整教程。

## 终端 UI 自动化测试深入研究

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

## 双智能体循环架构（原始研究视角）

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

**实现要点：** Generator 编写代码 + 创建 .tape；VHS 执行并生成 PNG；Evaluator 多模态分析截图并生成反馈；持续迭代直到通过或达到最大次数。

### 关键发现

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

## Playwright CLI 快速指南

> 轻量级 CLI 工具，让 Claude Code 高效完成 UI 测试

**相关文档**：**相关文档**：[[ui-design.md|UI 设计原则]]

## 安装

```bash
# 1. 安装 CLI
npm install -g @playwright/cli@latest

# 2. 安装浏览器
npx playwright install chromium

# 3. 安装 Skills（让 Claude Code 自动发现）
playwright-cli install --skills

# 4. 验证
playwright-cli --help
```

## 核心概念

- **Session（会话）**：浏览器实例，默认内存中
- **元素引用**：操作元素的标识（如 `e1`, `e2`）
- **快照**：每次操作后自动保存的页面状态
- **有头模式**：`--headed` 显示浏览器窗口

---

## 在 Claude Code 中使用

安装 `playwright-cli install --skills` 后，直接告诉 Claude Code 你想测试什么。

---

## 实用测试示例

### 示例 1：验证页面加载正常

```
你：打开 google.com，确认页面加载成功并截图
```

Claude Code：
1. `playwright-cli open https://google.com --headed`
2. `playwright-cli screenshot --filename=google.png`

**用途**：冒烟测试，确认页面能正常打开

---

### 示例 2：验证登录功能

```
你：测试 GitHub 登录，用户名 testuser，密码 test123，验证登录成功
```

Claude Code：
1. `playwright-cli open https://github.com/login --headed`
2. `playwright-cli snapshot`（查看表单元素）
3. `playwright-cli fill e1 "testuser"`
4. `playwright-cli fill e2 "test123"`
5. `playwright-cli click e3`（点击登录按钮）
6. `playwright-cli screenshot`（验证结果）

**用途**：功能测试，验证登录流程

---

### 示例 3：验证搜索功能

```
你：在百度搜索 "Claude Code"，验证搜索结果页面
```

Claude Code：
1. `playwright-cli open https://www.baidu.com --headed`
2. `playwright-cli snapshot`
3. `playwright-cli fill e1 "Claude Code"`
4. `playwright-cli press Enter`
5. `playwright-cli screenshot`（验证搜索结果）

**用途**：搜索功能测试

---

### 示例 4：验证 UI 响应式（手机尺寸）

```
你：用 iPhone 尺寸打开我的网站，验证移动端布局是否正常
```

Claude Code：
1. `playwright-cli open https://example.com --headed`
2. `playwright-cli resize 375 812`
3. `playwright-cli screenshot --filename=mobile.png`

**用途**：响应式布局测试

---

### 示例 5：验证弹窗交互

```
你：打开一个页面，点击"注册"按钮，验证弹窗出现
```

Claude Code：
1. `playwright-cli open <url> --headed`
2. `playwright-cli snapshot`
3. `playwright-cli click e1`（点击注册按钮）
4. `playwright-cli screenshot`（验证弹窗）
5. `playwright-cli click e2`（关闭弹窗）

**用途**：弹窗、模态框交互测试

---

### 示例 6：验证页面跳转

```
你：点击首页的"关于我们"链接，验证跳转到正确页面
```

Claude Code：
1. `playwright-cli open https://example.com --headed`
2. `playwright-cli snapshot`
3. `playwright-cli click e1`（点击"关于我们"）
4. `playwright-cli screenshot`（验证跳转结果）
5. 可选：`playwright-cli goto <expected-url>` 验证 URL

**用途**：链接跳转、导航测试

---

### 示例 7：验证表单验证

```
你：在注册页面填写无效邮箱，点击提交，验证错误提示
```

Claude Code：
1. `playwright-cli open <register-url> --headed`
2. `playwright-cli fill e1 "invalid-email"`
3. `playwright-cli click e2`（提交）
4. `playwright-cli screenshot`（验证错误提示）

**用途**：表单验证测试

---

### 示例 8：验证下拉选择

```
你：在筛选页面选择"价格从低到高"，验证筛选结果
```

Claude Code：
1. `playwright-cli open <url> --headed`
2. `playwright-cli snapshot`
3. `playwright-cli select e1 "price-asc"`
4. `playwright-cli screenshot`

**用途**：筛选、排序功能测试

---

## 高级测试示例

### 示例 9：Mock API 响应（网络拦截）

```
你：打开商品列表页，但把 /api/products 这个接口 mock 成返回测试数据
```

Claude Code：
1. `playwright-cli route "*/api/products" --status=200 --body='[{"id":1,"name":"Test Product","price":99}]'`
2. `playwright-cli open https://example.com/products --headed`
3. `playwright-cli screenshot`（验证 Mock 数据展示）

**用途**：
- 隔离外部依赖
- 测试不同 API 响应
- 模拟错误状态（500、404）
- 不等待后端完成时提前测试

---

### 示例 10：保存和复用登录状态

```
你：先手动登录 GitHub，然后保存登录状态，下次测试直接用这个状态
```

Claude Code：
1. `playwright-cli -s=github open https://github.com --headed`
2. 用户手动登录...
3. `playwright-cli state-save github-auth.json`

下次测试：
```
你：用之前保存的 GitHub 登录状态打开首页
```
Claude Code：
1. `playwright-cli -s=test open https://github.com --headed`
2. `playwright-cli state-load github-auth.json`
3. `playwright-cli goto https://github.com`
4. `playwright-cli screenshot`（已登录）

**用途**：避免每次测试重复登录，加快测试速度

---

### 示例 11：多用户同时测试

```
你：用两个浏览器同时测试：一个模拟管理员，一个模拟普通用户
```

Claude Code：
1. 会话 A（管理员）：`playwright-cli -s=admin open https://example.com/admin --headed`
2. 会话 B（用户）：`playwright-cli -s=user open https://example.com --headed`
3. 管理员操作：`playwright-cli -s=admin click e1`（删除内容）
4. 用户验证：`playwright-cli -s=user goto https://example.com`
5. `playwright-cli -s=user screenshot`（确认普通用户看到更新）

**用途**：多用户交互测试、并发测试

---

### 示例 12：监控网络请求

```
你：打开页面，列出所有网络请求，找出哪些请求失败了
```

Claude Code：
1. `playwright-cli open https://example.com --headed`
2. `playwright-cli network`（查看所有请求）
3. `playwright-cli console`（查看控制台日志）

**用途**：调试网络问题、检查 API 调用

---

### 示例 13：录制测试过程

```
你：录制整个测试过程的视频，用于复现问题
```

Claude Code：
1. `playwright-cli open https://example.com --headed`
2. `playwright-cli video-start`
3. 执行测试操作...
4. `playwright-cli video-stop test-recording.mp4`

或者轨迹录制：
1. `playwright-cli tracing-start`
2. 执行测试...
3. `playwright-cli tracing-stop`
4. 用 Playwright Trace Viewer 打开查看

**用途**：Bug 复现、测试报告

---

### 示例 14：模拟网络 Conditions

```
你：模拟慢速网络，测试页面在弱网下的表现
```

Claude Code 会使用配置文件或 `--config` 指定网络条件：
```bash
playwright-cli open https://example.com --config='{"network":{"slow":true}}'
```

或者通过 `playwright-cli route` 模拟延迟：
```bash
playwright-cli route "*/**" --delay=2000  # 所有请求延迟 2 秒
```

**用途**：弱网测试、性能测试

---

### 示例 15：验证 API 错误处理

```
你：打开页面，模拟 API 返回 500 错误，验证页面错误提示
```

Claude Code：
1. `playwright-cli route "*/api/*" --status=500`
2. `playwright-cli open https://example.com --headed`
3. `playwright-cli screenshot`（验证错误页面）

**用途**：异常场景测试、错误处理测试

---

## 常用操作速查

| 你想说 | Claude Code 执行 |
|--------|------------------|
| "打开 xxx，有头模式" | `playwright-cli open <url> --headed` |
| "截图" | `playwright-cli screenshot` |
| "先获取快照" | `playwright-cli snapshot` |
| "填入 xxx" | `playwright-cli fill e1 "xxx"` |
| "点击那个按钮" | `playwright-cli click e1` |
| "选择 xxx" | `playwright-cli select e1 "value"` |
| "后退/前进" | `playwright-cli go-back/go-forward` |
| "用手机尺寸" | `playwright-cli resize 375 812` |
| "Mock 这个 API" | `playwright-cli route <pattern> --status=200 --body='{}'` |
| "保存登录状态" | `playwright-cli state-save auth.json` |
| "加载登录状态" | `playwright-cli state-load auth.json` |
| "查看网络请求" | `playwright-cli network` |
| "查看控制台" | `playwright-cli console` |
| "开始录制视频" | `playwright-cli video-start` |

---

## 常见问题

| 问题 | 解决 |
|------|------|
| 找不到元素 | 说"先获取快照看看有哪些元素" |
| 浏览器打不开 | 运行 `npx playwright install chromium` |
| 会话消失 | 说"用持久化会话重新打开" |

---

## 配置文件

`.playwright/cli.config.json`：

```json
{
  "browser": { "browserName": "chromium" },
  "outputDir": "./screenshots"
}
```
