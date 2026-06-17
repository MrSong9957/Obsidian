---
name: Claude Code 配置详细参考
description: Claude Code 全部详细参考——CLAUDE.md/rules/Memory/StatusLine/诊断流程/子代理/Hooks/Token经济学/进阶技术
type: reference
updated: 2026-06-17
tags: claude-code, settings, claudemd, memory, rules
source: https://code.claude.com/docs/en/memory
---

# Claude Code 配置详细参考

> 入口文档：[[sources/claude-code/claude-code.md|Claude Code 配置核心]]

---

## CLAUDE.md 完整指南

> 官方文档：https://code.claude.com/docs/en/memory
> 实践参考：https://joseparreogarcia.substack.com/p/how-claude-code-rules-actually-work

### 三级作用域

| 作用域 | 位置 | 优先级 | 用途 |
|--------|------|--------|------|
| 组织级 | Windows: `C:\Program Files\ClaudeCode\CLAUDE.md` | 最高 | 公司级编码标准、安全政策 |
| 用户级 | `~/.claude/CLAUDE.md` | 中 | 个人偏好，跨项目生效 |
| 项目级 | `./CLAUDE.md` 或 `./.claude/CLAUDE.md` | 低 | 项目架构、构建命令、团队约定 |

### 加载机制

**每次会话开始时完整加载。** 与 MEMORY.md 的 200 行限制不同，CLAUDE.md 没有截断限制。

```
1. 从工作目录向上遍历目录树，加载每层的 CLAUDE.md
2. 子目录的 CLAUDE.md 在 Claude 读取该子目录文件时按需加载
3. 用户级 ~/.claude/CLAUDE.md 始终加载
4. 组织级 CLAUDE.md 始终加载，且无法被排除
```

#### Compaction 后的行为

**CLAUDE.md 完整存活于 compaction。** 执行 `/compact` 后，Claude 会从磁盘重新读取并重新注入上下文。

> 官方原话："CLAUDE.md fully survives compaction. After /compact, Claude re-reads your CLAUDE.md from disk and re-injects it fresh."

### 导入机制

用 `@path` 语法引用其他文件：

```markdown
见 @README.md 了解项目概述
@~/.claude/my-project-instructions.md
```

- 支持相对路径和绝对路径
- 最多递归 5 层导入

### 最佳使用方法

#### 写什么

| 适合写 | 不适合写 |
|--------|---------|
| 编码规范（缩进、类型要求） | 逐步骤工作流 |
| 构建和测试命令 | 只在特定目录才适用的规则 |
| 架构决策和项目结构 | 实验性或经常变化的实践 |
| 命名约定 | 外部文档的完整内容 |

#### 写作原则

1. **精简**：每个文件不超过 200 行
2. **具体**：可验证的指令（"使用 2 空格缩进"而非"格式化代码"）
3. **结构化**：用 Markdown 标题和列表
4. **无矛盾**：定期审查，用 `/memory` 检查
5. **注释给人类看**：HTML 注释会被 Claude 过滤，不消耗上下文

#### CLAUDE.md 作为索引

在项目 CLAUDE.md 中写"路线图"，引导 Claude 在不同场景下查阅不同规则：

```markdown
## 如何在此仓库工作

### 实验性工作
- 查阅 `.claude/rules/experiments.md`

### 生产代码修改
- `src/models/` 和 `src/pipelines/` 下有严格的 scoped rules
```

### 大型项目技巧

#### monorepo 排除不相关文件

```json
// .claude/settings.local.json
{
  "claudeMdExcludes": [
    "**/other-team/CLAUDE.md",
    "/home/user/monorepo/legacy/.claude/rules/**"
  ]
}
```

#### 子目录 CLAUDE.md

子目录中的 CLAUDE.md 不会在启动时加载，只有当 Claude 读取该子目录的文件时才加载：

```
project/
├── CLAUDE.md              # 项目级（启动时加载）
├── src/
│   ├── billing/
│   │   └── CLAUDE.md      # 只在编辑 billing/ 时加载
│   └── auth/
│       └── CLAUDE.md      # 只在编辑 auth/ 时加载
```

### CLAUDE.md 排错

