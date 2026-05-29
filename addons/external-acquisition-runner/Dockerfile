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
RUN cargo chef cook --release --locked --package nako-external-acquisition-runner --recipe-path recipe.json

FROM chef AS builder
WORKDIR /src
COPY . ./nako-official-addons
COPY --from=nako . ./nako
WORKDIR /src/nako-official-addons
COPY --from=cacher /src/nako-official-addons/target ./target
COPY --from=cacher /usr/local/cargo/registry /usr/local/cargo/registry
RUN cargo build --release --locked -p nako-external-acquisition-runner

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/nako-official-addons/target/release/nako-external-acquisition-runner /usr/local/bin/nako-external-acquisition-runner
ENV NAKO_EXTERNAL_ACQUISITION_RUNNER_LISTEN_ADDR=0.0.0.0:9160
ENV NAKO_EXTERNAL_ACQUISITION_RUNNER_BASE_URL=http://nako-external-acquisition-runner:9160
ENV NAKO_EXTERNAL_ACQUISITION_RUNNER_FIXTURE_PROFILE_ENABLED=true
ENV NAKO_EXTERNAL_ACQUISITION_RUNNER_DEFAULT_PROFILE_ID=fixture
EXPOSE 9160
ENTRYPOINT ["/usr/local/bin/nako-external-acquisition-runner"]
