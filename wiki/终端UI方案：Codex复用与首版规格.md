---
name: 终端UI方案：Codex复用与首版规格
description: Rust CLI 终端 UI 方案——Codex 源码按模块复用边界、Ratatui/Crossterm/ratatui-textarea 库选型与首版（MVP）渲染规格草稿
type: concept
updated: 2026-10-05
tags: [rust, tui, codex, ratatui, crossterm, terminal, spec]
source: self
---

# 终端UI方案：Codex复用与首版规格

## 核心思想

Codex CLI 的 Rust 代码开源（Apache-2.0），可以复制、修改，本项目也有现成库可直接复用。落地组合：**基础能力直接用库（Ratatui + Crossterm，输入框用 ratatui-textarea），布局与操作规则参考 Codex，库不能满足的部分再按模块摘取 Codex 小模块**。本文同时给出首版（MVP）终端 UI 渲染规格草稿。

## 复用可行性与许可证

Codex CLI 的 Rust 代码开源，可以复制、修改。它采用 Apache-2.0 许可证；发布复制改造的代码时，需要保留许可证、相关版权和 NOTICE，并注明修改。[Codex LICENSE](https://github.com/openai/codex/blob/main/LICENSE)

## 库选型

| 库 | 帮你完成什么 |
|---|---|
| [Ratatui](https://github.com/ratatui/ratatui) | 布局、文字样式、边框、消息区和状态栏绘制 |
| [Crossterm](https://github.com/crossterm-rs/crossterm) | 按键读取、光标控制、窗口变化、备用屏切换 |
| [ratatui-textarea](https://github.com/ratatui/ratatui-textarea) | 现成的多行输入框，支持编辑、光标移动和自动换行 |

Codex 的终端模块也使用 **Ratatui + Crossterm**；输入框可以按需采用 Ratatui 团队维护的 `ratatui-textarea`。[codex-rs/tui/Cargo.toml](https://github.com/openai/codex/blob/main/codex-rs/tui/Cargo.toml)

## Codex 代码的取用方式

复制 Codex 的代码可行，但建议**按模块取用**。核查结论：`codex-tui` 连着配置、登录、会话协议、应用服务器等大量内部模块，整套搬过来需要同时处理这些依赖，会增加当前项目的复杂度。[codex-rs/tui/Cargo.toml](https://github.com/openai/codex/blob/main/codex-rs/tui/Cargo.toml)

具体采用顺序：

1. **基础能力直接用库**，不用自己写终端绘制和输入编辑器。
2. **布局与操作规则参考 Codex**，例如消息、输入框和状态栏怎么配合。
3. **库不能满足的部分，再摘取 Codex 小模块**，先检查依赖，再决定移植范围。它的 Markdown 渲染文件也依赖多个内部模块，不能保证复制一个文件就能运行。[markdown_render.rs 源码](https://github.com/openai/codex/blob/main/codex-rs/tui/src/markdown_render.rs)

## 首版规格草稿：终端 UI 渲染（项目内编号 003）

> [!note] 状态
> MVP 范围建议，待确认；本轮未修改项目代码。核查日期：2026-10-04。

### 已明确的目标

现有 Rust 项目已经能调用模型、显示回复，并有模型目录与刷新功能。用户要求：终端样式尽量参考 Codex CLI；界面支持中文和英文；优先复用现成库与 Codex 中 MVP 需要的代码；保持项目简单。

这里的 MVP 指本项目首版功能范围。Codex 是参考实现，不预设其历史早期版本就是本项目应采用的版本。

首版目标：形成稳定、可日常使用的 Codex 风格聊天界面。模型调用和配置仍沿用现有功能。

### 建议的首版范围

| 部分 | 首版做到 |
| --- | --- |
| 整体布局 | 一种全屏布局：紧凑启动信息、上方对话、底部输入与提示；参考 Codex 的留白、弱分隔、消息标记和配色，使用自己的程序名称 |
| 对话显示 | 区分用户、Agent 和错误消息；保留本次运行的可见记录；支持长回复滚动和回到底部 |
| 输入编辑 | 中文与英文、多行编辑与粘贴、光标移动、删除、发送；提交后清空草稿；换行快捷键要在实际终端可识别 |
| Markdown | 标题、强调、列表、引用、行内代码和代码块；复杂表格先以可读文本降级，代码块先保持清楚可读 |
| 运行状态 | 就绪、等待动画与耗时、失败反馈；请求期间界面仍能响应绘制、窗口变化和退出；最多一个进行中的请求 |
| 中英界面 | 自有标题、占位符、状态、快捷提示和错误模板提供中文与英文；保持命令名称、模型 ID、用户输入、模型回复和厂商原始详情原样 |
| 终端兼容 | 缩放和窄窗口下重排，草稿保留；中英混排不乱码、不拆坏字符；退出恢复终端；非 TTY 沿用现有纯文本流程 |

显示已有聊天记录不等于新增模型上下文功能。首版仅延续当前模型请求方式：等待完整回复后显示，不为 UI 增加流式协议。

建议界面默认中文，以一个语言选项选择 zh-CN 或 en（例如启动参数 --lang）。首版同屏使用一种界面语言。是否同时显示“中文 / English”不在此处默认决定，当前建议采用可切换方式。

建议首版用全屏，接近当前 Codex 的默认体验。全屏布局和语言选项尚属方案建议；批准范围后再写具体尺寸、按键和状态转换规则。

### 保持简单的边界

- 底层绘制与终端事件复用 Ratatui、Crossterm；输入核心先核查 Codex 可提取部分，依赖过重时用 ratatui-textarea 实现同类编辑行为。
- 不引入 Codex 的登录、App Server、Core、工具执行、审批、MCP、会话持久化、后台任务或多 Agent 系统。
- 本轮不增加流式输出、请求重试、真正的请求取消、附件、复杂命令选择器、主题切换和历史搜索。
- 已有 /models、/refresh 等入口继续可用；界面只显示已有真实数据，不凭空展示 token、计费、权限或上下文用量。
- UI 重绘与模型调用分开：动画、滚动、语言选项和窗口变化不触发新的模型请求。
- 等待提示原位更新；错误信息进入可回看的消息记录；UI 管理终端输出，外来文本不执行终端控制指令，保留已有凭证脱敏要求。
- 双语采用少量集中管理的文案模板，不为首版引入完整国际化框架。

### Codex 源码核查与取用范围

官方仓库：[openai/codex](https://github.com/openai/codex)。

本轮核查的是 main 的可访问文件与模块声明。真正复制前应固定发布 tag 或 commit，并重新确认该版本的路径、依赖和许可证。官方 [0.157.0 发布说明](https://github.com/openai/codex/releases/tag/rust-v0.157.0) 已确认全屏对话默认启用；本轮没有完成该 tag 所有源码文件的逐项读取。

| 范围 | 用途 | 取用边界 |
| --- | --- | --- |
| [tui.rs](https://github.com/openai/codex/blob/main/codex-rs/tui/src/tui.rs) 及 tui/ | 终端进入、绘制调度、备用屏、退出恢复 | 提取本项目需要的终端逻辑，剥离其他运行时依赖 |
| [bottom_pane/mod.rs](https://github.com/openai/codex/blob/main/codex-rs/tui/src/bottom_pane/mod.rs) 声明的 chat_composer、textarea、footer 模块 | 输入编辑、提交、底部提示与布局 | 只核查聊天输入部分，排除审批、工具、附件和复杂选择器 |
| [public_widgets/composer_input.rs](https://github.com/openai/codex/blob/main/codex-rs/tui/src/public_widgets/composer_input.rs) | 参考“按键输入 → 提交字符串”的小接口 | 仍依赖内部 ChatComposer 和 AppEvent，不是独立现成 crate |
| [history_cell/messages.rs](https://github.com/openai/codex/blob/main/codex-rs/tui/src/history_cell/messages.rs) | 用户与 Agent 消息样式、文本清理 | 保留纯文字消息部分，排除图片、语音、推理块等当前不用的内容 |
| [markdown_render.rs](https://github.com/openai/codex/blob/main/codex-rs/tui/src/markdown_render.rs) 与其辅助模块 | Markdown 解析、样式与换行 | 从首版格式需求出发裁剪；此文件仍依赖主题、链接、宽度和换行等内部模块 |
| [status_indicator_widget.rs](https://github.com/openai/codex/blob/main/codex-rs/tui/src/status_indicator_widget.rs) | 等待提示、计时和重绘 | 保留等待展示，剥离任务、工具、运行时与高级效果依赖 |
| [chatwidget.rs](https://github.com/openai/codex/blob/main/codex-rs/tui/src/chatwidget.rs) | 只读参考消息、输入、状态如何配合 | 不整份复制其业务流程；当前源码包含大量 Agent 服务依赖 |

以上是候选取用范围，不是已验证能够独立编译的移植清单。底部模块的具体文件形态以及宽度、换行辅助代码，需在固定版本的本地源码中再次确认。

借用源码时同时参考对应测试；按本项目的中文、英文、长文本、窄窗口和输入规则调整断言。不要整套移植 Codex 的测试基础设施。

### 复用记录与许可证

每个实际借用的模块记录来源 URL、固定 tag/commit、修改内容和相关测试。遵守 Codex 的 Apache-2.0 许可证，保留复制部分涉及的版权与 NOTICE，注明修改，并核对相关第三方许可。

依赖选用互相兼容的版本，锁入 Cargo.lock；不照抄 Codex 的整个工作区依赖和 feature 配置。

### 首版完成标准

- 完成“启动 → 编辑中英混排消息 → 发送 → 等待 → 显示完整回复 → 下一轮 → 退出”，操作流畅。
- 等待动画不增加消息记录；窗口变化和滚动不重复调用 API；长回复完整保留。
- 中英文自有界面文案齐全；切换界面语言不改写消息内容，不要求模型重新回复。
- 网络失败后有明确反馈，仍可继续下一轮；错误信息继续满足现有脱敏要求。
- 窄窗口重排和终端恢复在实际运行环境验证；纯文本管道流程和已有后端测试继续通过。

## 待解决的问题

- 全屏布局与界面语言选项仍属方案建议，需先确认这份 MVP 范围。
- 确认后细化布局尺寸、输入键位、等待与错误状态规则，形成可交给 Agent 执行的规格。
- Codex 取用范围基于 main 分支核查；实际复制前需固定发布 tag 或 commit，并重新确认路径、依赖和许可证。

## 相关概念

- [[从零实现Agent]] — 本页是路线图第 27 步“终端界面”的落地方案
- [[多厂商LLM接入架构]] — 同项目第 28 步“多模型供应商”的落地方案
- [[规格文档]] — 本规格按该方法起草：想清楚 → 预测 → 实测 → 关键判断 → 坑记录
- [[mi-code-tui-experience]] — 同族项目 mi-code 从 Ink/React 迁移到自研渲染引擎的实战经验
- [[CLI 备用屏架构设计]] — 备用屏、缩放重建与退出恢复的架构原则，支撑“终端兼容”项
