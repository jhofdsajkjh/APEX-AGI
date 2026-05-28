#!/bin/sh
# =============================================================================
# ΩMEGA AGI SUPREMACY — Docker Entrypoint
# =============================================================================
set -e

# 加载 .env（如果存在）
if [ -f /var/lib/omega/.env ]; then
    set -a
    . /var/lib/omega/.env
    set +a
fi

# 创建数据目录
mkdir -p "${OMEGA_DATA_DIR:-/var/lib/omega/data}/memory"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     ΩMEGA AGI SUPREMACY                                    ║"
echo "║     11-Layer Autonomous AGI System                         ║"
echo "║     Running in Docker Container                            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 执行传入的命令（默认 check）
exec omega-agi "$@"
