---
name: Claude Code 配置
description: settings.json/CLAUDE.md/rules/记忆系统/技能位置 — 遇到Claude Code配置、持久化指令、记忆系统问题时参考
type: concept
updated: 2026-04-18
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
tags: [claude-code, settings, claudemd, memory, rules]
---
# Claude Code 配置

> 续篇：[[wiki/claude-code-p2.md|Claude Code 工具]]

## settings.json 关键字段

- `showClearContextOnPlanAccept: true` — Plan 模式审批时显示清空上下文选项，释放窗口空间
- `autoMemoryEnabled: false` — 关闭自动记忆
- `statusLine` — 状态栏配置（type: command/template）
- `claudeMdExcludes` — monorepo 排除不相关 CLAUDE.md
- `autoMemoryDirectory` — 自定义记忆存储位置（仅 user/local settings 可配）

## CLAUDE.md 结构规则

- 三级作用域：组织级 > 用户级 > 项目级（更具体的位置覆盖更宽泛的）
- 每次会话完整加载，无截断限制；compaction 后从磁盘重新读取并注入
- 每个文件不超过 200 行；超长内容用 `@path` 导入或拆分到 rules/
- 写具体可验证的指令，不写模糊指令；定期审查消除矛盾
- 子目录 CLAUDE.md 按需加载（读取该子目录文件时才触发）

## .claude/rules/ 使用结论

- 无条件规则：启动时全部加载，compaction 后重新加载
- Scoped 规则（带 `paths:`）：Claude 读取匹配路径文件时触发，不是每个工具调用
- 核心策略：CLAUDE.md 防全局误解，scoped rules 防局部错误
- 反模式：全部写成无条件规则 = 把 1000 行 CLAUDE.md 搬了个位置
- 验证：运行 `/memory` 或直接问 "你当前加载了哪些文件"

## 记忆系统行为

- MEMORY.md 加载前 200 行或 25KB，超限部分永远不加载；compaction 后不会重新注入
- 主题文件不启动加载，Claude 按 Read 工具按需读取
- 持久化优先级：Hooks > CLAUDE.md > 无条件 rules > scoped rules > MEMORY.md 前 200 行 > 主题文件 > 口头约定
- Active Recall 四阶段：扫描（200 文件前 30 行）→ 摘要 → Sonnet 选最多 5 条 → 注入（单文件 4KB 截断）
- 会话累计上限 60KB，单轮 20KB；压缩后计数器重置

## 技能存放位置

- 正确路径：`~/.claude/skills/<skill-name>/SKILL.md`
- 放好后用 `/reload-plugins` 刷新生效
- 项目 `skills/` 和 `~/.claude/plugins/` 都不是正确位置

## Hook 类型选择

- prompt hook 返回 `{"ok": false, "reason": "..."}` 可让主 Claude 继续工作，拥有完整对话上下文和全部工具权限
- command hook + `claude -p` 子进程需读 transcript 重建上下文，有冷启动和终端乱码风险
- stop_hook_active 机制防无限循环：Claude 继续→再停→hook 返回 `{"ok": true}`→结束
- 结论：Stop 事件中需利用主对话上下文的任务（如复盘），优先用 prompt hook

### Stop Hook 演进：V1 → V4

- **V1**（command hook + claude -p）：子进程需从 transcript 重建上下文，冷启动慢，终端乱码
- **V2**（prompt hook + 主对话内复盘）：不阻塞但消耗主对话 token
- **V3**（prompt hook + 后台 agent）：不阻塞、零 token 消耗，但评估器过度触发（几乎每次回复都启动后台 agent）+ GLM 评估器不严格输出 JSON 导致终端显示 "error"
- **V4**（关键词触发 + 后台 agent，当前方案）：移除 Stop hook，用户输入"复盘"关键词时 Claude 启动后台 agent
  - 零过度触发、零额外 API 调用、零配置（不需要 hook/settings.json）
  - 继承 V3 优势：不阻塞、零 token 消耗、完整工具权限
- **演进路线**：command+claude-p（V1）→ prompt hook ok:false（V2）→ prompt hook + 后台 agent（V3）→ 关键词触发 + 后台 agent（V4）
- **设计文档**：[[sources/stop-hook-design.md|完整设计文档]]
- **教训**：自动触发的 hook 评估器难以准确判断对话是否有复盘价值，用户主动触发（关键词）是更可靠的方案

## CLAUDE.md KV 缓存编排

- 稳定不变内容前置，易变项目信息后置；修改项目部分不会使前面缓存失效
- 删除可通过 ls/read 推导的内容，只保留无法推导的显式约束
- 双链统一带 `.md` 扩展名：`[[wiki/feedback_tdd.md|TDD 实践]]`

> 详细内容见 sources/claude-code-p1.md
