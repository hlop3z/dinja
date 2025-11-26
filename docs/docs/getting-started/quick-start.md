# Quick Start

Get up and running with Dinja in minutes.

## Python Quick Start

### 1. Create a Renderer

```python
from dinja import Renderer

renderer = Renderer()
```

### 2. Render Your First MDX

```python
from dinja import Input, Settings

result = renderer.render(
    Input(
        mdx={"hello.mdx": "# Hello World\n\nThis is my first MDX file!"},
        settings=Settings(output="html"),
    )
)
```

### 3. Access the Result

```python
entry = result["files"]["hello.mdx"]

if entry["status"] == "success":
    rendered = entry["result"]
    print(rendered["output"])  # <h1>Hello World</h1>...
    print(rendered["metadata"])  # {} (no frontmatter in this example)
else:
    print(f"Error: {entry.get('error')}")
```

## Rust Quick Start

### 1. Create a Render Service

```rust
use dinja_core::service::{RenderService, RenderServiceConfig};

let config = RenderServiceConfig::default();
let service = RenderService::new(config)?;
```

### 2. Render Your First MDX

```rust
let html = service.render_string(
    "hello.mdx",
    "# Hello World\n\nThis is my first MDX file!"
)?;
```

### 3. Use the Result

```rust
println!("{}", html);  // <h1>Hello World</h1>...
```

## Complete Example

=== "Python"

    ```python
    from dinja import Renderer, Input, Settings

    # Create renderer
    renderer = Renderer()

    # Define MDX content with frontmatter
    mdx_content = """---
    title: My First Page
    author: John Doe
    ---
    # Welcome to Dinja
    
    This is **markdown** with *formatting*.
    """

    # Render
    result = renderer.render(
        Input(
            mdx={"index.mdx": mdx_content},
            settings=Settings(output="html", minify=True),
        )
    )

    # Process results
    for filename, entry in result["files"].items():
        if entry["status"] == "success":
            rendered = entry["result"]
            metadata = rendered.get("metadata", {})
            output = rendered.get("output", "")
            
            print(f"File: {filename}")
            print(f"Title: {metadata.get('title')}")
            print(f"Author: {metadata.get('author')}")
            print(f"HTML: {output[:100]}...")
    ```

=== "Rust"

    ```rust
    use dinja_core::service::{RenderService, RenderServiceConfig};
    use std::collections::HashMap;

    let config = RenderServiceConfig::default();
    let service = RenderService::new(config)?;

    let mdx_content = r#"---
    title: My First Page
    author: John Doe
    ---
    # Welcome to Dinja
    
    This is **markdown** with *formatting*.
    "#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("index.mdx".to_string(), mdx_content.to_string());

    let input = dinja_core::models::Input {
        settings: dinja_core::models::Settings {
            output: dinja_core::models::OutputFormat::Html,
            minify: true,
            ..Default::default()
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input)?;

    for (filename, file_result) in outcome.files {
        if let Some(rendered) = file_result.result {
            println!("File: {}", filename);
            println!("Title: {:?}", rendered.metadata.get("title"));
            println!("HTML: {}...", &rendered.output.unwrap_or_default()[..100.min(rendered.output.as_ref().map(|s| s.len()).unwrap_or(0))]);
        }
    }
    ```

## Next Steps

- Learn about [Output Formats](output-formats.md)
- Explore [Component Support](components.md)
- Check out the [Python API Reference](../python/overview.md)

