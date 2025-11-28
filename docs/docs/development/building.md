# Building

Instructions for building Dinja from source.

## Prerequisites

- Rust toolchain (latest stable)
- Node.js 18+ (for TypeScript client)
- Python 3.13+ (for Python client)
- Go 1.21+ (for Go client)

## Building Rust Core

```bash
# Debug build
cargo build -p dinja-core

# Release build
cargo build -p dinja-core --release
```

## Running the Service

```bash
# Run directly
./target/release/dinja-core

# Or via cargo
cargo run -p dinja-core --release
```

The service runs on `http://localhost:8080` by default.

## Building TypeScript Client

```bash
cd clients/js
npm install
npm run build
```

## Building Python Client

The Python client is a pure Python HTTP client (no native bindings):

```bash
cd clients/py
pip install -e .
```

Or with uv:

```bash
cd clients/py
uv sync
```

## Building Go Client

```bash
cd clients/go
go build ./...
```

## Building Docker Image

```bash
docker build -t dinja .
```

## Building Documentation

```bash
cd docs
pip install mkdocs-material
mkdocs serve  # For local development
mkdocs build  # For production
```

## Full Build Script

Use the build script for common operations:

```bash
./utils/build.sh build           # Build Rust workspace
./utils/build.sh build-core      # Build core crate only
./utils/build.sh build-release   # Build in release mode
./utils/build.sh test            # Run tests
./utils/build.sh test-core       # Run core tests only
./utils/build.sh clean           # Clean build artifacts
```

## Running Tests

### Rust Tests

```bash
cargo test -p dinja-core
```

### Python Tests

```bash
cd clients/py
# Start service first
docker run -d -p 8080:8080 ghcr.io/hlop3z/dinja:latest

# Run tests
uv run pytest
```

### TypeScript Tests

```bash
cd clients/js
npm test
```

### Go Tests

```bash
cd clients/go
go test ./...
```

## Release Process

Versions are managed in the `VERSION` file and synced across all packages using `release.py`:

```bash
uv run release.py bump --version 0.3.0  # Bump all versions
uv run release.py release 0.3.0         # Create release tag
```
