<div align="center">

![Dinja Logo](docs/docs/assets/media/logo.png)

</div>

# Dinja

Safe, deterministic MDX rendering powered by a Rust core with batteries-included Python bindings.

## Installation

| Target         | Command                |
| -------------- | ---------------------- |
| Python package | `uv add dinja`         |
| Rust crate     | `cargo add dinja-core` |

## Usage

### Rust

```rust
use dinja_core::service::{RenderService, RenderServiceConfig};

fn main() -> anyhow::Result<()> {
    let config = RenderServiceConfig::default();
    let service = RenderService::new(config)?;

    let html = service.render_string("example.mdx", "# Hello **dinja**")?;
    println!("{html}");
    Ok(())
}
```

### Python

```python
from dinja import Renderer, Input, Settings

# Create a renderer instance (engine loads once)
renderer = Renderer()

# Render MDX content with type-safe dataclasses
result = renderer.render(
    Input(
        mdx={"example.mdx": "---\ntitle: Demo\n---\n# Hello **dinja**"},
        settings=Settings(output="html", minify=True, engine="base"),
    )
)

entry = result["files"]["example.mdx"]

if entry["status"] == "success":
    rendered = entry["result"]
    metadata = rendered.get("metadata", {})
    print("title:", metadata.get("title"))
    print("html:", rendered.get("output"))
else:
    print("error:", entry.get("error"))

# Reuse the same instance for multiple renders with different modes
result1 = renderer.render(
    Input(
        mdx={"page1.mdx": "# Page 1"},
        settings=Settings(output="html"),
    )
)

result2 = renderer.render(
    Input(
        mdx={"page2.mdx": "# Page 2"},
        settings=Settings(output="schema"),
    )
)
```

`rendered["output"]` contains HTML, JavaScript, or schema code depending on `settings.output`.

More examples live in `python-bindings/examples/`.

## License

BSD 3-Clause. See `LICENSE`.
