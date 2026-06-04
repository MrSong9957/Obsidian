---
name: OpenCode 配置指南
description: OpenCode 完整配置参考——安装、配置文件、Provider、MCP、指令文件、与 Claude Code 对比
type: concept
updated: 2026-06-02
tags: opencode, configuration, mcp, provider, agents-md
source: https://example.com
---

# OpenCode 配置指南

OpenCode（[anomalyco/opencode](https://github.com/anomalyco/opencode)）是开源终端 AI 编码工具，支持 75+ AI Provider、MCP 扩展、LSP 集成和自定义指令。

## 安装

```bash
# curl 脚本（推荐）
curl -fsSL https://opencode.ai/install | bash

# npm
npm i -g opencode-ai

# bun
bun add -g opencode-ai

# Homebrew（macOS/Linux）
brew install anomalyco/tap/opencode

# Windows
choco install opencode
scoop install opencode
```

其他：Arch Linux (`paru -S opencode` 或 `pacman -S opencode`)、mise (`mise use -g opencode`)、nix。

## 配置文件体系

### 文件格式

- JSON 或 JSONC（支持注释）
- Schema：`"$schema": "https://opencode.ai/config.json"`

### 配置位置

| 位置 | 路径 | 说明 |
|------|------|------|
| 全局 | `~/.config/opencode/opencode.json` | 用户级配置，跨项目生效 |
| 项目 | 项目根目录 `opencode.json` | 项目级配置，优先级最高 |
| 自定义路径 | `OPENCODE_CONFIG` 环境变量指定 | 在全局和项目之间加载 |
| 自定义目录 | `OPENCODE_CONFIG_DIR` 环境变量指定 | 替代 agents/commands/plugins 目录 |

### 优先级（低 → 高）

Remote（组织配置） → Global → Custom → Project → `.opencode` 目录 → Inline

### 变量替换

配置值支持两种变量替换语法：

```jsonc
// 环境变量引用，未设置时替换为空字符串
"apiKey": "{env:ANTHROPIC_API_KEY}"

// 文件内容引用，路径支持相对/绝对/~ 路径
"apiKey": "{file:~/.secrets/openai-key}"
```

### 完整配置示例

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "model": "anthropic/claude-sonnet-4-20250514",
  "small_model": "anthropic/claude-3-5-haiku-latest",
  "default_agent": "build",
  "instructions": ["./AGENTS.md", "./docs/style.md"],
  "compaction": { "auto": true, "tail_turns": 15 },
  "snapshot": true,
  "autoupdate": "notify"
}
```

## Provider 配置

### 支持的 Provider

OpenAI、Anthropic、Google Gemini、Amazon Bedrock、Azure OpenAI、Cloudflare、GitHub Copilot、OpenRouter、xAI、DeepSeek、Cerebras、Groq、Fireworks、Together，以及任何 OpenAI 兼容的自定义 Provider。

### API Key 配置

方式一：环境变量自动检测（推荐，无需配置文件）

```bash
export ANTHROPIC_API_KEY="sk-ant-xxx"
export OPENAI_API_KEY="sk-xxx"
export GOOGLE_API_KEY="xxx"
```

方式二：配置文件中指定

```jsonc
{
  "provider": {
    "anthropic": {
      "options": {
        "apiKey": "{env:ANTHROPIC_API_KEY}"
      }
    },
    "openai": {
      "options": {
        "apiKey": "{env:OPENAI_API_KEY}",
        "baseURL": "https://api.openai.com/v1"
      }
    }
  }
}
```

方式三：自定义 Provider（OpenAI 兼容）

```jsonc
{
  "provider": {
    "my-provider": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "My Provider",
      "options": {
        "baseURL": "https://api.myprovider.com/v1",
        "apiKey": "{env:MY_PROVIDER_KEY}"
      },
      "models": {
        "my-model": {
          "name": "My Model",
          "limit": { "context": 200000, "output": 65536 }
        }
      }
    }
  }
}
```

### Bedrock 配置

```bash
# 方式一：AWS Access Keys
AWS_ACCESS_KEY_ID=XXX AWS_SECRET_ACCESS_KEY=YYY opencode

# 方式二：Named Profile
AWS_PROFILE=my-profile opencode

# 方式三：Bearer Token
AWS_BEARER_TOKEN_BEDROCK=XXX opencode
```

## MCP 服务器配置

配置键名为 `mcp`（注意：与 Claude Code 的 `mcpServers` 不同）。

### 本地 MCP（type: "local"）

`command` 为字符串数组，`env` 传递环境变量：

```jsonc
{
  "mcp": {
    "my-server": {
      "type": "local",
      "command": ["npx", "-y", "my-mcp-server"],
      "enabled": true,
      "env": {
        "MY_API_KEY": "{env:MY_API_KEY}"
      }
    }
  }
}
```

### 远程 MCP（type: "remote"）

`url` 指定服务器地址，`headers` 传递认证信息：

```jsonc
{
  "mcp": {
    "my-remote": {
      "type": "remote",
      "url": "https://mcp.example.com/sse",
      "enabled": true,
      "headers": {
        "Authorization": "Bearer {env:MY_API_KEY}"
      }
    }
  }
}
```

### 智谱 AI 4 个 MCP 的 OpenCode 配置

将 [[sources/zhipu-mcp.md|智谱 MCP]] 转换为 OpenCode 格式：

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "zai-mcp-server": {
      "type": "local",
      "command": ["npx", "-y", "@z_ai/mcp-server"],
      "enabled": true,
      "env": {
        "Z_AI_API_KEY": "{env:Z_AI_API_KEY}"
      }
    },
    "web-search-prime": {
      "type": "remote",
      "url": "https://open.bigmodel.cn/api/mcp/web_search_prime/mcp",
      "enabled": true,
      "headers": {
        "Authorization": "Bearer {env:Z_AI_API_KEY}"
      }
    },
    "web-reader": {
      "type": "remote",
      "url": "https://open.bigmodel.cn/api/mcp/web_reader/mcp",
      "enabled": true,
      "headers": {
        "Authorization": "Bearer {env:Z_AI_API_KEY}"
      }
    },
    "zread": {
      "type": "remote",
      "url": "https://open.bigmodel.cn/api/mcp/zread/mcp",
      "enabled": true,
      "headers": {
        "Authorization": "Bearer {env:Z_AI_API_KEY}"
      }
    }
  }
}
```

