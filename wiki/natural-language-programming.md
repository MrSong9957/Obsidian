---
name: 人话编程规范
description: 用自然语言按编程结构描述逻辑的编程方法 — AI 双向翻译（人话↔代码）时参考
type: concept
sources:
  - "[[sources/learn-claude-code]]"
updated: 2026-04-05
tags: [ai, programming]
status: deprecated
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
---
# 人话编程规范

> 内容已迁移至 [[wiki/agent-arch-p2.md|agent-arch-p2]]。

> 创建日期：2026-03-31

---

## 1. 概述

### 定义

**人话编程**是一种用自然语言按照编程语言的结构（缩进、函数块、类块）来准确描述实现逻辑的编程方法。文件格式为 Markdown，内容结构直接对标代码结构，AI 负责在人与编程语言之间双向翻译。

### 核心理念

- **人写人话，AI 写代码** — 人类用自然语言描述逻辑，AI 翻译成任意编程语言
- **结构即代码结构** — 缩进、块、函数边界与目标编程语言一一对应
- **双向无损** — 代码 → 人话 → 代码，结构信息不丢失

### 与其他方法的区别

| 方法 | 特点 |
|------|------|
| 传统编程 | 用编程语言写文件，人学机器的语言 |
| 伪代码 | 不够严格，无法精确双向翻译 |
| Vibe Coding | 口头描述让 AI 自由发挥，不保留结构控制 |
| **人话编程** | 用自然语言 + 编程结构描述，AI 精确翻译，人保持控制 |

### 适用场景

- 编程新手入门（用中文理解编程概念）
- 开发者快速原型（用人话描述逻辑，AI 生成代码）
- 代码文档化（把代码翻译成人话，便于理解和审查）
- 跨语言移植（同一个人话描述翻译成不同语言）

### 适用语言范围

适用于**通用编程语言**（过程式、面向对象、函数式）：
Python、JavaScript、TypeScript、Go、Rust、C++、Java、C#、Ruby、PHP 等

**不适用于**领域特定语言（DSL）：
SQL（声明式查询）、HTML/CSS（标记语言）、正则表达式（模式匹配）、Shell（管道式）

原因：人话编程基于过程式结构（函数、条件、循环、异常），DSL 缺乏这些结构。

---

## 2. 基本规则

共 7 条，是整个人话编程的基石：

1. **缩进即结构** — 用 1 个 TAB 表示一层代码块。缩进层级与目标语言完全对应
2. **冒号开块** — 行尾冒号 `：` 表示接下来是一个缩进块（函数体、条件体、循环体等）
3. **自然语言表达** — 在结构框架内，用中文自然语言自由描述逻辑
4. **类型标注可选** — 可以用括号标注类型如 `n（整数）`，也可以省略
5. **字面量直写** — 数字、字符串、正则等直接写原文，不加引号（除非必要）
6. **原始名标注** — 反向翻译时，在函数名、变量名后用括号标注原始名称，如 `计算阶乘（factorial）：`、`用户列表（users） = []`
7. **注释保留** — 源代码中的注释用 `注：` 保留，并附上原始注释文本：`注：# Check if n is base case`

---

## 3. 基础语法元素

### 3.1 变量

```
名字 = "张三"
年龄 = 25
是否成年 = 年龄 >= 18
```

| Python | JavaScript | Go |
|--------|------------|----|
| `name = "张三"` | `let name = "张三"` | `name := "张三"` |
| `age = 25` | `let age = 25` | `age := 25` |
| `is_adult = age >= 18` | `let isAdult = age >= 18` | `isAdult := age >= 18` |

### 3.2 函数定义

```
计算阶乘：
	输入：n（整数）
	如果 n <= 1：
		返回 1
	否则：
		返回 n * 计算阶乘(n - 1)
```

| Python | JavaScript | Go |
|--------|------------|----|
| `def factorial(n):` | `function factorial(n) {` | `func factorial(n int) int {` |
| `    if n <= 1:` | `    if (n <= 1) {` | `    if n <= 1 {` |
| `        return 1` | `        return 1;` | `        return 1` |
| `    else:` | `    } else {` | `    } else {` |
| `        return n * factorial(n - 1)` | `        return n * factorial(n - 1);` | `        return n * factorial(n - 1)` |
| | `    }` | `    }` |
| | `}` | `}` |

### 3.3 函数表达方式

不同语言有多种定义函数的方式，反向翻译时通过行尾标记区分：

