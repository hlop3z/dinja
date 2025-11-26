# Testing

How to run and write tests for Dinja.

## Running Tests

### All Tests

```bash
./build.sh test
```

### Python Tests Only

```bash
cd python-bindings
pytest tests/
```

### Rust Tests Only

```bash
cd python-bindings
cargo test --lib
```

## Test Structure

- **Python tests**: `python-bindings/tests/`
- **Rust tests**: `python-bindings/src/lib.rs` (in `#[cfg(test)]` module)

## Writing Tests

### Python Test Example

```python
def test_basic_render():
    from dinja import Renderer, Input, Settings
    
    renderer = Renderer()
    result = renderer.render(
        Input(
            mdx={"test.mdx": "# Hello"},
            settings=Settings(output="html"),
        )
    )
    
    assert result["succeeded"] == 1
```

### Rust Test Example

```rust
#[test]
fn test_basic_render() {
    let service = init_test_service();
    let input = create_test_config(OutputFormat::Html, "# Hello");
    let outcome = service.render_batch(&input).unwrap();
    assert_eq!(outcome.succeeded, 1);
}
```

## Test Coverage

- Unit tests for core functionality
- Integration tests for end-to-end scenarios
- Performance tests for optimization validation