| 问题 | 检查方法 |
|------|---------|
| Claude 没遵守指令 | 运行 `/memory` 确认文件已加载 |
| 指令 compaction 后消失 | 该指令只存在于对话中，未写入 CLAUDE.md |
| 两个指令互相冲突 | 检查全局 + 项目 + 子目录的所有 CLAUDE.md |
| 指令太长被忽略 | 缩短到 200 行以内，拆分到 rules/ |

---

## .claude/rules/ 完整指南

### 两种规则

#### 无条件规则（Unconditional）

没有 `paths:` frontmatter 的文件，**启动时全部加载**。

```markdown
# .claude/rules/security.md
- 提交前检查无硬编码密钥
- 所有用户输入必须验证
```

#### Scoped 规则（路径条件）

带 `paths:` frontmatter 的文件，**只在 Claude 处理匹配文件时加载**。

```markdown
---
paths:
  - "notebooks/**"
---
# Notebook 规则
- Notebook 是探索性的，不是生产代码
```

> 官方原话："Path-scoped rules trigger when Claude reads files matching the pattern, not on every tool use."

### Glob 模式参考

| 模式 | 匹配 |
|------|------|
| `**/*.ts` | 任何目录下的 TypeScript 文件 |
| `src/**/*` | src/ 下所有文件 |
| `*.md` | 项目根目录的 Markdown 文件 |
| `src/components/*.tsx` | 指定目录的 React 组件 |
| `src/**/*.{ts,tsx}` | src/ 下所有 TS/TSX 文件 |
| `tests/**/*.test.ts` | 测试文件 |

支持多模式组合：

```yaml
paths:
  - "src/models/churn/**"
  - "src/pipelines/**"
  - "tests/**/*.test.ts"
```

### 目录结构

```
your-project/
├── CLAUDE.md                  # 主项目指令（入口）
├── .claude/
│   ├── CLAUDE.md              # 等同于 ./CLAUDE.md
│   └── rules/
│       ├── python-style.md    # 无条件：始终加载
│       ├── testing.md         # 无条件：始终加载
│       ├── security.md        # 无条件：始终加载
│       ├── notebooks.md       # Scoped：notebooks/ 时加载
│       ├── production.md      # Scoped：生产代码时加载
│       └── ml/                # 支持子目录组织
│           ├── training.md
│           └── pipelines.md
```

### 最佳使用方法

#### 无条件规则：放什么

只放**不管在做什么都适用的规则**：

| 适合 | 不适合 |
|------|--------|
| 基础编码标准（格式、类型） | 特定目录的工作流 |
| 安全硬要求（密钥检查、输入验证） | ML 建模选择 |
| 测试覆盖率要求 | 实验性实践 |

**反模式：** 把所有规则都写成无条件的。等于把问题从 CLAUDE.md 搬到了 rules/。

#### Scoped 规则：放什么

放**只在特定文件/目录下才有意义的规则**：

| 场景 | paths | 内容 |
|------|-------|------|
| Notebook 开发 | `notebooks/**` | 探索性、原型、松约束 |
| 生产 ML 代码 | `src/models/**`, `src/pipelines/**` | 严格测试、可重现 |
| API 端点 | `src/api/**/*.ts` | 输入验证、错误格式 |
| 前端组件 | `src/components/*.tsx` | 设计系统、无障碍要求 |

#### 符号链接共享规则

```bash
ln -s ~/shared-claude-rules .claude/rules/shared
ln -s ~/company-standards/security.md .claude/rules/security.md
```

### 什么情况不该用 Rules

| 情况 | 原因 | 替代方案 |
|------|------|---------|
| 项目很小，规则不到 50 行 | 直接写 CLAUDE.md 更简单 | 不拆分 |
| 需要真正强制执行的约束 | Rules 是上下文不是强制 | 用 managed settings |
| 规则只对当前对话有用 | 不需要跨会话持久化 | 直接在对话中说 |

### Rules 排错

| 问题 | 原因 | 解决 |
|------|------|------|
| Scoped 规则没触发 | Claude 没有读取匹配文件 | 在 CLAUDE.md 中加引导指向 |
| 规则被忽略 | 指令太模糊 | 写具体可验证的指令 |
| 多个规则冲突 | 不同文件对同一行为给出不同指导 | 合并或明确覆盖关系 |
| 不知道加载了什么 | 无法确认 | 运行 `/memory` 查看 |

