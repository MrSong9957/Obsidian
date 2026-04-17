---
name: Claude Code 全局技能存放位置
description: 自定义 SKILL 应放在 ~/.claude/skills/<name>/SKILL.md，不是项目目录或 plugins 目录
type: feedback
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
status: deprecated
---
> 内容已迁移至 [[wiki/claude-code-p1.md|claude-code-p1]]。

Claude Code 全局自定义技能的存放位置是 `~/.claude/skills/<skill-name>/SKILL.md`，不是项目内的 `CLI/Python/skills/` 目录，也不是 `~/.claude/plugins/` 目录。

**Why:** 实测放错三个位置都没生效：(1) 项目 `CLI/Python/skills/` 是 mi-code 项目自己的技能系统，不是 Claude Code 的；(2) `~/.claude/plugins/` 是插件市场的缓存目录，需要完整的 plugin.json 等结构；(3) 只有 `~/.claude/skills/` 是 Claude Code 扫描全局技能的正确路径。放好后用 `/reload-plugins` 刷新即可生效。

**How to apply:** 添加自定义技能时，直接 `mkdir -p ~/.claude/skills/<name>` 并放入 `SKILL.md`，然后 `/reload-plugins`。不要尝试放到项目或 plugins 目录。
