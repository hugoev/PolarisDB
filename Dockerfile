FROM rust:1.83 as builder

WORKDIR /usr/src/polarisdb

# Copy manifests first for caching
COPY Cargo.toml Cargo.lock ./
COPY polarisdb-core/Cargo.toml polarisdb-core/Cargo.toml
COPY polarisdb-server/Cargo.toml polarisdb-server/Cargo.toml
COPY polarisdb/Cargo.toml polarisdb/Cargo.toml
COPY py/Cargo.toml py/Cargo.toml

# Create dummy sources
RUN mkdir -p polarisdb-core/src && touch polarisdb-core/src/lib.rs
RUN mkdir -p polarisdb-server/src && echo "fn main() {}" > polarisdb-server/src/main.rs
RUN mkdir -p polarisdb/src && echo "fn main() {}" > polarisdb/src/main.rs
RUN mkdir -p py/src && touch py/src/lib.rs

# Build dependencies
RUN cargo build --release -p polarisdb-server

# Copy source code
COPY polarisdb-core/src polarisdb-core/src
COPY polarisdb-server/src polarisdb-server/src

# Build actual binary
RUN touch polarisdb-server/src/main.rs && cargo build --release -p polarisdb-server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/polarisdb/target/release/polarisdb-server /usr/local/bin/polarisdb-server

# Create data directory
RUN mkdir -p /app/data
VOLUME /app/data

EXPOSE 8080

CMD ["polarisdb-server"]