---

## Auto Memory 完整指南

### 存储位置

```
~/.claude/projects/<project>/memory/
├── MEMORY.md            # 索引入口，每次会话加载
├── debugging.md         # 主题文件，按需加载
├── api-conventions.md   # 主题文件，按需加载
└── ...
```

- `<project>` 路径由 git 仓库根目录推导
- 同一仓库的所有 worktree 共享一个 memory 目录
- Auto Memory 是**机器本地**的，不会跨设备同步

#### 自定义存储位置

```json
{
  "autoMemoryDirectory": "~/my-custom-memory-dir"
}
```

> 注意：此设置**不能**在项目 settings 中配置，防止共享项目重定向写入路径。

### 加载规则

| 文件 | 加载时机 | 限制 |
|------|---------|------|
| MEMORY.md | 会话开始时 | 前 200 行或 25KB |
| 主题文件 | Claude 按需 Read | 无限制 |

### Compaction 行为对比

| 场景 | CLAUDE.md | MEMORY.md |
|------|-----------|-----------|
| 会话开始 | 完整加载 | 前 200 行/25KB |
| `/compact` 后 | 从磁盘重新加载，完整恢复 | 已压缩的内容丢失 |

### 自动保存机制

Claude **不是**每次会话都保存。典型触发场景：
- 你说"记住这个"或"以后都这样做"
- 你纠正了 Claude 的做法
- Claude 了解到你的角色或偏好

### 开关控制

```json
// 方式 1：项目 settings
{ "autoMemoryEnabled": false }

// 方式 2：环境变量
CLAUDE_CODE_DISABLE_AUTO_MEMORY=1

// 方式 3：会话中运行 /memory，切换开关
```

### MEMORY.md 索引设计要点

1. **索引保持精简** — 每条一行，控制在 200 行以内
2. **详细信息放主题文件** — Claude 会按需用 Read 工具读取
3. **索引格式**：`- [标题](文件名) — 一句话摘要`
4. **定期清理** — 过时的记忆直接删除

#### 不该存什么

| 不要存 | 原因 |
|--------|------|
| 代码模式/架构 | 直接读代码就知道 |
| Git 历史 | `git log` 是权威来源 |
| 调试方案 | 修复已经在代码和 commit 里了 |
| 已在 CLAUDE.md 中的规则 | 重复浪费上下文空间 |

#### 适合存什么

| 适合存 | 原因 |
|--------|------|
| 你纠正过 Claude 的事 | 防止下次犯同样的错 |
| 你的沟通偏好 | "中文"、"原理优先" |
| 项目中不明显的关键上下文 | "这个 API key 下周过期" |

---

## 记忆系统 Active Recall 详解

### 四阶段筛选

每轮对话依次执行：**扫描 → 摘要 → 选择 → 注入**

1. **扫描阶段**：最多取 200 个记忆文件，仅读前 30 行（覆盖 frontmatter 元数据，成本极低）
2. **摘要阶段**：将元数据整合成记忆清单
3. **选择阶段**：用 Sonnet 模型选最多 **5 条**相关记忆，宁缺毋滥
4. **注入阶段**：读取选中文件内容，单文件超 4000 字节会截断并附提示

### 容量管控

| 维度 | 上限 | 说明 |
|------|------|------|
| 单轮注入 | 5 条共 20,000 字节 | 防单轮记忆占比过高 |
| 会话累计 | 60,000 字节 | 防上下文被记忆撑爆 |
| 压缩后 | 计数器重置 | 可重新注入已用过的记忆 |

### 去重与过期

- 同一会话中已注入的记忆文件会提前从候选集中过滤
- 注入超过 **2 天**的记忆时自动附加提醒：记忆为时间快照，可能过时

### 架构哲学

**「元数据粗筛 + 便宜模型精选」**两阶段架构，将记忆调用成本降低一个数量级，无需向量数据库或复杂相似度计算。

实践启示：
- 大量记忆时用 frontmatter 做低成本初筛
- 便宜模型做精选，贵的模型做决策
- 硬性上限防止单一维度膨胀

