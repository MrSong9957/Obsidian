# Memory

> **本文档为全仓库文档索引，任何记忆都拆分到独立文件。**

---

## 文档规则

1. **索引一行一条** — 索引中每个文档占一行，用 `- [标题](文件名) — 一句话摘要` 格式
2. **YAML 元数据必须** — 每个记忆文件开头必须包含 frontmatter：`name`、`description`、`type`
3. **符号链接** — Claude Code 的 memory 目录通过符号链接指向本仓库，零维护同步

---

## 索引

### Claude Code 参考

- [Everything-Claude-Code](References/Everything-Claude-Code.md) — 110+ Skills 完整索引
- [ECC 智能体](References/ECC-Agent-Usage-Instructions.md) — 25 个专业智能体使用说明
- [Superpowers](References/Superpowers.md) — 开发流程技能系统
- [ECC 安装](References/ecc-install.md) — 安装指南
- [状态栏](References/statusline-install.md) — 状态栏配置
- [Safety Net](References/safety-net-guide.md) — 安全插件指南
- [Harness Engineering](References/harness-engineering.md) — AI Agent-First 开发方法论
- [Playwright CLI](References/playwright-cli-guide.md) — 测试指南
- [UI 设计](References/ui-design.md) — 前端设计原则
- [终端 UI 测试](References/terminal-ui-testing.md) — VHS + AI 评估器方案

### 学习笔记

- [Learn Claude Code](References/learn-claude-code.md) — shareAI-lab/learn-claude-code 学习中
- [多智能体框架](References/multi-agent-framework.md) — 迭代框架精要
- [AI 时代技能](References/ai-era-skills.md) — L2-L6 技能体系
- [科学提升智商](References/科学提升智商：六大核心习惯与实践指南.md) — 六大核心习惯

### 经验与偏好

- [调试经验](References/debugging-lessons.md) — 调试教训记录
- [前端美学](References/frontend-aesthetics.md) — 设计原则
- [代码质量](References/code-quality.md) — 质量约束
- [状态栏记忆](References/statusline-setup.md) — 配置记录
- [终端测试记忆](References/terminal-ui-testing-memory.md) — 测试方案记录

---

## 设置方法

### 符号链接

将 Claude Code 项目记忆目录指向本仓库，实现零维护同步：

```bash
# WSL2 / Linux — 先删除旧 memory 目录，再创建符号链接
rm -rf ~/.claude/projects/<项目路径>/memory
ln -s /home/app/obsidian ~/.claude/projects/<项目路径>/memory

# Windows 原生 — 使用 junction（无需管理员权限）
rmdir "C:\Users\<用户>\.claude\projects\<项目路径>\memory"
mklink /J "C:\Users\<用户>\.claude\projects\<项目路径>\memory" "E:\Files\Obsidian\Claude Code Memory"
```

---

## 用户偏好

- **沟通语言** — 中文
- **代码学习** — 原理和逻辑优先，不逐行讲解
- **解释风格** — 嵌入术语的形象比喻，用比喻建直觉
- **项目推进** — 面向零基础，小步推进，每步可验证
- **核心工作流** — 自主闭环：自动验证、测试、迭代修复；多次失败才请求介入
- **调试原则** — 先观察再假设，第一件事加日志打印变量
- **技术选型** — 优先复用成熟方案，不重复造轮子
- **避免过度工程** — 只做必要更改，不加功能、不重构、不提前抽象
- **子代理** — 可并行或需隔离上下文时用子代理；简单顺序操作直接做
- **前端美学** — 避免"AI 泛滥"美学
- **代码方案** — 通用方案，不搞变通；测试验证正确性
- **代码质量** — AI 负责发现最佳实践
