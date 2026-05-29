# Email Assistant 多平台发布方案与一键发布工具

## Context

项目处于阶段 A（开源 Build in Public），需要设计一个覆盖代码分发 + 社交媒体的发布方案，并构建一键发布工具。优先复用现有方案，避免重复造轮子。

## 一、发布渠道（10 个平台）

按内容类型分四列，国内外对齐：

| 内容类型 | 国内 | 国外 |
|----------|------|------|
| 代码分发 | GitHub | GitHub |
| 长文 | 知乎 | Dev.to |
| 社交图文 | 小红书 / 公众号 | X / Reddit |
| 视频 | 抖音 / B站 | YouTube |

### 各平台发布方式

| 平台 | API 可用性 | 发布方式 |
|------|-----------|----------|
| **GitHub Release** | `gh` CLI | 自动 |
| **Dev.to** | REST API (免费) | 自动 |
| **知乎** | API (需申请) | 半自动 / 手动 |
| **小红书** | 无公开 API | 手动 |
| **公众号** | 官方 API (需认证服务号) | 半自动 / 手动 |
| **X / Twitter** | API (Basic $100/mo) | 手动（成本高） |
| **Reddit** | API (free tier) | 半自动 |
| **抖音** | 创作者平台 API | 手动上传视频 |
| **B站** | API (需申请) | 手动上传视频 |
| **YouTube** | Data API v3 | 半自动 |

## 二、通用发布格式

### 设计原则

1. **文字 + 图片 = 最小通用集**，所有 10 个平台都支持
2. **文字图片可合成视频**，同一份素材覆盖视频平台（抖音/B站/YouTube）
3. 一次写好内容，脚本按平台规则裁剪分发

### 内容结构

```
┌─────────────────────────────────────────────┐
│  Universal Post                              │
│                                              │
│  title        长文标题                        │
│  summary      一句话摘要（≤280字）            │
│  body         Markdown 正文（长文平台用）      │
│  images[]     图片列表（截图、Demo GIF）       │
│  video        视频文件（可选，或由图片自动生成）│
│  tags[]       标签                            │
│  link         项目链接（GitHub repo）          │
│  lang         zh / en / both                 │
└─────────────────────────────────────────────┘
```

### 各平台字段映射

| 字段 | GitHub Release | Dev.to / 知乎 | 小红书 / X / Reddit | 抖音 / B站 / YouTube |
|------|---------------|---------------|---------------------|---------------------|
| title | release 标题 | 文章标题 | - | 视频标题 |
| summary | - | - | 帖子正文 | 视频描述 |
| body | release notes | 文章正文 | - | - |
| images | 内嵌图片 | 内嵌图片 | 附图 / 图文帖 | 封面 / 视频素材源 |
| video | - | - | - | 视频本身 |
| tags | - | 标签 | #话题 | #话题 |
| link | 自动 | 文内链接 | 评论区附链 | 简介 |

### 视频生成策略

视频平台的素材来自同一份文字+图片，通过工具合成：

```
images[] + summary + body
        ↓
  图片轮播视频 / 录屏讲解视频
        ↓
  抖音（竖屏 9:16）/ B站（横屏 16:9）/ YouTube（横屏 16:9）
```

可选工具：
- **FFmpeg** — 图片序列转视频，命令行可脚本化
- **remotion** — React 视频框架，可编程生成
- **手动录屏** — 终端操作演示，效果最好但无法自动化

### 模板文件格式

```
release-notes/
└── v0.1.0/
    ├── post.md              # 统一内容模板（frontmatter + body）
    ├── images/              # 图片素材
    │   ├── demo.gif
    │   └── screenshot.png
    └── video/               # 可选：手动录屏或自动生成
        └── demo.mp4
```

`post.md` 模板：