---

## StatusLine 配置详解

- stdin JSON 字段：`model.display_name` / `workspace.current_dir` / `context_window.used_percentage`
- 配置：settings.json 中 `statusLine.type` = command 或 template
- 推荐 Python 版本（比 Bash 版本更稳定）；脚本放 `~/.claude/scripts/statusline.py`
- 多状态栏合并：用管道 `py statusline.py | npx cc-safety-net --statusline`（Windows 必须用 `py`，`python` 返回 exit code 49）

### Worktree 兼容

- **分支检测 bug：** `get_git_branch()` 用 `os.path.isdir(".git")` 查找仓库，但 worktree 中 `.git` 是文件（指向主仓库），导致跳过 worktree 向上找到主仓库，返回错误分支。修复：改为 `isdir(...) or isfile(...)`
- **目录显示：** worktree 路径 `<project>/.claude/worktrees/<name>/` 过长且无意义，检测后压缩为 `项目名:worktree名` 格式（如 `project:persona-style`）
- **普通目录：** 只保留最后 2 层（如 `~/a/b/c` → `b/c`），不足 2 层保持原样

## Safety Net 详细配置

- Hook 机制在命令执行前拦截，CLAUDE.md 的"软"规则无法替代"硬"技术约束
- 拦截：git checkout -- / git reset --hard / git push --force / rm -rf（cwd 外）/ git branch -D / find -delete
- 放行：git checkout -b / git branch -d / git push --force-with-lease / rm -rf ./...
- 绕过方式：Safety Net 阻止批量删除时，先预览目标列表（如 `find ... -print`），再逐个删除
- 高级：Shell 包装检测、解释器单行检测、审计日志、自定义规则（`~/.cc-safety-net/config.json`）
- 安装：`/plugin install safety-net@cc-marketplace` → 配置 PreToolUse Hook
- Hook 参数：v0.9.0+ 用 `--claude-code`（从 stdin 读 JSON），旧参数 `--hook "$TOOL_INPUT"` 已废弃 ← [[sources/claude-code/2026-05-13-safety-net-hook-fix.md]] L32-46

## 切换 API KEY 后诊断流程

切换 `ANTHROPIC_AUTH_TOKEN` 后按以下优先级排查，99% 问题出在 MCP token：

### 1. MCP 断连（99% 是 token 问题）

**症状：** `claude mcp list` 显示服务器 Disconnected

**快速修复：** 智谱 HTTP MCP 和 zai-mcp-server 已配置为 `${ANTHROPIC_AUTH_TOKEN}` 环境变量引用，切换后无需手动更新。验证：`claude mcp list` 确认全部 Connected。

### 2. StatusLine 失效（Windows `py` vs `python`）

Windows 上 `python` 命令返回 exit code 49（不存在），必须用 `py`。确认 settings.json 中 `statusLine.command` 使用 `py statusline.py`。

### 3. Hooks 报错

ECC 插件升级后 `plugins/cache/` 路径变化，settings.json 中旧 hooks 失效。ECC v2+ 自带 `hooks/hooks.json` 自解析路径，settings.json 只保留 Safety Net 和 block-no-verify。

## GitHub MCP 插件认证

**问题：** `github@claude-plugins-official` 报错 `Incompatible auth server: does not support dynamic client registration`

**解决：** 在 `~/.claude/settings.json` 的 `env` 中添加 `"GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxx"`，`/reload-plugins` 后生效

## 创建自定义子代理详解

详见 [[sources/claude-code/creating-subagents.md|创建自定义 Subagents 指南]]。

### Frontmatter 核心参数

| 参数 | 必填 | 说明 |
|------|------|------|
| `name` | 是 | 唯一标识符，小写字母和连字符 |
| `description` | 是 | **最关键** — Claude 据此决定何时自动委托任务 |
| `tools` | 否 | 允许的工具列表（限制工具 = 聚焦执行） |
| `model` | 否 | `sonnet`（复杂推理）或 `haiku`（简单研究，快速便宜） |
| `memory` | 否 | `project` 或 `user`，开启持久内存 |

### 五大效率提升特性

