# Memory

## 管理规则
- 此文件超过 200 行会被截断，每条保持单行，不超过 150 字符。
- 只保存常用的、跨会话的用户个人习惯和偏好。
- wiki/ 里的文档必须有 yaml 元数据：type/name/description/updated/tags，双链统一带 `.md` 扩展名。
- 知识库三层架构：sources/（只读）→ wiki/（可读写）→ CLAUDE.md（行为规则），详见 [LLM-WIKI-ARCHITECTURE](../../LLM-WIKI-ARCHITECTURE.md)。
- 个人习惯和经验教训条目格式：`- {场景}：{摘要} → [[wiki/xxx.md|详情]](...)`。
- wiki 按问题领域组织，同一领域写进同一个文档，超 600 行拆分。
- 记忆查找优先级：MEMORY.md（热，始终在上下文中）→ README.md（冷，MEMORY.md 未命中时查阅）。

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
- 智谱MCP：web reader 可能遗漏JS内容或返回缓存旧版，用 curl+grep 或 no_cache=true → [[wiki/zhipu-mcp.md|详情]]([wiki/zhipu-mcp.md](wiki/zhipu-mcp.md))
- Claude Code：自定义技能放 ~/.claude/skills/<name>/SKILL.md → [[wiki/claude-code-p1.md|详情]]([wiki/claude-code-p1.md](wiki/claude-code-p1.md))
- 首次配置：插件/SafetyNet/StatusLine/智谱MCP 完整安装步骤 → [[wiki/claude-code-p2.md|详情]]([wiki/claude-code-p2.md](wiki/claude-code-p2.md))
- 提示词：固定前置动态后置缓存命中；协作用情境简报/魔鬼代言人/范围锁定等 10 技巧 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- 文档：CLAUDE.md 稳定前置 + 双链带 .md → [[wiki/claude-code-p1.md|详情]]([wiki/claude-code-p1.md](wiki/claude-code-p1.md))
- 删除操作：rm -rf 和 find -delete 前先预览，Safety Net 会阻止批量删除 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- 上下文管理：当前完整保留，旧的才压缩（截断+落盘→占位符→摘要）→ [[wiki/agent-arch-p1.md|详情]]([wiki/agent-arch-p1.md](wiki/agent-arch-p1.md))
- Hooks：V4 关键词"复盘"触发后台 agent 复盘，无需 hook 配置（V3 因评估器过度触发已废弃）→ [[wiki/claude-code-p1.md|详情]]([wiki/claude-code-p1.md](wiki/claude-code-p1.md))
- 决策：跨学科思维模型框架（芒格100模型），高频模型含反向思考/概率思维/二阶思维 → [[wiki/mental-models.md|详情]]([wiki/mental-models.md](wiki/mental-models.md))
- Git 认证：remote URL 禁止明文 token，用 credential store + .env 读取 → [[wiki/git-security.md|详情]]([wiki/git-security.md](wiki/git-security.md))
- Safety Net 状态栏：管道接入 `statusline.py | npx cc-safety-net --statusline`，Windows 需设 SAFETY_NET_PLUGIN_ROOT → [[wiki/claude-code-p2.md|详情]]([wiki/claude-code-p2.md](wiki/claude-code-p2.md))

## 知识库索引
- 本文件未覆盖的内容 → [README.md](README.md)（冷数据，按领域导航）
- Wiki 目录 → [wiki/](wiki/) | 原始资料 → [sources/](sources/)
