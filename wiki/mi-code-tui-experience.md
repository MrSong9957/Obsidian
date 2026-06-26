# MiCode TUI 开发经验汇总

> 来源：mi-code 项目实战经验，从 Ink/React 迁移到自研差量渲染引擎的完整记录。
> 创建日期：2026-06-26

---

## 一、架构演进：Ink → 自研渲染引擎

### 第一阶段：Ink/React（已废弃）

**依赖**：ink v7 + ink-text-input + react = 42 个运行时包

**代码模式**：
```tsx
import { render, Box, Text } from 'ink';
import TextInput from 'ink-text-input';
// React 组件模型，useState/useCallback/React.memo
```

**放弃原因**：
1. 启动慢（42 包加载 + tsx 类型检查）
2. 渲染卡（React reconciler 全组件树 diff）
3. 优化无效（memo/useCallback 只缓解不根治）
4. ESM 强制（ink v7 要求 `"type": "module"`）

---

### 第二阶段：自研差量渲染引擎

**依赖**：0 个运行时包

**模块划分**（8 文件，~620 行）：
```
src/renderer/
├── cell.ts           — Cell 接口 + 属性常量
├── pool.ts           — CharPool + StylePool（整数 ID 复用）
├── screen-buffer.ts  — 双缓冲 + Damage 区域 + diff
├── optimizer.ts      — 补丁合并优化
├── writer.ts         — ANSI 输出
├── colors.ts         — 颜色名→ANSI 代码
├── renderer.ts       — RenderNode→ScreenBuffer 渲染
└── index.ts          — 公共导出
```

---

### 核心数据流

```
RenderNode 树
    ↓ renderNode()
ScreenBuffer (cells[])
    ↓ diffBuffers(prev, curr)
Patch[] (cursorTo/style/write)
    ↓ optimize()
Patch[] (合并后)
    ↓ writePatches()
stdout.write(buf)  // 单次系统调用
```

---

### 关键设计决策

#### 1. Cell 用整数 ID 而非字符串

```typescript
interface Cell {
  char: number;   // CharPool 索引
  fg: number;     // StylePool 索引
  bg: number;
  attrs: number;  // 位标志：bold=1, dim=2, italic=4, underline=8
}
```

**Why**：帧间比较用 `===`，比字符串比较快 10x+。

#### 2. CharPool 的 ASCII 快速路径

```typescript
class CharPool {
  private ascii = new Int32Array(128).fill(-1);  // O(1) 查找
  private map = new Map<string, number>();        // 非 ASCII 兜底
}
```

**Why**：ASCII 字符占比 90%+，用数组直接索引避免 Map 开销。

#### 3. Damage 区域追踪

```typescript
interface ScreenBuffer {
  damage: DamageRegion | null;  // setCell 时自动更新
}

// diffBuffers 只遍历 damage 区域
```

**Why**：全屏 diff O(width*height) 太慢，Damage 追踪只 diff 变化区域。

**踩坑**：最初没有 Damage 追踪，diff 引擎有 bug 导致文字显示错误，不得不暂时禁用改为全屏重绘（commit 0779144）。后来添加 Damage 追踪才修复（commit d8c6eff）。

#### 4. 补丁优化器

```typescript
function optimize(patches: Patch[]): Patch[] {
  // 合并连续 write → 单次 write
  // 合并连续 style → 只保留最后一个
  // 跳过空 write
}
```

**Why**：减少 `process.stdout.write()` 调用次数。

---

### 布局系统

#### RenderNode 类型

```typescript
interface RenderNode {
  type: 'box' | 'text';
  children?: RenderNode[];
  text?: string;
  props: {
    color?: string;
    bold?: boolean;
    dim?: boolean;
    flexDirection?: 'row' | 'column';  // 默认 column
    justifyContent?: 'flex-start' | 'flex-end' | 'space-between';
    paddingX?: number;
    paddingY?: number;
    marginY?: number;
    borderStyle?: 'single';
    borderColor?: string;
  };
}
```

#### curY 推进规则（关键！）

- **text 节点**：渲染后 `curY++`（占一行）
- **column box**：子元素依次渲染，curY 由子元素推进
- **row box**：所有子元素在同一行渲染，最后 `curY++`
- **border box**：上边框 `curY++` → 内容 → 下边框 `curY++`

#### 边框绘制顺序（关键！）

```
1. 上边框：┌─┐  → curY++
2. 内容区域：递归渲染子元素
3. 左右边框：回填 │ 到内容行
4. 下边框：└─┘  → curY++
```

**踩坑**：必须先画内容再画左右边框，因为内容行数未知。

#### space-between 布局

```typescript
// 第一个元素左对齐，最后一个元素右对齐
curX = x + paddingX;
for (let i = 0; i < children.length - 1; i++) {
  renderNode(buffer, child, curX, curY, ...);
  curX += childWidth;
}
// 最后一个元素
renderNode(buffer, lastChild, lastX, curY, lastWidth, maxHeight);
```

