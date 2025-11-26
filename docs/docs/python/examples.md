# Python Examples

Complete examples demonstrating Dinja's Python API.

## Basic Rendering

```python
from dinja import Renderer, Input, Settings

renderer = Renderer()

result = renderer.render(
    Input(
        mdx={"hello.mdx": "# Hello World\n\nThis is **markdown**!"},
        settings=Settings(output="html"),
    )
)

entry = result["files"]["hello.mdx"]
if entry["status"] == "success":
    print(entry["result"]["output"])
```

## With Frontmatter

```python
from dinja import Renderer, Input, Settings

mdx_content = """---
title: My Page
author: John Doe
---

# Welcome

This page has frontmatter metadata.
"""

renderer = Renderer()
result = renderer.render(
    Input(
        mdx={"page.mdx": mdx_content},
        settings=Settings(output="html"),
    )
)

entry = result["files"]["page.mdx"]
if entry["status"] == "success":
    rendered = entry["result"]
    print("Metadata:", rendered["metadata"])
    print("Output:", rendered["output"])
```

## Custom Components

```python
from dinja import Renderer, Input, Settings

components = {
    "Button": "function Component(props) { return <button>{props.children}</button>; }",
    "Card": "function Component(props) { return <div class='card'>{props.children}</div>; }",
}

mdx_content = """
# My Page

<Card>
  <Button>Click me</Button>
</Card>
"""

renderer = Renderer()
result = renderer.render(
    Input(
        mdx={"page.mdx": mdx_content},
        settings=Settings(engine="custom"),
        components=components,
    )
)
```

## Multiple Output Formats

```python
from dinja import Renderer, Input, Settings

renderer = Renderer()
mdx = {"page.mdx": "# Hello World"}

# HTML output
html_result = renderer.render(
    Input(
        mdx=mdx,
        settings=Settings(output="html"),
    )
)

# JavaScript output
js_result = renderer.render(
    Input(
        mdx=mdx,
        settings=Settings(output="javascript"),
    )
)

# Schema output
schema_result = renderer.render(
    Input(
        mdx=mdx,
        settings=Settings(output="schema"),
    )
)
```

## Batch Rendering

```python
from dinja import Renderer, Input, Settings

renderer = Renderer()

result = renderer.render(
    Input(
        mdx={
            "index.mdx": "# Home Page",
            "about.mdx": "# About Us",
            "contact.mdx": "# Contact",
        },
        settings=Settings(output="html"),
    )
)

print(f"Total: {result['total']}")
print(f"Succeeded: {result['succeeded']}")
print(f"Failed: {result['failed']}")

for filename, entry in result["files"].items():
    if entry["status"] == "success":
        print(f"✓ {filename}")
    else:
        print(f"✗ {filename}: {entry.get('error')}")
```

## Error Handling

```python
from dinja import Renderer, Input, Settings

renderer = Renderer()

try:
    result = renderer.render(
        Input(
            mdx={"page.mdx": "# Content"},
            settings=Settings(output="html"),
        )
    )
    
    for filename, entry in result["files"].items():
        if entry["status"] == "success":
            print(f"✓ {filename} rendered successfully")
        else:
            print(f"✗ {filename} failed: {entry.get('error')}")
            
    if result["errors"]:
        for error in result["errors"]:
            print(f"Error in {error['file']}: {error['message']}")
            
except ValueError as e:
    print(f"Invalid request: {e}")
except RuntimeError as e:
    print(f"Runtime error: {e}")
```

## Custom Retry Configuration

```python
from dinja import Renderer

# Configure for high-load scenarios
renderer = Renderer(
    max_retries=5,
    retry_delay=0.1,
    backoff_factor=2.0
)

# Use the renderer as normal
result = renderer.render({
    "settings": {"output": "html"},
    "mdx": {"page.mdx": "# Content"},
})
```

