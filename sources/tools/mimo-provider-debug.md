---
name: MiMo Provider 切换调试记录
description: Claude Code 切换到小米 MiMo provider 的调试过程 — 401 认证字段、模型后缀大小写、域名拼写错误（xiaomimimo vs xiaomimimimo）三重根因
type: debugging-story
updated: 2026-05-29
tags: claude-code, mimo, provider, debugging, dns
source: self
---
# MiMo Provider 切换调试记录

**日期：** 2026-05-29
**问题：** Claude Code 切换到 MiMo 后无法使用，提示 401 Invalid API Key / 超时

## 背景

Claude Code 支持通过 `ANTHROPIC_BASE_URL` 切换到第三方 Anthropic-compatible 提供方。已有的 `switch-provider.sh` 脚本支持 Zhipu（智谱）和 MiMo（小米）两个 provider。

Zhipu 一直正常使用。切换到 MiMo 后，先报 401，修复认证字段后变成超时。

## 调试过程

### 阶段 1：认证字段排查

**症状：** `401 Invalid API Key`，重试 10 次后失败

对比官方文档发现：
- 脚本用的是 `ANTHROPIC_API_KEY`（发送 `x-api-key` 头）
- MiMo Token Plan 要求用 `ANTHROPIC_AUTH_TOKEN`（发送 `Authorization: Bearer` 头）

修复：切换脚本中 MiMo 使用 `ANTHROPIC_AUTH_TOKEN`，Zhipu 使用 `ANTHROPIC_API_KEY`，切换时清除另一个变量。

同时发现模型名后缀问题：
- 脚本用大写 `[1M]`
- 官方文档用小写 `[1m]`

修复：统一改为小写 `[1m]`。

### 阶段 2：超时排查

**症状：** 修复认证后，401 消失但变成超时（`Retrying in 20s · attempt 8/10`）

对比两个 provider 的网络路径：

| | Zhipu（可用） | MiMo（不可用） |
|---|---|---|
| DNS | ✅ 容器直接解析 `open.bigmodel.cn` | ❌ 容器 DNS 无法解析 |
| 代理 | 绕过（在 `NO_PROXY` 中） | 经过代理 → TLS 握手被远端关闭 |
| 连接 | 直连成功 | 代理隧道建立但 SSL EOF |

深入测试：
- `api.xiaomimimo.com`：DNS 解析 ✅，TLS 通过代理 ✅
- `token-plan-cn.xiaomimimimo.com`：DNS 解析 ❌，代理 TLS ❌
- `httpbin.org`：代理 TLS ✅

说明代理本身没问题，问题在 `token-plan-cn.xiaomimimimo.com` 这个域名。

### 阶段 3：域名拼写错误（根因）

**关键发现：** 对比官网域名和脚本中的域名：

- 官网：`platform.xiaomimimo.com`（xiaomi**mimo**，10 字符）
- 脚本：`token-plan-cn.xiaomimimimo.com`（xiaomi**mimi**mo，12 字符）

多了一个 `mi`！

验证：
```
token-plan-cn.xiaomimimo.com   → DNS ✅ (111.13.213.62) → API 200 ✅
token-plan-cn.xiaomimimimo.com → DNS ❌ → 不可达
```

修复：域名从 `xiaomimimimo.com` 改为 `xiaomimimo.com`。

## 三个修复汇总

| # | 问题 | 旧值 | 新值 |
|---|------|------|------|
| 1 | 认证字段 | `ANTHROPIC_API_KEY` | `ANTHROPIC_AUTH_TOKEN` |
| 2 | 上下文后缀 | `[1M]`（大写） | `[1m]`（小写） |
| 3 | 域名拼写 | `xiaomimimimo.com` | `xiaomimimo.com` |

## 调试教训

1. **域名拼写错误是最难发现的**：DNS 报错容易被误认为是网络问题，而非拼写问题。多个 `mi` 看起来几乎一样。
2. **对比法有效但需要精确**：对比了代理、TLS、DNS 多个层面，但始终把错误域名当作正确域名来排查。直到最后才对比官网域名发现拼写差异。
3. **官方文档也可能有误**：多个搜索结果都写的是 `xiaomimimimo.com`（带额外 mi），但实际 DNS 只认 `xiaomimimo.com`。不要盲目信任文档中的 URL。
4. **先验证最基本的连通性**：curl 测试 API 调用是最快验证方式。在排查代理/TLS 复杂问题前，应该先确认域名是否可达。

## 涉及文件

- `/home/app/project/scripts/switch-provider.sh` — provider 切换脚本
- `~/.claude/settings.json` — Claude Code 配置（env 字段）
- `D:\Files\.env` — 全局环境变量（MIMO_API_KEY）

## 最终可用配置

### 环境变量（D:\Files\.env）

```bash
# Zhipu AI (GLM) — docker-compose 里作为 ANTHROPIC_API_KEY 使用
ANTHROPIC_API_KEY=<智谱 key>

# MiMo (小米) — 切换脚本读取此变量，映射到 ANTHROPIC_AUTH_TOKEN
MIMO_API_KEY=<MiMo Token Plan key，tp- 开头>
```

### settings.json 中的 env 字段（切换后）

MiMo 模式：
```json
{
  "env": {
    "ANTHROPIC_AUTH_TOKEN": "tp-xxx...",
    "ANTHROPIC_BASE_URL": "https://token-plan-cn.xiaomimimo.com/anthropic",
    "ANTHROPIC_MODEL": "mimo-v2.5-pro[1m]",
    "ANTHROPIC_REASONING_MODEL": "mimo-v2.5-pro[1m]",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "mimo-v2.5-pro[1m]",
    "ANTHROPIC_DEFAULT_OPUS_MODEL": "mimo-v2.5-pro[1m]",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "mimo-v2.5[1m]",
    "ANTHROPIC_DEFAULT_SONNET_MODEL_NAME": "mimo-v2.5-pro",
    "ANTHROPIC_DEFAULT_OPUS_MODEL_NAME": "mimo-v2.5-pro",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME": "mimo-v2.5"
  }
}
```

Zhipu 模式：
```json
{
  "env": {
    "ANTHROPIC_API_KEY": "<智谱 key>",
    "ANTHROPIC_BASE_URL": "https://open.bigmodel.cn/api/anthropic",
    "ANTHROPIC_MODEL": "glm-5.1",
    "ANTHROPIC_REASONING_MODEL": "glm-5.1",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "glm-5-turbo",
    "ANTHROPIC_DEFAULT_OPUS_MODEL": "glm-5.1",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "glm-5-turbo",
    "ANTHROPIC_DEFAULT_SONNET_MODEL_NAME": "glm-5-turbo",
    "ANTHROPIC_DEFAULT_OPUS_MODEL_NAME": "glm-5.1",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME": "glm-5-turbo"
  }
}
```

### MiMo 官方参考

- 官网：https://platform.xiaomimimo.com
- Claude Code 集成文档：https://platform.xiaomimimo.com/docs/zh-CN/integration/claudecode
- Token Plan 请求地址：`https://token-plan-cn.xiaomimimo.com/anthropic`
- API 请求地址：`https://api.xiaomimimo.com/anthropic`（需 sk- 开头的 key，非 Token Plan）
