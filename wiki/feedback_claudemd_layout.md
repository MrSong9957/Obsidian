---
name: 文档编排与双链规范
description: CLAUDE.md 的 KV 缓存编排原则 + Obsidian 双链带 .md 扩展名的跨系统兼容方案
type: feedback
updated: 2026-04-16
tags: [kv-cache, claudemd, prompt-engineering, obsidian, 双链]
originSessionId: a40cbcbb-0a0a-4e5c-95c1-213ce2eb6a6c
status: deprecated
---
# 文档编排与双链规范

> 内容已迁移至 [[wiki/claude-code-p1.md|claude-code-p1]]。

## CLAUDE.md KV 缓存编排

CLAUDE.md 按 KV 缓存效率编排：稳定不变的内容放最前面，易变的项目信息放最后。精简掉 Claude 能自己从代码推导的内容（目录结构、运行命令、通用编码规范），只保留无法推导的显式约束。修改项目部分时不会使前面稳定部分的缓存失效。

**Why:** Anthropic prompt caching 对前缀不变的内容有缓存优势，越靠前越稳定命中率越高。CLAUDE.md 是 system prompt 的一部分，每个字节都占用 token 和缓存空间。

**How to apply:** 编排 CLAUDE.md 时：1) 最稳定的行为规则/贡献指南置顶；2) 技术栈、架构等中等稳定内容居中；3) 项目当前状态等易变信息放末尾；4) 删除可通过 ls/read 代码推导的内容。

## Obsidian 双链规范

双链统一带 `.md` 扩展名：`[[wiki/feedback_tdd.md|TDD 实践]]`，不带扩展名的格式 `[[wiki/feedback_tdd|TDD 实践]]` 禁用。

**Why:** 不带 `.md` 时 LLM 需要推断路径（去括号、补扩展名），存在解析歧义。带 `.md` 后路径确定性为 100%，零推断。Obsidian 完全支持带 `.md` 的双链，图谱和反向链接不受影响。

**How to apply:** 所有 MEMORY.md、README.md、wiki 文档中的 Obsidian 双链一律使用 `[[path/to/file.md|显示名]]` 格式。
