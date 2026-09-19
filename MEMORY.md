# Memory

## 管理规则
- 本文件是**文件记忆入口**：服务通过目录 Junction/链接把本 vault 用作项目记忆的场景（如 obsidian-link 建立的链接），维护少量必要提示与长期有效的知识指针，不是全库索引。
- 行数上限 200 行；条目单行不超过 150 字符，格式：`- {场景}：{摘要} → [[wiki/xxx.md|详情]]([wiki/xxx.md](wiki/xxx.md))`。
- 只保存跨会话长期有效的偏好与经验指针，不保存进度性内容；知识库操作以本 vault 的 AGENTS.md 为准，本入口不替代外部项目自身的规则。
- 查找方式：环境有可用 wiki MCP 时优先 `wiki_search`；无 MCP 时经 [README.md](README.md) 按领域定位详情，必要时允许文件搜索（rg / glob）。

## 个人习惯
- 语言：中文
- 调试：先观察再修改，三段式可观测性 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- 方法论：Harness Engineering → [[wiki/agent-arch.md|详情]]([wiki/agent-arch.md](wiki/agent-arch.md))
- 子代理：可并行或需隔离时用；简单操作直接做
- 学习习惯：学霸七法（框架/反向/深层/逻辑/分科/渐进/理解>数量）+AI辅助 → [[wiki/learning.md|详情]]([wiki/learning.md](wiki/learning.md))
- 技术栈：原型 HTML/CSS/JS，正式 Flutter，后端 Python/Rust → [[wiki/frontend.md|详情]]([wiki/frontend.md](wiki/frontend.md))
- 开发方法：默认 TDD，代码质量由 AI 发现最佳实践 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- 编程认知：必须用四层分离法（设计/验证/诊断/决策），禁止 vibe coding → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))

## 经验教训
- Claude Code：技能存放/SKILL开发/文档双链/Hooks复盘 → [[wiki/claude-code.md|详情]]([wiki/claude-code.md](wiki/claude-code.md))
- 开发实践：提示词缓存/协作技巧、rm 安全操作 → [[wiki/dev-practice.md|详情]]([wiki/dev-practice.md](wiki/dev-practice.md))
- Learn ShareAI：s01-s19 从零构建 Agent，各章摘要、关键数据结构、实现要点 → [[wiki/learn-shareai.md|详情]](wiki/learn-shareai.md)
- 上下文管理：当前完整保留，旧的才压缩（截断+落盘→占位符→摘要）→ [[wiki/agent-arch.md|详情]]([wiki/agent-arch.md](wiki/agent-arch.md))
- 决策：跨学科思维模型框架（芒格100模型），高频模型含反向思考/概率思维/二阶思维 → [[wiki/mental-models.md|详情]]([wiki/mental-models.md](wiki/mental-models.md))
- 凭证/Provider：token 集中管理在 D:\Files\.env；MiMo 用 AUTH_TOKEN 非 API_KEY，域名 xiaomimimo.com → [[wiki/token-management.md|详情]](wiki/token-management.md)
- Yazi：WinGet 安装需重启终端；`file.exe` 必须在 PATH（Git usr/bin）；`ya pkg` 缺 LICENSE 需手动部署 → [[wiki/yazi.md|详情]]([wiki/yazi.md](wiki/yazi.md))
- 技能链调用：技能内引用其他技能时必须用 Skill 工具加载完整内容，不能跳过 → [[wiki/claude-code.md|详情]](wiki/claude-code.md)
- 子代理技能注入：无法调用 Skill 工具，父代理预加载内容注入 prompt → [[sources/claude-code/subagent-skill-test-2026-05-10.md|详情]]
- Python 邮件开发：IMAP/SMTP asyncio 踩坑（BODY.PEEK、multipart 递归、QQ 邮箱特性） → [[wiki/email-assistant.md|详情]](wiki/email-assistant.md)
- CC 进阶工作流：上下文管理/Hooks/子代理/并行 + Session Memory/Token经济学/Evals/持续学习 → [[wiki/claude-code.md|详情]](wiki/claude-code.md)
- 新项目初始化：Docker 容器 → 验证镜像配置 → 补充缺失项（Git/CLAUDE.md/ECC rules/目录结构） → [[wiki/new-project-setup.md|详情]](wiki/new-project-setup.md)
- Rust 安装：容器用 USTC 镜像脚本；Windows 免 VS 选 GNU 工具链 `x86_64-pc-windows-gnu` → [[sources/dev/rust.md|详情]](sources/dev/rust.md)
- OpenCode 桥接 v2：log 修复/parseHookOutput 写回/permission.asked；input.tool 是字符串 → [[wiki/opencode-bridge.md|详情]]
- ECC→OpenCode 集成：agent frontmatter 须转换（model 删除、tools 数组→对象、color 映射） → [[wiki/opencode-bridge.md|详情]]
- OpenCode 代理配置：opencode.jsonc 的 agent 部分添加/覆盖代理，mode:"primary" 在切换列表 → [[wiki/opencode-bridge.md|详情]]
- MiMo-Code 源码阅读：4 模块 19 章路线（核心闭环→系统加固→任务运行时→多Agent） → [[sources/projects/mimocode-reading-guide.md|详情]]
- cc switch 重置：切 KEY 时旧模板重写 settings.json，通用配置已修正 → [[sources/claude-code/claude-code-setup-references.md#cc switch 通用配置模板|详情]]
- 缓存优化：`cch` 哈希变化致命中率低，`CLAUDE_CODE_ATTRIBUTION_HEADER=0` 可部分解决 → [[sources/claude-code/claude-code-setup-references.md#缓存优化（Prompt Caching）|详情]]
- wiki-query MCP：exe 移出 target 解耦；ZCode 读 `~/.zcode/cli/config.json` 的 mcp.servers；GitHub Actions 推 tag 触发 → [[wiki/wiki-query-deploy.md|详情]]

## 知识库索引
- 本文件未覆盖的内容 → [README.md](README.md)（导航索引，按领域查找）
- Wiki 目录 → [wiki/](wiki/) | 原始资料 → [sources/](sources/)
- MiMoCode Obsidian 链接：Junction/symlink 方法 → [[sources/claude-code/claude-code-setup-references.md#MiMoCode Obsidian 链接方法|详情]]
