---
title: PR merge 后本地分支的安全同步
name: PR merge 后本地分支的安全同步
description: 本地 master 有未推送提交 + 工作区有脏文件时,PR squash merge 后不能机械用 pull/reset --hard;先看拓扑和最终树内容,再做最小受控同步——保住脏文件、对齐到 merge commit
doc_type: concept
updated: 2026-08-02
tags:
  - git
  - pr-merge
  - squash-merge
  - local-sync
  - dirty-worktree
  - diverged-branches
  - safe-reset
---

# PR merge 后本地分支的安全同步

> **场景**: PR 已在远端 squash merge 到 `master`,但本地 `master` 与 `origin/master` 已分叉(本地有未推送提交),且工作区有**必须保留**的脏文件。如何把本地 `master` 安全同步到 merge commit,而不破坏脏文件、不丢失本地提交内容。

## 1. 问题本质:两条独立约束

PR merge 后的本地同步,同时受两条正确性约束,而常规命令会破坏其中之一:

| 约束 | 含义 | 哪个常规命令会破坏它 |
|---|---|---|
| **拓扑正确** | 本地与远端的分叉结构决定 `pull`/`ff-only` 是否可行 | `git pull` / `git merge --ff-only` 在分叉时直接报错失败 |
| **脏文件不可触碰** | 工作区里不属于本次 PR 的未提交修改必须原样保留 | `git reset --hard` / `git checkout -- .` 会丢弃/还原脏文件 |

**关键洞察**: 这两条约束**互相独立**——拓扑信息告诉你能不能快进,脏文件信息告诉你能不能 hard reset。机械套用任一命令都会破坏另一个不变量。必须**先看清拓扑 + 最终树内容,再决定同步路径**。

## 2. 标准同步流程(拓扑优先 + 最小受控操作)

### Step 1: 看清拓扑(决定同步路径)

```bash
# 本地 master 是否是 origin/master 的祖先?
git merge-base --is-ancestor master origin/master && echo "FF 可行" || echo "已分叉"

# 分叉时,看分叉点 + 两边各有哪些提交
git merge-base master origin/master          # 共同祖先
git log --oneline origin/master..master       # 本地有、远端没有的提交
git log --oneline master..origin/master       # 远端有、本地没有的提交(含 merge commit)
```

### Step 2: 确认远端 merge commit 的最终树内容(决定能不能信任它)

squash merge 会把 PR 分支相对其 base 的 diff 压成一个提交。**务必确认 merge commit 包含了所有期望内容**(尤其当本地有未推送提交时):

```bash
# merge commit 相对共同祖先改了哪些文件
git diff --name-only <merge-base> <merge-commit>

# 关键文件是否在 merge commit 中存在(内容是否权威)
git show <merge-commit>:<path/to/file> > /dev/null && echo "存在" || echo "缺失"
```

> 这一步防止 squash 只压了部分文件。本次实例中,本地 master 有 5 个 docs 提交未推送,但 squash commit `a914d11` 完整包含了 9 个计划文件 + 2 个 docs(因为 PR 分支基于包含这些 docs 的本地 master 构建),所以 merge commit 的树内容是权威完整的。

### Step 3: 最小受控同步(两步分离,脏文件始终可控)

把"移动分支指针"和"更新工作树"**分成两步**,让脏文件始终处于显式可控范围:

```bash
# (1) 移动分支指针到 merge commit,不动工作树
git reset --mixed origin/master

# (2) 从新 HEAD 恢复需要同步的文件(精确列文件,排除脏文件)
git checkout HEAD -- <file1> <file2> ...   # 不要写脏文件路径
```

**为什么不用复合命令?**

| 命令 | 问题 |
|---|---|
| `git reset --hard origin/master` | 丢弃脏文件(违反约束 2) |
| `git stash && git reset --hard && git stash pop` | stash 可能冲突;且约束常禁止 stash |
| `git pull` | 分叉时失败;即使成功也可能引入 merge commit 污染线性历史 |
| `git checkout -- .` | 还原所有脏文件,包括必须保留的那个 |

两步分离的优势:**每一步可单独验证**。`reset --mixed` 后 `git status` 能看到哪些文件待恢复;`checkout HEAD -- <列表>` 只动指定文件,脏文件从不进入操作范围。

### Step 4: 验证闭环

```bash
git branch --show-current          # 当前分支 = master
git rev-parse HEAD                 # HEAD = merge commit SHA
git status --short                 # 仅剩必须保留的脏文件
git ls-remote origin <feature>     # 远端功能分支已删除(空)
git branch --list <feature>        # 本地功能分支已删除(空)
```

## 3. 本次实例的具体拓扑(供对照)

```
共同祖先: 8ae5a75
本地 master: 8ae5a75 → [5 个 docs 提交] → c8ec6b6  (未推送)
origin/master: 8ae5a75 → a914d11 (squash: 9 计划文件 + 2 docs)

工作区脏文件: src/prompts/planner.generated.ts (LF→CRLF,用户原有修改)
```

- `git pull`: 因分叉失败。
- `git reset --hard origin/master`: 会丢 `planner.generated.ts` 的 CRLF 修改 → 禁止。
- **实际采用**: `reset --mixed origin/master` + `checkout HEAD -- <9 个计划文件>`,严格排除 `planner.generated.ts`。
- 结果: master 指向 `a914d11`,工作树与之一致,`planner.generated.ts` 原样保留。

## 4. 防御边界

- **禁止 stash / 还原 / 覆盖 / 暂存 / 提交**那些"不属于本次 PR 的脏文件"——它们是用户/其他任务的在途状态,PR merge 收尾无权触碰。
- **不要为了图省事用 `reset --hard`**——它是最常见的脏文件杀手。哪怕工作区"看起来只有一两个文件",也要先 `git diff --name-only` 确认脏文件清单。
- **squash merge 不保证包含本地未推送提交**——必须 Step 2 显式确认 merge commit 的树内容,不能假设。
- **两步分离的成本为零,收益是确定性**——比起 debug 一个被 hard reset 丢掉的脏文件,多敲一条 `checkout` 命令几乎无成本。

## 5. 何时本流程适用

- ✅ PR 已远端 merge,本地需同步,且工作区有必须保留的脏文件。
- ✅ 本地 master 与 origin/master 分叉(有未推送提交)。
- ✅ 需要保持线性历史(不要本地 merge commit)。
- ❌ 工作区完全干净 → 直接 `git pull --ff-only` 或 `git reset --hard origin/master` 即可,无需本流程。
- ❌ 本地未推送提交的内容**不在** merge commit 中 → 不能 reset 到 origin/master(会丢提交),需先 push 或 rebase。
