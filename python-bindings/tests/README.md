# Dinja Test Suite

This directory contains comprehensive tests for the dinja library, including v8 isolate management tests.

## Test Files

- `test_render.py` - Basic integration tests for the render function
- `test_v8_isolate.py` - Comprehensive v8 isolate behavior tests

## Running Tests

### Python Tests

Run all Python tests:
```bash
cd python-bindings
pytest tests/
```

Run specific test file:
```bash
pytest tests/test_v8_isolate.py
```

Run the v8 isolate test script directly:
```bash
python tests/test_v8_isolate.py
```

### Rust Tests

Run Rust unit tests (requires Rust toolchain):
```bash
cd python-bindings
cargo test
```

Run specific test:
```bash
cargo test test_renderer_rapid_iterations
```

Run tests with output:
```bash
cargo test -- --nocapture
```

## Test Coverage

### Python Tests (`test_v8_isolate.py`)

1. **Renderer class with multiple modes** - Tests `dinja.Renderer` with different output modes
2. **Rapid iterations (Renderer)** - Stress test with `Renderer` class (50 iterations)
3. **Stress test** - Maximum stress test with `Renderer` class (50 iterations)

### Rust Tests (`src/lib.rs`)

1. **Stateless rapid iterations** - Tests creating new service instances (1 iteration, minimal)
2. **Renderer rapid iterations** - Tests reusing service instance (50 iterations)
3. **Rapid mode switching** - Tests switching between modes rapidly (100 iterations)
4. **Stress test** - Maximum stress test (200 iterations)
5. **Concurrent-style renders** - Simulates concurrent requests (20 batch items)
6. **Performance comparison** - Compares stateless vs reusable approaches
7. **Component rendering** - Tests rendering with custom components
8. **All output formats** - Tests all output formats (html, javascript, schema)

## Expected Results

### With `Renderer` Class

- Handles rapid successive renders without v8 isolate issues
- Reuses the same service instance for all renders
- Recommended for all use cases, especially with different modes
- Significantly faster for multiple renders

## Performance Benchmarks

The Rust tests include performance benchmarks comparing:
- **Stateless approach**: Creates new service per render (minimal iterations for comparison)
- **Reusable approach**: Uses single service instance

Expected speedup: 2-5x faster with reusable approach for multiple renders.

## Troubleshooting

If tests fail with v8 isolate errors:

1. **Ensure you're using the Renderer class** - the stateless function has been removed
2. **Reduce iteration counts** if running on resource-constrained systems
3. **Check system resources** - v8 isolates require memory
4. **Run tests individually** to isolate specific failures

## CI/CD Integration

These tests can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions
- name: Run Python tests
  run: |
    cd python-bindings
    pytest tests/ -v

- name: Run Rust tests
  run: |
    cd python-bindings
    cargo test --release
```

