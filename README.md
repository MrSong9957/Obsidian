# Obsidian 知识库

## 仓库概览

```
obsidian-vault/
├── README.md                    # 本文件 — 全仓库导航枢纽
├── Attachments/                 # 图片、PDF 等附件
├── Templates/                   # 笔记模板
├── Learning/                    # 学习笔记、知识整理
│   └── 科学提升智商：六大核心习惯与实践指南.md
├── References/                  # 长期参考材料（10 篇）
│   ├── Everything-Claude-Code.md
│   ├── ECC-Agent-Usage-Instructions.md
│   ├── Superpowers.md
│   ├── ecc-install.md
│   ├── statusline-install.md
│   ├── safety-net-guide.md
│   ├── ui-design.md
│   ├── playwright-cli-guide.md
│   ├── harness-engineering.md
│   └── terminal-ui-testing.md
├── claude-memory/               # Claude Code 全局记忆（6 篇）
│   ├── MEMORY.md                # 总索引
│   ├── multi-agent-framework.md
│   ├── terminal-ui-testing.md
│   ├── debugging-lessons.md
│   ├── frontend-aesthetics.md
│   └── code-quality.md
└── claude-code-system-prompts/  # Claude Code 系统提示词提取（110+ 篇）
    ├── README.md
    └── system-prompts/          # 按类型分类的提示词文件
```

---

## 快速导航

### 学习笔记

| 笔记 | 说明 |
|------|------|
| [[Learning/科学提升智商：六大核心习惯与实践指南\|科学提升智商]] | 六大核心习惯与实践指南 |

### 参考文档

#### Claude Code 技能系统

| 文档 | 说明 |
|------|------|
| [[References/Everything-Claude-Code\|ECC 技能集]] | 110+ Skills 完整索引 |
| [[References/ECC-Agent-Usage-Instructions\|ECC 智能体]] | 25 个专业智能体使用说明 |
| [[References/Superpowers\|Superpowers]] | 开发流程技能系统 |

#### 安装与配置

| 文档 | 说明 |
|------|------|
| [[References/ecc-install\|ECC 安装]] | Everything-Claude-Code 安装指南 |
| [[References/statusline-install\|状态栏]] | Claude Code 状态栏配置 |
| [[References/safety-net-guide\|Safety Net]] | Safety Net 安全插件指南 |

#### 开发指南

| 文档 | 说明 |
|------|------|
| [[References/ui-design\|UI 设计]] | 前端 UI 设计原则 |
| [[References/playwright-cli-guide\|Playwright]] | Playwright CLI 测试指南 |
| [[References/harness-engineering\|Harness Engineering]] | AI Agent-First 软件开发方法论 |
| [[References/terminal-ui-testing\|终端 UI 测试]] | VHS + AI 评估器自动化测试方案 |

### Claude 记忆

> 跨会话持久化的开发知识与偏好

| 文档 | 说明 |
|------|------|
| [[claude-memory/MEMORY\|MEMORY]] | 总索引 — 学习进度、用户偏好、项目经验 |
| [[claude-memory/multi-agent-framework\|多智能体框架]] | 多智能体迭代框架精要 |
| [[claude-memory/terminal-ui-testing\|终端 UI 测试]] | 终端 UI 自动化测试方案 |
| [[claude-memory/debugging-lessons\|调试经验]] | 调试经验记录与教训 |
| [[claude-memory/frontend-aesthetics\|前端美学]] | 前端美学原则 |
| [[claude-memory/code-quality\|代码质量]] | 代码质量约束 |

### Claude Code 系统提示词

> 外部仓库：110+ Claude Code 系统提示词提取，按类型分类存储

[[claude-code-system-prompts/README|查看完整目录]]

### 附件与模板

| 文件夹 | 用途 |
|--------|------|
| [[Attachments/]] | 图片、PDF 等附件 |
| [[Templates/]] | 笔记模板 |

---

## 管理方法

### 分类原则

| 文件夹 | 用途 | 示例 |
|--------|------|------|
| Inbox/ | 新笔记先放这里，定期整理 | 临时想法、快速记录 |
| Projects/ | 具体项目相关 | 项目笔记、进度追踪 |
| Learning/ | 学习笔记、知识整理 | 读书笔记、课程记录 |
| References/ | 长期参考（方法、原则、最佳实践） | 工作流、规范、模板 |
| Templates/ | 笔记模板 | 日报模板、会议记录模板 |
| Attachments/ | 图片、PDF 等附件 | 截图、文档 |
| claude-memory/ | Claude Code 全局记忆 | 跨会话持久化配置 |

### 核心原则

1. **根目录保持简洁** - 只放 README.md
2. **Inbox 先行** - 新笔记先进 Inbox，定期归档
3. **极简分类** - 避免过度分类，够用即可

---

### 关系图谱（Graph View）

#### 原理

关系图谱显示笔记之间的链接关系。**有链接才有连线**，没有相互链接的笔记在图谱中看不到连接。

#### 如何打开

左侧工具栏（Ribbon）点击 **"打开关系图谱"** 图标（网络图图标）

#### 如何创建内部链接

在笔记中输入 `[[笔记名称]]` 即可创建链接：

```markdown
# 示例
今天学到了[[思维模型]]的概念...
参考[[深度思考方法]]了解更多...
```

**快捷键**：输入 `[[` 后自动弹出笔记列表，选择即可

#### 链接语法

| 语法 | 效果 |
|------|------|
| `[[笔记名]]` | 链接到整篇笔记 |
| `[[笔记名#标题]]` | 链接到特定标题 |
| `[[笔记名#标题#子标题]]` | 链接到嵌套标题 |
| `[[笔记名|显示文字]]` | 自定义显示文字 |

#### 图谱操作

- **悬停** - 高亮该笔记的所有连接
- **点击** - 打开该笔记
- **右键** - 显示更多操作
- **滚轮** - 缩放
- **拖动** - 移动图谱

#### 图谱设置（左上角齿轮）

- **Filters** - 过滤显示哪些笔记
- **Orphans** - 显示孤立笔记（无链接的笔记）
- **Groups** - 按颜色分组

#### 常见问题

**问题：图谱中看不到连线**

原因：笔记之间没有相互链接

解决：在笔记中添加 `[[其他笔记名]]` 链接

#### 参考资源

- [Graph view - Obsidian Help](https://help.obsidian.md/plugins/graph)
- [Internal links - Obsidian Help](https://help.obsidian.md/Linking+notes+and+files/Internal+links)
