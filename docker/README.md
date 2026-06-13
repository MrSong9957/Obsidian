# Docker Dev Container

Fully isolated development environment, supporting Zhipu AI and Xiaomi MiMo dual model switching.

## Architecture

```
.env (API Key + Model Config) -> docker-compose.yml -> container env vars -> Claude Code
```

Config is injected via `.env`, **completely isolated from host**.

## Auto Config Restore

On startup the container automatically checks and restores Claude config:

1. Check if `~/.claude.json` exists
2. If not, try restoring from `~/.claude/backups/` (latest backup)
3. If no backup, create default config
4. Also initialize `~/.claude/settings.json`

## Quick Start

```bash
# 1. Edit .env, fill in API Key
cp .env.example .env
# Edit .env with your API Key and GitHub Token

# 2. Build and start
docker compose up -d --build

# 3. Enter container
docker exec -it -u app dev-container bash

# 4. First run of Claude Code
claude
# When prompted "Do you want to use this API key?" select Yes

# 5. Test
claude -p "hello"
```

## Model Switching

All model configs are in `.env`. Switch by commenting/uncommenting:

### Zhipu AI (default)

```env
ANTHROPIC_BASE_URL=https://open.bigmodel.cn/api/anthropic
ANTHROPIC_API_KEY=your-zhipu-api-key
ANTHROPIC_DEFAULT_HAIKU_MODEL=glm-5-turbo
ANTHROPIC_DEFAULT_SONNET_MODEL=glm-5-turbo
ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5.2
ANTHROPIC_MODEL=glm-5.2
ANTHROPIC_REASONING_MODEL=glm-5.2
```

### Xiaomi MiMo

```env
ANTHROPIC_BASE_URL=https://token-plan-cn.xiaomimimo.com/anthropic
ANTHROPIC_API_KEY=your-xiaomi-api-key
ANTHROPIC_DEFAULT_HAIKU_MODEL=mimo-v2.5
ANTHROPIC_DEFAULT_SONNET_MODEL=mimo-v2.5
ANTHROPIC_DEFAULT_OPUS_MODEL=mimo-v2.5-pro
ANTHROPIC_MODEL=mimo-v2.5-pro
ANTHROPIC_REASONING_MODEL=mimo-v2.5-pro
```

After switching, run `docker compose up -d` to reload.

## Isolation

| Directory | Type | Description |
|-----------|------|-------------|
| /home/app/project | Bind Mount | Project dir, two-way sync with host |
| /home/app/.claude | Docker Volume | Claude config, fully isolated |
| /home/app/.config/opencode | Docker Volume | OpenCode config, fully isolated |

## Troubleshooting

```bash
# Manual config restore when files are lost
docker exec -it dev-container ls /home/app/.claude/backups/
docker exec -it dev-container cp /home/app/.claude/backups/.claude.json.backup.XXX /home/app/.claude.json

# Rebuild container (preserves Volume data)
docker compose down && docker compose up -d

# Full reset (deletes all config)
docker compose down -v && docker compose up -d --build
```

## Common Commands

```bash
docker compose logs -f          # View logs
docker compose down             # Stop
docker compose build --no-cache # Rebuild
docker volume ls                # List volumes
```
