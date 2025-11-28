# Python API Overview

Dinja provides a Python HTTP client for the Dinja MDX rendering service with both sync and async support.

## Installation

```bash
pip install dinja
```

### Async Support (Optional)

For async support, install with your preferred HTTP library:

```bash
pip install dinja[httpx]    # recommended
pip install dinja[aiohttp]  # alternative
```

## Requirements

Start the Dinja service via Docker:

```bash
docker pull ghcr.io/hlop3z/dinja:latest
docker run -p 8080:8080 ghcr.io/hlop3z/dinja:latest
```

## Core Classes

### Renderer (Sync)

The synchronous entry point for rendering MDX content.

```python
from dinja import Renderer

renderer = Renderer(
    base_url="http://localhost:8080",  # Service URL
    timeout=30.0,                       # Request timeout in seconds
)
```

### AsyncRenderer (Async)

The asynchronous entry point for rendering MDX content.

```python
from dinja import AsyncRenderer

# Use as async context manager (recommended)
async with AsyncRenderer("http://localhost:8080") as renderer:
    result = await renderer.html(views={"page.mdx": "# Hello"})

# Or manually manage lifecycle
renderer = AsyncRenderer("http://localhost:8080")
result = await renderer.html(views={"page.mdx": "# Hello"})
await renderer.close()
```

#### Backend Selection

```python
# Auto-detect (uses httpx if available, then aiohttp)
renderer = AsyncRenderer("http://localhost:8080")

# Or specify explicitly
renderer = AsyncRenderer("http://localhost:8080", backend="httpx")
renderer = AsyncRenderer("http://localhost:8080", backend="aiohttp")
```

### Render Methods

Both `Renderer` and `AsyncRenderer` support the same methods:

```python
# Sync
result = renderer.html(views={...}, components={...}, utils="...", minify=True)

# Async
result = await renderer.html(views={...}, components={...}, utils="...", minify=True)
```

Available methods:

```python
# Render to HTML
result = renderer.html(views={...})

# Render to JavaScript
result = renderer.javascript(views={...})

# Extract schema (component names)
result = renderer.schema(views={...})

# Render to JSON tree
result = renderer.json(views={...})

# Generic render with output format
result = renderer.render(output="html", views={...})

# Health check
is_healthy = renderer.health()
```

### Result Object

```python
result = renderer.html(views={"page.mdx": "# Hello"})

# Check success
result.is_all_success()  # True if all files succeeded

# Get output for a file
result.get_output("page.mdx")  # Returns HTML string or None

# Get metadata for a file
result.get_metadata("page.mdx")  # Returns dict

# Access individual files
result.files["page.mdx"].success
result.files["page.mdx"].output
result.files["page.mdx"].metadata
result.files["page.mdx"].error  # If failed
```

## Usage Examples

### Sync Example

```python
from dinja import Renderer, Component

# Connect to the service
renderer = Renderer("http://localhost:8080")

# Check health
if renderer.health():
    print("Service is running!")

# Render MDX to HTML
result = renderer.html(
    views={"page.mdx": "# Hello World\n\n<Button>Click me</Button>"},
    components={
        "Button": "export default function Component(props) { return <button>{props.children}</button>; }"
    },
    utils="export default { greeting: 'Hello' }",
)

# Get the output
if result.is_all_success():
    print(result.get_output("page.mdx"))
```

### Async Example

```python
import asyncio
from dinja import AsyncRenderer

async def main():
    async with AsyncRenderer("http://localhost:8080") as renderer:
        if await renderer.health():
            print("Service is running!")

        result = await renderer.html(
            views={"page.mdx": "# Hello World\n\n<Button>Click me</Button>"},
            components={
                "Button": "export default function Component(props) { return <button>{props.children}</button>; }"
            },
        )

        if result.is_all_success():
            print(result.get_output("page.mdx"))

asyncio.run(main())
```

## Types

```python
from dinja import (
    Renderer,       # Sync HTTP client class
    AsyncRenderer,  # Async HTTP client class (requires httpx or aiohttp)
    Input,          # Input dataclass
    Result,         # Batch result dataclass
    FileResult,     # Individual file result
    Component,      # Component definition
    Output,         # Type alias: "html" | "javascript" | "schema" | "json"
)
```

## API Reference

- [Renderer Class](renderer.md) – Complete Renderer documentation
- [Data Classes](dataclasses.md) – All dataclass definitions
- [Examples](examples.md) – Usage examples
