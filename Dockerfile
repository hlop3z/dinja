# Multi-stage Dockerfile for Dinja HTTP Server
# Creates a minimal Docker image using scratch base (~10MB final image)

# Stage 1: Builder - Build the Rust binary
FROM rust:latest AS builder

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

# Build the HTTP server with release optimizations
RUN cargo build --release \
    --features http \
    --package dinja-core \
    --bin dinja-core

# Stage 2: Runtime - Minimal debian-slim image
FROM debian:trixie-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /app/target/release/dinja-core /usr/local/bin/dinja-core

# Copy static files (JavaScript libraries)
COPY core/static /static

# Set environment variables
ENV STATIC_DIR=/static
ENV PORT=8080

# Expose the HTTP port
EXPOSE 8080

# Run the server
ENTRYPOINT ["dinja-core"]
