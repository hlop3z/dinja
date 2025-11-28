# Run Dinja Examples

Run example code to demonstrate dinja functionality.

## Arguments
- `$ARGUMENTS` - Example type: `python` (default), `rust`, `js`, `go`, or `all`

## Instructions

### For `python` (default):
Run the Python example:
```bash
cd clients/py && uv run python examples/basic_example.py
```

Or for the dict-based API example:
```bash
cd clients/py && uv run python examples/example.py
```

### For `rust`:
Run the Rust example:
```bash
cargo run --example test_render -p dinja-core
```

### For `js`:
Run the JavaScript example:
```bash
cd clients/js && node examples/example.js
```

### For `go`:
Run the Go example:
```bash
cd clients/go && go run examples/main.go
```

### For `all`:
Run all examples in sequence and show the output from each.

## Example Output
Each example demonstrates:
- MDX content with frontmatter
- Custom component rendering
- Different output formats (HTML, JavaScript, Schema)
- Utility function injection

Explain the output and how it relates to the dinja API.