**踩坑**：最初没有实现 space-between，状态栏无法右对齐（commit 62ea766）。

---

### 帧渲染流程

```typescript
function renderTree(node: RenderNode): void {
  const buffer = createScreenBuffer(width, height, charPool, stylePool);
  clearBuffer(buffer);  // 必须清空！否则残留上帧数据
  renderNode(buffer, node, 0, 0, width, height);

  if (prevBuffer) {
    const patches = diffBuffers(prevBuffer, buffer);
    if (patches.length > 0) {  // 必须检查！否则空补丁也会输出
      writePatches(optimize(patches));
    }
  } else {
    writeFullFrame(buffer);  // 首帧全屏输出
  }
  prevBuffer = buffer;
}
```

**关键检查点**：
1. `clearBuffer` 必须调用，否则残留上帧数据
2. `patches.length > 0` 必须检查，否则空补丁也会触发 write
3. 首帧必须 `writeFullFrame`，否则 diff 无基准

---

### 样式缓存优化

```typescript
// writeFullFrame 中的样式缓存
let lastFg = -1, lastBg = -1, lastAttrs = -1;
for (每个 cell) {
  if (cell.fg !== lastFg || cell.bg !== lastBg || cell.attrs !== lastAttrs) {
    output += 构建 ANSI 代码;
    lastFg = cell.fg; lastBg = cell.bg; lastAttrs = cell.attrs;
  }
  output += charStr;
}
// 换行后重置缓存！
if (y < height - 1) {
  output += '\n';
  lastFg = lastBg = lastAttrs = -1;  // 必须重置
}
```

**踩坑**：换行后不重置缓存会导致样式错乱。

---

### 性能对比

| 指标 | Ink/React | 自研引擎 |
|------|-----------|----------|
| 运行时依赖 | 42 包 | 0 包 |
| 启动时间 | ~500ms | ~50ms |
| 内存占用 | ~30MB | ~5MB |
| 渲染方式 | 全量 diff | 差量输出 |

---

### 从零开发 Checklist

1. **cell.ts**：定义 Cell 接口，ATTR 常量用位标志
2. **pool.ts**：CharPool 用 Int32Array(128) 做 ASCII 快速路径
3. **screen-buffer.ts**：双缓冲 + Damage 追踪 + diffBuffers
4. **optimizer.ts**：合并连续 write/style
5. **writer.ts**：拼接成单次 stdout.write
6. **colors.ts**：颜色名→ANSI 代码映射
7. **renderer.ts**：RenderNode 递归渲染，注意 curY 推进规则
8. **index.ts**：首帧 writeFullFrame，后续 diff

---

## 二、TUI 中文/CJK 字符处理三要素

### 1. 输入编码：Buffer + 手动 UTF-8 解码

**问题**：raw mode 下 `setEncoding('utf8')` 会把中文（3 字节 UTF-8）拆成多个 `data` 事件，`key.length === 1` 过滤会把不完整字节当普通字符 → 乱码。

**方案**：移除 `setEncoding`，用 Buffer 接收，手动解码：

```typescript
process.stdin.setRawMode(true);
process.stdin.resume();
let pending = Buffer.alloc(0);  // 存储不完整的字节

process.stdin.on('data', (buf: Buffer) => {
  const data = Buffer.concat([pending, buf]);
  pending = Buffer.alloc(0);

  for (let i = 0; i < data.length; ) {
    const byte = data[i]!;

    // 特殊键必须在 UTF-8 解码前处理！
    if (byte === 0x03) { process.exit(0); }           // Ctrl+C
    if (byte === 0x0d || byte === 0x0a) { /* 回车 */ } // CR/LF
    if (byte === 0x08 || byte === 0x7F) { /* 退格 */ } // BS/DEL
    if (byte < 0x20) { i++; continue; }               // 其他控制字符跳过

    // UTF-8 多字节判断
    let charLen = 1;
    if ((byte & 0xE0) === 0xC0) charLen = 2;       // 110xxxxx
    else if ((byte & 0xF0) === 0xE0) charLen = 3;   // 1110xxxx（中文）
    else if ((byte & 0xF8) === 0xF0) charLen = 4;   // 11110xxx（emoji）

    // 字节不够 → 存入 pending 等下次
    if (i + charLen > data.length) {
      pending = data.subarray(i);
      break;
    }

    const char = data.subarray(i, i + charLen).toString('utf8');
    input += char;
    i += charLen;
  }
});
```

**关键检查点**：
1. 不设 `setEncoding`，用 Buffer 原始字节
2. 特殊键（Ctrl+C/回车/退格）必须在 UTF-8 解码前用字节值判断
3. 字节不够时存入 `pending`，不能丢弃
4. `Buffer.concat([pending, buf])` 拼接未完成字节

**退格处理**：用 `[...input]` 展开为字符数组再 splice，确保中文字符作为单个元素：

