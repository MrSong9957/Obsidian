---
created: 2026-06-01
updated: 2026-06-01
tags: [git, quick-reference, backup]
---

# Git 快速参考（兜底手册）

> 无法使用 Claude Code 或其他 AI 工具时，按此文档手动操作。

## 一、Obsidian 仓库（日常笔记）

```bash
# 1. 进入仓库目录
cd E:/Files/Obsidian

# 2. 查看哪些文件有改动
git status --short

# 3. 暂存所有改动
git add -A

# 或只暂存指定文件
git add wiki/某个文件.md wiki/另一个文件.md

# 4. 提交
git commit -m "docs: 简要描述改了什么"

# 5. 推送到远程
git push
```

### 提交信息格式

```
<类型>: <简要描述>

类型：
  docs     — 文档更新
  feat     — 新增内容
  fix      — 修正错误
  refactor — 重组/重命名
  chore    — 杂项维护
```

### 常见场景

| 场景 | 命令 |
|------|------|
| 改了几个笔记，想提交 | `git add -A && git commit -m "docs: 更新笔记" && git push` |
| 只想提交一个文件 | `git add wiki/文件名.md && git commit -m "docs: 更新xxx" && git push` |
| 提交信息写错了，想修改 | `git commit --amend -m "新的提交信息"` （仅限未推送的情况） |
| 推送失败（远程有新提交） | `git pull --rebase` 然后 `git push` |
| 想看最近 10 条提交记录 | `git log --oneline -10` |
| 想撤销某个文件的改动 | `git checkout -- wiki/文件名.md` |

## 二、其他项目仓库

```bash
# 进入项目目录
cd E:/Files/PycharmProjects/项目名

# 后续步骤同上：status → add → commit → push
```

## 三、分支操作（偶尔用）

```bash
# 查看当前分支
git branch

# 创建并切换到新分支
git checkout -b feat/新功能名

# 切回主分支
git checkout main

# 合并分支到 main
git merge feat/新功能名

# 删除已合并的分支
git branch -d feat/新功能名
```

## 四、安全注意事项

### Remote URL 禁止明文 token

**禁止：** 在 `git remote set-url` 中嵌入 token（如 `https://TOKEN@github.com/...`）
**原因：** `git remote -v` 会暴露明文 token，有泄露风险
**替代方案：**

```bash
# 1. remote URL 不含 token
git remote set-url origin https://github.com/user/repo.git

# 2. 用 credential store 持久化认证
echo "https://username:TOKEN@github.com" > ~/.git-credentials
chmod 600 ~/.git-credentials
git config --global credential.helper store

# 3. Token 从 .env 读取，不硬编码
export GITHUB_TOKEN=$(grep GITHUB_TOKEN path/to/.env | cut -d= -f2)
```

## 五、应急处理

| 问题 | 解决 |
|------|------|
| `git push` 被拒绝 | `git pull --rebase` 再 `git push` |
| 误提交了敏感文件 | 立即处理，参考 `.gitignore` 或 `git rm --cached` |
| 不确定改了什么 | `git diff` 查看具体改动内容 |
| 想丢弃所有未提交改动 | `git checkout -- .` （慎用，不可恢复） |
| 想查看某个文件的历史 | `git log --oneline wiki/文件名.md` |

---

> 核心流程就三步：`git add` → `git commit` → `git push`。记住这个就不会慌。
