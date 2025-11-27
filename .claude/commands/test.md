# Run Dinja Tests

Run tests for the specified component(s) of the dinja project.

## Arguments
- `$ARGUMENTS` - Component to test: `core`, `python`, `js`, or `all` (default: all)

## Instructions

Based on the argument provided, run the appropriate tests:

### For `core` or `all`:
```bash
cargo test --all-features
```

### For `python` or `all`:
```bash
cd python-bindings && uv run pytest -v
```

### For `js` or `all`:
```bash
cd js-bindings && npm test
```

## Test Order
When testing `all`, run in this order:
1. Core Rust tests
2. Python binding tests
3. JavaScript binding tests

Report test results clearly. If tests fail, analyze the failure and suggest fixes.
