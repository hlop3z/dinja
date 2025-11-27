# Data Classes

Dinja provides type-safe dataclasses that match the Rust type definitions.

## Input

Input structure for batch MDX rendering requests.

```python
@dataclass
class Input:
    mdx: dict[str, str]  # Required: Map of file names to MDX content
    settings: Settings = Settings()  # Rendering settings
    components: dict[str, ComponentDefinition] | dict[str, str] | None = None
```

### Auto-Conversion

If you provide `components` as a `dict[str, str]` (name → code), it will be automatically converted to `dict[str, ComponentDefinition]`:

```python
# Simple dict format (automatically converted)
input_data = Input(
    mdx={"page.mdx": "# Hello"},
    components={
        "Button": "function Component(props) { return <button>{props.children}</button>; }"
    }
)
```

## Settings

Rendering configuration.

```python
@dataclass
class Settings:
    output: OutputFormat = "html"  # "html" | "javascript" | "schema" | "json"
    minify: bool = True
```

## ComponentDefinition

Component definition with code and metadata.

```python
@dataclass
class ComponentDefinition:
    code: str  # Required: Component code (JSX/TSX)
    name: str | None = None
    docs: str | None = None
    args: Any | None = None  # JSON value
```

### Helper Methods

#### from_name_code()

Create a component definition from name and code:

```python
comp = ComponentDefinition.from_name_code(
    "Button",
    "function Component(props) { return <button>{props.children}</button>; }"
)
```

#### from_dict()

Convert a dictionary of name→code to name→ComponentDefinition:

```python
components_dict = {
    "Button": "function Component(props) { return <button>{props.children}</button>; }",
    "Card": "function Component(props) { return <div>{props.children}</div>; }",
}

components = ComponentDefinition.from_dict(components_dict)
```

## Type Aliases

```python
OutputFormat = Literal["html", "javascript", "schema", "json"]
```

## Examples

### Using Dataclasses

```python
from dinja import (
    Input,
    Settings,
    ComponentDefinition,
    Renderer
)

# Create settings
settings = Settings(
    output="html",
    minify=True
)

# Create components (simple dict format)
components = {
    "Button": "function Component(props) { return <button>{props.children}</button>; }"
}

# Create input
input_data = Input(
    mdx={"page.mdx": "# Hello <Button>Click</Button>"},
    settings=settings,
    components=components  # Auto-converted to ComponentDefinition
)

# Render
renderer = Renderer()
result = renderer.render(input_data)
```

### Using Dictionary Format

```python
# Still works for backward compatibility
result = renderer.render({
    "settings": {
        "output": "html",
        "minify": True
    },
    "mdx": {"page.mdx": "# Hello"},
    "components": {
        "Button": {
            "code": "function Component(props) { return <button>{props.children}</button>; }"
        }
    }
})
```

