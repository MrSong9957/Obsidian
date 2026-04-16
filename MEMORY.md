# Memory

## 管理规则：
1. 此文件超过 200 行会被截断，每条保持单行，不超过 150 字符。
2. 只保存常用的、跨会话的用户个人习惯和偏好，不保存文档索引。
3. 唯一的知识库索引是底部的 [README](README.md)，由用户管理。
4. wiki/ 里的文档必须有 yaml 元数据：type/name/description/updated/tags（有来源时加 sources），双链统一带 `.md` 扩展名。
5. 知识库三层架构：sources/（只读）→ wiki/（可读写）→ CLAUDE.md（行为规则），详见 [LLM-WIKI-ARCHITECTURE](../../LLM-WIKI-ARCHITECTURE.md)。
6. 偏好条目如有对应 wiki 详档，行末加 → [[wiki/xxx.md|详情]]([wiki/xxx.md](wiki/xxx.md)) 链接。
7. feedback/project/reference 类详档放 wiki/，根目录只放 MEMORY.md、README.md、LLM-WIKI-ARCHITECTURE.md。

## 用户偏好
- 语言：中文
- 推进：小步迭代，每步可验证
- 调试：先观察再修改，用 before/after/failed 三段式事件日志建立可观测性再定位 → [[wiki/feedback_three_stage_observability.md|详情]]([wiki/feedback_three_stage_observability.md](wiki/feedback_three_stage_observability.md))
- 工作流：自主闭环，多次失败才请求介入
- 技术选型：复用成熟方案，不重复造轮子
- 方法论：Harness Engineering — 人类驾驶 Agent 执行，设计环境/意图/反馈循环 → [[wiki/harness-engineering.md|详情]]([wiki/harness-engineering.md](wiki/harness-engineering.md))
- 技术栈：原型 HTML/CSS/JS，正式前端 Flutter，后端默认 Python 高性能 Rust → [[wiki/tech_stack_strategy.md|详情]]([wiki/tech_stack_strategy.md](wiki/tech_stack_strategy.md))
- 原则：只做必要更改，不提前抽象
- 前端：避免 AI 泛滥美学 → [[wiki/frontend-aesthetics.md|详情]]([wiki/frontend-aesthetics.md](wiki/frontend-aesthetics.md))
- 子代理：可并行或需隔离时用；简单操作直接做
- 代码质量：AI 负责发现最佳实践 → [[wiki/code-quality.md|详情]]([wiki/code-quality.md](wiki/code-quality.md))
- 代码讲解：底层逻辑优先，遇新概念零基础教学，分析时从问题出发拆解 → [[wiki/feedback_code_teaching.md|详情]]([wiki/feedback_code_teaching.md](wiki/feedback_code_teaching.md))
- 开发方法：默认 TDD，安全/工具函数先写边界 case 再实现 → [[wiki/feedback_tdd.md|详情]]([wiki/feedback_tdd.md](wiki/feedback_tdd.md))
- 提示词编排：固定内容前置、动态内容后置，最大化 KV 缓存命中率 → [[wiki/kv-cache-prompt-design.md|详情]]([wiki/kv-cache-prompt-design.md](wiki/kv-cache-prompt-design.md))
- 文档编排：CLAUDE.md 稳定前置 + 双链统一带 `.md` → [[wiki/feedback_claudemd_layout.md|详情]]([wiki/feedback_claudemd_layout.md](wiki/feedback_claudemd_layout.md))

## 知识库索引
[README](README.md) | [Wiki 目录](wiki/) | [原始资料](sources/)
