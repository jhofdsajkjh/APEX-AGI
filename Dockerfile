# ============================================================
# OMEGA AGI System - Production Dockerfile
# Multi-stage build: Rust compile -> Python runtime
# ============================================================

# ---------- Stage 1: Rust Build ----------
FROM rust:1.85-slim-bookworm AS rust-builder

WORKDIR /build

# Copy Rust crate manifests first (layer caching: rebuild only when deps change)
COPY omega-agi/Cargo.toml /build/omega-agi/
COPY omega-agi/hypercore/Cargo.toml /build/omega-agi/hypercore/
COPY omega-agi/runtime/Cargo.toml /build/omega-agi/runtime/
COPY omega-agi/engineering/Cargo.toml /build/omega-agi/engineering/
COPY omega-agi/evolution/Cargo.toml /build/omega-agi/evolution/
COPY omega-agi/adapters/Cargo.toml /build/omega-agi/adapters/

# Create dummy source files to cache dependencies
RUN mkdir -p /build/omega-agi/src \
    && echo "fn main() {}" > /build/omega-agi/src/main.rs \
    && for dir in hypercore runtime engineering evolution adapters; do \
         mkdir -p /build/omega-agi/$dir/src && echo "" > /build/omega-agi/$dir/src/lib.rs; \
       done \
    && cd /build/omega-agi && cargo build --release 2>/dev/null || true

# Now copy actual source code
COPY omega-agi/ /build/omega-agi/

# Build all Rust crates in release mode
RUN cd /build/omega-agi && cargo build --release

# ---------- Stage 2: Python Runtime ----------
FROM python:3.11-slim

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd --gid 1000 omega && \
    useradd --uid 1000 --gid omega --shell /bin/bash --create-home omega

WORKDIR /opt/omega-agi

# Copy Python pipeline code
COPY omega_pipeline/  /opt/omega-agi/omega_pipeline/
COPY examples/        /opt/omega-agi/examples/
COPY scripts/         /opt/omega-agi/scripts/
COPY web_ui/          /opt/omega-agi/web_ui/
COPY tests/           /opt/omega-agi/tests/

# Copy compiled Rust binary
COPY --from=rust-builder /build/omega-agi/target/release/omega-agi /usr/local/bin/omega-agi

# Copy Rust shared libraries for runtime use
COPY --from=rust-builder /build/omega-agi/target/release/libomega_hypercore*.so  /opt/omega-agi/rust-lib/ 2>/dev/null || true
COPY --from=rust-builder /build/omega-agi/target/release/libomega_runtime*.so    /opt/omega-agi/rust-lib/ 2>/dev/null || true
COPY --from=rust-builder /build/omega-agi/target/release/libomega_engineering*.so /opt/omega-agi/rust-lib/ 2>/dev/null || true
COPY --from=rust-builder /build/omega-agi/target/release/libomega_evolution*.so  /opt/omega-agi/rust-lib/ 2>/dev/null || true
COPY --from=rust-builder /build/omega-agi/target/release/libomega_adapters*.so   /opt/omega-agi/rust-lib/ 2>/dev/null || true

# Also copy full source for development capabilities
COPY --from=rust-builder /build/omega-agi/ /opt/omega-agi/rust-src/

# Create persistent directories
RUN mkdir -p /opt/omega-agi/evolution_runs /opt/omega-agi/memory && \
    chown -R omega:omega /opt/omega-agi

# Set environment variables
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    RUST_BACKTRACE=1 \
    PATH="/opt/omega-agi:${PATH}"

# Switch to non-root user
USER omega

# Health check
HEALTHCHECK --interval=60s --timeout=10s --start-period=30s --retries=3 \
    CMD python3 -c "import sys; sys.exit(0)"

# Default command: run self-evolution loop
CMD ["python3", "omega_pipeline/self_evolution_loop.py", "--auto"]
