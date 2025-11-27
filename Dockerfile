# Multi-stage Dockerfile for Dinja HTTP Server
# Creates a minimal Docker image using debian-slim base

# Stage 1: Builder - Build the Rust binary
FROM rust:slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy only core (the only package needed for the HTTP server)
COPY core ./core

# Create a standalone Cargo.toml for the Docker build
RUN printf '%s\n' \
    '[workspace]' \
    'members = ["core"]' \
    'resolver = "2"' \
    '' \
    '[workspace.package]' \
    'version = "0.3.1"' \
    'edition = "2021"' \
    'authors = ["dinja contributors"]' \
    'license = "BSD-3-Clause"' \
    'repository = "https://github.com/hlop3z/dinja"' \
    'description = "Safe MDX renderer with a Rust core"' \
    'readme = "README.md"' \
    'documentation = "https://github.com/hlop3z/dinja#readme"' \
    > Cargo.toml

# Build the HTTP server with release optimizations
RUN cargo build --release \
    --features http \
    --package dinja-core \
    --bin dinja-core

# Stage 2: Runtime - Minimal debian-slim image
FROM debian:bookworm-slim

# Install runtime dependencies (only libssl and ca-certificates)
RUN apt-get update && apt-get install -y --no-install-recommends \
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
