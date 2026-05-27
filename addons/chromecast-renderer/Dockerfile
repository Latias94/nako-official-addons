# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.95
ARG CARGO_CHEF_VERSION=0.1.77

FROM rust:${RUST_VERSION}-bookworm AS chef
ARG CARGO_CHEF_VERSION
WORKDIR /src
RUN cargo install cargo-chef --locked --version ${CARGO_CHEF_VERSION}

FROM chef AS planner
COPY . ./nako-official-addons
WORKDIR /src/nako-official-addons
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cacher
WORKDIR /src/nako-official-addons
COPY --from=planner /src/nako-official-addons/recipe.json recipe.json
RUN cargo chef cook --release --locked --package nako-chromecast-renderer --recipe-path recipe.json

FROM chef AS builder
WORKDIR /src
COPY . ./nako-official-addons
WORKDIR /src/nako-official-addons
COPY --from=cacher /src/nako-official-addons/target ./target
COPY --from=cacher /usr/local/cargo/registry /usr/local/cargo/registry
RUN cargo build --release --locked -p nako-chromecast-renderer

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/nako-official-addons/target/release/nako-chromecast-renderer /usr/local/bin/nako-chromecast-renderer
ENV NAKO_CHROMECAST_RENDERER_LISTEN_ADDR=0.0.0.0:9120
ENV NAKO_CHROMECAST_RENDERER_BASE_URL=http://nako-chromecast-renderer:9120
ENV NAKO_CHROMECAST_RENDERER_LIVE_DISCOVERY_ENABLED=false
ENV NAKO_CHROMECAST_RENDERER_LIVE_CONTROL_ENABLED=false
EXPOSE 9120
ENTRYPOINT ["/usr/local/bin/nako-chromecast-renderer"]