| 人话写法 | JavaScript | 说明 |
|----------|------------|------|
| `函数名：` | `function fn() {}` | 普通函数声明 |
| `函数名（箭头）：` | `const fn = () => {}` | 箭头函数 |
| `函数名（方法）：` | `fn() {}`（在 class 内） | 类方法 |
| `函数名（异步）：` | `async function fn() {}` | 异步函数 |
| `函数名（异步箭头）：` | `const fn = async () => {}` | 异步箭头函数 |

示例：

```
读取文件内容（箭头）：
	输入：file（文件路径）
	读取文件，编码为 utf-8
	返回内容

异步获取数据（异步箭头）：
	输入：url（字符串）
	发送 GET 请求到 url
	解析响应 JSON
	返回数据
```

翻译为 JavaScript：

```javascript
const readFileContent = (file) => {
  const content = readFileSync(file, 'utf-8');
  return content;
};

const fetchData = async (url) => {
  const response = await fetch(url);
  const data = await response.json();
  return data;
};
```

不标注时默认为普通函数声明。反向翻译时，如果原始代码是箭头函数，必须标注 `（箭头）`。

### 3.4 条件判断

```
如果 成绩 >= 90：
	等级 = "优秀"
否则如果 成绩 >= 60：
	等级 = "及格"
否则：
	等级 = "不及格"
```

对应关系：
- `如果` → `if`
- `否则如果` → `elif` / `else if`
- `否则` → `else`

### 3.5 循环

**遍历循环：**

```
遍历 1 到 10 的每个数字 i：
	打印 i
```

**条件循环：**

```
当 还有数据 时重复：
	读取下一行
	处理该行数据
```

对应关系：
- `遍历...每个...` → `for ... in ...`
- `当...时重复` → `while ...`
- `跳出` → `break`
- `跳过` → `continue`

### 3.6 其他基本元素

返回值、注释、导入依赖：

```
返回 结果                          → return result
注：这是一个注释                    → # 这是一个注释 / // 这是一个注释
引入 math, os                      → import math, os
引入 requests 作为 req             → import requests as req
```

---

## 4. 高级语法元素

### 4.1 类与对象

```
类 用户管理器：
	属性：
		用户列表 = 空列表
		数据库连接

	初始化（数据库地址）：
		连接到数据库地址
		将连接保存到 self.数据库连接

	添加用户（名字，年龄）：
		如果 名字 为空：
			抛出 "用户名不能为空"
		创建新用户 = {名字: 名字, 年龄: 年龄, id: 生成ID()}
		添加到 用户列表
		返回 新用户的 id

	查找用户（用户id）：
		遍历 用户列表 的每个 用户：
			如果 用户.id == 用户id：
				返回 用户
		返回 空
```

### 4.2 异常处理

```
尝试：
	打开文件 "data.txt"
	读取内容
捕获 文件不存在错误：
	打印 "文件不存在，使用默认配置"
	内容 = 默认配置
捕获 其他错误 为 e：
	打印 "发生错误：" + e
	内容 = 空
最终：
	关闭文件（如果已打开）
```

对应关系：
- `尝试` → `try`
- `捕获 ...错误` → `catch` / `except`
- `捕获 ... 为 e` → `catch(e)` / `except ... as e`
- `最终` → `finally`

### 非异常错误处理

Go、Rust、C 等语言不使用异常，而是通过返回值传递错误。人话中用自然语言描述错误检查逻辑，AI 根据目标语言选择对应模式：

**Go 风格（多返回值）：**

```
读取文件内容：
	内容, 错误 = 读取文件(文件路径)
	如果 错误 不为空：
		返回空, 错误
	返回 内容, 空
```

→ Go: `content, err := os.ReadFile(path); if err != nil { return "", err }; return string(content), nil`

**Rust 风格（Result 类型）：**

```
读取文件内容：
	结果 = 读取文件(文件路径)
	如果 结果 为错误：
		返回错误
	内容 = 解包结果
	返回 内容
```

→ Rust: `let content = fs::read_to_string(path)?;`

**原则**：人话不绑定特定错误处理模型。使用 `尝试/捕获` 描述异常模式，用 `错误不为空`/`为错误` 描述返回值模式。AI 识别语义后适配目标语言。

### 4.3 导入依赖

```
引入 math, os
引入 requests 作为 req
```

→ Python: `import math, os` / `import requests as req`
→ JS: `import * as math from 'math'` / `import req from 'requests'`
→ Go: `import ("math"; "os")`

### 4.4 并发（简述）

```
并发执行：
	任务1 = 启动任务 获取用户信息
	任务2 = 启动任务 获取订单信息
	用户信息 = 等待 任务1
	订单信息 = 等待 任务2
	返回 合并(用户信息, 订单信息)
```

---

## 5. 完整示例

