# syntax=docker/dockerfile:1.7
# Optimized multi-stage build for Wiki.js Meilisearch module delivery.

ARG VERSION=dev
ARG RUST_IMAGE=rust:1.91-bookworm
ARG NODE_IMAGE=node:24-slim
ARG PNPM_VERSION=10.19.0

########################
# 1) Base toolchain
########################
FROM ${RUST_IMAGE} AS rust-base

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

ENV VERSION=$VERSION 

# Reuse dependency artifacts
COPY --from=deps /app/target /app/target
COPY --from=deps /usr/local/cargo /usr/local/cargo

# Build wasm package and the delivery binary with caching
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    RUSTFLAGS="" wasm-pack build --release --out-dir pkg && \
    echo "$VERSION" > VERSION

FROM ${NODE_IMAGE} AS node-base


WORKDIR /app

COPY ./package.json ./
COPY ./pnpm-lock.yaml ./

ENV NODE_ENV=production
RUN apt-get update -y && apt-get upgrade -y && rm -rf /var/lib/apt/lists/* && \
    corepack enable && corepack prepare pnpm@${PNPM_VERSION} --activate

# Install JS deps with cache for pnpm store
RUN --mount=type=cache,id=pnpm-store,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile --prefer-offline

COPY . .
COPY --from=builder /app/pkg ./pkg

# Build the JS bundle with VERSION propagated for dist/VERSION
RUN VERSION=$VERSION pnpm run build:js


########################
# 5) Minimal runtime image (glibc)
########################
FROM debian:bookworm-slim AS runtime
ARG VERSION=dev
LABEL org.opencontainers.image.title="Wiki.js Meilisearch Module" \
    org.opencontainers.image.description="WASM-powered Meilisearch module for Wiki.js" \
    org.opencontainers.image.version=$VERSION \
    org.opencontainers.image.source="https://github.com/mbround18/wikijs-module-meilisearch"
WORKDIR /

# Default locations inside the container for copy tool
ENV SOURCE=/modules/meilisearch \
    DESTINATION=/wiki/server/modules/meilisearch

# Module assets (bundled engine + pkg + metadata)
COPY --from=node-base /app/dist /modules/meilisearch
COPY ./docs/assets/logo.png /modules/meilisearch/docs/assets/logo.png

# Nice entrypoint banner + command passthrough
COPY ./scripts/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh && useradd -m -u 10001 appuser \
    && echo "${VERSION}" > /modules/meilisearch/VERSION
USER appuser
ENTRYPOINT ["/entrypoint.sh"]
