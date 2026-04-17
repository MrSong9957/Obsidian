# QA 验证指南

> **截图路径硬性规则**：每次调用 `browser_take_screenshot` 时，`filename` 参数必须使用你收到的输出目录的完整绝对路径（不以 / 开头的路径都是错误的）。
> 禁止使用裸文件名（如 `X-fullpage.png`）。裸文件名会保存到 `.playwright-mcp/` 默认目录，评估者无法找到。
> 正确：`filename: "/absolute/path/to/outputs/X-fullpage.png"`
> 错误：`filename: "X-fullpage.png"` ← 会保存到 .playwright-mcp/

测试者和评估者的完整操作手册。包含前端验证、后端验证、截图规范和裁决标准。

## 目录

- [测试者操作指南](#测试者操作指南)
  - [前端 Playwright 验证](#前端-playwright-验证)
  - [后端 pytest 验证](#后端-pytest-验证)
- [截图与证据规范](#截图与证据规范)
- [测试报告格式](#测试报告格式)
- [评估者操作指南](#评估者操作指南)
  - [裁决标准](#裁决标准)
  - [截图审查要点](#截图审查要点)
  - [评估报告格式](#评估报告格式)

---

## 测试者操作指南

### 前端 Playwright 验证

按以下步骤逐一执行。每一步都必须记录结果。

#### Step 1：打开页面

```
browser_navigate → 打开 HTML 文件
browser_wait_for → text: "body"，确认页面已加载
```

如果页面无法打开（404、空白），直接判定所有标准 FAIL。

#### Step 2：截图取证

以下所有 `{输出目录}` 代表你在 prompt 中收到的输出目录完整绝对路径，使用时替换为实际路径。

```
browser_take_screenshot → fullPage: true, filename: "{输出目录}/{variant}-fullpage.png"
browser_take_screenshot → fullPage: false, filename: "{输出目录}/{variant}-viewport.png"
```

- 全页截图用于整体审查
- 首屏截图用于模拟用户第一印象
- 两个截图都必须保存，评估者会查看

#### Step 3：Console 错误检查

```
browser_console_messages → level: "error"
```

记录所有 error 级别消息。如果有 error，标记 AC-UI-3 为 FAIL。

#### Step 4：Accessibility 快照

```
browser_snapshot → 获取页面元素树
```

检查关键元素是否存在：
- 主内容区域
- 所有按钮和链接有可识别的文本
- 表单元素有 label
- 图片有 alt 文本

#### Step 5：逐条验收标准验证

对每条涉及交互的验收标准：

1. 用 `browser_snapshot` 找到目标元素
2. 用 `browser_click` / `browser_type` 执行操作
3. 用 `browser_take_screenshot` 截图记录操作结果
4. 判断 PASS 或 FAIL

截图命名规范：`{输出目录}/{variant}-ac{编号}-{动作}.png`
例如：`{输出目录}/X-ac02-upload.png`

#### Step 6：响应式验证

```
browser_resize → width: 375, height: 667
browser_take_screenshot → filename: "{输出目录}/{variant}-mobile.png"
browser_snapshot → 检查移动端布局
```

#### Step 7：恢复桌面尺寸

```
browser_resize → width: 1280, height: 720
```

### 后端 pytest 验证

#### Step 1：运行测试套件

```bash
cd {输出目录}
python -m pytest test_variant_X.py -v --tb=short > pytest_output_X.txt 2>&1
python -m pytest test_variant_Y.py -v --tb=short > pytest_output_Y.txt 2>&1
```

记录所有测试结果（通过数 / 失败数 / 错误数）。

**原始输出文件是硬性要求**——评估者会运行 pytest 做独立验证，需要对比测试者的报告与原始输出。不保存原始输出等同于证据缺失。

#### Step 2：边界输入测试

对每个核心函数，测试以下输入：
- `None` / 空字符串 / 空列表
- 超长字符串（1000+ 字符）
- 特殊字符（`<script>`、SQL 注入、路径遍历）
- 负数 / 零 / 超大数

#### Step 3：输出格式验证

运行核心功能，检查输出：
- 格式是否正确（JSON schema、CSV 列数、数据类型）
- 编码是否正确（中文不乱码）
- 空值处理是否合理

#### Step 4：逐条验收标准验证

对每条验收标准，设计对应的测试用例并运行：
- 编写最小测试代码
- 运行并记录结果
- 如果验收标准是"无 TODO"，则用 grep 搜索代码

---

## 截图与证据规范

### 文件命名

```
{输出目录}/{variant}-fullpage.png        # 全页截图
{输出目录}/{variant}-viewport.png        # 首屏截图
{输出目录}/{variant}-mobile.png          # 移动端截图
{输出目录}/{variant}-ac{编号}-{动作}.png  # 验收标准验证截图
```

其中 `{variant}` 是 X 或 Y，`{编号}` 是验收标准的两位数编号。

### 保存位置

所有截图保存在测试报告同目录下：
```
{输出目录}/
  test_report.md
  X-fullpage.png
  X-viewport.png
  X-ac02-upload.png
  ...
  Y-fullpage.png
  Y-viewport.png
  ...
```

### 截图要求

- **必须**使用 PNG 格式
- 全页截图必须包含完整内容（`fullPage: true`）
- 交互截图必须包含操作前后的状态对比（可以截两张）
- 移动端截图必须是 375x667 视口

---

## 测试报告格式

测试者必须产出以下格式的 `test_report.md`，加上原始输出文件：

```markdown
# 测试报告

**测试时间**：{ISO 时间}
**测试者**：测试子代理（盲验模式）

---

## 原始输出文件

| 变体 | 文件路径 |
|------|---------|
| X | pytest_output_X.txt（或截图目录） |
| Y | pytest_output_Y.txt（或截图目录） |

---

## 变体 X

### 验收标准检查

| 编号 | 标准名称 | 结果 | 证据 |
|------|---------|------|------|
| AC-1 | {名称} | PASS/FAIL | {截图文件名 + 分析 / pytest 输出引用（含行号）} |
| AC-2 | {名称} | PASS/FAIL | ... |
| ... | ... | ... | ... |

**证据引用规范**（每行必须可追溯到原始输出）：
- ✅ `AC-3: passed — pytest output shows test_report_format passed (see pytest_output_X.txt line 23)`
- ✅ `AC-1: PASS — 截图 X-viewport.png 显示完整文件列表，每行含文件名和大小`
- ❌ `AC-3: passed`（无证据引用，不可接受）

### Console 错误
{列出所有 error 级别的 console 消息，或"无 console 错误"}

### 截图清单
- {输出目录}/X-fullpage.png：全页截图
- {输出目录}/X-viewport.png：首屏截图
- {输出目录}/X-mobile.png：移动端截图
- {输出目录}/X-ac{N}-{action}.png：验收标准验证截图

### 额外发现
{测试过程中发现的、未在验收标准中定义的问题。如果没有，写"无额外发现"}

---

## 变体 Y

{同上格式}
```

**必须额外保存的原始输出文件**：
- `pytest_output_X.txt` — 变体 X 的完整 pytest 输出
- `pytest_output_Y.txt` — 变体 Y 的完整 pytest 输出

这些文件是评估者独立验证的基础。缺少原始输出文件将导致评估者无法独立验证，测试报告的可信度大幅降低。

---

## 评估者操作指南

### 裁决标准

**最严格原则**：评估者的职责是找问题，不是找理由放过。

判定规则：
1. 任何一条验收标准 FAIL → **CONTINUE**
2. Console 有 error 级别错误 → **CONTINUE**
3. 测试者报告了额外问题，且该问题影响用户体验 → **CONTINUE**
4. 以上都不满足，所有验收标准 PASS → **PASS**

**绝对不能**：
- 因为"大部分通过了"就给 PASS
- 跳过查看截图直接根据文字报告裁决
- 自己猜测实现的质量，只基于测试者的证据
- 只抄测试者报告的结论，不做独立验证

### 独立验证（不可跳过）

评估者必须自己动手验证，不能只看测试者的报告：

**前端评估**：
- 用 Read 工具查看每张截图（已在截图审查要点中详述）
- 对比截图内容与测试者判定，不一致时以自己的判断为准

**后端评估**：
- 运行 `pytest {输出目录}/test_variant_X.py -v` 和 `test_variant_Y.py -v`
- 对比实际通过数与测试者报告的通过数
- 读取 `pytest_output_X.txt` 和 `pytest_output_Y.txt`，检查测试者是否遗漏了失败项
- 如果发现不一致，降级该变体为 CONTINUE

**抽查验证**：
- 随机选取 2-3 条验收标准
- 自己执行该标准的验证方法（运行代码、检查输出、查看文件等）
- 在评估报告中记录抽查过程和发现

### 截图审查要点

评估者必须用 Read 工具查看每张截图，审查以下要点：

1. **首屏截图**（viewport）：
   - 布局是否正常（无错位、重叠、溢出）
   - 核心功能是否可见
   - 视觉质量是否可接受（配色、间距、字体）

2. **全页截图**（fullpage）：
   - 内容是否完整（没有截断或缺失）
   - 底部内容是否正常

3. **交互截图**：
   - 操作后的状态变化是否符合预期
   - 错误提示是否友好

4. **移动端截图**：
   - 布局是否适应小屏幕
   - 文字是否可读
   - 按钮是否可点击

5. **后端测试输出**：
   - 所有测试是否真的通过
   - 是否有 warning 级别的问题被忽略

### 问题分类

CONTINUE 时必须分类，决定循环回哪个角色：

| 问题类型 | 循环回 | 示例 |
|---------|--------|------|
| 代码 Bug | 实现者 | 功能不工作、console 错误、样式错乱、测试失败 |
| 需求理解错误 | 研究者 | 实现方向与用户意图不符、关键功能遗漏 |
| 默认 | 实现者 | 任何不明确分类的问题 |

### 评估报告格式

```markdown
# 评估报告

**评估时间**：{ISO 时间}
**评估者**：评估子代理

---

## 变体 X

**裁决**：PASS / CONTINUE
**问题类型**（仅 CONTINUE 时）：代码 Bug / 需求理解错误

### 验收标准审查

{逐条审查，结合测试者报告 + 截图分析}

#### AC-1：{标准名称}
- 测试者报告：PASS
- 截图确认：{查看截图后的分析}
- 评估结论：同意/不同意（如不同意，说明原因）

#### AC-2：{标准名称}
- ...

### 截图审查发现
{通过查看截图发现的、测试者报告可能遗漏的问题}

### 问题清单（仅 CONTINUE 时）
1. {问题描述，引用具体截图和验收标准编号}
   - **影响**：{对用户体验的影响}
   - **建议修复方式**：{具体建议}

---

## 变体 Y

{同上格式}

---

## 总体建议

{如果两个都 PASS：推荐哪个变体及理由，考虑功能完整度、用户体验、代码质量}
{如果有 CONTINUE：列出需要修复的具体问题，明确循环回哪个角色}
```
