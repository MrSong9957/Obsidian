# 来源与适配说明

- 官方仓库：[anthropics/claude-code-security-review](https://github.com/anthropics/claude-code-security-review)。
- 固定提交：`0c6a49f1fa56a1d472575da86a94dbc1edb78eda`。
- 原始命令：[.claude/commands/security-review.md](https://github.com/anthropics/claude-code-security-review/blob/0c6a49f1fa56a1d472575da86a94dbc1edb78eda/.claude/commands/security-review.md)。
- 许可证：MIT，完整原文见 [LICENSE](LICENSE)。

这是基于官方命令整理的技能适配版，并非 Anthropic 原生发布的 Skill 包。

[SKILL.md](SKILL.md) 直接包含完整官方审查指令正文，未删节、改写其审查流程或过滤规则。本地仅补充技能元数据、来源标识和运行环境适配说明。

该审查流程使用 Agent 自身的 Git、文件读取和子任务工具，无需附带 GitHub Action 的实现、测试或运行脚本。本目录仅保留 `SKILL.md`、`LICENSE` 和本文件。
