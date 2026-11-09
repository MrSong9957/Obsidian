---
name: 001-move-adr-to-sources
description: 把根目录架构决策记录迁入 sources/projects，根目录只留入口文件
type: concept
updated: 2026-11-09
tags: [adr, knowledge-management, cleanup]
source: self
---

# 001-move-adr-to-sources

状态：已确认（用户，2026-11-09，AskUserQuestion 选定方案 B）

## 1. 标题

`001-move-adr-to-sources`

## 2. 本轮目标

- 做：把根目录 `LLM-WIKI-ARCHITECTURE.md` 迁入 `sources/projects/`，根目录只剩 AGENTS.md、CLAUDE.md、MEMORY.md、README.md 四个入口文件。
- 不做：不重写 ADR 正文、不建 wiki 入口页、不改 AGENTS.md、不改文件名。

## 3. 触发与行为

1. 用户确认方案（2026-11-09 已确认，选方案 B 迁移）。
2. `git mv` 迁移文件到 `sources/projects/`。
3. 按 sources 资料格式补 frontmatter（`source: self`），正文一字不改。
4. 更新 `README.md` 两处引用为新路径。
5. `wiki_sync` 同步索引，提交并推送。

## 4. 输入输出与边界

- 输入：`LLM-WIKI-ARCHITECTURE.md`（86 行，git 已跟踪）。
- 输出：`sources/projects/LLM-WIKI-ARCHITECTURE.md`。
- 边界：
  - 文件名保持不变——库协议规定迁移资料保持文件名不变，覆盖本规格草案中原定的改名。
  - `README.md` 目录树注释与正文 wikilink 两处必须同步。
  - 正文内 `[[AGENTS.md]]` 等 wikilink 为全库名称解析，迁移后仍有效，不动。
  - 「前身见 `git show 573eb26:LLM-WIKI-ARCHITECTURE.md`」注记指向历史提交路径，保留原样。

## 5. 约束与取舍

- 用 `git mv`，保 rename 历史。
- 迁 `sources/` 不迁 `wiki/`：它不是入口知识，是决策详史；`sources/projects/` 分类即「项目规划、进度记录」。
- 取舍记录：方案 A 直接删除被否决——§4 方案取舍、§5 架构演进两节为全库独有内容，删除后仅 git 历史可见。

## 6. 验收结果

对应第 3 节，每条可观察、可复核：

- [ ] 1. 根目录无 `LLM-WIKI-ARCHITECTURE.md`，四个入口文件齐全。
- [ ] 2. `rg -l "LLM-WIKI-ARCHITECTURE"` 命中文件内路径全部指向 `sources/projects/`，无失效引用。
- [ ] 3. README 两处引用指向新路径，正文未改动。
- [ ] 4. `wiki_sync` 成功，查重报告无意外重复。
- [ ] 5. git 历史显示 rename，提交已推送。

## 7. 术语

无。

## 8. 调研链接

不适用（2026-11-09 会话讨论即调研）。
