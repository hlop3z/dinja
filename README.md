<div align="center">

![Dinja Logo](https://raw.githubusercontent.com/hlop3z/dinja/main/docs/docs/assets/media/logo.png)

[![Documentation](https://img.shields.io/badge/docs-github.io-blue?style=flat-square)](https://hlop3z.github.io/dinja)
[![Python](https://img.shields.io/pypi/v/dinja?style=flat-square)](https://pypi.org/project/dinja/)
[![Rust](https://img.shields.io/crates/v/dinja-core?style=flat-square)](https://crates.io/crates/dinja-core)
[![GitHub stars](https://img.shields.io/github/stars/hlop3z/dinja?style=flat-square)](https://github.com/hlop3z/dinja/stargazers)
[![License](https://img.shields.io/github/license/hlop3z/dinja?style=flat-square)](LICENSE)

</div>

# Dinja

Safe, deterministic MDX rendering powered by a Rust core with batteries-included Python bindings.

## Links

- [Read the Docs](https://hlop3z.github.io/dinja)
- [GitHub](https://github.com/hlop3z/dinja)
- [PyPI](https://pypi.org/project/dinja)
- [Crates.io](https://crates.io/crates/dinja-core)

## Installation

| Target         | Command                |
| -------------- | ---------------------- |
| Python package | `uv add dinja`         |
| Rust crate     | `cargo add dinja-core` |

## Usage

### Rust

```rust
use dinja_core::service::{RenderService, RenderServiceConfig};
use dinja_core::models::{NamedMdxBatchInput, RenderSettings, OutputFormat};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    // Create a renderer instance (engine loads once)
    let config = RenderServiceConfig::default();
    let service = RenderService::new(config)?;

    // Render MDX content with full settings
    let mut mdx_files = HashMap::new();
    mdx_files.insert(
        "example.mdx".to_string(),
        "---\ntitle: Demo\n---\n# Hello **dinja**".to_string(),
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: true,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input)?;

    // Access results
    if let Some(entry) = outcome.files.get("example.mdx") {
        if entry.status == "success" {
            if let Some(ref result) = entry.result {
                println!("title: {:?}", result.metadata.get("title"));
                println!("html: {}", result.output);
            }
        } else {
            println!("error: {:?}", entry.error);
        }
    }

    // Reuse the same instance for multiple renders with different modes
    let mut page1 = HashMap::new();
    page1.insert("page1.mdx".to_string(), "# Page 1".to_string());

    let result1 = service.render_batch(&NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            ..Default::default()
        },
        mdx: page1,
        components: None,
    })?;

    let mut page2 = HashMap::new();
    page2.insert("page2.mdx".to_string(), "# Page 2".to_string());

    let result2 = service.render_batch(&NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Schema,
            ..Default::default()
        },
        mdx: page2,
        components: None,
    })?;

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
        settings=Settings(output="html", minify=True),
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

### Accessing Metadata in MDX

Metadata from YAML frontmatter is available via the `context` function:

```python
result = renderer.render(
    Input(
        mdx={
            "page.mdx": """
---
title: Welcome
author: Alice
---
# {context('title')}

By {context('author')}
"""
        },
        settings=Settings(output="html"),
    )
)
```

The `context` function supports nested paths: `context('author.name')` for nested metadata.

### Using Global Utils

You can inject global JavaScript utilities that are available in all components:

```python
result = renderer.render(
    Input(
        mdx={"page.mdx": "<Greeting name='Alice' />"},
        settings=Settings(
            output="html",
            utils="export default { greeting: 'Hello', emoji: '👋' }",
        ),
        components={
            "Greeting": """
                export default function Component(props) {
                    return <div>{utils.greeting} {props.name} {utils.emoji}</div>;
                }
            """,
        },
    )
)
```

The `utils` object must be exported using `export default { ... }` and will be available globally as `utils` in all component code. Invalid utils code is silently ignored.

`rendered["output"]` contains HTML, JavaScript, or schema code depending on `settings.output`.

More examples live in `python-bindings/examples/`.

## License

BSD 3-Clause. See `LICENSE`.
