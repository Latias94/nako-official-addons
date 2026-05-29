FROM rust:1.95 AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p nako-subtitle-provider

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/nako-subtitle-provider /usr/local/bin/nako-subtitle-provider
EXPOSE 9140
ENV NAKO_SUBTITLE_PROVIDER_LISTEN_ADDR=0.0.0.0:9140
CMD ["nako-subtitle-provider"]
