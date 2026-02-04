# ---- Stage 1: Build ----
FROM rust:1.93-slim-bookworm AS builder

RUN apt-get update && apt-get install -y protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace manifests and lock for dependency caching
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY blog-server/Cargo.toml blog-server/Cargo.toml
COPY blog-client/Cargo.toml blog-client/Cargo.toml
COPY blog-cli/Cargo.toml blog-cli/Cargo.toml
COPY blog-wasm/Cargo.toml blog-wasm/Cargo.toml

# Dummy source files so cargo can resolve the workspace and cache dependencies
RUN mkdir -p blog-server/src blog-client/src blog-cli/src blog-wasm/src && \
    echo "fn main() {}" > blog-server/src/main.rs && \
    echo "fn main() {}" > blog-cli/src/main.rs && \
    touch blog-client/src/lib.rs && \
    touch blog-wasm/src/lib.rs

# Proto files and build scripts needed at compile time
COPY blog-server/build.rs blog-server/build.rs
COPY blog-server/proto blog-server/proto
COPY blog-client/build.rs blog-client/build.rs
COPY blog-client/proto blog-client/proto

# Pre-build dependencies (may partially fail on dummy source, but deps get cached)
RUN cargo build --release -p blog-server || true

# Copy real source code
COPY blog-server/src blog-server/src
COPY blog-server/migrations blog-server/migrations
COPY blog-client/src blog-client/src

# Invalidate cache for the real source and build
RUN touch blog-server/src/main.rs blog-client/src/lib.rs && \
    cargo build --release --bin blog-server

# ---- Stage 2: Runtime ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

RUN addgroup --system app && adduser --system --ingroup app app

COPY --from=builder /app/target/release/blog-server /usr/local/bin/blog-server

WORKDIR /app

USER app

EXPOSE 3000 50051

HEALTHCHECK --interval=30s --timeout=3s \
    CMD curl -f http://localhost:3000/api/v1/health || exit 1

CMD ["blog-server"]