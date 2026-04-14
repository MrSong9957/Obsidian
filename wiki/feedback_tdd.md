---
type: feedback
name: TDD 默认实践
description: 安全函数和工具函数必须先写边界测试用例再实现，参考 safe_path 越界遗漏的教训
updated: 2026-04-12
tags: [TDD, 测试驱动开发, 安全函数, 工具函数, safe_path]
originSessionId: e7a7837a-73ac-4168-b6bb-ba35ba9bb1b3
---
默认使用 TDD（测试驱动开发）。安全相关函数和工具函数，实现前先写边界 case 并验证。

**Why:** safe_path 函数定义了 PROJECT_ROOT 却没使用，也没有边界检查（`is_relative_to`），写完后只测了正常路径没测越界路径，导致 `../../etc/passwd` 可以畅通无阻。直到用户对照教程指出才修复。

**How to apply:**
- 安全函数（路径校验、权限检查、输入过滤）：第一个测试就是"必须被拦住的那个 case"
- 工具函数：先写 `pytest` 测试覆盖正常/边界/错误三路，再写实现
- 有参考代码时，逐行对齐关键步骤，不是"差不多"就行
