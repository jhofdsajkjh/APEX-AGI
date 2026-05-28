# =============================================================================
# ΩMEGA AGI SUPREMACY — Makefile
# =============================================================================
# 目标：构建 · 测试 · 部署 · 一键运行
# =============================================================================

.PHONY: all build check test release run clean install uninstall \
        docker docker-build docker-run docker-stop \
        setup help env lint clippy fmt docs \
        install-linux install-service upgrade

SHELL := /bin/bash

APP_NAME := omega-agi
BINARY := target/release/$(APP_NAME)

# === 颜色输出 ===
RED    := \033[0;31m
GREEN  := \033[0;32m
YELLOW := \033[1;33m
CYAN   := \033[0;36m
NC     := \033[0m

# =============================================================================
# 构建
# =============================================================================

all: check build release

## 编译（调试模式）
build:
	@echo -e "$(CYAN)🔧 Building $(APP_NAME) (debug)...$(NC)"
	cargo build --workspace --all-features
	@echo -e "$(GREEN)✅ Build complete (debug)$(NC)"

## 编译发布模式
release:
	@echo -e "$(CYAN)🔧 Building $(APP_NAME) (release)...$(NC)"
	cargo build --release --features full
	@echo -e "$(GREEN)✅ Build complete (release)$(NC)"
	@ls -lh $(BINARY)

## 编译检查（快速）
check:
	@echo -e "$(CYAN)🔍 Checking $(APP_NAME)...$(NC)"
	cargo check --workspace --all-features
	@echo -e "$(GREEN)✅ Check passed$(NC)"

## 清理
clean:
	@echo -e "$(YELLOW)🧹 Cleaning...$(NC)"
	cargo clean
	rm -rf data/
	@echo -e "$(GREEN)✅ Cleaned$(NC)"

# =============================================================================
# 测试 & 质量
# =============================================================================

## 运行全部测试
test:
	@echo -e "$(CYAN)🧪 Running tests...$(NC)"
	cargo test --workspace --all-features
	@echo -e "$(GREEN)✅ All tests passed$(NC)"

## Lint 检查
lint: clippy fmt

## Clippy
clippy:
	@echo -e "$(CYAN)🔍 Running clippy...$(NC)"
	cargo clippy --workspace --all-features -- -D warnings
	@echo -e "$(GREEN)✅ Clippy passed$(NC)"

## 格式化检查
fmt:
	@echo -e "$(CYAN)🔍 Checking formatting...$(NC)"
	cargo fmt --all --check
	@echo -e "$(GREEN)✅ Formatting check passed$(NC)"

## 生成文档
docs:
	@echo -e "$(CYAN)📖 Generating docs...$(NC)"
	cargo doc --workspace --no-deps --document-private-items
	@echo -e "$(GREEN)✅ Docs generated at target/doc/$(APP_NAME)/$(NC)"

# =============================================================================
# 运行
# =============================================================================

## 运行健康检查
run:
	@echo -e "$(CYAN)🚀 Starting $(APP_NAME)...$(NC)"
	cargo run -- check

## 交互模式
chat:
	@echo -e "$(CYAN)💬 Starting interactive mode...$(NC)"
	cargo run -- interactive

## 自我进化
evolve:
	@echo -e "$(CYAN)🧬 Running self-evolution...$(NC)"
	cargo run -- evolve

## APEX 进化
apex:
	@echo -e "$(CYAN)⚡ Running Φ_APEX*∞ evolution...$(NC)"
	cargo run -- apex --verbose

## 运行发布二进制
run-release:
	@echo -e "$(CYAN)🚀 Starting $(APP_NAME) (release)...$(NC)"
	$(BINARY) check

# =============================================================================
# 安装 & 部署
# =============================================================================

## 安装到系统（Linux/macOS）
install: release
	@echo -e "$(CYAN)📦 Installing $(APP_NAME)...$(NC)"
	@mkdir -p /usr/local/bin /var/lib/omega/data
	@cp $(BINARY) /usr/local/bin/$(APP_NAME)
	@cp .env.example /var/lib/omega/.env 2>/dev/null || true
	@echo -e "$(GREEN)✅ Installed: /usr/local/bin/$(APP_NAME)$(NC)"
	@echo -e "$(GREEN)✅ Data dir: /var/lib/omega/data$(NC)"
	@echo -e "$(YELLOW)⚠️  Edit /var/lib/omega/.env to set your API keys$(NC)"

## 卸载
uninstall:
	@echo -e "$(YELLOW)🗑️  Uninstalling $(APP_NAME)...$(NC)"
	@rm -f /usr/local/bin/$(APP_NAME)
	@echo -e "$(GREEN)✅ Uninstalled$(NC)"

## 安装为 systemd 服务
install-service: install
	@echo -e "$(CYAN)🔧 Installing systemd service...$(NC)"
	@cat > /tmp/omega-agi.service << 'SERVICEEOF'
[Unit]
Description=ΩMEGA AGI SUPREMACY — 11-Layer Autonomous AGI System
Documentation=https://github.com/omega-agi/omega-agi
After=network.target

[Service]
Type=simple
User=omega
Group=omega
WorkingDirectory=/var/lib/omega
EnvironmentFile=/var/lib/omega/.env
Environment=OMEGA_DATA_DIR=/var/lib/omega/data
ExecStart=/usr/local/bin/omega-agi check
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SERVICEEOF
	@sudo mv /tmp/omega-agi.service /etc/systemd/system/omega-agi.service
	@sudo useradd -r -s /sbin/nologin -d /var/lib/omega omega 2>/dev/null || true
	@sudo chown -R omega:omega /var/lib/omega
	@sudo systemctl daemon-reload
	@echo -e "$(GREEN)✅ Service installed$(NC)"
	@echo "  Start:  sudo systemctl start omega-agi"
	@echo "  Enable: sudo systemctl enable omega-agi"
	@echo "  Status: sudo systemctl status omega-agi"
	@echo "  Logs:   sudo journalctl -u omega-agi -f"

## 升级
upgrade: release
	@echo -e "$(CYAN)⬆️  Upgrading $(APP_NAME)...$(NC)"
	@sudo systemctl stop omega-agi 2>/dev/null || true
	@sudo cp $(BINARY) /usr/local/bin/$(APP_NAME)
	@sudo systemctl start omega-agi 2>/dev/null || true
	@echo -e "$(GREEN)✅ Upgrade complete$(NC)"

# =============================================================================
# Docker
# =============================================================================

## Docker 构建
docker-build:
	@echo -e "$(CYAN)🐳 Building Docker image...$(NC)"
	docker compose build
	@echo -e "$(GREEN)✅ Docker build complete$(NC)"

## Docker 启动
docker-run:
	@echo -e "$(CYAN)🐳 Starting Docker services...$(NC)"
	docker compose up -d
	@echo -e "$(GREEN)✅ Docker services started$(NC)"
	docker compose logs -f

## Docker 停止
docker-stop:
	@echo -e "$(YELLOW)🛑 Stopping Docker services...$(NC)"
	docker compose down
	@echo -e "$(GREEN)✅ Docker services stopped$(NC)"

## Docker 重新构建启动
docker: docker-build docker-run

# =============================================================================
# 初始化
# =============================================================================

## 初始化开发环境
setup:
	@echo -e "$(CYAN)🔧 Setting up development environment...$(NC)"
	@rustup show
	@rustc --version
	@cargo --version
	@cp -n .env.example .env 2>/dev/null || true
	@echo -e "$(GREEN)✅ Environment ready$(NC)"
	@echo "  - Edit .env with your API keys"
	@echo "  - Run 'make check' to verify build"

## 显示帮助
help:
	@echo ""
	@echo "╔══════════════════════════════════════════════════════╗"
	@echo "║      ΩMEGA AGI SUPREMACY — Makefile Help            ║"
	@echo "╚══════════════════════════════════════════════════════╝"
	@echo ""
	@echo "  $(CYAN)Build:$(NC)"
	@echo "    make          check + build + release"
	@echo "    make build    编译 (debug)"
	@echo "    make release  编译 (release)"
	@echo "    make check    快速编译检查"
	@echo "    make clean    清理构建产物"
	@echo ""
	@echo "  $(CYAN)Test:$(NC)"
	@echo "    make test     运行全部测试"
	@echo "    make lint     clippy + fmt"
	@echo "    make clippy   代码质量检查"
	@echo "    make fmt      格式化检查"
	@echo ""
	@echo "  $(CYAN)Run:$(NC)"
	@echo "    make run      运行健康检查"
	@echo "    make chat     交互模式"
	@echo "    make evolve   自我进化"
	@echo "    make apex     Φ_APEX*∞ 进化"
	@echo ""
	@echo "  $(CYAN)Deploy:$(NC)"
	@echo "    make install  安装到系统"
	@echo "    make install-service 安装为 systemd 服务"
	@echo "    make uninstall  卸载"
	@echo "    make upgrade   升级"
	@echo ""
	@echo "  $(CYAN)Docker:$(NC)"
	@echo "    make docker      构建 + 启动"
	@echo "    make docker-build 构建镜像"
	@echo "    make docker-run   启动服务"
	@echo "    make docker-stop  停止服务"
	@echo ""
	@echo "  $(CYAN)Other:$(NC)"
	@echo "    make setup   初始化开发环境"
	@echo "    make docs    生成文档"
	@echo "    make help    显示此帮助"
	@echo ""
	@echo "  $(YELLOW)Examples:$(NC)"
	@echo "    make setup && make check && make test"
	@echo "    make docker"
	@echo "    make install-service && sudo systemctl start omega-agi"
	@echo ""

# 默认目标
.DEFAULT_GOAL := help
