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
RUN cargo chef cook --release --locked --package nako-metadata-scraper --recipe-path recipe.json

FROM chef AS builder
WORKDIR /src
COPY . ./nako-official-addons
WORKDIR /src/nako-official-addons
COPY --from=cacher /src/nako-official-addons/target ./target
COPY --from=cacher /usr/local/cargo/registry /usr/local/cargo/registry
RUN cargo build --release --locked -p nako-metadata-scraper

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/nako-official-addons/target/release/nako-metadata-scraper /usr/local/bin/nako-metadata-scraper
ENV NAKO_METADATA_SCRAPER_LISTEN_ADDR=0.0.0.0:9100
ENV NAKO_METADATA_SCRAPER_BASE_URL=http://127.0.0.1:9100
ENV NAKO_METADATA_SCRAPER_LANGUAGE=en-US
ENV NAKO_METADATA_SCRAPER_PROVIDER_FIXTURE_ENABLED=true
ENV NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED=false
ENV NAKO_METADATA_SCRAPER_PROVIDER_BANGUMI_ENABLED=false
ENV NAKO_METADATA_SCRAPER_BANGUMI_USER_AGENT="Latias94/nako-official-addons/nako-metadata-scraper/0.1.0-alpha.1 (https://github.com/Latias94/nako-official-addons)"
EXPOSE 9100
ENTRYPOINT ["/usr/local/bin/nako-metadata-scraper"]
