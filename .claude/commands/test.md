# Run Dinja Tests

Run tests for the specified component(s) of the dinja project using the centralized build script.

## Arguments
- `$ARGUMENTS` - Component to test: `core`, `python`, `js`, or `all` (default: all)

## Instructions

Use `./utils/build.sh` for all test operations. This script handles Python environment setup automatically using uv.

### For `core`:
Run only the Rust core tests (no Python required):
```bash
./utils/build.sh test-core
```

### For `python`:
Run Python tests (requires uv and Python):
```bash
./utils/build.sh test
```
Note: This runs both Rust and Python tests. The script handles virtualenv setup, maturin develop, and pytest execution.

### For `all` (default):
Run all tests (falls back to core tests if Python unavailable):
```bash
./utils/build.sh test
```

### For `js`:
JavaScript tests (not yet integrated with build.sh):
```bash
cd js-bindings && npm test
```

## Test Commands Reference
| Command | Description |
|---------|-------------|
| `./utils/build.sh test` | Run Rust + Python tests |
| `./utils/build.sh test-core` | Run core crate tests only (no Python) |

## Test Order
When testing `all`, the script runs in this order:
1. Core Rust tests (`cargo test -p dinja-core`)
2. Python binding tests (`pytest` in managed virtualenv)

Report test results clearly. If tests fail, analyze the failure and suggest fixes.
