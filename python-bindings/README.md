# Dinja Python Client

HTTP client for the Dinja MDX rendering service.

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

## Usage

```python
from dinja import Renderer, Input, Result

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

## Render Methods

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

## Components

```python
result = renderer.html(
    views={"app.mdx": "# App\n\n<Button>Click me</Button>"},
    components={
        "Button": "function Component(props) { return <button>{props.children}</button>; }"
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
    Renderer,      # HTTP client class
    Input,         # Input dataclass
    Result,        # Batch result dataclass
    FileResult,    # Individual file result
    Component,     # Component definition
    Output,        # Type alias: "html" | "javascript" | "schema" | "json"
)
```

## License

BSD-3-Clause
