<div align="center">

<img src="assets/media/logo.png" alt="Dinja Logo" />

</div>

# Dinja

**Safe, deterministic MDX rendering powered by a Rust core with batteries-included Python bindings.**

Dinja is a high-performance MDX renderer that combines the speed of Rust with the ease of Python, providing identical output across both languages.

## Why Dinja?

- **🚀 Native Performance** – The renderer is written in Rust and avoids Node.js runtime costs
- **🔄 Identical Output** – Both the Rust crate and Python bindings call into the same engine
- **⚡ Zero Setup** – Static JS dependencies are embedded and extracted on demand
- **📦 Prebuilt Binaries** – PyPI ships wheels for Linux, macOS (x86/arm), and Windows (abi3-py313+)
- **🛡️ Safe & Deterministic** – No external dependencies, predictable rendering behavior

## Quick Example

=== "Python"

    ```python
    from dinja import Renderer, Input, Settings

    renderer = Renderer()
    
    result = renderer.render(
        Input(
            mdx={"page.mdx": "# Hello **dinja**"},
            settings=Settings(output="html"),
        )
    )
    
    print(result["files"]["page.mdx"]["result"]["output"])
    ```

=== "Rust"

    ```rust
    use dinja_core::service::{RenderService, RenderServiceConfig};

    let config = RenderServiceConfig::default();
    let service = RenderService::new(config)?;
    
    let html = service.render_string("page.mdx", "# Hello **dinja**")?;
    println!("{html}");
    ```

## Features

### Multiple Output Formats

Dinja supports three output formats:

- **HTML** – Rendered HTML ready for display
- **JavaScript** – Transformed JavaScript code
- **Schema** – Component schema representation

### Component Support

Define custom components and use them in your MDX:

```python
from dinja import Renderer, Input, Settings

components = {
    "Button": "function Component(props) { return <button>{props.children}</button>; }"
}

renderer = Renderer()
result = renderer.render(
    Input(
        mdx={"page.mdx": "# Hello <Button>Click me</Button>"},
        settings=Settings(engine="custom"),
        components=components,
    )
)
```

### Automatic Retry Logic

The Python `Renderer` class includes automatic retry logic for transient v8 isolate errors:

```python
# Automatically retries on v8 isolate errors
renderer = Renderer(max_retries=3, retry_delay=0.05)
```

## Installation

=== "Python"

    ```bash
    uv add dinja
    # or
    pip install dinja
    ```

=== "Rust"

    ```bash
    cargo add dinja-core
    ```

## What's Next?

- [Installation Guide](getting-started/installation.md) – Get started with Dinja
- [Quick Start](getting-started/quick-start.md) – Your first MDX render
- [Python API Reference](python/overview.md) – Complete Python documentation
- [Rust API Reference](rust/overview.md) – Complete Rust documentation

## License

BSD 3-Clause. See [LICENSE](https://github.com/hlop3z/dinja/blob/main/LICENSE) for details.

