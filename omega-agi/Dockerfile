# =============================================================================
# ΩMEGA AGI SUPREMACY — 多阶段 Docker 构建
# =============================================================================
# 构建阶段
FROM rust:1.78-slim-bookworm AS builder

LABEL maintainer="OMEGA AGI <omega@agi.system>"
LABEL description="ΩMEGA AGI SUPREMACY — 11-Layer Autonomous AGI System"

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev musl-tools \
    && rm -rf /var/lib/apt/lists/*

# 创建项目目录
WORKDIR /build

# 先复制 Cargo.toml/Cargo.lock 以缓存依赖
COPY Cargo.toml ./
COPY hypercore/Cargo.toml ./hypercore/
COPY runtime/Cargo.toml ./runtime/
COPY engineering/Cargo.toml ./engineering/
COPY evolution/Cargo.toml ./evolution/
COPY adapters/Cargo.toml ./adapters/
COPY agent/Cargo.toml ./agent/
COPY research/Cargo.toml ./research/
COPY life_harness/Cargo.toml ./life_harness/
COPY superpowers/Cargo.toml ./superpowers/
COPY avatar/Cargo.toml ./avatar/
COPY transcendence/Cargo.toml ./transcendence/

# 创建 src 目录结构使依赖能够解析
RUN mkdir -p src \
    hypercore/src runtime/src engineering/src evolution/src \
    adapters/src agent/src research/src \
    life_harness/src superpowers/src avatar/src transcendence/src \
    && echo "fn main() {}" > src/main.rs \
    && echo "" > src/lib.rs \
    && for d in hypercore runtime engineering evolution adapters agent research life_harness superpowers avatar transcendence; do \
         echo "" > "$d/src/lib.rs"; \
       done

# 构建依赖缓存
RUN cargo build --release --features full 2>/dev/null || true

# 复制实际源代码
COPY src/ ./src/
COPY hypercore/src/ ./hypercore/src/
COPY runtime/src/ ./runtime/src/
COPY engineering/src/ ./engineering/src/
COPY evolution/src/ ./evolution/src/
COPY adapters/src/ ./adapters/src/
COPY agent/src/ ./agent/src/
COPY research/src/ ./research/src/
COPY life_harness/src/ ./life_harness/src/
COPY superpowers/src/ ./superpowers/src/
COPY avatar/src/ ./avatar/src/
COPY transcendence/src/ ./transcendence/src/

# 完整构建
RUN cargo build --release --features full

# =============================================================================
# 运行时阶段 — distroless for minimal attack surface
# =============================================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建 omega 用户
RUN groupadd -r omega && useradd -r -g omega -d /var/lib/omega -s /sbin/nologin omega

WORKDIR /var/lib/omega

# 从构建阶段复制二进制
COPY --from=builder /build/target/release/omega-agi /usr/local/bin/omega-agi

# 创建数据目录
RUN mkdir -p /var/lib/omega/data && chown -R omega:omega /var/lib/omega

# 复制入口脚本
COPY scripts/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

USER omega

ENV OMEGA_DATA_DIR=/var/lib/omega/data
ENV OMEGA_LOG_LEVEL=info
ENV OMEGA_TRANSCENDENCE=true

EXPOSE 8080

ENTRYPOINT ["/entrypoint.sh"]
CMD ["check"]

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD omega-agi check || exit 1
