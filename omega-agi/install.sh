#!/usr/bin/env bash
# =============================================================================
# ΩMEGA AGI SUPREMACY — 一键安装脚本 (Linux / macOS)
# =============================================================================
# 使用方法:
#   curl -fsSL https://raw.githubusercontent.com/omega-agi/omega-agi/main/install.sh | bash
#   或
#   chmod +x install.sh && ./install.sh
# =============================================================================

set -euo pipefail

# ── 颜色 ──
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# ── 检测平台 ──
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)   PLATFORM="linux-$ARCH" ;;
    Darwin)  PLATFORM="macos-$ARCH" ;;
    *)
        echo -e "${RED}❌ 不支持的操作系统: $OS${NC}"
        echo "请使用 Docker 部署: docker compose up --build"
        exit 1
        ;;
esac

echo -e "${CYAN}"
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     ΩMEGA AGI SUPREMACY — 一键安装                        ║"
echo "║     11-Layer Autonomous AGI System                         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# ── 检查依赖 ──
echo -e "${YELLOW}🔍 检查系统依赖...${NC}"

# Rust
if ! command -v rustc &>/dev/null; then
    echo -e "${YELLOW}📦 安装 Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "${GREEN}✅ Rust 安装完成: $(rustc --version)${NC}"
else
    echo -e "${GREEN}✅ Rust: $(rustc --version)${NC}"
fi

# Cargo
if ! command -v cargo &>/dev/null; then
    echo -e "${RED}❌ Cargo 未找到，请重新加载 shell 或运行: source \$HOME/.cargo/env${NC}"
    exit 1
fi

# Git
if ! command -v git &>/dev/null; then
    echo -e "${YELLOW}📦 安装 Git...${NC}"
    if [ "$OS" = "Linux" ]; then
        sudo apt-get update && sudo apt-get install -y git
    elif [ "$OS" = "Darwin" ]; then
        xcode-select --install 2>/dev/null || true
    fi
    echo -e "${GREEN}✅ Git 安装完成${NC}"
fi

# ── 克隆 / 进入目录 ──
REPO_DIR="omega-agi"

if [ -d "$REPO_DIR" ]; then
    echo -e "${YELLOW}📂 目录已存在，更新...${NC}"
    cd "$REPO_DIR"
    git pull --ff-only || echo -e "${YELLOW}⚠️  更新失败，继续使用现有代码${NC}"
else
    echo -e "${CYAN}📦 克隆仓库...${NC}"
    git clone --depth 1 https://github.com/omega-agi/omega-agi.git
    cd "$REPO_DIR"
fi

# ── 配置 ──
if [ ! -f .env ]; then
    cp .env.example .env
    echo -e "${YELLOW}⚠️  请编辑 .env 文件填入你的 API Key${NC}"
    echo "  nano .env"
fi

# ── 构建 ──
echo -e "${CYAN}🔧 构建项目（release）...${NC}"
cargo build --release --features full

# ── 安装到系统 ──
echo -e "${CYAN}📦 安装到系统...${NC}"
sudo mkdir -p /usr/local/bin /var/lib/omega/data
sudo cp target/release/omega-agi /usr/local/bin/omega-agi
sudo chmod +x /usr/local/bin/omega-agi

# ── 验证 ──
echo ""
echo -e "${GREEN}✅ ΩMEGA AGI 安装完成！${NC}"
echo ""
echo -e "  二进制路径: ${CYAN}/usr/local/bin/omega-agi${NC}"
echo -e "  数据目录:   ${CYAN}/var/lib/omega/data${NC}"
echo ""
echo -e "  ${YELLOW}快速启动:${NC}"
echo "    omega-agi check          # 系统健康检查"
echo "    omega-agi interactive     # 交互模式"
echo "    omega-agi evolve          # 自我进化"
echo "    omega-agi apex --verbose   # Φ_APEX*∞ 进化"
echo ""
echo -e "  ${YELLOW}安装为 systemd 服务:${NC}"
echo "    sudo make install-service"
echo "    sudo systemctl start omega-agi"
echo "    sudo journalctl -u omega-agi -f"
echo ""
echo -e "  ${YELLOW}Docker 部署:${NC}"
echo "    docker compose up --build -d"
echo ""

# 运行验证
echo -e "${CYAN}🔍 运行验证...${NC}"
/usr/local/bin/omega-agi check || echo -e "${YELLOW}⚠️  请配置 .env 后重试${NC}"
