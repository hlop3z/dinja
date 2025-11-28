# Dinja Python Client

HTTP client for the Dinja MDX rendering service.

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

## Sync Usage

```python
from dinja import Renderer

# Connect to the service
renderer = Renderer("http://localhost:8080")

# Check health
if renderer.health():
    print("Service is running!")

# Render MDX to HTML
result = renderer.html(
    views={"page.mdx": "# Hello World\n\nThis is **bold** text."},
    utils="export default { greeting: 'Hello' }",
)

# Get the output
print(result.get_output("page.mdx"))
# Output: <h1>Hello World</h1><p>This is <strong>bold</strong> text.</p>
```

## Async Usage

```python
import asyncio
from dinja import AsyncRenderer

async def main():
    # Use as async context manager
    async with AsyncRenderer("http://localhost:8080") as renderer:
        # Check health
        if await renderer.health():
            print("Service is running!")

        # Render MDX to HTML
        result = await renderer.html(
            views={"page.mdx": "# Hello World"},
        )
        print(result.get_output("page.mdx"))

asyncio.run(main())
```

### Specifying Backend

```python
# Auto-detect (uses httpx if available, then aiohttp)
renderer = AsyncRenderer("http://localhost:8080")

# Or specify explicitly
renderer = AsyncRenderer("http://localhost:8080", backend="httpx")
renderer = AsyncRenderer("http://localhost:8080", backend="aiohttp")
```

## Render Methods

Both `Renderer` and `AsyncRenderer` support the same methods:

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
result = renderer.render("html", views={...})
```

For async, just add `await`:

```python
result = await renderer.html(views={...})
```

## Components

```python
result = renderer.html(
    views={"app.mdx": "# App\n\n<Button>Click me</Button>"},
    components={
        "Button": "export default function Component(props) { return <button>{props.children}</button>; }"
    },
)
```

## Options

All render methods accept these parameters:

- `views`: Dict mapping view names to MDX content (required)
- `components`: Dict mapping component names to code (optional)
- `utils`: JavaScript utilities code (optional)
- `minify`: Enable minification (default: True)
- `directives`: List of directive prefixes for schema extraction (optional)

## Result Object

```python
result = renderer.html(views={...})

# Check success
result.is_all_success()  # True if all files succeeded

# Get output for a file
result.get_output("page.mdx")

# Get metadata for a file
result.get_metadata("page.mdx")

# Access individual files
result.files["page.mdx"].success
result.files["page.mdx"].output
result.files["page.mdx"].metadata
result.files["page.mdx"].error  # If failed
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

## License

BSD-3-Clause
