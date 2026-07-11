# syntax=docker/dockerfile:1.7

FROM rust:1.90-slim-bookworm AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache deps — build a stub main so dep compilation is cached independently of src changes
COPY Cargo.toml Cargo.lock ./
COPY xtask/Cargo.toml xtask/Cargo.toml
RUN --mount=type=cache,id=unifi-rmcp-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=unifi-rmcp-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=unifi-rmcp-cargo-target,target=/app/target,sharing=locked \
    mkdir -p src xtask/src && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > xtask/src/main.rs && \
    cargo build --release --locked -p rustifi && \
    rm -rf src

# Build real binary
COPY data/ data/
COPY src/ src/
RUN --mount=type=cache,id=unifi-rmcp-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=unifi-rmcp-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=unifi-rmcp-cargo-target,target=/app/target,sharing=locked \
    touch src/main.rs && cargo build --release --locked -p rustifi && cp target/release/runifi /usr/local/bin/runifi

# Runtime — minimal debian image with ca-certificates for TLS to the UniFi controller
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl gosu && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/runifi /usr/local/bin/runifi
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

RUN groupadd --gid 1000 unifi && \
    useradd --uid 1000 --gid unifi --no-create-home --shell /sbin/nologin unifi && \
    mkdir -p /data && chown unifi:unifi /data

ENV RUST_LOG=info

EXPOSE 40030/tcp

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -sf http://localhost:40030/health || exit 1

LABEL io.modelcontextprotocol.server.name="tv.tootie/unifi-rmcp"

# Entrypoint runs as root, validates env, chowns /data, then execs as 1000:1000
ENTRYPOINT ["/entrypoint.sh"]
CMD ["serve", "mcp"]