### 示例 1：斐波那契数列

```
计算第n个斐波那契数：
	输入：n（非负整数）
	如果 n <= 1：
		返回 n
	前一个 = 0
	当前 = 1
	遍历 2 到 n 的每个 i：
		下一个 = 前一个 + 当前
		前一个 = 当前
		当前 = 下一个
	返回 当前
```

翻译成 Python：

```python
def fibonacci(n):
    if n <= 1:
        return n
    prev = 0
    curr = 1
    for i in range(2, n + 1):
        next_val = prev + curr
        prev = curr
        curr = next_val
    return curr
```

### 示例 2：API 数据处理（生产级）

```
引入 requests

获取天气（get_weather）：
	输入：城市名（city_name，字符串）
	前置校验：
		如果 城市名 为空或仅含空白：
			抛出 ValueError "城市名不能为空"
	尝试：
		注：调用天气 API // Call weather API
		发送 GET 请求到 https://api.weather.com/v1
			超时：10 秒
			参数：{city: 城市名, unit: "metric", lang: "zh_cn"}
			请求头：{Accept: "application/json"}
		如果 响应.状态码 == 404：
			返回 null（城市不存在，非错误）
		如果 响应.状态码 == 429：
			等待 5 秒后重试，最多 2 次
			仍失败则抛出 "天气 API 限流，请稍后重试"
		如果 响应.状态码 != 200：
			抛出 "天气 API 请求失败：" + 响应.状态码
		数据 = 解析JSON(响应.文本)
			如果解析失败或缺少 main 字段：
				抛出 "天气 API 返回格式异常"
		返回 {
			温度: 数据.main.temp（浮点数，摄氏度）,
			湿度: 数据.main.humidity（整数，百分比）,
			描述: 数据.weather[0].description（字符串）,
			更新时间: 数据.dt（UNIX 时间戳）
		}
	捕获 requests.ConnectionError：
		记录警告日志 "网络连接失败，城市：" + 城市名
		返回 null
	捕获 requests.Timeout：
		记录警告日志 "天气 API 超时，城市：" + 城市名
		返回 null
```

翻译成 Python：

```python
import requests
import logging

logger = logging.getLogger(__name__)

def get_weather(city_name, _retries=0):
    if not city_name or not city_name.strip():
        raise ValueError("城市名不能为空")

    try:
        # Call weather API
        response = requests.get(
            "https://api.weather.com/v1",
            params={"city": city_name, "unit": "metric", "lang": "zh_cn"},
            headers={"Accept": "application/json"},
            timeout=10
        )

        if response.status_code == 404:
            return None
        if response.status_code == 429:
            if _retries < 2:
                import time
                time.sleep(5)
                return get_weather(city_name, _retries=_retries + 1)
            raise Exception("天气 API 限流，请稍后重试")
        if response.status_code != 200:
            raise Exception(f"天气 API 请求失败：{response.status_code}")

        data = response.json()
        if "main" not in data:
            raise Exception("天气 API 返回格式异常")

        return {
            "temp": float(data["main"]["temp"]),
            "humidity": int(data["main"]["humidity"]),
            "description": str(data["weather"][0]["description"]),
            "updated_at": int(data["dt"])
        }
    except requests.ConnectionError:
        logger.warning(f"网络连接失败，城市：{city_name}")
        return None
    except requests.Timeout:
        logger.warning(f"天气 API 超时，城市：{city_name}")
        return None
```

### 示例 3：反向翻译 — Python 代码转人话

原始代码：

```python
class DataProcessor:
    def __init__(self, source_path):
        self.data = []
        self.source_path = source_path

    def load(self):
        with open(self.source_path, 'r') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#'):
                    self.data.append(line.split(','))

    def filter_by(self, column, value):
        return [row for row in self.data if row[column] == value]

    def count(self):
        return len(self.data)
```

反向翻译结果：

```
类 数据处理器（DataProcessor）：
	属性：
		数据（data） = 空列表
		源文件路径（source_path）

	初始化（source_path）：
		self.源文件路径（source_path） = source_path

	加载（load）：
		注：从文件读取并解析 CSV 数据 // Read and parse CSV data from source file
		打开 self.源文件路径 用于读取，作为 f：
			遍历 f 的每一行：
				去除行首尾空白
				如果 行不为空 且 不以 # 开头：
					按逗号分割行，添加到 self.data

	按列筛选（filter_by）：
		输入：column（列号），value（值）
		返回 self.data 中所有该列等于指定值的行

	计数（count）：
		返回 self.data 的长度
```

---

## 6. 反向翻译规则

从编程语言翻译为人话时，遵循以下规则：

### 结构保留

