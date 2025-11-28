# Renderer Class

The `Renderer` class is the main entry point for rendering MDX content in Python. It is an HTTP client that connects to the Dinja service.

## Constructor

```python
Renderer(
    base_url: str = "http://localhost:8080",
    timeout: float = 30.0
)
```

### Parameters

- **base_url** (`str`, default: `"http://localhost:8080"`) – URL of the Dinja service
- **timeout** (`float`, default: `30.0`) – Request timeout in seconds

## Methods

### html()

Renders MDX content to HTML.

```python
def html(
    self,
    views: dict[str, str],
    components: dict[str, Component] | dict[str, str] | None = None,
    minify: bool = True,
    utils: str | None = None,
    directives: list[str] | None = None,
) -> Result
```

### javascript()

Renders MDX content to JavaScript.

```python
def javascript(
    self,
    views: dict[str, str],
    components: dict[str, Component] | dict[str, str] | None = None,
    minify: bool = True,
    utils: str | None = None,
    directives: list[str] | None = None,
) -> Result
```

### schema()

Extracts custom component names from MDX content.

```python
def schema(
    self,
    views: dict[str, str],
    components: dict[str, Component] | dict[str, str] | None = None,
    minify: bool = True,
    utils: str | None = None,
    directives: list[str] | None = None,
) -> Result
```

### json()

Renders MDX content to JSON tree representation.

```python
def json(
    self,
    views: dict[str, str],
    components: dict[str, Component] | dict[str, str] | None = None,
    minify: bool = True,
    utils: str | None = None,
    directives: list[str] | None = None,
) -> Result
```

### render()

Generic render method with configurable output format.

```python
def render(
    self,
    output: Output,  # "html" | "javascript" | "schema" | "json"
    views: dict[str, str],
    components: dict[str, Component] | dict[str, str] | None = None,
    minify: bool = True,
    utils: str | None = None,
    directives: list[str] | None = None,
) -> Result
```

### health()

Checks if the Dinja service is running.

```python
def health(self) -> bool
```

## Parameters

All render methods accept the same parameters:

- **views** (`dict[str, str]`) – Map of file names to MDX content strings
- **components** (`dict[str, Component] | dict[str, str] | None`) – Optional custom components
- **minify** (`bool`, default: `True`) – Whether to minify output
- **utils** (`str | None`) – Optional JavaScript utilities available to all components
- **directives** (`list[str] | None`) – Optional directive prefixes

## Return Type

All render methods return a `Result` object:

```python
@dataclass
class Result:
    total: int                      # Total files processed
    succeeded: int                  # Number of successful renders
    failed: int                     # Number of failed renders
    files: dict[str, FileResult]    # Individual file results
    errors: list[dict[str, str]]    # Error list
```

### Result Methods

```python
result.is_all_success()           # True if all files succeeded
result.get_output("page.mdx")     # Get output for a file
result.get_metadata("page.mdx")   # Get metadata for a file
```

## Examples

### Basic Usage

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")

result = renderer.html(views={"page.mdx": "# Hello World"})

if result.is_all_success():
    print(result.get_output("page.mdx"))
```

### With Components

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")

result = renderer.html(
    views={"page.mdx": "# Hello\n\n<Button>Click</Button>"},
    components={
        "Button": "export default function Component(props) { return <button>{props.children}</button>; }"
    },
)
```

### Custom Timeout

```python
# Longer timeout for large batches
renderer = Renderer(
    base_url="http://localhost:8080",
    timeout=60.0
)
```

### Multiple Renders

```python
renderer = Renderer("http://localhost:8080")

# Reuse the same instance for multiple renders
result1 = renderer.html(views={"page1.mdx": "# Page 1"})
result2 = renderer.schema(views={"page2.mdx": "# Page 2"})
```

## Best Practices

1. **Reuse Renderer Instances** – Create one `Renderer` instance and reuse it for multiple renders
2. **Check Health First** – Use `renderer.health()` to verify the service is running
3. **Handle Results** – Always check `result.is_all_success()` before accessing output
4. **Use Components Dict** – Simple `dict[str, str]` format is auto-converted to `dict[str, Component]`
