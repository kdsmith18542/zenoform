FROM rust:1.95-slim AS rust-builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ghcr.io/software-mansion/scarb:2.18.0 AS scarb-builder
WORKDIR /app/cairo/terrain_v1
COPY cairo/terrain_v1 .
RUN scarb build

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=rust-builder /app/target/release/zenoform-cli /usr/local/bin/zenoform
COPY --from=scarb-builder /app/cairo/terrain_v1/target /app/cairo/target
ENTRYPOINT ["zenoform"]
