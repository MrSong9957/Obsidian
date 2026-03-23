# ai-terminal 项目记忆

## UI 设计原则

**Ctrl+P 是唯一主菜单入口**，遵循以下原则：

1. **一个功能一个位置** — 不能重复占用菜单选项
2. **功能整合** — 相关功能必须合并为一个菜单项
   - 例如：选择会话 + 删除会话 → 整合为"会话管理"
3. **慎重添加** — 主菜单选项必须精简，不能随意扩展

```
主菜单结构（示例）:
├── 会话管理     ← 整合：选择/新建/删除/重命名
├── 模型选择     ← 未来
└── 设置        ← 未来
```

## 功能参考标准

**以 Claude Code、OpenCode 为参考**，确定 ai-terminal 的功能方向和学习路径。

参考项目：
- [Claude Code](https://github.com/anthropics/claude-code) - Anthropic 官方 CLI
- [OpenCode](https://github.com/opencode-ai/opencode) - 开源替代方案
