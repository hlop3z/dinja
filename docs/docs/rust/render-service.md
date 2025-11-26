# RenderService

The `RenderService` is the main entry point for rendering MDX content in Rust.

## Creating a Service

```rust
use dinja_core::service::{RenderService, RenderServiceConfig};

let config = RenderServiceConfig::default();
let service = RenderService::new(config)?;
```

### RenderServiceConfig

```rust
pub struct RenderServiceConfig {
    pub static_dir: PathBuf,           // Directory with static JS files
    pub max_cached_renderers: usize,   // Max cached renderers per profile
    pub resource_limits: ResourceLimits, // Resource limits
}
```

## Methods

### render_batch()

Renders a batch of MDX files.

```rust
pub fn render_batch(
    &self,
    input: &NamedMdxBatchInput  // Note: In Python, this is aliased as Input
) -> Result<RenderBatchOutcome, RenderBatchError>
```

### render_string()

Convenience method to render a single MDX string.

```rust
pub fn render_string(
    &self,
    filename: &str,
    content: &str
) -> Result<String, anyhow::Error>
```

## Examples

### Basic Rendering

```rust
use dinja_core::service::{RenderService, RenderServiceConfig};
use dinja_core::models::{NamedMdxBatchInput, Settings, OutputFormat};
use std::collections::HashMap;

let config = RenderServiceConfig::default();
let service = RenderService::new(config)?;

let mut mdx = HashMap::new();
mdx.insert("page.mdx".to_string(), "# Hello World".to_string());

// Note: In Rust, the type is NamedMdxBatchInput (aliased as Input in Python)
let input = NamedMdxBatchInput {
    settings: Settings {
        output: OutputFormat::Html,
        ..Default::default()
    },
    mdx,
    components: None,
};

let outcome = service.render_batch(&input)?;
```

### Using render_string()

```rust
let service = RenderService::new(RenderServiceConfig::default())?;
let html = service.render_string("page.mdx", "# Hello World")?;
println!("{}", html);
```

