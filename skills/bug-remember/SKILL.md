---
name: bug-remember
description: >
  Query, fix, and record bug fixes from a local experience library.
  TRIGGER when: user says "/bug-remember", or conversation contains phrases like
  "你已经改了超过3次了", "又报错了", "之前也遇到过", "之前碰到过", "忘了上次怎么改的",
  "同样的问题", "又踩坑了", "这个坑记一下", "same bug again",
  "happened before", "forgot how I fixed this", "remember this fix".
  Also trigger on any sign that a bug has been encountered and fixed repeatedly.
  When triggered: FIRST check the bugs/ experience library for matching solutions,
  then apply the fix or record a new entry.
---

# Bug Remember

A self-learning bug fix system. It has three capabilities:

1. **Query** — Search the experience library for known fixes
2. **Apply** — Use known fixes to resolve current issues
3. **Record** — Capture new bug fixes into the experience library

## Experience Library

All bug records live in the `bugs/` folder of this skill directory.

```
~/.claude/skills/bug-remember/
├── SKILL.md
├── bugs/              ← experience library
│   ├── canvas-render-flicker.md
│   ├── phaser-collision-tunnel.md
│   └── ...
├── scripts/
│   └── keyword-detector.sh
└── evals/
```

Each file is one bug record. Use `ls bugs/` and `grep` to search.

## Trigger Paths

This skill has two trigger paths with different behaviors:

### Path A: Hook passive trigger (keyword detection)
The PostToolUse hook detects keywords and passes a reason to Claude.
**Behavior: Auto-query, no confirmation needed.** Go directly to Step 1 (Query).
The user is in pain — act fast, search the experience library immediately.

**Keywords (Chinese):**
- "你已经改了超过3次了"
- "又报错了"
- "之前也遇到过"
- "之前碰到过"
- "忘了上次怎么改的"
- "同样的问题"
- "又踩坑了"
- "这个坑记一下"

**Keywords (English):**
- "same bug again"
- "happened before"
- "forgot how I fixed this"
- "remember this fix"
- "we fixed this before"
- "same issue as last time"

### Path B: User manual trigger (/bug-remember)
The user explicitly invokes the skill.
**Behavior: Record mode.** Go directly to Step 3 (Record).
This is the "post-mortem" path — the bug is already fixed, user wants to save the experience.

## Execution Flow

### Step 1: Query — search the experience library (Path A entry point)

Before doing ANYTHING else, search the `bugs/` folder:

1. List all files: `ls ~/.claude/skills/bug-remember/bugs/`
2. Extract keywords from the current problem (error messages, framework, symptoms)
3. Search file contents for matching keywords: `grep -ril "keyword" ~/.claude/skills/bug-remember/bugs/`
4. Also check YAML frontmatter `keywords` fields across all files

If you find a match:

- Read the full matching bug document
- **Verify relevance**: Check that the bug document's `framework` field matches the current project's framework. A grep hit on a generic keyword like "渲染" or "error" is not enough — the framework and symptom must both align. If the framework doesn't match, discard the result.
- Read the **失败路径** section to know what NOT to try
- Apply the documented solution directly
- Tell the user: "在经验库中找到了解决方案: [bug-name]，直接应用"
- If the fix works → done
- If the fix doesn't work → proceed to Step 3 (Record) to update the entry

If no match found:

- Tell the user: "经验库中没有找到匹配的记录，开始排查"
- Proceed to normal debugging
- After fixing → proceed to Step 3 (Record) to save the experience

### Step 2: Apply — use a known fix

When a matching bug document is found:

1. Read the **解决方案** section for the fix
2. Read the **失败路径** section to know what NOT to try
3. Apply the fix
4. Verify it works
5. If the fix needs updating (new version, different environment), update the bug document's `updated` date and add a note

### Step 3: Record — capture a new bug fix (Path B entry point)

**Edge case:** If `/bug-remember` is invoked but the current conversation contains no bug-fixing work (no errors, no debugging, no failed attempts), tell the user: "当前对话中没有发现需要复盘的 bug 修复过程。请先描述你遇到的 bug 和修复步骤，然后我来帮你记录。" Do not create an empty bug document.

#### 3a. Retrospective

Walk through the conversation chronologically:

1. **问题定义** — Exact error message, visual glitch, or unexpected behavior
2. **失败路径** — Each failed attempt with reasons it didn't work
3. **成功路径** — What finally fixed it
4. **根因分析** — Why this bug happens
5. **适用条件** — Framework version, environment, code patterns

Present the retrospective to the user for review. Ask: "以上总结准确吗？有遗漏的步骤吗？"

#### 3b. Check for existing records

Search `bugs/` one more time with the finalized keywords. If a similar entry exists, ask: update or create new?

#### 3c. Write bug document

Create a file in `~/.claude/skills/bug-remember/bugs/` using the template below.

**File naming:** Short, descriptive slug in English:
- `canvas-render-flicker.md`
- `phaser-collision-tunnel.md`
- `react-hydration-mismatch.md`

## Bug Document Template

```markdown
---
name: canvas-render-flicker
framework: pixi.js
version: "7.x"
keywords: [闪烁, flicker, canvas, 渲染, render, 渲染闪烁]
created: 2026-04-08
updated: 2026-04-08
---

## 问题
{When does this bug occur — specific conditions, framework version, environment}

## 错误表现
{Exact error message, visual glitch, or unexpected behavior}

## 根因
{Why this bug happens — API misunderstanding, framework quirk, version issue}

## 解决方案
{The specific fix — code change, config update, dependency version}

## 失败路径
- 尝试 A: {approach} → 失败原因: {why it didn't work}
- 尝试 B: {approach} → 失败原因: {why it didn't work}

## 适用条件
{Framework versions, environments, code patterns that trigger this bug}
```

## YAML Metadata Fields

| Field | Required | Purpose |
|-------|----------|---------|
| `name` | Yes | File identifier, matches filename without .md |
| `framework` | Yes | Technology stack (pixi.js, phaser, react, etc.) |
| `version` | Yes | Framework version range |
| `keywords` | Yes | Search terms in both Chinese and English |
| `created` | Yes | First recording date |
| `updated` | Yes | Last update date |

## Guidelines

- **Query first, always.** Do not start debugging without checking bugs/ first
- **Be specific:** Include file paths, error messages, version numbers
- **Be bilingual:** Keywords should include both Chinese and English terms
- **Be concise:** Each section 2-5 lines — enough context, not a novel
- **No secrets:** Never record API keys, tokens, or passwords
- **Update dates:** Always update the `updated` field when modifying a record
