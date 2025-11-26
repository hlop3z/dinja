# Output Formats

Dinja supports three output formats, each serving different use cases.

## HTML

Rendered HTML ready for display. This is the most common format.

```python
from dinja import Settings

settings = Settings(output="html")
```

**Output**: `<h1>Hello World</h1><p>Content here</p>`

## JavaScript

Transformed JavaScript code that can be executed to render the component.

```python
settings = Settings(output="javascript")
```

**Output**: JavaScript code representing the component structure.

## Schema

Component schema representation showing the structure before rendering.

```python
settings = Settings(output="schema")
```

**Output**: JSON-like schema representation of the component.

## Examples

=== "HTML"

    ```python
    from dinja import Renderer, Input, Settings

    renderer = Renderer()
    result = renderer.render(
        Input(
            mdx={"page.mdx": "# Hello"},
            settings=Settings(output="html"),
        )
    )
    
    # Output: <h1>Hello</h1>
    ```

=== "JavaScript"

    ```python
    result = renderer.render(
        Input(
            mdx={"page.mdx": "# Hello"},
            settings=Settings(output="javascript"),
        )
    )
    
    # Output: JavaScript code
    ```

=== "Schema"

    ```python
    result = renderer.render(
        Input(
            mdx={"page.mdx": "# Hello"},
            settings=Settings(output="schema"),
        )
    )
    
    # Output: Schema representation
    ```

## Choosing the Right Format

- **HTML**: Use when you need the final rendered output for display
- **JavaScript**: Use when you need to manipulate or transform the component structure
- **Schema**: Use for debugging or understanding the component structure

