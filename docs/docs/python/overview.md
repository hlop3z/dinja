# Python API Overview

Dinja provides a Python API built on top of the Rust core, offering both type-safe dataclasses and flexible dictionary-based interfaces.

## Core Classes

### Renderer

The main entry point for rendering MDX content. Includes automatic retry logic for v8 isolate errors.

```python
from dinja import Renderer

renderer = Renderer(
    max_retries=3,      # Number of retry attempts (default: 3)
    retry_delay=0.05,   # Initial delay between retries (default: 0.05s)
    backoff_factor=1.5  # Exponential backoff multiplier (default: 1.5)
)
```

### Input

Type-safe input structure for batch rendering requests.

```python
from dinja import Input, Settings

input_data = Input(
    mdx={"file.mdx": "# Content"},
    settings=Settings(output="html"),
    components=None  # Optional
)
```

### Settings

Configuration for rendering behavior.

```python
from dinja import Settings

settings = Settings(
    output="html",      # "html" | "javascript" | "schema" | "json"
    minify=True,        # Enable minification
)
```

## Usage Patterns

### Type-Safe (Recommended)

```python
from dinja import Renderer, Input, Settings

renderer = Renderer()
result = renderer.render(
    Input(
        mdx={"page.mdx": "# Hello"},
        settings=Settings(output="html"),
    )
)
```

### Dictionary-Based (Backward Compatible)

```python
from dinja import Renderer

renderer = Renderer()
result = renderer.render({
    "settings": {"output": "html"},
    "mdx": {"page.mdx": "# Hello"},
})
```

## API Reference

- [Renderer Class](renderer.md) – Complete Renderer documentation
- [Data Classes](dataclasses.md) – All dataclass definitions
- [Examples](examples.md) – Usage examples

