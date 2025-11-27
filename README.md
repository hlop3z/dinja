<div align="center">

![Dinja Logo](docs/docs/assets/media/logo.png)

[![License](https://img.shields.io/github/license/hlop3z/dinja?style=flat-square)](LICENSE)
[![Python](https://img.shields.io/pypi/v/dinja?style=flat-square)](https://pypi.org/project/dinja/)
[![Rust](https://img.shields.io/crates/v/dinja-core?style=flat-square)](https://crates.io/crates/dinja-core)
[![Python Version](https://img.shields.io/pypi/pyversions/dinja?style=flat-square)](https://pypi.org/project/dinja/)
[![GitHub stars](https://img.shields.io/github/stars/hlop3z/dinja?style=flat-square)](https://github.com/hlop3z/dinja/stargazers)
[![GitHub issues](https://img.shields.io/github/issues/hlop3z/dinja?style=flat-square)](https://github.com/hlop3z/dinja/issues)
[![GitHub forks](https://img.shields.io/github/forks/hlop3z/dinja?style=flat-square)](https://github.com/hlop3z/dinja/network/members)

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