| 代码结构 | 人话对应 |
|----------|----------|
| `def` / `function` / `func` | `函数名：` 或直接 `名称：` |
| `class` | `类 名称：` |
| `if` / `elif` / `else` | `如果` / `否则如果` / `否则` |
| `for ... in ...` | `遍历 ... 的每个 ...` |
| `while ...` | `当 ... 时重复` |
| `try` / `catch` / `finally` | `尝试` / `捕获` / `最终` |
| `return` | `返回` |
| `import` | `引入` |
| `#` / `//` 注释 | `注：` |
| 缩进层级 | 保持相同层级（TAB） |

### 翻译原则

1. **函数名直译 + 原始名标注**：`get_weather` → `获取天气（get_weather）：`，`calculate_total` → `计算总数（calculate_total）：`
2. **变量名直译 + 原始名标注**：`user_name` → `用户名（user_name）`，`is_valid` → `是否有效（is_valid）`
3. **保留代码结构**：该缩进的地方必须缩进，该有块的地方必须用冒号开块
4. **字面量原样保留**：字符串、数字、URL、正则表达式等不做翻译
5. **运算符可选翻译**：`==` 可以写成 `等于`，也可以保留原文；`>=` 可以写成 `大于等于`，也可以保留
6. **注释双向保留**：源码注释翻译为 `注：中文翻译 // 原始注释`，确保还原时能找回原文

### 不翻译的部分

- 字符串字面量中的内容
- URL 和文件路径
- 正则表达式
- API 端点和参数名
- 数据库字段名

### 原始名标注规则

反向翻译时，所有标识符（函数名、变量名、参数名、类名）必须在首次出现时标注原始名称。格式：`中文名（original_name）`。完整示例见第 5 节示例 3。

---

## 7. 写作指南

### 句式约定

| 场景 | 写法 | 示例 |
|------|------|------|
| 描述动作 | 祈使句 | `打开文件`、`读取内容` |
| 描述条件 | `如果...则...` | `如果 年龄 >= 18：` |
| 描述循环 | `遍历` / `当...时重复` | `遍历 用户列表 的每个 用户：` |
| 描述赋值 | `=` 或 `设为` | `总数 = 0` 或 `将总数设为 0` |
| 描述返回 | `返回` | `返回 计算结果` |
| 描述异常 | `抛出` | `抛出 "文件不存在"` |

### 避免歧义的技巧

1. **明确作用对象**：不说 `删除它`，说 `删除临时文件`
2. **明确执行顺序**：用 `先...再...` 或有序步骤
3. **明确数据类型**：在首次出现时用括号标注类型，如 `结果（布尔值）`
4. **明确范围**：不说 `所有`，说 `列表中的每个元素`
5. **复杂逻辑拆步骤**：一个缩进块内不要塞太多逻辑，拆成有序步骤

### 命名建议

- 函数名用动词开头：`获取用户`、`计算总数`、`验证输入`
- 变量名用名词：`用户列表`、`当前页码`、`最大重试次数`
- 布尔变量用 `是否` 开头：`是否登录`、`是否有效`
- 常量全大写风格可保留：`MAX_RETRY` 或翻译为 `最大重试次数`

### 生产级写作清单

教程级人话只描述核心逻辑，生产级人话需要额外覆盖以下维度。除明确标注为「教程示例」的内容外，所有面向实际使用的人话文档都应按此清单检查：

**每个函数应覆盖：**

- [ ] **前置校验** — 输入为 null/空/越界时的行为
- [ ] **超时** — 网络请求、文件操作的超时时间
- [ ] **重试** — 哪些错误重试、最多几次、退避策略（固定/指数）
- [ ] **错误分类** — 哪些错误可恢复（重试）、哪些不可恢复（直接抛出）、哪些降级（返回默认值）
- [ ] **边界值** — 空输入、极大输入、并发访问时的行为
- [ ] **资源释放** — 文件句柄、网络连接、锁的释放时机

**人话中的写法示例：**

```
前置校验：
	如果 输入 为 null 或 undefined，抛出 TypeError "text is required"
	如果 输入长度 为 0，直接返回 0，不发请求

超时与重试：
	请求超时 30 秒
	如果 429 限流：
		从响应头读取 Retry-After，若无则默认 5 秒
		重试最多 3 次
		3 次仍失败，抛出异常
	如果 >= 500 服务端错误：
		指数退避重试最多 2 次（1秒、2秒）
		仍失败，抛出异常
	如果 401 认证失败：
		直接抛出，不重试

降级处理：
	如果请求失败，打印警告到 stderr，返回 null
	不抛异常，不中断调用方
```
