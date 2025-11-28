# Run Dinja Tests

Run tests for the specified component(s) of the dinja project using the centralized build script.

## Arguments
- `$ARGUMENTS` - Component to test: `core`, `python`, `js`, `go`, or `all` (default: all)

## Instructions

Use `./utils/build.sh` for all test operations. This script handles Python environment setup automatically using uv.

### For `core`:
Run only the Rust core tests (no Python required):
```bash
./utils/build.sh test-core
```

### For `python`:
Run Python tests (requires service running):
```bash
./utils/build.sh test-python
```
Note: This runs the Python HTTP client tests. Requires the Dinja service to be running.

### For `all` (default):
Run all tests (falls back to core tests if Python unavailable):
```bash
./utils/build.sh test-all
```

### For `js`:
JavaScript tests:
```bash
cd clients/js && npm test
```

### For `go`:
Go tests:
```bash
cd clients/go && go test ./...
```

## Test Commands Reference
| Command | Description |
|---------|-------------|
| `./utils/build.sh test` | Run Rust core tests |
| `./utils/build.sh test-core` | Run core crate tests only |
| `./utils/build.sh test-python` | Run Python client tests (requires service) |
| `./utils/build.sh test-js` | Run JavaScript client tests |
| `./utils/build.sh test-all` | Run all tests |

## Test Order
When testing `all`, the script runs in this order:
1. Core Rust tests (`cargo test -p dinja-core`)
2. Python client tests (`pytest`)
3. JavaScript client tests (`npm test`)

Report test results clearly. If tests fail, analyze the failure and suggest fixes.
