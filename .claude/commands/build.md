# Build Dinja Components

Build the specified component(s) of the dinja project using the centralized build script.

## Arguments
- `$ARGUMENTS` - Component to build: `core`, `python`, `js`, `go`, `release`, or `all` (default: all)

## Instructions

Use `./utils/build.sh` for all build operations. This script handles Python environment setup automatically using uv.

### For `core`:
Build only the Rust core crate (no Python required):
```bash
./utils/build.sh build-core
```

### For `release`:
Build the Rust workspace in release mode:
```bash
./utils/build.sh build-release
```

### For `python`:
Sync Python client dependencies:
```bash
cd clients/py && uv sync --dev
```

### For `all` (default):
Build the entire Rust workspace (falls back to core if Python unavailable):
```bash
./utils/build.sh build
```

### For `js`:
JavaScript client:
```bash
cd clients/js && npm install && npm run build
```

### For `go`:
Go client:
```bash
cd clients/go && go build ./...
```

## Build Commands Reference
| Command | Description |
|---------|-------------|
| `./utils/build.sh build` | Build Rust workspace (debug) |
| `./utils/build.sh build-core` | Build core crate only (no Python) |
| `./utils/build.sh build-release` | Build workspace (release mode) |
| `./utils/build.sh build-core-release` | Build core (release, no Python) |
| `./utils/build.sh test` | Run all tests |
| `./utils/build.sh test-core` | Run core tests only |
| `./utils/build.sh clean` | Clean build artifacts |

## Build Order
When building `all`, the script builds in this order:
1. Core (Rust library and HTTP service)
2. Clients are built separately (py, js, go)

Report any build errors clearly and suggest fixes based on the error messages.
