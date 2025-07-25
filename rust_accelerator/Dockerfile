# ------------------------------------------------------------------------------
# Rust Microservice Dockerfile - Multi-stage Build for Production
#
# This Dockerfile builds a Rust microservice in two stages:
# 1. Build Stage: Uses official Rust image to compile the project in release mode.
#    - Caches dependencies for faster rebuilds.
#    - Produces an optimized binary.
#
# 2. Runtime Stage: Uses a minimal Debian slim image for small, secure runtime.
#    - Copies the binary from the build stage.
#    - Installs necessary runtime libraries (e.g., ca-certificates for HTTPS).
#    - Exposes the service port.
#    - Runs the compiled binary as the container entrypoint.
#
# Detailed steps and reasoning:
#
# Stage 1 - Build Stage:
#   - FROM rust:1.88 as builder
#     Selects official Rust image with compiler and tools.
#
#   - WORKDIR /usr/src/rust_accelerator
#     Sets working directory inside container; all commands run here.
#
#   - COPY rust_accelerator/Cargo.toml rust_accelerator/Cargo.lock* ./
#     Copies only manifest files to leverage Docker layer caching.
#
#   - RUN mkdir src && echo "fn main() {}" > src/main.rs
#     Creates dummy main.rs to allow cargo to compile dependencies without full source.
#
#   - RUN cargo build --release || true
#     Builds dependencies in release mode to cache them.
#
#   - RUN rm -f src/main.rs
#     Removes dummy main.rs to prepare for actual source code.
#
#   - COPY rust_accelerator/src ./src
#     Copies real source code into container.
#
#   - RUN cargo build --release
#     Compiles optimized binary of your microservice.
#
# Stage 2 - Runtime Stage:
#   - FROM debian:bullseye-slim
#     Starts with minimal Debian image for small runtime footprint.
#
#   - RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
#     Installs SSL certificates for HTTPS requests; cleans apt cache for smaller image.
#
#   - COPY --from=builder /usr/src/rust_accelerator/target/release/rust_accelerator /usr/local/bin/rust_accelerator
#     Copies compiled binary from build stage.
#
#   - EXPOSE 5000
#     Declares the port the container listens on.
#
#   - CMD ["rust_accelerator"]
#     Default command to run when container starts - runs the binary.
# ------------------------------------------------------------------------------

# Stage 1: Build stage
FROM rust:1.88 AS builder

WORKDIR /usr/src/rust_accelerator

COPY Cargo.toml Cargo.lock* ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release || true

RUN rm -f src/main.rs

COPY src ./src

RUN cargo build --release

# Stage 2: Runtime stage (smallest possible image)
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/rust_accelerator/target/release/rust_accelerator /usr/local/bin/rust_accelerator

EXPOSE 5000

CMD ["/usr/local/bin/rust_accelerator"]
