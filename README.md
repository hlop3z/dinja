# Dinja

Safe, deterministic MDX rendering powered by a Rust core with batteries-included Python bindings.

## Why dinja?

- **Native performance** – the renderer is written in Rust and avoids Node.js runtime costs.
- **Identical output everywhere** – both the Rust crate and the Python bindings call into the same engine.
- **Zero setup** – static JS dependencies are embedded and extracted on demand.
- **Prebuilt binaries** – PyPI ships wheels for Linux, macOS (x86/arm), and Windows (abi3-py313+).

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
from dinja import Renderer

# Create a renderer instance (engine loads once)
renderer = Renderer()

# Render MDX content
result = renderer.render({
    "settings": {"output": "html", "minify": True, "engine": "base"},
    "mdx": {"example.mdx": "---\ntitle: Demo\n---\n# Hello **dinja**"},
})

entry = result["files"]["example.mdx"]

if entry["status"] == "success":
    rendered = entry["result"]
    metadata = rendered.get("metadata", {})
    print("title:", metadata.get("title"))
    print("html:", rendered.get("output"))
else:
    print("error:", entry.get("error"))

# Reuse the same instance for multiple renders with different modes
result1 = renderer.render({
    "settings": {"output": "html"},
    "mdx": {"page1.mdx": "# Page 1"},
})

result2 = renderer.render({
    "settings": {"output": "schema"},
    "mdx": {"page2.mdx": "# Page 2"},
})
```

`rendered["output"]` contains HTML, JavaScript, or schema code depending on `settings["output"]`.

More examples live in `python-bindings/examples/`.

## Development

| Dev Docs                  | Contents                                                      |
| ------------------------- | ------------------------------------------------------------- |
| `.dev-ops/DEVELOPMENT.md` | Repo layout, local workflows, release overview                |
| `.dev-ops/RELEASE.md`     | Full release playbook (bump variants, flags, troubleshooting) |

## License

BSD 3-Clause. See `LICENSE`.