1. **隔离高容量操作** — 委托给 subagent 在独立窗口处理，只返回摘要
2. **路径规则按需加载** — `.claude/rules/` 配合 `paths` 字段按需加载
3. **持久内存** — `memory: project` 让代理跨会话记住项目架构
4. **显式调用** — `@code-reviewer` 直接启动特定代理
5. **灵活搭配模型** — 简单研究任务用 `haiku`，速度快费用低

## 子代理 Skill 工具可用性

**结论：** 子代理和团队成员都不能使用 Skill 工具。父代理必须预加载技能内容注入 prompt。 ← [[sources/claude-code/subagent-skill-test-2026-05-10.md]]

**注入方式：** 父代理先用 Skill 加载技能完整内容，用 `---<SKILL_NAME> START/END---` 标记包裹注入子代理 prompt。

**失败的方案（勿再尝试）：** ToolSearch 加载 Skill → 假阳性；`--allowedTools Skill` → 参数被忽略；`--plugin-dir superpowers/` → 插件提供内容不提供工具。

## Hook 类型选择

- prompt hook 返回 `{"ok": false, "reason": "..."}` 可让主 Claude 继续工作，拥有完整对话上下文和全部工具权限
- command hook + `claude -p` 子进程需读 transcript 重建上下文，有冷启动和终端乱码风险
- 结论：Stop 事件中需利用主对话上下文的任务（如复盘），优先用 prompt hook

### Stop Hook 演进：V1 → V4

- **V1**（command hook + claude -p）：子进程需从 transcript 重建上下文，冷启动慢，终端乱码
- **V2**（prompt hook + 主对话内复盘）：不阻塞但消耗主对话 token
- **V3**（prompt hook + 后台 agent）：不阻塞、零 token 消耗，但评估器过度触发
- **V4**（关键词触发 + 后台 agent，当前方案）：移除 Stop hook，用户输入"复盘"关键词时启动后台 agent
- **教训**：自动触发的 hook 评估器难以准确判断对话是否有复盘价值，用户主动触发更可靠
- **设计文档**：[[sources/claude-code/stop-hook-design.md|完整设计文档]]

## 进阶技术详解

### Session Memory 持久化

- 跨会话恢复：`~/.claude/sessions/YYYY-MM-DD-topic.tmp`，每会话一个文件
- 文件内容：有效方法（附证据）+ 失败方法 + 未尝试方法 + 剩余待办
- 触发时机：撞上下文限制、天结束时保存、次日提供路径恢复

### Strategic Compacting

- 禁用自动压缩，在逻辑节点手动 `/compact`：探索→执行切换时、里程碑完成后
- 计数器 Hook（绑定 PreToolUse Edit/Write）：达 50 次工具调用时提醒考虑压缩

### Dynamic System Prompt Injection

- `claude --system-prompt "$(cat dev.md)"` 按 CLI 别名切换场景（dev/review/research）
- **指令权重**：System Prompt > User Messages > Tool Results，比 `.claude/rules/` 更高优先级

### Continuous Learning

- **Stop Hook 自动评估**：会话结束时提取模式（错误方案、调试技巧、变通方案），保存到 `~/.claude/skills/learned/`
- **`/learn` 手动提取**：会话中途随时运行，即时提取刚解决的模式
- **用 Stop 而非 UserPromptSubmit**：后者每条消息触发，开销大；Stop 只触发一次

### Token 经济学

- **Haiku + Opus 组合**性价比最高（5x 价差），Sonnet + Opus 只有 1.67x 不值得
- Sonnet 做 90% 编码；首次失败/跨 5+ 文件/架构决策 → Opus；重复/明确/worker → Haiku
- mgrep 替代 grep/ripgrep 减少约 50% token
- System Prompt 瘦身可从 ~18k 减至 ~10k tokens

### Sub-Agent 上下文问题

- 子代理只有查询字面意思，缺乏背后的目的和推理 → 摘要常遗漏关键细节
- **迭代检索模式**：评估返回值 → 不充分则追问 → 最多 3 轮循环
- **传递目标上下文**，不只是查询——帮子代理判断该包含什么

### 序列阶段编排

```
Research → Plan → Implement → Review → Verify
```
- 每个 agent 一个输入一个输出，输出存文件不靠记忆，agent 间 `/clear` 刷新上下文

