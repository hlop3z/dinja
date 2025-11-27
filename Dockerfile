# Multi-stage Dockerfile for Dinja HTTP Server
# Minimal image using scratch with only required libraries

# Stage 1: Builder - Build the Rust binary
FROM rust:slim-bookworm AS builder

# Version can be passed as build argument (defaults to 0.0.0 for local builds)
ARG VERSION=0.0.0

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY core ./core

RUN printf '%s\n' \
    '[workspace]' \
    'members = ["core"]' \
    'resolver = "2"' \
    '' \
    '[workspace.package]' \
    "version = \"${VERSION}\"" \
    'edition = "2021"' \
    'authors = ["dinja contributors"]' \
    'license = "BSD-3-Clause"' \
    'repository = "https://github.com/hlop3z/dinja"' \
    'description = "Safe MDX renderer with a Rust core"' \
    'readme = "README.md"' \
    'documentation = "https://github.com/hlop3z/dinja#readme"' \
    > Cargo.toml

RUN CARGO_PROFILE_RELEASE_LTO=true \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
    CARGO_PROFILE_RELEASE_STRIP=symbols \
    CARGO_PROFILE_RELEASE_OPT_LEVEL=z \
    cargo build --release \
    --features http \
    --package dinja-core \
    --bin dinja-core

RUN strip --strip-all /app/target/release/dinja-core || true

# Stage 2: Collect minimal runtime dependencies
FROM debian:bookworm-slim AS libs

RUN apt-get update && apt-get install -y --no-install-recommends \
    libgcc-s1 \
    libstdc++6 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Stage 3: Final minimal image from scratch
FROM scratch

# Copy only the required shared libraries (minimal set)
COPY --from=libs /lib/x86_64-linux-gnu/libc.so.6 /lib/x86_64-linux-gnu/
COPY --from=libs /lib/x86_64-linux-gnu/libm.so.6 /lib/x86_64-linux-gnu/
COPY --from=libs /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/x86_64-linux-gnu/
COPY --from=libs /lib/x86_64-linux-gnu/libstdc++.so.6 /lib/x86_64-linux-gnu/
COPY --from=libs /lib64/ld-linux-x86-64.so.2 /lib64/

# Copy CA certificates for HTTPS
COPY --from=libs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the binary
COPY --from=builder /app/target/release/dinja-core /dinja-core

# Copy static files
COPY core/static /static

# Set environment variables
ENV RUST_CMS_STATIC_DIR=/static
ENV PORT=8080

EXPOSE 8080

ENTRYPOINT ["/dinja-core"]
