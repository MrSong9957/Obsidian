# Memory

## 管理规则
- 此文件超过 200 行会被截断，每条保持单行，不超过 150 字符。
- 只保存常用的、跨会话的用户个人习惯和偏好。
- 条目格式：`- {场景}：{摘要} → [[wiki/xxx.md|详情]](...)`。
- 记忆查找流程（必须遵循）：① MEMORY.md 中查找 → ② 未命中则必须读 README.md 按领域导航查找 → ③ README.md 命中后读对应 wiki/ 文件。禁止跳过 README.md 直接 grep/find。

## 个人习惯
- 语言：中文
- 推进：小步迭代，每步可验证
- 调试：先观察再修改，三段式可观测性 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- 工作流：自主闭环，多次失败才请求介入
- 方法论：Harness Engineering → [[wiki/agent-arch.md|详情]]([wiki/agent-arch.md](wiki/agent-arch.md))
- 原则：只做必要更改，不提前抽象，复用成熟方案
- 子代理：可并行或需隔离时用；简单操作直接做
- 学习习惯：学霸七法（框架/反向/深层/逻辑/分科/渐进/理解>数量）+AI辅助 → [[wiki/learning.md|详情]]([wiki/learning.md](wiki/learning.md))
- 技术栈：原型 HTML/CSS/JS，正式 Flutter，后端 Python/Rust → [[wiki/frontend.md|详情]]([wiki/frontend.md](wiki/frontend.md))
- 开发方法：默认 TDD，代码质量由 AI 发现最佳实践 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- 编程认知：必须用四层分离法（设计/验证/诊断/决策），禁止 vibe coding → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))

## 经验教训
- Claude Code：技能存放/SKILL开发/文档双链/Hooks复盘 → [[wiki/claude-code.md|详情]]([wiki/claude-code.md](wiki/claude-code.md))
- 开发实践：提示词缓存/协作技巧、rm 安全操作 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- Learn ShareAI：s01-s19 从零构建 Agent，各章摘要、关键数据结构、实现要点见 → [[wiki/learn-shareai.md|详情]](wiki/learn-shareai.md)
- 上下文管理：当前完整保留，旧的才压缩（截断+落盘→占位符→摘要）→ [[wiki/agent-arch.md|详情]]([wiki/agent-arch.md](wiki/agent-arch.md))
- 决策：跨学科思维模型框架（芒格100模型），高频模型含反向思考/概率思维/二阶思维 → [[wiki/mental-models.md|详情]]([wiki/mental-models.md](wiki/mental-models.md))
- 凭证/Provider：token 集中管理在 D:\Files\.env；MiMo 用 AUTH_TOKEN 非 API_KEY，域名 xiaomimimo.com（非 xiaomimimimo.com） → [[wiki/token-management.md|详情]](wiki/token-management.md)
- Yazi：WinGet 安装需重启终端；`file.exe` 必须在 PATH（Git usr/bin）；`ya pkg` 缺 LICENSE 会报错需手动部署 → [[wiki/yazi.md|详情]]([wiki/yazi.md](wiki/yazi.md))
- 技能链调用：技能内引用其他技能时必须用 Skill 工具加载完整内容，不能跳过 → [feedback-skill-chain-discipline](feedback-skill-chain-discipline.md)
- 子代理/团队成员技能注入：均无法调用 Skill 工具，父代理预加载内容注入 prompt → [feedback-subagent-skill-injection](feedback-subagent-skill-injection.md)
- 技能文件同步：改 project/email_cli.py 或 SKILL.md 后必须 cp 到 ~/.claude/skills/email-assistant/，否则技能调用用旧版 → [feedback-skill-file-sync](feedback-skill-file-sync.md)
- Python 邮件开发：IMAP/SMTP asyncio 踩坑（BODY.PEEK、multipart 递归、QQ 邮箱特性） → [[wiki/email-assistant.md|详情]](wiki/email-assistant.md)
- 项目路线图：三阶段递进 A 开源 Build in Public → B Skill 框架 → C SaaS，含 10 平台发布方案 → [[wiki/todos.md|详情]](wiki/todos.md)
- 技能触发纪律：邮件等关键词请求必须先 Skill 工具调用，不能直接执行 CLI → [feedback-skill-trigger-discipline](feedback-skill-trigger-discipline.md)
- CC 进阶工作流：上下文管理/Hooks/子代理/并行 + Session Memory/Token经济学/Evals/持续学习 → [[wiki/claude-code.md|详情]](wiki/claude-code.md)
- 邮件详情展示：末尾必须追加 webmail Link 行（按账户变化） → [feedback-email-detail-link](feedback-email-detail-link.md)
- 人格化邮件回复：已合并 master，待办已迁移 → [[wiki/todos.md|待办事项]](wiki/todos.md)
- 新项目初始化：Docker 容器 → 验证镜像配置 → 补充缺失项（Git/CLAUDE.md/ECC rules/目录结构） → [[wiki/new-project-setup.md|详情]](wiki/new-project-setup.md)
- Rust 安装：容器用 USTC 镜像脚本；Windows 免 VS 选 GNU 工具链 `x86_64-pc-windows-gnu` → [[sources/dev/rust.md|详情]](sources/dev/rust.md)
- OpenCode 桥接 v2：log 修复/parseHookOutput/output.args 写回/permission.asked；input.tool 是字符串；Stop/SessionEnd 在 CLI 不触发；watcher 实测 40+ 次 PostToolUse 写入确认 100% 工作 → [[wiki/opencode-bridge.md|详情]]([wiki/opencode-bridge.md](wiki/opencode-bridge.md))
- ECC→OpenCode 集成：agent frontmatter 格式必须转换（model 删除、tools 数组→对象、color 映射）；不能把全部 .md 复制到 agents/ 目录 → [[wiki/opencode-bridge.md#ECC Agent 格式兼容性|详情]]([wiki/opencode-bridge.md](wiki/opencode-bridge.md))
- OpenCode 代理配置：通过 opencode.jsonc 的 agent 部分添加/覆盖代理，mode:"primary" 出现在切换列表；源码修改仅在需要自定义逻辑时 → [[wiki/opencode-bridge.md#代理配置|详情]]([wiki/opencode-bridge.md](wiki/opencode-bridge.md))
- MiMo-Code 源码阅读：4 模块 19 章路线（参考 Learn ShareAI s01-s19），核心闭环→系统加固→任务运行时→多Agent → [[sources/projects/mimocode-reading-guide.md|详情]]([sources/projects/mimocode-reading-guide.md](sources/projects/mimocode-reading-guide.md))
- cc switch 重置：切换 API KEY 时用过时模板重写 settings.json，通用配置已修正存入 → [[sources/claude-code/claude-code-setup-references.md#cc switch 通用配置模板|详情]](sources/claude-code/claude-code-setup-references.md)
- 缓存优化：第三方 API 缓存命中率低，根因是 `cch` 哈希值变化；设置 `CLAUDE_CODE_ATTRIBUTION_HEADER=0` 可部分解决 → [[sources/claude-code/claude-code-setup-references.md#缓存优化（Prompt Caching）|详情]](sources/claude-code/claude-code-setup-references.md)

## 知识库索引
- 本文件未覆盖的内容 → [README.md](README.md)（冷数据，按领域导航）
- Wiki 目录 → [wiki/](wiki/) | 原始资料 → [sources/](sources/)
- MiMoCode Obsidian 链接：Junction/symlink 链接方法 → [[sources/claude-code/claude-code-setup-references.md#MiMoCode Obsidian 链接方法|详情]]