```markdown
---
version: 0.1.0
lang: both

title_en: Email Assistant v0.1.0 — AI-Powered Email in Your Terminal
title_zh: 用 AI 写了一个终端邮件助手

summary_en: "Built an AI email assistant that runs in your terminal. Check, reply, forward emails with natural language. Open source."
summary_zh: "用一个周末，让 AI 帮我管邮箱。终端里收发邮件，自动分类摘要。开源了。"

tags_en: [ai, email, cli, open-source, claude]
tags_zh: [AI, 终端, 邮箱, 开源]

images: [images/demo.gif, images/screenshot.png]
video: video/demo.mp4

link: https://github.com/MrSong9957/email-assistant

# 平台开关（true=自动发布，draft=草稿，manual=输出内容手动发，false=跳过）
github: true
devto: draft
zhihu: manual
xiaohongshu: manual
wechat: false
twitter: manual
reddit: manual
douyin: manual
bilibili: manual
youtube: manual
---

## What's New

（Markdown 正文，GitHub Release / Dev.to / 知乎 共用）

### Features

- Natural language email management
- AI-powered categorization
- Multi-account support (QQ Mail, Gmail)

### Screenshots

![Demo](images/demo.gif)
![Email list view](images/screenshot.png)
```

## 三、一键发布工具：`publish.sh`

### 架构

```
release-notes/v0.1.0/post.md
            ↓
      publish.sh (CLI)
      ├── 解析 frontmatter + body
      ├── 按平台开关分发
      ├── auto/draft 平台 → API 调用
      ├── manual 平台 → 格式化输出到终端 + 复制到剪贴板
      └── video 平台 → 检查视频文件，输出上传指引
```

### 交互流程

```bash
./publish.sh v0.1.0 --dry-run    # 预览：解析模板，输出各平台裁剪内容
./publish.sh v0.1.0              # 正式发布
./publish.sh v0.1.0 --platform github,devto  # 指定平台
```

### 文件结构

```
project/
├── publish.sh                    # 一键发布脚本
├── release-notes/
│   └── v0.1.0/
│       ├── post.md              # 统一内容模板
│       ├── images/              # 图片素材
│       └── video/               # 可选视频
├── .env.publish                 # API keys（加入 .gitignore）
└── .env.publish.example         # 模板
```

### `.env.publish` 需要的 keys

```bash
# GitHub — gh CLI 已认证则不需要额外 key
DEVTO_API_KEY=xxx
# 可选，按需配置
ZHIHU_COOKIE=xxx
REDDIT_CLIENT_ID=xxx
REDDIT_CLIENT_SECRET=xxx
REDDIT_REFRESH_TOKEN=xxx
YOUTUBE_API_KEY=xxx
```

### 格式适配逻辑

```
post.md
  ├── GitHub Release → title_en + body + images
  ├── Dev.to → title_en + body + tags_en + images
  ├── 知乎 → title_zh + body + images
  ├── 小红书 → summary_zh + images（图文帖）
  ├── 公众号 → title_zh + body + images（长文）
  ├── X → summary_en + image[0] + link
  ├── Reddit → title_en + link + body（发到 r/SideProject 等）
  ├── 抖音 → video + summary_zh
  ├── B站 → video + title_zh + summary_zh
  └── YouTube → video + title_en + summary_en
```

## 四、实施步骤

1. **创建 `release-notes/v0.1.0/post.md`** — 首版发布内容
2. **创建 `.env.publish.example`** — API key 模板
3. **创建 `publish.sh`** — 核心脚本（frontmatter 解析 + GitHub Release + Dev.to + 手动平台输出）
4. **注册各平台账号 + 获取 API key**
5. **`./publish.sh v0.1.0 --dry-run` 验证**
6. **正式首发** — `./publish.sh v0.1.0`

## 五、验证

- `--dry-run` 正确解析 frontmatter，按平台输出裁剪后的内容
- GitHub Release 创建成功（`--draft` 模式测试）
- Dev.to 文章出现在草稿列表
- manual 平台内容格式正确，已复制到剪贴板
- 视频平台输出上传指引（视频文件路径 + 标题描述）