## 指令文件（AGENTS.md）

OpenCode 的指令系统等效于 Claude Code 的 `CLAUDE.md`。

### 默认指令文件

项目根目录的 `AGENTS.md` 自动加载（与 Claude Code 的 `CLAUDE.md` 类似）。

### 自定义指令

通过 `instructions` 字段指定额外指令文件，支持 glob 模式和远程 URL：

```jsonc
{
  "instructions": [
    "CONTRIBUTING.md",
    "docs/guidelines.md",
    ".cursor/rules/*.md"
  ]
}
```

```jsonc
{
  "instructions": [
    "https://raw.githubusercontent.com/my-org/shared-rules/main/style.md"
  ]
}
```

### AGENTS.md 中的文件引用

使用 `@path/to/file.md` 语法懒加载外部文件：

```markdown
## 开发规范

TypeScript 代码风格：@docs/typescript-guidelines.md
React 组件架构：@docs/react-patterns.md
REST API 设计：@docs/api-standards.md
```

## 与 Claude Code 的配置对比

### 配置文件位置

| | Claude Code | OpenCode |
|---|---|---|
| 全局配置 | `~/.claude.json` | `~/.config/opencode/opencode.json` |
| 全局设置 | `~/.claude/settings.json` | 同上（合并在一个文件中） |
| 项目配置 | 项目级 `.claude/settings.json` | 项目根目录 `opencode.json` |
| 指令文件 | `CLAUDE.md`（全局 + 项目级） | `AGENTS.md` + `instructions` 字段 |
| 规则文件 | `.claude/rules/` | `.opencode/commands/` |

### MCP 配置格式对比

| | Claude Code | OpenCode |
|---|---|---|
| 配置键名 | `mcpServers` | `mcp` |
| 本地类型 | `"type": "stdio"` | `"type": "local"` |
| 远程类型 | `"type": "http"` / `"sse"` | `"type": "remote"` |
| 命令格式 | `"command": "npx", "args": ["-y", "pkg"]` | `"command": ["npx", "-y", "pkg"]` |
| 环境变量 | `"env": {"KEY": "value"}` | `"env": {"KEY": "value"}` |
| 认证头 | `"headers": {"Authorization": "Bearer xxx"}` | `"headers": {"Authorization": "Bearer xxx"}` |
| 启用/禁用 | 无（通过 CLI 添加/删除） | `"enabled": true/false` |
| 变量替换 | 不支持 | `{env:VAR}` / `{file:path}` |

### 同一 MCP 服务器配置转换示例

以智谱视觉理解 MCP 为例：

**Claude Code（`~/.claude.json`）：**

```json
{
  "mcpServers": {
    "zai-mcp-server": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@z_ai/mcp-server"],
      "env": {
        "Z_AI_API_KEY": "your-actual-key"
      }
    }
  }
}
```

**OpenCode（`~/.config/opencode/opencode.json`）：**

```jsonc
{
  "mcp": {
    "zai-mcp-server": {
      "type": "local",
      "command": ["npx", "-y", "@z_ai/mcp-server"],
      "enabled": true,
      "env": {
        "Z_AI_API_KEY": "{env:Z_AI_API_KEY}"
      }
    }
  }
}
```

关键差异：
1. 键名：`mcpServers` → `mcp`
2. 类型：`stdio` → `local`
3. 命令：`command` + `args` → `command` 数组
4. API Key：硬编码值 → `{env:VAR}` 变量引用
5. 显式启用：`"enabled": true`

### 环境变量共享

两个工具使用相同的环境变量名：

| 环境变量 | 用途 |
|---------|------|
| `ANTHROPIC_API_KEY` | Anthropic API Key |
| `OPENAI_API_KEY` | OpenAI API Key |
| `GOOGLE_API_KEY` | Google API Key |

在同一环境中设置这些变量后，两个工具均可直接使用，无需重复配置。

## Docker 容器中的 OpenCode

> Docker 容器部署详见 [[skills/docker-dev-container/SKILL.md|Docker 开发容器技能]]，新项目初始化流程见 [[wiki/new-project-setup.md|新项目初始化]]。

### 配置目录隔离

```yaml
volumes:
  opencode-config:/home/app/.config/opencode  # Docker Volume，与宿主机隔离
```

### 基础镜像预装

基础镜像 `s7620605/claude-code-opencode:latest` 通过 `npm install -g opencode-ai` 预装 OpenCode。更新方式：

```bash
# 容器内更新
npm update -g opencode-ai

# 或通过 docker commit 更新基础镜像（推荐）
docker exec -it -u root temp-update bash
npm update -g opencode-ai
docker commit temp-update s7620605/claude-code-opencode:latest
```

### 与 Claude Code 共享 API Key

容器通过 `docker-compose.yml` 的 `env_file` 注入环境变量。两个工具共享相同的 API Key 变量（`ANTHROPIC_API_KEY`、`ANTHROPIC_AUTH_TOKEN`），无需分别配置。

### entrypoint 中的权限处理

```bash
chown -R app:app /home/app/.config/opencode 2>/dev/null || true
```

确保 OpenCode 配置目录归属正确用户，避免权限问题。