### Agent Abstraction 分层

- **Tier 1（直接增益）**：Subagents、Metaprompting、开头多问用户
- **Tier 2（高门槛）**：Long-running agents、Parallel multi-agent、Role-based、Computer use
- 从 Tier 1 开始，有真实需求才进阶 Tier 2

### Evaluations 体系

- **Checkpoint-Based**：线性工作流，每个检查点验证后继续；**Continuous**：每 N 分钟运行测试套件
- **Grader**：Code-Based（快/客观/脆弱）、Model-Based（灵活/贵/不稳定）、Human（金标准/慢/贵）
- **pass@k**（至少一次成功）vs **pass^k**（全部成功）

## 常用设置速查表

完整设置清单见 [[sources/claude-code/settings-reference.md|Claude Code 完整设置清单]]（100+ 配置项）。

### UI 与显示
| 设置 | 说明 | 推荐值 |
|------|------|--------|
| `theme` | 颜色主题 | `auto` |
| `editorMode` | 键盘模式 | `normal` 或 `vim` |
| `verbose` | 显示完整工具输出 | `false` |
| `showTurnDuration` | 显示每轮对话耗时 | `false` |
| `showMessageTimestamps` | 显示消息时间戳 | `false` |
| `tui` | 终端 UI 渲染器 | `default` 或 `fullscreen` |
| `showClearContextOnPlanAccept` | 计划审批时显示"清除上下文"选项 | `true` |

### 模型与性能
| 设置 | 说明 | 推荐值 |
|------|------|--------|
| `model` | 默认模型 | - |
| `alwaysThinkingEnabled` | 启用扩展思考 | `true` |
| `effortLevel` | 思考深度 | — |
| `fastMode` | 快速模式 | `false` |
| `autoCompactEnabled` | 上下文满时自动压缩 | `true` |

### 权限与安全
| 设置 | 说明 | 推荐值 |
|------|------|--------|
| `permissions.defaultMode` | 默认权限模式 | `default` |
| `permissions.allow` | 允许的工具规则列表 | `["Bash(npm *)", "Bash(git *)"]` |
| `permissions.deny` | 拒绝的工具规则列表 | `["Bash(rm -rf *)"]` |

### 记忆与持久化
| 设置 | 说明 | 推荐值 |
|------|------|--------|
| `autoMemoryEnabled` | 启用自动记忆 | `true` |
| `autoMemoryDirectory` | 自定义记忆存储路径 | - |
| `fileCheckpointingEnabled` | 编辑前快照文件（支持 /rewind） | `true` |

### 通知与语音
| 设置 | 说明 | 推荐值 |
|------|------|--------|
| `preferredNotifChannel` | 首选通知渠道 | `auto` |
| `inputNeededNotifEnabled` | 权限提示时推送通知 | `true` |
| `voiceEnabled` | 启用语音模式 | `false` |

### 自动模式
| 设置 | 说明 | 推荐值 |
|------|------|--------|
| `permissions.defaultMode` | 默认权限模式 | `auto` |
| `useAutoModeDuringPlan` | 计划模式使用自动模式语义 | `true` |

## CLAUDE.md KV 缓存编排

- 稳定不变内容前置，易变项目信息后置；修改项目部分不会使前面缓存失效
- 删除可通过 ls/read 推导的内容，只保留无法推导的显式约束
- 双链统一带 `.md` 扩展名：`[[wiki/dev-practice.md|TDD 实践]]`

## MCP → Skills 替代

- 很多 MCP 是 CLI 封装（GitHub/Supabase/Vercel），用 skills+commands 包装 CLI 调用可省上下文
- Lazy Loading 已解决启动时上下文占用，但 **token 使用和成本**仍未解决
- CLI + skills 方式在重操作（DB 查询、部署）时仍显著省 token

## 安装教训

- Safety Net：Windows 需手动配 Hook 和 `SAFETY_NET_PLUGIN_ROOT` 环境变量
- 任何插件安装前备份：`cp -r ~/.claude/rules ~/.claude/rules.backup`
- Rules 和 Hooks 都是追加合并，绝不能直接覆盖
