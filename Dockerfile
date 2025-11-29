 # syntax=docker/dockerfile:1.7
 # Optimized multi-stage build for Wiki.js Meilisearch module delivery.
 # Uses cargo-chef + sccache + BuildKit cache mounts for fast incremental rebuilds.

########################
# 1) Base toolchain
########################
FROM rust:1.91-bookworm AS rust-base

ARG VERSION=dev
ENV VERSION=$VERSION \
    CARGO_TERM_COLOR=always \
    CARGO_HOME=/usr/local/cargo \
    RUSTFLAGS="-C debuginfo=0 -C link-arg=-fuse-ld=mold"

RUN apt-get update && apt-get install -y --no-install-recommends \
      pkg-config libssl-dev ca-certificates clang mold curl nodejs npm && \
    rm -rf /var/lib/apt/lists/*

# Ensure wasm target is available up front (avoids rustup network during builds)
RUN rustup target add wasm32-unknown-unknown

# Tooling: cargo-chef, wasm-pack
RUN cargo install --locked cargo-chef wasm-pack

########################
# 2) Cargo Chef prepare (dependency graph)
########################
FROM rust-base AS chef
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
# (If multi-crate workspace, copy each crate's Cargo.toml similarly.)
RUN cargo chef prepare --recipe-path recipe.json

########################
# 3) Dependency build layer
########################
FROM rust-base AS deps
WORKDIR /app
COPY --from=chef /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo chef cook --release --recipe-path recipe.json

########################
# 4) Build source (wasm + binary)
########################
FROM rust-base AS builder
WORKDIR /app
COPY . .

ARG VERSION=dev
ENV VERSION=$VERSION 

# Reuse dependency artifacts
COPY --from=deps /app/target /app/target
COPY --from=deps /usr/local/cargo /usr/local/cargo

# Build wasm package (bundler target) and the delivery binary with caching
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    # Unset RUSTFLAGS for wasm build (rust-lld rejects -fuse-ld=mold) then restore for native build
    RUSTFLAGS="" wasm-pack build --release --target bundler --out-dir pkg && \
    echo "$VERSION" > VERSION && \
    cargo build --release --locked --bin wiki_meilisearch

########################
# 5) Minimal runtime image (glibc)
########################
FROM gcr.io/distroless/cc-debian13:nonroot AS runtime
WORKDIR /

# Default locations inside the container for copy tool
ENV SOURCE=/modules/meilisearch \
    DESTINATION=/wiki/server/modules/meilisearch

# Module assets
COPY --from=builder /app/pkg /modules/meilisearch/pkg
COPY --from=builder /app/engine.js /modules/meilisearch/engine.js
COPY --from=builder /app/definition.yml /modules/meilisearch/definition.yml
COPY --from=builder /app/README.md /modules/meilisearch/README.md
COPY --from=builder /app/LICENSE /modules/meilisearch/LICENSE
COPY --from=builder /app/VERSION /modules/meilisearch/VERSION
COPY --from=builder /app/target/release/wiki_meilisearch /wiki_meilisearch
# Optional logo (ignore if absent)
COPY --from=builder /app/docs/assets/logo.png /modules/meilisearch/docs/assets/logo.png

USER nonroot
ENTRYPOINT ["/wiki_meilisearch"]
CMD ["copy"]

# ---- Notes ----
# For even smaller runtime, consider switching reqwest to rustls and building a MUSL target, then
#   FROM gcr.io/distroless/static:nonroot
#   (Requires: rustup target add x86_64-unknown-linux-musl and dependencies adjusted.)
# Enable BuildKit for best performance: DOCKER_BUILDKIT=1 docker build .
