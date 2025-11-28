# Renderer Classes

Dinja provides two renderer classes: `Renderer` for synchronous code and `AsyncRenderer` for asynchronous code.

## Renderer (Sync)

The synchronous HTTP client for the Dinja MDX rendering service.

### Constructor

```python
Renderer(
    base_url: str = "http://localhost:8080",
    timeout: float = 30.0
)
```

#### Parameters

- **base_url** (`str`, default: `"http://localhost:8080"`) – URL of the Dinja service
- **timeout** (`float`, default: `30.0`) – Request timeout in seconds

## AsyncRenderer (Async)

The asynchronous HTTP client for the Dinja MDX rendering service.

### Installation

Requires an async HTTP library:

```bash
pip install dinja[httpx]    # recommended
pip install dinja[aiohttp]  # alternative
```

### Constructor

```python
AsyncRenderer(
    base_url: str = "http://localhost:8080",
    timeout: float = 30.0,
    backend: Literal["httpx", "aiohttp"] | None = None
)
```

#### Parameters

- **base_url** (`str`, default: `"http://localhost:8080"`) – URL of the Dinja service
- **timeout** (`float`, default: `30.0`) – Request timeout in seconds
- **backend** (`"httpx" | "aiohttp" | None`, default: `None`) – HTTP backend to use. If `None`, auto-detects available library.

### Context Manager

`AsyncRenderer` supports async context manager for automatic cleanup:

```python
async with AsyncRenderer("http://localhost:8080") as renderer:
    result = await renderer.html(views={...})
# Connection automatically closed
```

Or manually:

```python
renderer = AsyncRenderer("http://localhost:8080")
try:
    result = await renderer.html(views={...})
finally:
    await renderer.close()
```

## Methods

Both `Renderer` and `AsyncRenderer` share the same methods. For async, just add `await`.

### html()

Renders MDX content to HTML.

```python
# Sync
def html(
    self,
    views: dict[str, str],
    components: dict[str, Component] | dict[str, str] | None = None,
    minify: bool = True,
    utils: str | None = None,
    directives: list[str] | None = None,
) -> Result

# Async
async def html(...) -> Result
```

### javascript()

Renders MDX content to JavaScript.

```python
# Sync
def javascript(
    self,
    views: dict[str, str],
    components: dict[str, Component] | dict[str, str] | None = None,
    minify: bool = True,
    utils: str | None = None,
    directives: list[str] | None = None,
) -> Result

# Async
async def javascript(...) -> Result
```

### schema()

Extracts custom component names from MDX content.

```python
# Sync
def schema(
    self,
    views: dict[str, str],
    components: dict[str, Component] | dict[str, str] | None = None,
    minify: bool = True,
    utils: str | None = None,
    directives: list[str] | None = None,
) -> Result

# Async
async def schema(...) -> Result
```

### json()

Renders MDX content to JSON tree representation.

```python
# Sync
def json(
    self,
    views: dict[str, str],
    components: dict[str, Component] | dict[str, str] | None = None,
    minify: bool = True,
    utils: str | None = None,
    directives: list[str] | None = None,
) -> Result

# Async
async def json(...) -> Result
```

### render()

Generic render method with configurable output format.

```python
# Sync
def render(
    self,
    output: Output,  # "html" | "javascript" | "schema" | "json"
    views: dict[str, str],
    components: dict[str, Component] | dict[str, str] | None = None,
    minify: bool = True,
    utils: str | None = None,
    directives: list[str] | None = None,
) -> Result

# Async
async def render(...) -> Result
```

### health()

Checks if the Dinja service is running.

```python
# Sync
def health(self) -> bool

# Async
async def health(self) -> bool
```

### close() (Async only)

Closes the HTTP client connection.

```python
async def close(self) -> None
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

### Basic Sync Usage

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")

result = renderer.html(views={"page.mdx": "# Hello World"})

if result.is_all_success():
    print(result.get_output("page.mdx"))
```

### Basic Async Usage

```python
import asyncio
from dinja import AsyncRenderer

async def main():
    async with AsyncRenderer("http://localhost:8080") as renderer:
        result = await renderer.html(views={"page.mdx": "# Hello World"})
        if result.is_all_success():
            print(result.get_output("page.mdx"))

asyncio.run(main())
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

### Specifying Async Backend

```python
# Use httpx explicitly
renderer = AsyncRenderer(
    base_url="http://localhost:8080",
    backend="httpx"
)

# Use aiohttp explicitly
renderer = AsyncRenderer(
    base_url="http://localhost:8080",
    backend="aiohttp"
)
```

### Multiple Renders

```python
# Sync
renderer = Renderer("http://localhost:8080")
result1 = renderer.html(views={"page1.mdx": "# Page 1"})
result2 = renderer.schema(views={"page2.mdx": "# Page 2"})

# Async
async with AsyncRenderer("http://localhost:8080") as renderer:
    result1 = await renderer.html(views={"page1.mdx": "# Page 1"})
    result2 = await renderer.schema(views={"page2.mdx": "# Page 2"})
```

## Best Practices

1. **Reuse Renderer Instances** – Create one instance and reuse it for multiple renders
2. **Use Context Manager for Async** – Always use `async with` to ensure proper cleanup
3. **Check Health First** – Use `health()` to verify the service is running
4. **Handle Results** – Always check `result.is_all_success()` before accessing output
5. **Use Components Dict** – Simple `dict[str, str]` format is auto-converted to `dict[str, Component]`
