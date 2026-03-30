---
name: learn-claude-code
description: shareAI-lab/learn-claude-code 仓库学习笔记
type: project
---

# Learn Claude Code 学习笔记

> 学习仓库：https://github.com/shareAI-lab/learn-claude-code

---

## safe_path() — 路径穿越防御

```python
def safe_path(p: str) -> Path:
    path = (WORKDIR / p).resolve()
    if not path.is_relative_to(WORKDIR):
        raise ValueError(f"Path escapes workspace: {p}")
    return path
```

### 三步机制

1. **拼接 + resolve**：`(WORKDIR / p).resolve()` 把用户输入拼到沙箱目录下，`.resolve()` 展开所有 `..`、`.`、符号链接，得到真正的绝对路径
2. **校验边界**：`is_relative_to(WORKDIR)` 检查最终路径是否仍在沙箱内
3. **通过返回 / 否则拒绝**：只有安全路径才返回

### 关键点

- **必须 resolve**：只检查字符串中的 `..` 不够，符号链接也能逃逸。`resolve()` 连符号链接一起展开
- **防御 Path Traversal**（OWASP Top 10）：典型攻击 `p = "../../etc/passwd"`，resolve 后变成 `/etc/passwd`，被 `is_relative_to` 拦截
- **比喻**：X 光安检——不管地址套多少层，透视到最终目的地再判断是否越界

---

## run_read() — 安全读取文件内容

```python
def run_read(path: str, limit: int = None) -> str:
    text = safe_path(path).read_text()
    lines = text.splitlines()
    if limit and limit < len(lines):
        lines = lines[:limit]
    return "\n".join(lines)[:50000]
```

### 流程

1. **安检 + 读取**：`safe_path(path).read_text()` — 先过路径安检，安全后才读文件
2. **拆行**：`splitlines()` 按换行符拆成列表
3. **行数限制**：`if limit and limit < len(lines)` — 用户指定了限制且文件超长时，截取前 N 行
4. **字符硬顶**：`[:50000]` — 最多返回 5 万字符，兜底保护

### 关键点

- **两层限流**：`limit` 是用户可控的软限制，`50000` 是系统硬天花板
- **安检复用**：直接调用 `safe_path()`，路径安全逻辑只写一次

---

## TOOL_HANDLERS — 工具路由表

```python
TOOL_HANDLERS = {
    "bash":       lambda **kw: run_bash(kw["command"]),
    "read_file":  lambda **kw: run_read(kw["path"], kw.get("limit")),
    "write_file": lambda **kw: run_write(kw["path"], kw["content"]),
    "edit_file":  lambda **kw: run_edit(kw["path"], kw["old_text"],
                                         kw["new_text"]),
}
```

### 是什么

字典，把工具名映射到对应的处理函数。AI 请求到达后，查表找到函数并执行。

### 为什么用 lambda

每个工具参数不同，但 AI 传来的格式统一是字典。lambda 是适配器——接收统一的 `**kw` 字典，从中提取该工具需要的参数，传给实际函数。

### 好处

- 调用代码只有一行：`TOOL_HANDLERS[tool](**kw)`，不需要 if/elif
- 加新工具只需在字典里加一行，不用改其他代码

### 完整链路

1. 系统把工具说明书（名称、参数、说明）发给 AI
2. AI 根据用户需求决定调哪个工具，返回工具名和参数
3. 系统查 TOOL_HANDLERS，找到对应函数执行
4. 结果返回给 AI

---
