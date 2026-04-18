# Memory

## 管理规则
1. 此文件超过 200 行会被截断，每条保持单行，不超过 150 字符。
2. 只保存常用的、跨会话的用户个人习惯和偏好，不保存文档索引。
3. 唯一的知识库索引是底部的 [README](README.md)，由用户管理。
4. wiki/ 里的文档必须有 yaml 元数据：type/name/description/updated/tags，双链统一带 `.md` 扩展名。
5. 知识库三层架构：sources/（只读）→ wiki/（可读写）→ CLAUDE.md（行为规则），详见 [LLM-WIKI-ARCHITECTURE](../../LLM-WIKI-ARCHITECTURE.md)。
6. 个人习惯和经验教训条目格式：`- {场景}：{摘要} → [[wiki/xxx.md|详情]](...)`。
7. wiki 按问题领域组织，同一领域写进同一个文档，超 600 行拆分。

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
- 网络：web reader 可能遗漏JS内容或返回缓存旧版，用 curl+grep 或 no_cache=true → [[wiki/network-api.md|详情]]([wiki/network-api.md](wiki/network-api.md))
- Claude Code：自定义技能放 ~/.claude/skills/<name>/SKILL.md → [[wiki/claude-code-p1.md|详情]]([wiki/claude-code-p1.md](wiki/claude-code-p1.md))
- 提示词：固定前置动态后置最大化 KV 缓存命中 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- 文档：CLAUDE.md 稳定前置 + 双链带 .md → [[wiki/claude-code-p1.md|详情]]([wiki/claude-code-p1.md](wiki/claude-code-p1.md))
- 删除操作：rm -rf 和 find -delete 前先预览，Safety Net 会阻止批量删除 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- 上下文管理：当前完整保留，旧的才压缩（截断+落盘→占位符→摘要）→ [[wiki/agent-arch-p1.md|详情]]([wiki/agent-arch-p1.md](wiki/agent-arch-p1.md))
- Hooks：V4 关键词"复盘"触发后台 agent 复盘，无需 hook 配置（V3 因评估器过度触发已废弃）→ [[wiki/claude-code-p1.md|详情]]([wiki/claude-code-p1.md](wiki/claude-code-p1.md))
- Skill 触发：description 含重叠关键词会竞争匹配，用户触发的改斜杠命令 → [[wiki/claude-code-p2.md|详情]]([wiki/claude-code-p2.md](wiki/claude-code-p2.md))

## 知识库索引
[README](README.md) | [Wiki 目录](wiki/) | [原始资料](sources/)
（在上面找不到的都在这里找）
