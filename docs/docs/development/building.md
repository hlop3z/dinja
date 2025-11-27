# Building

Instructions for building Dinja from source.

## Prerequisites

- Rust toolchain (latest stable)
- Python 3.13+
- `maturin` for Python bindings

## Building Rust Core

```bash
cd core
cargo build --release
```

## Building Python Bindings

```bash
cd python-bindings
maturin develop
```

Or for production:

```bash
maturin build --release
```

## Building Documentation

```bash
cd docs
mkdocs serve  # For local development
mkdocs build  # For production
```

## Full Build

Use the build script:

```bash
./utils/build.sh
```

This will:
1. Build the Rust core
2. Build Python bindings
3. Run tests
4. Build documentation

