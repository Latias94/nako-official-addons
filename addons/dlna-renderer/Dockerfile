FROM rust:1.95 AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p nako-dlna-renderer

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/nako-dlna-renderer /usr/local/bin/nako-dlna-renderer
EXPOSE 9150
ENV NAKO_DLNA_RENDERER_LISTEN_ADDR=0.0.0.0:9150
CMD ["nako-dlna-renderer"]
