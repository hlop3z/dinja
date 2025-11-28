# Python Examples

Complete examples demonstrating Dinja's Python HTTP client API.

## Prerequisites

Start the Dinja service:

```bash
docker pull ghcr.io/hlop3z/dinja:latest
docker run -p 8080:8080 ghcr.io/hlop3z/dinja:latest
```

## Basic Rendering

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")

result = renderer.html(views={"hello.mdx": "# Hello World\n\nThis is **markdown**!"})

if result.is_all_success():
    print(result.get_output("hello.mdx"))
```

## With Frontmatter

```python
from dinja import Renderer

mdx_content = """---
title: My Page
author: John Doe
---

# Welcome

This page has frontmatter metadata.
"""

renderer = Renderer("http://localhost:8080")
result = renderer.html(views={"page.mdx": mdx_content})

if result.is_all_success():
    output = result.get_output("page.mdx")
    metadata = result.get_metadata("page.mdx")

    print("Title:", metadata.get("title"))
    print("Author:", metadata.get("author"))
    print("Output:", output)
```

## Custom Components

```python
from dinja import Renderer

components = {
    "Button": "export default function Component(props) { return <button>{props.children}</button>; }",
    "Card": "export default function Component(props) { return <div class='card'>{props.children}</div>; }",
}

mdx_content = """
# My Page

<Card>
  <Button>Click me</Button>
</Card>
"""

renderer = Renderer("http://localhost:8080")
result = renderer.html(
    views={"page.mdx": mdx_content},
    components=components,
)

if result.is_all_success():
    print(result.get_output("page.mdx"))
```

## Global Utilities

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")

result = renderer.html(
    views={"page.mdx": "<Greeting />"},
    components={
        "Greeting": "export default function Component() { return <div>{utils.message}</div>; }"
    },
    utils="export default { message: 'Hello World' }",
)
```

## Multiple Output Formats

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")
views = {"page.mdx": "# Hello World"}

# HTML output
html_result = renderer.html(views=views)

# JavaScript output
js_result = renderer.javascript(views=views)

# Schema output (extract component names)
schema_result = renderer.schema(views=views)

# JSON tree output
json_result = renderer.json(views=views)
```

## Batch Rendering

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")

result = renderer.html(
    views={
        "index.mdx": "# Home Page",
        "about.mdx": "# About Us",
        "contact.mdx": "# Contact",
    }
)

print(f"Total: {result.total}")
print(f"Succeeded: {result.succeeded}")
print(f"Failed: {result.failed}")

for filename, file_result in result.files.items():
    if file_result.success:
        print(f"✓ {filename}")
    else:
        print(f"✗ {filename}: {file_result.error}")
```

## Error Handling

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")

# Check if service is running
if not renderer.health():
    print("Dinja service is not running!")
    exit(1)

result = renderer.html(views={"page.mdx": "# Content"})

if result.is_all_success():
    print("All files rendered successfully!")
    print(result.get_output("page.mdx"))
else:
    print(f"Some files failed: {result.failed} of {result.total}")
    for error in result.errors:
        print(f"Error in {error['file']}: {error['message']}")
```

## Using Component Dataclass

```python
from dinja import Renderer, Component

renderer = Renderer("http://localhost:8080")

# Full Component definition with metadata
components = {
    "Button": Component(
        code="export default function Component(props) { return <button>{props.children}</button>; }",
        name="Button",
        docs="A button component",
        args={"children": "ReactNode"},
    )
}

result = renderer.html(
    views={"page.mdx": "<Button>Click me</Button>"},
    components=components,
)
```

## Custom Directives

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")

result = renderer.html(
    views={"page.mdx": "# Content with directives"},
    directives=["use client", "use server"],
)
```

## With Context Function

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")

result = renderer.html(
    views={
        "blog.mdx": """---
title: My Post
author: Alice
---

# {context('title')}

By {context('author')}
"""
    }
)

if result.is_all_success():
    print(result.get_output("blog.mdx"))
    print(result.get_metadata("blog.mdx"))
```
