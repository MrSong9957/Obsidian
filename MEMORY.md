# Memory

## 管理规则
- 此文件超过 200 行会被截断，每条保持单行，不超过 150 字符。
- 只保存常用的、跨会话的用户个人习惯和偏好。
- wiki/ 里的文档必须有 yaml 元数据：type/name/description/updated/tags，双链统一带 `.md` 扩展名。
- 知识库三层架构：sources/（只读）→ wiki/（可读写）→ CLAUDE.md（行为规则），详见 [LLM-WIKI-ARCHITECTURE](../../LLM-WIKI-ARCHITECTURE.md)。
- 个人习惯和经验教训条目格式：`- {场景}：{摘要} → [[wiki/xxx.md|详情]](...)`。
- wiki 按问题领域组织，同一领域写进同一个文档，超 600 行拆分。
- 记忆查找流程（必须遵循）：① MEMORY.md 中查找 → ② 未命中则必须读 README.md 按领域导航查找 → ③ README.md 命中后读对应 wiki/ 文件。禁止跳过 README.md 直接 grep/find。

## 个人习惯
- 语言：中文
- 推进：小步迭代，每步可验证
- 调试：先观察再修改，三段式可观测性 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- 工作流：自主闭环，多次失败才请求介入
- 方法论：Harness Engineering → [[wiki/agent-arch-p2.md|详情]]([wiki/agent-arch-p2.md](wiki/agent-arch-p2.md))
- 原则：只做必要更改，不提前抽象，复用成熟方案
- 子代理：可并行或需隔离时用；简单操作直接做
- 代码讲解：底层逻辑优先，不跳过用户的问题 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- 技术栈：原型 HTML/CSS/JS，正式 Flutter，后端 Python/Rust → [[wiki/frontend.md|详情]]([wiki/frontend.md](wiki/frontend.md))
- 开发方法：默认 TDD，代码质量由 AI 发现最佳实践 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))

## 经验教训
- Claude Code：技能存放/SKILL开发/文档双链/Hooks复盘 → [[wiki/claude-code-p1.md|详情]]([wiki/claude-code-p1.md](wiki/claude-code-p1.md))
- 开发实践：提示词缓存/协作技巧、rm 安全操作 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- Learn ShareAI：s01-s19 从零构建 Agent，各章摘要、关键数据结构、实现要点见 → [[wiki/learn-shareai.md|详情]](wiki/learn-shareai.md)
- 上下文管理：当前完整保留，旧的才压缩（截断+落盘→占位符→摘要）→ [[wiki/agent-arch-p1.md|详情]]([wiki/agent-arch-p1.md](wiki/agent-arch-p1.md))
- 决策：跨学科思维模型框架（芒格100模型），高频模型含反向思考/概率思维/二阶思维 → [[wiki/mental-models.md|详情]]([wiki/mental-models.md](wiki/mental-models.md))
- 凭证：所有 token 集中管理在 D:\Files\.env，docker-compose 通过 env_file 引用；各服务获取方式见速查表 → [[wiki/token-management.md|详情]](wiki/token-management.md)
- Yazi：Windows 配置在 `%AppData%\yazi\config\`，非 Linux `~/.config/yazi/`；PowerShell 写多行 TOML 用数组语法 → [[wiki/yazi.md|详情]]([wiki/yazi.md](wiki/yazi.md))
- 技能链调用：技能内引用其他技能时必须用 Skill 工具加载完整内容，不能跳过 → [feedback-skill-chain-discipline](feedback-skill-chain-discipline.md)
- 子代理/团队成员技能注入：均无法调用 Skill 工具，父代理预加载内容注入 prompt → [feedback-subagent-skill-injection](feedback-subagent-skill-injection.md)
- 技能文件同步：改 project/email_cli.py 或 SKILL.md 后必须 cp 到 ~/.claude/skills/email-assistant/，否则技能调用用旧版 → [feedback-skill-file-sync](feedback-skill-file-sync.md)
- Python 邮件开发：IMAP/SMTP asyncio 踩坑（BODY.PEEK、multipart 递归、QQ 邮箱特性） → [[wiki/email-dev.md|详情]](wiki/email-dev.md)
- 项目路线图：三阶段递进 A 开源 Build in Public → B Skill 框架 → C SaaS，含 10 平台发布方案 → [[wiki/project-roadmap.md|详情]](wiki/project-roadmap.md)
- 技能触发纪律：邮件等关键词请求必须先 Skill 工具调用，不能直接执行 CLI → [feedback-skill-trigger-discipline](feedback-skill-trigger-discipline.md)
- CC 进阶工作流：上下文管理/Hooks/子代理/并行 + Session Memory/Token经济学/Evals/持续学习 → [[wiki/claude-code-p3.md|详情]](wiki/claude-code-p3.md)
- 邮件详情展示：末尾必须追加 webmail Link 行（按账户变化） → [feedback-email-detail-link](feedback-email-detail-link.md)
- 人格化邮件回复：代码完成待实测，分支 worktree-persona-style，实测后执行 finishing-a-development-branch → [[sources/2026-05-14-persona-style-progress.md|详情]](sources/2026-05-14-persona-style-progress.md)
- 新项目初始化：Docker 容器 → 验证镜像配置 → 补充缺失项（Git/CLAUDE.md/ECC rules/目录结构） → [[wiki/new-project-setup.md|详情]](wiki/new-project-setup.md)

## 知识库索引
- 本文件未覆盖的内容 → [README.md](README.md)（冷数据，按领域导航）
- Wiki 目录 → [wiki/](wiki/) | 原始资料 → [sources/](sources/)
