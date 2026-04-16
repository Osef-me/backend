FROM rust:bookworm AS builder

WORKDIR /app

# Cache dependencies first (layer cache)
COPY processor/Cargo.toml processor/Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Build real source
COPY processor/src/ src/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/processor /usr/local/bin/processor

CMD ["processor"]
