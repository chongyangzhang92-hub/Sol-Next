FROM rust:1.85-slim AS builder

WORKDIR /src
COPY Cargo.toml /src/Cargo.toml
COPY src /src/src
RUN cargo build --release --bins

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl android-tools-adb \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/sol-cli /usr/local/bin/sol-cli
COPY --from=builder /src/target/release/sol-mcp /usr/local/bin/sol-mcp
ENTRYPOINT ["/usr/local/bin/sol-mcp"]