```typescript
if (byte === 0x08 || byte === 0x7F) {
  if (cursorPos > 0) {
    const chars = [...input];
    chars.splice(cursorPos - 1, 1);
    input = chars.join('');
    cursorPos--;
  }
}
```

---

### 2. 渲染间距：宽字符"第二格"标记

**问题**：CJK 字符宽度=2，需要在第二格放空字符标记。原代码标记放在 `x + col`（字符后面），导致下一个字符从 `x + col + 1` 开始 → 中间多一格空白。

**方案**：标记放在 `x + col - 1`（字符占用的第二格），不额外推进 col：

```typescript
let col = 0;
for (let i = startIdx; i < chars.length && col < maxWidth; i++) {
  const w = charWidths[i]!;
  if (col + w > maxWidth) break;
  setCell(buffer, x + col, curY, chars[i]!, fg, '', attrs);
  col += w;
  // 标记在第二格，不是后面！
  if (w === 2) {
    setCell(buffer, x + col - 1, curY, '', fg, '', attrs);
    // 不要 col++！col 已经通过 col += w 正确推进了
  }
}
```

**关键检查点**：
1. `col += w` 在标记之前（col 先推进到字符后面）
2. 标记位置是 `x + col - 1`（字符的第二格），不是 `x + col`
3. 标记后不 `col++`（col 已经正确）

**文本溢出截断**：当文本超过宽度时，从末尾往前找能放下的起始位置：

```typescript
let startIdx = 0;
if (totalWidth > maxWidth) {
  let w = 0;
  for (let i = chars.length - 1; i >= 0; i--) {
    w += charWidths[i]!;
    if (w > maxWidth) {
      startIdx = i + 1;
      break;
    }
  }
}
```

---

### 3. 光标定位：ANSI 转义序列

**问题**：终端光标定位 `\x1b[row;colH` 是 1-based，需要精确计算输入框内容的起始行列。

**方案**：逐层累加布局偏移：

```typescript
function positionCursor() {
  // 行：root paddingY + banner marginY + banner 行数 + messages marginY + messages 行数 + input marginY + border 上
  const messageLines = messages.length === 0 ? 1 : messages.length;
  const inputRow = 1 + 1 + 3 + 1 + messageLines + 1 + 1;

  // 列：root 内容起始(2) + border 左(1) + paddingX(1) + 前缀"❯ "(2) + 光标前文本宽度
  const chars = [...input];
  const beforeWidth = chars.slice(0, cursorPos).reduce((w, c) => w + charWidth(c), 0);
  const inputCol = 2 + 1 + 1 + 2 + beforeWidth;

  process.stdout.write(`\x1b[${inputRow};${inputCol}H`);
}
```

**关键检查点**：
1. root 内容从第 **2** 列开始（第 1 列是 root 的 paddingX）— 常见错误是用 1
2. `charWidth` 函数需要在 index.ts 中**重复定义**（renderer.ts 中也有，但不导出）
3. 渲染后必须调用 `positionCursor()` 定位光标
4. 显示终端光标：`process.stdout.write('\x1b[?25h')`

**charWidth 完整实现**（需要处理代理对）：

```typescript
function charWidth(char: string): number {
  const code = char.codePointAt(0)!;  // 用 codePointAt 处理代理对
  if (code >= 0x4E00 && code <= 0x9FFF) return 2;   // CJK
  if (code >= 0x3000 && code <= 0x303F) return 2;   // CJK 符号
  if (code >= 0xFF00 && code <= 0xFFEF) return 2;   // 全角字符
  if (code >= 0x2E80 && code <= 0x2FDF) return 2;   // CJK 部首
  if (code >= 0x1F000 && code <= 0x1FFFF) return 2; // Emoji
  return 1;
}
```

**注意**：用 `codePointAt(0)` 而非 `charCodeAt(0)`，正确处理 emoji 等代理对字符。

---

### 从零开发 Checklist

1. **输入**：不设 encoding，Buffer 手动解码，特殊键用字节值判断
2. **渲染**：宽字符标记在 `x + col - 1`，不额外 `col++`
3. **光标**：root 内容从第 2 列开始，`charWidth` 用 `codePointAt`
4. **charWidth**：需要在 renderer.ts 和 index.ts 两处定义
5. **退格**：用 `[...input]` 展开字符数组再 splice

---

## 三、经验总结

### 为什么记录这些

MiCode TUI 项目在实现中文输入时踩了三个独立的坑，互相影响但根源不同。从 Ink/React 迁移到自研引擎是最关键的架构决策。

### 如何应用

- 新建 TUI 时优先考虑自研差量渲染，参考此项目的模块划分、数据结构和踩坑记录
- 中文输入处理直接参考 Buffer 手动解码模式，避免重复踩坑
- 特别注意 root 内容起始列是 2 不是 1

---

*来源：mi-code 项目 | 生成日期：2026-06-26*
