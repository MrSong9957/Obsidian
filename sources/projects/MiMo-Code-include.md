---
name: MiMo Code 的 @ 引用机制
description: MiMo Code 对 @ 引用语法的支持方式——核心正则实现与和 Claude Code @include 机制的区别
type: concept
updated: 2026-06-27
tags: [mimocode, include, markdown, config]
source: self
---

**是的，MiMo Code 支持 `@` 引用语法**，但和 Claude Code 的 `@include` 有重要区别。

---

## MiMo Code 的 `@` 引用机制

### 核心正则

**关键代码** — `packages/opencode/src/config/markdown.ts:6`：

```typescript
export const FILE_REGEX = /(?<![\w`])@(\.?[^\s`,.]*(?:\.[^\s`,.]+)*)/g
```

这个正则匹配所有 `@路径` 格式的引用，支持：

| 格式 | 示例 | 支持 |
|------|------|------|
| 相对路径 | `@src/utils.ts` | ✅ |
| 绝对路径 | `@/etc/config.json` | ✅ |
| 家目录路径 | `@~/.bashrc` | ✅ |
| 隐藏文件 | `@.env` | ✅ |
| 多层目录 | `@src/components/Button.tsx` | ✅ |
| 多扩展名 | `@file.test.ts` | ✅ |
| 反引号内 | `` `@not/matched` `` | ❌ 不匹配 |
| 邮箱 | `user@example.com` | ❌ 不匹配 |

### 解析流程

**关键代码** — `packages/opencode/src/session/prompt.ts:313-345`：

```typescript
const resolvePromptParts = Effect.fn("SessionPrompt.resolvePromptParts")(function* (template: string) {
  const parts: PromptInput["parts"] = [{ type: "text", text: template }]
  const files = ConfigMarkdown.files(template)  // 用正则提取所有 @引用

  for (const match of files) {
    const name = match[1]  // 提取路径部分
    const filepath = name.startsWith("~/")
      ? path.join(os.homedir(), name.slice(2))  // ~/ 开头 → 家目录
      : path.resolve(ctx.worktree, name)         // 其他 → 相对于工作区

    const info = yield* fsys.stat(filepath).pipe(Effect.option)
    if (Option.isNone(info)) {
      // 文件不存在 → 尝试作为 agent 名称查找
      const found = yield* agents.get(name)
      if (found) parts.push({ type: "agent", name: found.name })
      return
    }
    // 文件存在 → 作为文件附件添加
    parts.push({
      type: "file",
      url: pathToFileURL(filepath).href,
      filename: name,
      mime: stat.type === "Directory" ? "application/x-directory" : "text/plain",
    })
  }
  return parts
})
```

### 它在哪里被使用？

**主要用于用户输入和命令模板**，而不是 CLAUDE.md/AGENTS.md 文件：

1. **用户在 TUI 输入框中输入 `@文件路径`** → 文件内容作为附件发送给模型
2. **命令（command）模板中的 `@` 引用** → 命令执行时解析并注入

**关键代码** — `packages/opencode/src/session/prompt.ts:3574`：

```typescript
// 在命令处理中解析 @引用
const templateParts = yield* resolvePromptParts(template)
```

---

## 和 Claude Code 的 `@include` 对比

| 特性 | Claude Code `@include` | MiMo Code `@` 引用 |
|------|------------------------|---------------------|
| **作用范围** | CLAUDE.md 文件内 | 用户输入、命令模板 |
| **语法** | `@path/to/file.md` | `@path/to/file.md` |
| **是否递归** | 支持循环引用检测 | 不递归（只解析一层） |
| **处理方式** | 内容内联到 CLAUDE.md | 文件作为附件发送 |
| **CLAUDE.md 内可用** | ✅ | ❌（CLAUDE.md 被直接读取，不解析 @） |

### 关键区别

**Claude Code**：在 CLAUDE.md 中写 `@rules/typescript.md`，系统会把那个文件的内容**内联**到 CLAUDE.md 中，然后一起注入 system prompt。

**MiMo Code**：CLAUDE.md/AGENTS.md 是**直接读取原样注入**的，不会解析其中的 `@` 引用。`@` 引用只在用户输入和命令模板中生效。

---

## MiMo Code 的替代方案

如果你想在 AGENTS.md 中引用其他文件，MiMo Code 用的是不同的机制：

### 1. `instructions` 配置项

在 `mimocode.json` 中配置多个指令文件：

```json
{
  "instructions": [
    "AGENTS.md",
    "rules/typescript.md",
    "rules/testing.md",
    "~/.config/mimo/global-rules.md"
  ]
}
```

**关键代码** — `packages/opencode/src/session/instruction.ts:161-175`：

```typescript
if (config.instructions) {
  for (const raw of config.instructions) {
    if (raw.startsWith("https://") || raw.startsWith("http://")) continue
    const instruction = raw.startsWith("~/") ? path.join(os.homedir(), raw.slice(2)) : raw
    const matches = yield* (
      path.isAbsolute(instruction)
        ? fs.glob(path.basename(instruction), { cwd: path.dirname(instruction) })
        : relative(instruction)
    ).pipe(Effect.catch(() => Effect.succeed([] as string[])))
    matches.forEach((item) => paths.add(path.resolve(item)))
  }
}
```

### 2. 支持 HTTP URL

```json
{
  "instructions": [
    "AGENTS.md",
    "https://example.com/team-rules.md"
  ]
}
```

### 3. shell 命令模板

命令模板中支持 `!` 反引号执行 shell 命令：

```markdown
# 在命令模板中：
当前 git 分支: !`git branch --show-current`
```

**关键代码** — `packages/opencode/src/config/markdown.ts:7`：

```typescript
export const SHELL_REGEX = /!`([^`]+)`/g
```

---

## 总结

| 问题 | 答案 |
|------|------|
| MiMo Code 支持 `@` 引用吗？ | ✅ 支持，但只在用户输入和命令模板中 |
| AGENTS.md/CLAUDE.md 中能用 `@include` 吗？ | ❌ 不能，这些文件被直接读取 |
| 如何在 AGENTS.md 中引用其他文件？ | 用 `mimocode.json` 的 `instructions` 配置 |
| 支持循环引用检测吗？ | ❌ 不递归，只解析一层 |

如果你想实现类似 Claude Code 的 `@include` 效果，目前的方案是在 `mimocode.json` 中用 `instructions` 数组列出所有需要加载的文件。