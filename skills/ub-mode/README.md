# ub-mode

`ub-mode` 是一个 Agent 技能。它在编写、修改代码时，强制执行规范的软件工程原则。
本技能来自《整洁代码之道》（Clean Code）作者 Robert C. Martin（Uncle Bob）的 GitHub 仓库 [unclebob/ub-mode](https://github.com/unclebob/ub-mode)，MIT 许可。本目录 `SKILL.md` 为上游原件（仅末行行尾符不同）。
上游自述：规则参考 @poteto 的规则（poteto-mode / pstack）精简而来，并搭配可复现的质量门禁工具：

1. **先让代码可用** — 懒协议、底层思考、TDD、基于第一性原理重设计、质疑前提、先删减再新增、结果导向执行、搭建杠杆、架构规范（领域建模、边界约束、类型系统约束、操作幂等、共享状态先拆分再序列化）。
2. **再让代码正确** — 修复根因、拆分可验证单元、测试行为（不测试实现）、Crapper（CRAP 值 < 10）、Dryer（重复度 < 0.8）、Mutator（覆盖未覆盖分支、清除存活变异体、删除死代码）、重构。
3. **工程惯例** — 留存日志、管控上下文窗口、工作区清理、有序小粒度提交推送。

## 配套工具仓库

`ub-mode` 依靠确定性质量门禁工具，管控结构、复杂度、重复代码、测试完备度：
- unclebob/crapper — 计算每个函数的圈复杂度、测试覆盖率、CRAP 分数，保证所有函数 CRAP 低于阈值（< 10）。
- unclebob/dryer — 检测结构化代码重复，消除相似度 ≥ 0.8 的重复代码块。
- unclebob/mutator — 执行变异测试，找出未覆盖代码、测试没能捕获的存活变异体。
- unclebob/uml-viewer — 可视化组件、覆盖率、CRAP 分数、变异分数、依赖方向。

## 安装

把 `SKILL.md` 复制到 Agent 的技能目录，路径为 `ub-mode/SKILL.md`。
示例路径：
`~/.grok/skills/ub-mode/SKILL.md`
`~/.claude/skills/ub-mode/SKILL.md`
`~/.cursor/skills/ub-mode/SKILL.md`
复制完成后，调用 `/ub-mode` 启用。
