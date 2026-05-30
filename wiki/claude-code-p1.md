---
name: Claude Code 配置
description: settings.json/CLAUDE.md/rules/记忆系统/技能位置 — 遇到Claude Code配置、持久化指令、记忆系统问题时参考
type: concept
updated: 2026-05-30
tags: claude-code, settings, claudemd, memory, rules, reference
source: https://example.com
---
# Claude Code 配置

> 续篇：[[wiki/claude-code-p2.md|Claude Code 工具]]

## settings.json 关键字段

- `showClearContextOnPlanAccept: true` — Plan 模式审批时显示清空上下文选项，释放窗口空间
- `autoMemoryEnabled: true` — 启用自动记忆
- `fileCheckpointingEnabled: true` — 编辑前自动快照，支持 `/rewind` 撤销
- `autoCompactEnabled: true` — 上下文满时自动压缩，避免会话中断
- `todoFeatureEnabled: true` — 启用任务追踪面板，多步骤任务更清晰
- `statusLine` — 状态栏配置（type: command/template）
- `claudeMdExcludes` — monorepo 排除不相关 CLAUDE.md
- `autoMemoryDirectory` — 自定义记忆存储位置（仅 user/local settings 可配）

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
| `effortLevel` | 思考深度 | `xhigh` |
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

### 通知
| 设置 | 说明 | 推荐值 |
|------|------|--------|
| `preferredNotifChannel` | 首选通知渠道 | `auto` |
| `inputNeededNotifEnabled` | 权限提示时推送通知 | `true` |

### 语音
| 设置 | 说明 | 推荐值 |
|------|------|--------|
| `voiceEnabled` | 启用语音模式 | `false` |
| `voice.mode` | 模式 | `hold` |

### 自动模式
| 设置 | 说明 | 推荐值 |
|------|------|--------|
| `permissions.defaultMode` | 默认权限模式 | `auto` |
| `useAutoModeDuringPlan` | 计划模式使用自动模式语义 | `true` |

## 常见配置场景

### 场景 1：开启自动记忆
```json
{
  "autoMemoryEnabled": true,
  "autoMemoryDirectory": "~/.claude/projects/my-project/memory"
}
```

### 场景 2：配置自动格式化 Hook
```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "prettier --write $FILE"
          }
        ]
      }
    ]
  }
}
```

### 场景 3：允许 npm 命令不弹确认
```json
{
  "permissions": {
    "allow": ["Bash(npm *)"]
  }
}
```

### 场景 4：开启快速模式和扩展思考
```json
{
  "fastMode": true,
  "alwaysThinkingEnabled": true,
  "effortLevel": "xhigh"
}
```

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

## Skill 开发与部署

- 存放路径：`~/.claude/skills/<skill-name>/SKILL.md`，放好后用 `/reload-plugins` 刷新生效
- **SKILL.md 结构**：纯自然语言指令，定义触发条件 → 操作流程（含 bash 命令模板）→ 输出格式（含 box-drawing 表格示例）→ 安全约束（发送前确认、不自动批量） ← [[sources/projects/email-assistant-journey.md]] L1-10
- **双目录部署**：项目目录（开发）和 `~/.claude/skills/`（运行）是两份独立副本，改代码后必须 `cp` 同步，否则 Skill 调用仍用旧版 ← [[sources/projects/email-assistant-journey.md]] L53-54
- **.env 加载策略**：Skill 的 `_load_dotenv()` 用 fallback 列表加载多个 .env，`os.environ.setdefault` 保证先加载的优先，解决部署目录和开发目录凭证分散问题 ← [[sources/projects/email-assistant-journey.md]] L71-75
- **技能链调用**：Skill 内引用其他 Skill 时必须用 Skill 工具加载完整内容，不能跳过
- **子代理限制**：子代理/团队成员无法调用 Skill 工具，父代理必须预加载技能内容并注入 prompt

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
- **设计文档**：[[sources/claude-code/stop-hook-design.md|完整设计文档]]
- **教训**：自动触发的 hook 评估器难以准确判断对话是否有复盘价值，用户主动触发（关键词）是更可靠的方案

## CLAUDE.md KV 缓存编排

- 稳定不变内容前置，易变项目信息后置；修改项目部分不会使前面缓存失效
- 删除可通过 ls/read 推导的内容，只保留无法推导的显式约束
- 双链统一带 `.md` 扩展名：`[[wiki/dev-practice.md|TDD 实践]]`

> 详细内容见 sources/claude-code/claude-code-p1.md
