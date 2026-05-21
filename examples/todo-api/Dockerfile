# syntax=docker/dockerfile:1
#
# Imagen de produccion de todo-api.
# Construye un binario estatico con MUSL y lo empaqueta en FROM scratch.
#
# Uso:
#   docker build -t todo-api .
#   docker run -e DATABASE_URL=postgresql://... -p 8080:8080 todo-api

FROM rust:1.95-slim AS builder

# Instala las herramientas necesarias para compilacion estatica con MUSL.
RUN apt-get update && apt-get install -y --no-install-recommends \
        musl-tools \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copia rust-toolchain.toml primero para que rustup use el canal correcto
# antes de instalar el target MUSL.
COPY rust-toolchain.toml ./
RUN rustup target add x86_64-unknown-linux-musl

# Copia el resto del workspace.
COPY Cargo.toml Cargo.lock ./
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
