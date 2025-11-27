# Development Environment Setup

Set up or verify the development environment for dinja using the centralized build script.

## Arguments
- `$ARGUMENTS` - Action: `check` (default), `install`, or `full`

## Instructions

### For `check` (default):
Verify all required tools are installed:
```bash
rustc --version
cargo --version
uv --version
node --version
npm --version
```

Also check:
- Python virtual environment exists in `python-bindings/.venv`
- Node modules installed in `js-bindings/node_modules`

### For `install`:
Install/update development dependencies:

1. **Rust toolchain:**
   ```bash
   rustup update
   ```

2. **Python bindings (using build.sh):**
   ```bash
   ./utils/build.sh dev
   ```
   This handles virtualenv creation, maturin installation via uv, and development mode installation.

3. **JavaScript bindings:**
   ```bash
   cd js-bindings && npm install
   ```

### For `full`:
Complete setup including building all components:
```bash
./utils/build.sh all
```
This runs build, test, and dev mode setup in sequence.

## build.sh Commands Reference
| Command | Description |
|---------|-------------|
| `./utils/build.sh help` | Show all available commands |
| `./utils/build.sh build` | Build Rust workspace |
| `./utils/build.sh build-core` | Build core only (no Python) |
| `./utils/build.sh test` | Run all tests |
| `./utils/build.sh test-core` | Run core tests only |
| `./utils/build.sh dev` | Install Python bindings in dev mode |
| `./utils/build.sh build-python` | Build Python wheels |
| `./utils/build.sh clean` | Clean build artifacts |
| `./utils/build.sh all` | Build, test, and install dev mode |

## Required Tools
- Rust 1.75+ (with cargo)
- Python 3.13+ (with uv package manager recommended)
- Node.js 18+ (with npm)
- uv (https://github.com/astral-sh/uv) - handles maturin and Python env automatically
