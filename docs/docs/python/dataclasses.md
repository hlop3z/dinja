# Data Classes

Dinja provides type-safe dataclasses for the Python HTTP client.

## Component

Component definition with code and metadata.

```python
from dinja import Component

@dataclass
class Component:
    code: str              # Required: Component code (JSX/TSX)
    name: str | None       # Optional: Component name
    docs: str | None       # Optional: Component documentation
    args: Any | None       # Optional: Component arguments (JSON value)
```

### Usage

```python
from dinja import Component

# Simple string format (recommended)
components = {
    "Button": "export default function Component(props) { return <button>{props.children}</button>; }"
}

# Full Component definition
components = {
    "Button": Component(
        code="export default function Component(props) { return <button>{props.children}</button>; }",
        name="Button",
        docs="A button component",
        args={"children": "ReactNode"},
    )
}
```

### Helper Methods

#### from_dict()

Convert a dictionary of name→code to name→Component:

```python
components_dict = {
    "Button": "export default function Component(props) { return <button>{props.children}</button>; }",
    "Card": "export default function Component(props) { return <div>{props.children}</div>; }",
}

components = Component.from_dict(components_dict)
```

## Input

Input structure for batch MDX rendering requests.

```python
from dinja import Input

@dataclass
class Input:
    views: dict[str, str]                              # Required: Map of view names to MDX content
    utils: str | None = None                           # Optional: JavaScript utilities
    components: dict[str, Component] | dict[str, str] | None = None  # Optional: Components
    minify: bool = True                                # Enable minification
    directives: list[str] | None = None                # Optional: Directive prefixes
```

### Auto-Conversion

If you provide `components` as a `dict[str, str]` (name → code), it will be automatically converted to `dict[str, Component]`:

```python
# Simple dict format (automatically converted)
input_data = Input(
    views={"page.mdx": "# Hello"},
    components={
        "Button": "export default function Component(props) { return <button>{props.children}</button>; }"
    }
)
```

## Result

Result of a batch render operation.

```python
from dinja import Result

@dataclass
class Result:
    total: int                      # Total number of files processed
    succeeded: int                  # Number of successful renders
    failed: int                     # Number of failed renders
    files: dict[str, FileResult]    # File results
    errors: list[dict[str, str]]    # Error list
```

### Methods

```python
result.is_all_success()           # True if all files succeeded
result.get_output("page.mdx")     # Get output for a file
result.get_metadata("page.mdx")   # Get metadata for a file
```

## FileResult

Result for a single rendered file.

```python
from dinja import FileResult

@dataclass
class FileResult:
    success: bool                   # Whether rendering succeeded
    metadata: dict[str, Any]        # Parsed YAML frontmatter
    output: str | None              # Rendered output
    error: str | None               # Error message if failed
```

## Type Aliases

```python
from dinja import Output

Output = Literal["html", "javascript", "schema", "json"]
```

### Output Format Details

- **`html`**: Returns rendered HTML
- **`javascript`**: Returns JavaScript code (Preact syntax with h() and Fragment)
- **`schema`**: Returns a JSON array of unique component names found in the MDX content
- **`json`**: Returns a JSON representation of the transformed JSX tree

## Examples

### Using Dataclasses

```python
from dinja import Renderer, Component, Input

renderer = Renderer("http://localhost:8080")

# Create input
input_data = Input(
    views={"page.mdx": "# Hello <Button>Click</Button>"},
    components={
        "Button": "export default function Component(props) { return <button>{props.children}</button>; }"
    },
    minify=True,
)

# Note: Input is for reference, methods accept kwargs directly
result = renderer.html(
    views=input_data.views,
    components=input_data.components,
    minify=input_data.minify,
)
```

### Direct Method Calls (Recommended)

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")

# Direct kwargs
result = renderer.html(
    views={"page.mdx": "# Hello <Button>Click</Button>"},
    components={
        "Button": "export default function Component(props) { return <button>{props.children}</button>; }"
    },
    minify=True,
)

if result.is_all_success():
    print(result.get_output("page.mdx"))
```
