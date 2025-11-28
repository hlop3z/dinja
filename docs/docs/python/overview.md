# Python API Overview

Dinja provides a Python HTTP client for the Dinja MDX rendering service.

## Installation

```bash
pip install dinja
```

## Requirements

Start the Dinja service via Docker:

```bash
docker pull ghcr.io/hlop3z/dinja:latest
docker run -p 8080:8080 ghcr.io/hlop3z/dinja:latest
```

## Core Classes

### Renderer

The main entry point for rendering MDX content.

```python
from dinja import Renderer

renderer = Renderer(
    base_url="http://localhost:8080",  # Service URL
    timeout=30.0,                       # Request timeout in seconds
)
```

### Render Methods

```python
# Render to HTML
result = renderer.html(views={...}, components={...}, utils="...", minify=True)

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

## Usage Example

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

## Types

```python
from dinja import (
    Renderer,      # HTTP client class
    Input,         # Input dataclass
    Result,        # Batch result dataclass
    FileResult,    # Individual file result
    Component,     # Component definition
    Output,        # Type alias: "html" | "javascript" | "schema" | "json"
)
```

## API Reference

- [Renderer Class](renderer.md) – Complete Renderer documentation
- [Data Classes](dataclasses.md) – All dataclass definitions
- [Examples](examples.md) – Usage examples
