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
go version
```

Also check:
- Python virtual environment exists in `clients/py/.venv`
- Node modules installed in `clients/js/node_modules`
- Go module initialized in `clients/go`

### For `install`:
Install/update development dependencies:

1. **Rust toolchain:**
   ```bash
   rustup update
   ```

2. **Python client:**
   ```bash
   cd clients/py && uv sync --dev
   ```

3. **JavaScript client:**
   ```bash
   cd clients/js && npm install
   ```

4. **Go client:**
   ```bash
   cd clients/go && go mod tidy
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
| `./utils/build.sh clean` | Clean build artifacts |
| `./utils/build.sh all` | Build and test |

## Required Tools
- Rust 1.75+ (with cargo)
- Python 3.13+ (for Python client)
- Node.js 18+ (for JavaScript client)
- Go 1.21+ (for Go client)
- uv (https://github.com/astral-sh/uv) - handles Python env automatically
