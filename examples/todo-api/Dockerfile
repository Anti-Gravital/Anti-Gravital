# syntax=docker/dockerfile:1
#
# Imagen de produccion de todo-api.
# Construye un binario estatico con MUSL y lo empaqueta en FROM scratch.
#
# Uso:
#   docker build -t todo-api .
#   docker run -e DATABASE_URL=postgresql://... -p 8080:8080 todo-api

FROM rust:1.79-slim AS builder

# Instala las herramientas necesarias para compilacion estatica con MUSL.
RUN apt-get update && apt-get install -y --no-install-recommends \
        musl-tools \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /build

# Copia el workspace completo para aprovechar la cache de capas Docker.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ ./crates/
COPY examples/todo-api/ ./examples/todo-api/
COPY templates/ ./templates/

RUN cargo build --release --target x86_64-unknown-linux-musl -p todo-api

FROM scratch

COPY --from=builder \
    /build/target/x86_64-unknown-linux-musl/release/todo-api \
    /todo-api

EXPOSE 8080

ENTRYPOINT ["/todo-api"]
