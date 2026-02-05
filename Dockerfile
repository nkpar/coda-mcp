# Build stage
FROM rust:1.93-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release \
    && rm -rf src \
    && rm -f target/release/coda-mcp* \
    && rm -f target/release/deps/coda_mcp* \
    && rm -rf target/release/.fingerprint/coda-mcp*

# Copy actual source and build
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/coda-mcp /usr/local/bin/

LABEL io.modelcontextprotocol.server.name="io.github.nkpar/coda"

# MCP servers communicate via stdio
ENTRYPOINT ["coda-mcp"]
