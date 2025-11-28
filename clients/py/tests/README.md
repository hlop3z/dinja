# Dinja Test Suite

This directory contains comprehensive tests for the dinja library.

## Test Files

- `test_render.py` - Basic integration tests for the render function
- `test_v8_isolate.py` - Comprehensive v8 isolate behavior tests

## Running Tests

### Python Tests

Run all Python tests:
```bash
cd clients/py
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

## Test Coverage

### Python Tests (`test_v8_isolate.py`)

1. **Renderer class with multiple modes** - Tests `dinja.Renderer` with different output modes
2. **Rapid iterations (Renderer)** - Stress test with `Renderer` class (50 iterations)
3. **Stress test** - Maximum stress test with `Renderer` class (50 iterations)

## Expected Results

### With `Renderer` Class

- Handles rapid successive renders without v8 isolate issues
- Reuses the same service instance for all renders
- Recommended for all use cases, especially with different modes
- Significantly faster for multiple renders

## Performance Benchmarks

Expected speedup: 2-5x faster with reusable approach for multiple renders.

## Troubleshooting

If tests fail with connection errors:

1. **Ensure the Dinja service is running** - the tests require the HTTP service
2. **Start the service**: `docker run -p 8080:8080 ghcr.io/hlop3z/dinja:latest`
3. **Check system resources** - v8 isolates require memory
4. **Run tests individually** to isolate specific failures

## CI/CD Integration

These tests can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions
- name: Start Dinja service
  run: |
    docker run -d -p 8080:8080 ghcr.io/hlop3z/dinja:latest
    sleep 5  # Wait for service to start

- name: Run Python tests
  run: |
    cd clients/py
    uv run pytest tests/ -v
```
