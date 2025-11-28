<div align="center">

![Dinja Logo](https://raw.githubusercontent.com/hlop3z/dinja/main/docs/docs/assets/media/logo.png)

[![Documentation](https://img.shields.io/badge/docs-github.io-blue?style=flat-square)](https://hlop3z.github.io/dinja)
[![Python](https://img.shields.io/pypi/v/dinja?style=flat-square)](https://pypi.org/project/dinja/)
[![npm](https://img.shields.io/npm/v/@dinja/core?style=flat-square)](https://www.npmjs.com/package/@dinja/core)
[![Rust](https://img.shields.io/crates/v/dinja-core?style=flat-square)](https://crates.io/crates/dinja-core)
[![GitHub stars](https://img.shields.io/github/stars/hlop3z/dinja?style=flat-square)](https://github.com/hlop3z/dinja/stargazers)
[![License](https://img.shields.io/github/license/hlop3z/dinja?style=flat-square)](LICENSE)

</div>

# Dinja

Safe, deterministic MDX rendering powered by a Rust HTTP service with Python and TypeScript clients.

## Links

- [Documentation](https://hlop3z.github.io/dinja)
- [GitHub](https://github.com/hlop3z/dinja)
- [Crates.io](https://crates.io/crates/dinja-core)
- [Docker](https://github.com/hlop3z/dinja/pkgs/container/dinja)
- [PyPI](https://pypi.org/project/dinja)
- [NPM](https://www.npmjs.com/package/@dinja/core)

## Quick Start

### 1. Start the Service

```bash
docker pull ghcr.io/hlop3z/dinja:latest
docker run -p 8080:8080 ghcr.io/hlop3z/dinja:latest
```

**Test with curl:**

```bash
curl -X POST http://localhost:8080/render \
  -H "Content-Type: application/json" \
  -d '{"settings":{"output":"html"},"mdx":{"page.mdx":"# Hello **World**"}}'
```

### 2. Install a Client

| Target     | Command                   |
| ---------- | ------------------------- |
| Python     | `pip install dinja`       |
| TypeScript | `npm install @dinja/core` |

### 3. Render MDX

**Python:**

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")

result = renderer.html(
    views={"page.mdx": "# Hello **World**"},
)

print(result.get_output("page.mdx"))
# Output: <h1>Hello <strong>World</strong></h1>
```

**TypeScript:**

```typescript
import { Renderer, getOutput } from "@dinja/core";

const renderer = new Renderer({ baseUrl: "http://localhost:8080" });

const result = await renderer.html({
  views: { "page.mdx": "# Hello **World**" },
});

console.log(getOutput(result, "page.mdx"));
// Output: <h1>Hello <strong>World</strong></h1>
```

## Output Formats

| Format       | Description                    |
| ------------ | ------------------------------ |
| `html`       | Rendered HTML                  |
| `javascript` | JavaScript code                |
| `schema`     | Extract custom component names |
| `json`       | JSON tree representation       |

## Components

Define custom components that render in your MDX:

**Python:**

```python
result = renderer.html(
    views={"app.mdx": "# App\n\n<Button>Click me</Button>"},
    components={
        "Button": "export default function Component(props) { return <button>{props.children}</button>; }"
    },
)
```

**TypeScript:**

```typescript
const result = await renderer.html({
  views: { "app.mdx": "# App\n\n<Button>Click me</Button>" },
  components: {
    Button:
      "export default function Component(props) { return <button>{props.children}</button>; }",
  },
});
```

## Global Utilities

Inject JavaScript utilities available to all components:

```python
result = renderer.html(
    views={"page.mdx": "<Greeting />"},
    components={
        "Greeting": "export default function Component() { return <div>{utils.message}</div>; }"
    },
    utils="export default { message: 'Hello World' }",
)
```

## YAML Frontmatter

Extract metadata from YAML frontmatter:

```python
result = renderer.html(
    views={
        "blog.mdx": """---
title: My Post
author: Alice
---

# {context('title')}

By {context('author')}
"""
    }
)

metadata = result.get_metadata("blog.mdx")
print(metadata["title"])  # "My Post"
```

## API Reference

### Python

```python
from dinja import Renderer, Component, Output

renderer = Renderer(base_url="http://localhost:8080", timeout=30.0)

# Render methods
result = renderer.html(views={...}, components={...}, utils="...", minify=True, directives=[...])
result = renderer.javascript(views={...})
result = renderer.schema(views={...})
result = renderer.json(views={...})
result = renderer.render(output="html", views={...})

# Health check
is_healthy = renderer.health()

# Result methods
result.is_all_success()
result.get_output("filename.mdx")
result.get_metadata("filename.mdx")
```

### TypeScript

```typescript
import { Renderer, isAllSuccess, getOutput, getMetadata } from '@dinja/core';

const renderer = new Renderer({ baseUrl: 'http://localhost:8080', timeout: 30000 });

// Render methods
const result = await renderer.html({ views: {...}, components: {...}, utils: '...', minify: true, directives: [...] });
const result = await renderer.javascript({ views: {...} });
const result = await renderer.schema({ views: {...} });
const result = await renderer.json({ views: {...} });
const result = await renderer.render('html', { views: {...} });

// Health check
const isHealthy = await renderer.health();

// Helper functions
isAllSuccess(result);
getOutput(result, 'filename.mdx');
getMetadata(result, 'filename.mdx');
```

## Rust Core

For direct Rust integration, see [dinja-core on crates.io](https://crates.io/crates/dinja-core).

```rust
use dinja_core::service::{RenderService, RenderServiceConfig};
use dinja_core::models::{NamedMdxBatchInput, RenderSettings, OutputFormat};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    let service = RenderService::new(RenderServiceConfig::default())?;

    let mut mdx = HashMap::new();
    mdx.insert("page.mdx".to_string(), "# Hello **World**".to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: true,
            utils: None,
            directives: None,
        },
        mdx,
        components: None,
    };

    let outcome = service.render_batch(&input)?;

    if let Some(entry) = outcome.files.get("page.mdx") {
        if let Some(ref result) = entry.result {
            println!("{}", result.output);
        }
    }

    Ok(())
}
```

## License

BSD 3-Clause. See `LICENSE`.
