#!/bin/bash
# =============================================================================
# OMEGA AGI - 新加坡节点快速部署脚本
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

echo "=============================================="
echo "  OMEGA AGI Supremacy - 新加坡节点部署"
echo "=============================================="
echo ""

# 1. 检查环境
log_step "检查部署环境..."
if [ ! -f ".env" ]; then
    log_warn ".env 文件不存在，创建默认配置..."
    cp .env.example .env
    log_warn "请编辑 .env 文件配置必要的认证信息"
fi

# 2. 检查Docker
if ! command -v docker &> /dev/null; then
    log_error "Docker 未安装"
    exit 1
fi

if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    log_error "Docker Compose 未安装"
    exit 1
fi

log_info "Docker 环境检查通过 ✓"

# 3. 创建目录
log_step "创建数据目录..."
mkdir -p data/{core,swarm,evolution}
mkdir -p logs
log_info "目录创建完成 ✓"

# 4. 构建镜像
log_step "构建 Docker 镜像..."
docker build -t omega-agi:local -f omega-agi/Dockerfile omega-agi/ || {
    log_warn "omega-agi 构建失败，尝试使用 Dockerfile..."
    docker build -t omega-agi:local . || {
        log_error "Docker 镜像构建失败"
        exit 1
    }
}
log_info "镜像构建完成 ✓"

# 5. 配置检查
log_step "配置检查..."
if grep -q "your_github_token_here\|your_app_secret_here" .env 2>/dev/null; then
    log_warn ".env 中包含未配置的项目，请检查："
    grep -E "^[A-Z_]+=.+(your_|xxx|example)" .env 2>/dev/null || true
fi

# 6. 启动服务
log_step "启动 OMEGA AGI 服务..."

# 使用 docker compose 或 docker-compose
if docker compose version &> /dev/null; then
    COMPOSE_CMD="docker compose"
else
    COMPOSE_CMD="docker-compose"
fi

# 启动核心服务
$COMPOSE_CMD up -d omega-core 2>/dev/null || {
    log_warn "docker compose 不可用，直接启动容器..."
    docker run -d \
        --name omega-core \
        -p 8080:8080 \
        -p 9090:9090 \
        -p 5000:5000 \
        --restart unless-stopped \
        -e OMEGA_MODE=production \
        -v $(pwd)/data:/app/data \
        omega-agi:local
}

log_info "服务启动完成 ✓"

# 7. 等待服务就绪
log_step "等待服务就绪..."
sleep 5

# 8. 健康检查
log_step "执行健康检查..."
if curl -sf http://localhost:8080/health &> /dev/null; then
    log_info "Core 服务健康 ✓"
else
    log_warn "Core 服务可能未就绪，请检查日志"
fi

if curl -sf http://localhost:5000/api/health &> /dev/null; then
    log_info "Web UI 服务健康 ✓"
fi

# 9. 显示状态
echo ""
echo "=============================================="
echo "  部署完成!"
echo "=============================================="
echo ""
echo "访问地址："
echo "  • Core API:     http://localhost:8080"
echo "  • Metrics:      http://localhost:9090"
echo "  • Web UI:       http://localhost:5000"
echo "  • Feishu Bot:   配置后可用"
echo ""
echo "常用命令："
echo "  • 查看日志:    docker logs -f omega-core"
echo "  • 停止服务:    $COMPOSE_CMD down"
echo "  • 重启服务:    $COMPOSE_CMD restart"
echo "  • Web UI日志:  tail -f logs/web_ui.log"
echo ""
echo "配置文件: $SCRIPT_DIR/.env"
echo "数据目录: $SCRIPT_DIR/data"
echo ""
