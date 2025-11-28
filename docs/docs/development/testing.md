# Testing

How to run and write tests for Dinja.

## Running Tests

### All Tests

```bash
./utils/build.sh test
```

### Python Tests Only

```bash
cd clients/py
# Start the service first
docker run -d -p 8080:8080 ghcr.io/hlop3z/dinja:latest
pytest tests/
```

### Rust Tests Only

```bash
cargo test -p dinja-core
```

### JavaScript Tests Only

```bash
cd clients/js
npm test
```

### Go Tests Only

```bash
cd clients/go
go test ./...
```

## Test Structure

- **Rust tests**: `core/src/` (in `#[cfg(test)]` modules)
- **Python tests**: `clients/py/tests/`
- **JavaScript tests**: `clients/js/test/`
- **Go tests**: `clients/go/*_test.go`

## Writing Tests

### Python Test Example

```python
def test_basic_render():
    from dinja import Renderer
    
    renderer = Renderer()
    result = renderer.html(views={"test.mdx": "# Hello"})
    
    assert result.succeeded == 1
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

### Go Test Example

```go
func TestBasicRender(t *testing.T) {
    renderer := dinja.New()
    result, err := renderer.HTML(context.Background(), dinja.Input{
        Views: map[string]string{"test.mdx": "# Hello"},
    })
    if err != nil {
        t.Fatalf("unexpected error: %v", err)
    }
    if !result.IsAllSuccess() {
        t.Error("expected all success")
    }
}
```

## Test Coverage

- Unit tests for core functionality
- Integration tests for end-to-end scenarios
- Performance tests for optimization validation
