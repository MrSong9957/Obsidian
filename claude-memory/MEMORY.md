# Memory

> **本文档为索引，详细内容见链接文件。新增记忆超过 10 行请拆分到独立文件。**

---

## Obsidian 仓库规则

### obsidian-vault 同步规则

obsidian-vault 仓库**只保存全局记忆**，不保存项目记忆。修改时同步到 `/home/app/.claude/projects/-home-app-project/memory/MEMORY.md`。

### MEMORY.md 文档结构

文档固定两个板块，按顺序排列：

1. **仓库规则** — 同步规则、管理方法
2. **索引** — 所有详细文档的链接

**新增内容归类原则：**
- 所有内容都拆分为独立文档，在索引中添加链接
- 不在 MEMORY.md 中内联正文内容

### Obsidian 管理方法

**文件夹结构：**
```
obsidian-vault/
├── README.md           # 管理方法说明（根目录）
├── Attachments/        # 图片、PDF 等附件
├── Templates/          # 笔记模板
├── References/         # 长期参考材料（方法、原则、最佳实践）
├── Projects/           # 项目相关笔记
├── Learning/           # 学习笔记、知识整理
├── claude-memory/      # Claude Code 全局记忆
└── Inbox/              # 待整理笔记
```

**分类原则：**
- 新笔记先放 Inbox/，定期整理
- 根目录只放 README.md，保持简洁
- References/ 存放长期可复用的知识

### 记忆文档规则

1. **索引一行一条** — 索引中每个文档占一行，用 `- [标题](文件名) — 一句话摘要` 格式
2. **YAML 元数据必须** — 每个记忆文件开头必须包含 frontmatter：`name`、`description`、`type`

---

## 索引

- ✅ [ai-terminal 架构学习](./ai-terminal-architecture.md) — 引导式教学，已完结
- ✅ [多智能体迭代框架](./multi-agent-framework.md) — 动态发现，已验证
- ✅ [AI 时代人类技能发展](./ai-era-skills.md) — L2-L6 技能体系
- ✅ [终端 UI 自动化测试](./terminal-ui-testing.md) — VHS + AI 评估器
- 📄 [前端美学](./frontend-aesthetics.md)
- 📄 [代码质量](./code-quality.md)
- 📄 [状态栏配置](./statusline-setup.md)
- 📄 [调试经验](./debugging-lessons.md)
- 📄 [用户偏好](./user-preferences.md)
