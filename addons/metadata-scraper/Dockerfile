# syntax=docker/dockerfile:1.7

FROM rust:1.89-bookworm AS builder
WORKDIR /src
COPY nako-official-addons ./nako-official-addons
COPY taru/crates/taru-addon-protocol ./taru/crates/taru-addon-protocol
WORKDIR /src/nako-official-addons
RUN cargo build --release -p nako-metadata-scraper

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/nako-official-addons/target/release/nako-metadata-scraper /usr/local/bin/nako-metadata-scraper
ENV NAKO_METADATA_SCRAPER_LISTEN_ADDR=0.0.0.0:9100
EXPOSE 9100
ENTRYPOINT ["/usr/local/bin/nako-metadata-scraper"]
