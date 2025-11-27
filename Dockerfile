# Multi-stage Dockerfile for Dinja HTTP Server
# Creates a minimal Docker image using scratch base (~10MB final image)

# Stage 1: Builder - Build the Rust binary
FROM rust:1.83-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all workspace members
COPY core ./core
COPY python-bindings ./python-bindings

# Build the HTTP server with release optimizations and static linking
# Target x86_64-unknown-linux-musl for fully static binary
RUN rustup target add x86_64-unknown-linux-musl && \
    cargo build --release \
    --target x86_64-unknown-linux-musl \
    --features http \
    --package dinja-core \
    --bin dinja-core

# Stage 2: Runtime - Minimal scratch-based image
FROM scratch

# Copy CA certificates for HTTPS (if needed)
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the static binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/dinja-core /dinja-core

# Copy static files (JavaScript libraries)
COPY core/static /static

# Set environment variables
ENV STATIC_DIR=/static
ENV PORT=8080

# Expose the HTTP port
EXPOSE 8080

# Run the server
ENTRYPOINT ["/dinja-core"]
