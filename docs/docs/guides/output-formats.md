# Output Formats

Dinja supports four output formats, each serving different use cases.

## HTML

Rendered HTML ready for display. This is the most common format.

```python
from dinja import Settings

settings = Settings(output="html")
```

**Input**: `# Hello\n\n<Button>Click</Button>`
**Output**: `<h1>Hello</h1><button>Click</button>`

## JavaScript

Transformed JavaScript code (Preact syntax with h() and Fragment) that can be executed to render the component.

```python
settings = Settings(output="javascript")
```

**Input**: `# Hello\n\n<Button>Click</Button>`
**Output**: JavaScript code with Preact function calls

## Schema

Extracts unique component names from the MDX content. Returns a JSON array of all custom components (elements starting with capital letters).

```python
settings = Settings(output="schema")
```

**Input**: `# Hello\n\n<Button>Click</Button>\n<Card>Test</Card>\n<Button>Another</Button>`
**Output**: `["Button", "Card"]` (sorted, unique)

**Use cases**:
- Analyze which components are used in MDX content
- Validate component usage
- Generate component dependency lists
- Fast component discovery without rendering

## JSON

Returns a JSON representation of the transformed JSX tree structure.

```python
settings = Settings(output="json")
```

**Input**: `# Hello\n\n<Button>Click</Button>`
**Output**: JSON tree representing the component structure

**Use cases**:
- Debugging component structure
- Serializing component trees
- Building custom renderers

## Examples

### Complete Example with All Formats

```python
from dinja import Renderer, Input, Settings

renderer = Renderer()

# Sample MDX with custom components
mdx_content = """
# Welcome

<Alert type="info">Important message</Alert>

<Card title="Getting Started">
  <Button primary>Click here</Button>
  <Button>Learn more</Button>
</Card>
"""

# HTML Output
html_result = renderer.render(
    Input(
        mdx={"page.mdx": mdx_content},
        settings=Settings(output="html"),
        components={
            "Alert": "function Component(props) { return <div class={'alert-' + props.type}>{props.children}</div>; }",
            "Card": "function Component(props) { return <div class='card'><h3>{props.title}</h3>{props.children}</div>; }",
            "Button": "function Component(props) { return <button class={props.primary ? 'btn-primary' : 'btn'}>{props.children}</button>; }",
        },
    )
)
print(html_result["files"]["page.mdx"]["result"]["output"])
# Output: <h1>Welcome</h1><div class="alert-info">Important message</div>...

# JavaScript Output
js_result = renderer.render(
    Input(
        mdx={"page.mdx": mdx_content},
        settings=Settings(output="javascript"),
    )
)
# Output: JavaScript code with h() and Fragment

# Schema Output - Fast component discovery
schema_result = renderer.render(
    Input(
        mdx={"page.mdx": mdx_content},
        settings=Settings(output="schema"),
    )
)
print(schema_result["files"]["page.mdx"]["result"]["output"])
# Output: ["Alert", "Button", "Card"]

# JSON Output - Full tree structure
json_result = renderer.render(
    Input(
        mdx={"page.mdx": mdx_content},
        settings=Settings(output="json"),
        components={
            "Alert": "function Component(props) { return <div>{props.children}</div>; }",
            "Card": "function Component(props) { return <div>{props.children}</div>; }",
            "Button": "function Component(props) { return <button>{props.children}</button>; }",
        },
    )
)
# Output: JSON tree structure
```

## Choosing the Right Format

- **HTML**: Use when you need the final rendered output for display in browsers
- **JavaScript**: Use when you need to execute or transform components on the client side
- **Schema**: Use for fast component discovery, validation, or dependency analysis (no rendering needed)
- **JSON**: Use for debugging, serialization, or building custom renderers

