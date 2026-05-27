# Stage 1: Build
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /build
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

RUN cargo build --release --manifest-path omega-agi/Cargo.toml -p omega-agi

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates git && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/omega-agi/target/release/omega-agi /usr/local/bin/omega-agi
COPY --from=builder /build/.env.example /etc/omega-agi/.env.example

WORKDIR /workspace
VOLUME ["/workspace/data"]

EXPOSE 8080

ENTRYPOINT ["omega-agi"]
CMD ["--help"]
