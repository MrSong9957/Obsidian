---
name: Playwright CLI 指南
description: CLI 工具让 Claude Code 高效完成 UI 测试 — 用 Playwright 进行 UI 自动化测试时参考
type: summary
sources:
  - "[[sources/claude-code-system-prompts/README]]"
updated: 2026-04-05
tags: [testing, playwright]
status: deprecated
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# Playwright CLI 快速指南

> 内容已迁移至 [[wiki/frontend.md|frontend]]。

> 轻量级 CLI 工具，让 Claude Code 高效完成 UI 测试

**相关文档**：[[Everything-Claude-Code.md|ECC 技能集]] | [[ui-design.md|UI 设计原则]]

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
