# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.95
ARG CARGO_CHEF_VERSION=0.1.77

FROM rust:${RUST_VERSION}-bookworm AS chef
ARG CARGO_CHEF_VERSION
WORKDIR /src
RUN cargo install cargo-chef --locked --version ${CARGO_CHEF_VERSION}

FROM chef AS planner
# Build with `--build-context nako=../nako` so local SDK path dependencies
# resolve in the same sibling layout used by developers and CI.
COPY . ./nako-official-addons
COPY --from=nako . ./nako
WORKDIR /src/nako-official-addons
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cacher
WORKDIR /src/nako-official-addons
COPY --from=planner /src/nako-official-addons/recipe.json recipe.json
RUN cargo chef cook --release --locked --package nako-resource-search --recipe-path recipe.json

FROM chef AS builder
WORKDIR /src
COPY . ./nako-official-addons
COPY --from=nako . ./nako
WORKDIR /src/nako-official-addons
COPY --from=cacher /src/nako-official-addons/target ./target
COPY --from=cacher /usr/local/cargo/registry /usr/local/cargo/registry
RUN cargo build --release --locked -p nako-resource-search

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/nako-official-addons/target/release/nako-resource-search /usr/local/bin/nako-resource-search
ENV NAKO_RESOURCE_SEARCH_LISTEN_ADDR=0.0.0.0:9130
ENV NAKO_RESOURCE_SEARCH_BASE_URL=http://nako-resource-search:9130
ENV NAKO_RESOURCE_SEARCH_FIXTURE_PROVIDER_ENABLED=true
ENV NAKO_RESOURCE_SEARCH_PANSOU_PROVIDER_ENABLED=false
EXPOSE 9130
ENTRYPOINT ["/usr/local/bin/nako-resource-search"]
