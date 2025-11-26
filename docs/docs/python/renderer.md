# Renderer Class

The `Renderer` class is the main entry point for rendering MDX content in Python. It wraps the native Rust renderer and provides automatic retry logic for transient v8 isolate errors.

## Constructor

```python
Renderer(
    max_retries: int = 3,
    retry_delay: float = 0.05,
    backoff_factor: float = 1.5
)
```

### Parameters

- **max_retries** (`int`, default: `3`) – Maximum number of retry attempts on v8 isolate errors
- **retry_delay** (`float`, default: `0.05`) – Initial delay between retries in seconds
- **backoff_factor** (`float`, default: `1.5`) – Multiplier for exponential backoff

## Methods

### render()

Renders MDX content with automatic retry on v8 isolate errors.

```python
def render(
    self, 
    input: Input | dict[str, Any]
) -> dict[str, Any]
```

#### Parameters

- **input** – Either a `Input` dataclass instance or a dictionary containing:
  - `settings`: Dictionary with `output`, `minify`, `engine`, `components`
  - `mdx`: Dictionary mapping file names to MDX content strings
  - `components`: Optional dictionary mapping component names to code strings

#### Returns

Dictionary containing:
- `total`: Total number of files processed
- `succeeded`: Number of files that rendered successfully
- `failed`: Number of files that failed to render
- `errors`: List of error dictionaries with `file` and `message` keys
- `files`: Dictionary mapping file names to render outcomes

#### Raises

- `ValueError` – If the request is invalid after all retries
- `RuntimeError` – If an internal error occurs after all retries

## Examples

### Basic Usage

```python
from dinja import Renderer, Input, Settings

renderer = Renderer()
result = renderer.render(
    Input(
        mdx={"page.mdx": "# Hello World"},
        settings=Settings(output="html"),
    )
)
```

### Custom Retry Configuration

```python
# More aggressive retry settings
renderer = Renderer(
    max_retries=5,
    retry_delay=0.1,
    backoff_factor=2.0
)
```

### Multiple Renders

```python
renderer = Renderer()

# Reuse the same instance for multiple renders
result1 = renderer.render({
    "settings": {"output": "html"},
    "mdx": {"page1.mdx": "# Page 1"},
})

result2 = renderer.render({
    "settings": {"output": "schema"},
    "mdx": {"page2.mdx": "# Page 2"},
})
```

## Retry Behavior

The `Renderer` class automatically retries on v8 isolate errors, which can occur during rapid successive renders or mode switching. Non-retryable errors (like `ValueError`) are raised immediately.

Retry delays use exponential backoff:
- Attempt 1: `retry_delay` seconds
- Attempt 2: `retry_delay * backoff_factor` seconds
- Attempt 3: `retry_delay * backoff_factor^2` seconds
- And so on...

## Best Practices

1. **Reuse Renderer Instances** – Create one `Renderer` instance and reuse it for multiple renders
2. **Use Type-Safe Inputs** – Prefer `Input` over dictionaries for better type checking
3. **Handle Errors** – Always check the `status` field in the result
4. **Configure Retries** – Adjust retry settings based on your use case

