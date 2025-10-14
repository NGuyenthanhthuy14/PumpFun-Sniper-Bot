FROM rust:1.85 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src
RUN cargo build --release || true

COPY src ./src

RUN cargo build --release

FROM ubuntu:22.04

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ /usr/local/bin/

CMD ["sniper_mode"]