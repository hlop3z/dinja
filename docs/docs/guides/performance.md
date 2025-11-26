# Performance

Dinja is designed for high performance with several optimization strategies.

## Renderer Reuse

The most important performance tip: **reuse Renderer instances**.

```python
# Good: Create once, reuse many times
renderer = Renderer()
for mdx_file in many_files:
    result = renderer.render(...)

# Bad: Creating new renderer for each render
for mdx_file in many_files:
    renderer = Renderer()  # Expensive!
    result = renderer.render(...)
```

## Batch Rendering

Render multiple files in a single batch call:

```python
# Good: Single batch call
result = renderer.render(
    Input(
        mdx={
            "file1.mdx": "...",
            "file2.mdx": "...",
            "file3.mdx": "...",
        },
        settings=Settings(output="html"),
    )
)

# Less efficient: Multiple calls
for file in files:
    result = renderer.render(
        Input(mdx={file: content}, ...)
    )
```

## Minification

Enable minification for production:

```python
settings = Settings(
    output="html",
    minify=True,  # Reduces output size
)
```

## Retry Configuration

For high-load scenarios, adjust retry settings:

```python
renderer = Renderer(
    max_retries=3,      # Default is usually sufficient
    retry_delay=0.05,   # Minimal delay
    backoff_factor=1.5  # Moderate backoff
)
```

## Rust Performance

The Rust API provides the best performance:

```rust
// Single service instance for all renders
let service = RenderService::new(config)?;

// Batch rendering
let outcome = service.render_batch(&input)?;
```

## Benchmarks

Dinja's Rust core provides:
- **2-5x faster** than creating new service instances for each render
- **Native performance** without Node.js overhead
- **Efficient memory usage** with renderer pooling

## Best Practices

1. ✅ Reuse Renderer instances
2. ✅ Use batch rendering for multiple files
3. ✅ Enable minification in production
4. ✅ Use Rust API for maximum performance
5. ❌ Don't create new Renderer for each render
6. ❌ Don't render files one-by-one when batching is possible

