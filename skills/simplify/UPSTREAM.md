# 来源与适配说明

- 官方来源：Claude Code 内置（bundled）技能 `/simplify`，随 CLI 本体二进制分发。官方命令参考将其标注为 "**[Skill]**: a bundled skill"。
- 提取版本：Claude Code 2.1.312（changelog 当时最新条目），技能文本提取自本机原生安装 `~/.local/share/claude/versions/2.1.312` 二进制内嵌的技能模板。
- 无独立公开源码仓库：[anthropics/claude-code](https://github.com/anthropics/claude-code) 仅含 issue 与 changelog，CLI 以二进制分发；[anthropics/skills](https://github.com/anthropics/skills) 是插件市场（文档技能与示例），不包含本体内置技能。
- 官方文档：[命令参考](https://code.claude.com/docs/en/commands.md)、[技能文档](https://code.claude.com/docs/en/skills.md)（"Bundled skills" 一节，可用 `disableBundledSkills` 关闭）。
- 演进：v2.1.63 首次加入 `/simplify`；v2.1.147 一度并入 `/code-review`；v2.1.152–154 拆分定型为当前"仅质量清理、不查 bug"形态。
- 许可证：技能文本随 Claude Code 本体分发，无单独开源许可证文件，本目录不附加 LICENSE。

[SKILL.md](SKILL.md) 完整收录官方运行时的两个变体（Agent 工具可用时的 4-agent 并行版、不可用时的单遍版），未删节、改写其审查流程与四个审查角度（Reuse / Simplification / Efficiency / Altitude）。Phase 0 与各审查角度在官方实现中是共享模板段，此处按原样展开。本地仅补充技能元数据、变体选择与参数适配说明。

该技能仅使用 Agent 自身的 git、文件检索与子代理工具，无需附带实现、测试或运行脚本。本目录仅保留 `SKILL.md` 和本文件。
