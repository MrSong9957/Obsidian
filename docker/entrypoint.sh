#!/bin/bash
set -e

export PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

CLAUDE_CONFIG="/home/app/.claude.json"
CLAUDE_DIR="/home/app/.claude"
BACKUP_PATTERN="/home/app/.claude/backups/.claude.json.backup.*"

# 确保目录存在
mkdir -p "$CLAUDE_DIR/backups"

# 恢复配置文件
restore_config() {
    if [ ! -f "$CLAUDE_CONFIG" ]; then
        echo "⚠️  Claude 配置文件不存在，尝试恢复..."
        LATEST_BACKUP=$(ls -t $BACKUP_PATTERN 2>/dev/null | head -1)
        if [ -n "$LATEST_BACKUP" ] && [ -f "$LATEST_BACKUP" ]; then
            cp "$LATEST_BACKUP" "$CLAUDE_CONFIG"
            echo "✅ 已从备份恢复: $LATEST_BACKUP"
        else
            echo '{"hasCompletedOnboarding": true}' > "$CLAUDE_CONFIG"
            echo "✅ 已创建默认配置文件"
        fi
        chown app:app "$CLAUDE_CONFIG" 2>/dev/null || true
    fi
}

# 初始化 settings.json
init_settings() {
    SETTINGS_FILE="$CLAUDE_DIR/settings.json"
    if [ ! -f "$SETTINGS_FILE" ]; then
        echo "⚠️  Claude settings.json 不存在，创建默认配置..."
        cat > "$SETTINGS_FILE" << 'EOF'
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://open.bigmodel.cn/api/anthropic",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "glm-5-turbo",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "glm-5-turbo",
    "ANTHROPIC_DEFAULT_OPUS_MODEL": "glm-5.2",
    "ANTHROPIC_DEFAULT_OPUS_MODEL_NAME": "glm-5.2",
    "ANTHROPIC_MODEL": "glm-5.2",
    "ANTHROPIC_REASONING_MODEL": "glm-5.2",
    "CLAUDE_CODE_EFFORT_LEVEL": "max",
    "DISABLE_AUTOUPDATER": "1",
    "API_TIMEOUT_MS": "3000000",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1"
  }
}
EOF
        chown app:app "$SETTINGS_FILE" 2>/dev/null || true
        echo "✅ 已创建默认 settings.json"
    fi
}

# 设置 Bun 环境
setup_bun() {
    if [ -d "/home/app/.bun" ]; then
        export BUN_INSTALL="/home/app/.bun"
        export PATH="/home/app/.bun/bin:$PATH"
    fi
}

# 以 app 用户身份执行初始化
if [ "$(id -u)" = "0" ]; then
    # root 用户：修复权限后初始化
    chown -R app:app /home/app/.claude /home/app/.config/opencode 2>/dev/null || true
    restore_config
    init_settings
    setup_bun
    chown -R app:app /home/app/.claude /home/app/.config/opencode 2>/dev/null || true
else
    # 非 root 用户：直接初始化
    restore_config
    init_settings
    setup_bun
fi

echo "🚀 MiMo-Code 开发容器已就绪"
exec "$@"
