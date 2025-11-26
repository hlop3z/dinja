# Rust API Overview

Dinja's Rust API provides direct access to the core rendering engine with full control over the rendering process.

## Core Types

### RenderService

The main service for rendering MDX content.

```rust
use dinja_core::service::{RenderService, RenderServiceConfig};

let config = RenderServiceConfig::default();
let service = RenderService::new(config)?;
```

### Input (NamedMdxBatchInput in Rust)

Input structure for batch rendering requests.

```rust
use dinja_core::models::{NamedMdxBatchInput, Settings, OutputFormat};

let input = NamedMdxBatchInput {
    settings: Settings {
        output: OutputFormat::Html,
        minify: true,
        ..Default::default()
    },
    mdx: {
        let mut mdx = std::collections::HashMap::new();
        mdx.insert("file.mdx".to_string(), "# Content".to_string());
        mdx
    },
    components: None,
};
```

## Quick Example

```rust
use dinja_core::service::{RenderService, RenderServiceConfig};
use dinja_core::models::{NamedMdxBatchInput, Settings, OutputFormat};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    let config = RenderServiceConfig::default();
    let service = RenderService::new(config)?;

    let mut mdx = HashMap::new();
    mdx.insert("hello.mdx".to_string(), "# Hello World".to_string());

    // Note: In Rust, the type is NamedMdxBatchInput (aliased as Input in Python)
    let input = NamedMdxBatchInput {
        settings: Settings {
            output: OutputFormat::Html,
            minify: true,
            ..Default::default()
        },
        mdx,
        components: None,
    };

    let outcome = service.render_batch(&input)?;

    for (filename, file_result) in outcome.files {
        if let Some(rendered) = file_result.result {
            println!("{}: {}", filename, rendered.output.unwrap_or_default());
        }
    }

    Ok(())
}
```

## API Reference

- [RenderService](render-service.md) – Complete service documentation

