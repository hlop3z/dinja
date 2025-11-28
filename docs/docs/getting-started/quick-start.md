# Quick Start

Get up and running with Dinja in minutes.

## 1. Start the Service

```bash
docker pull ghcr.io/hlop3z/dinja:latest
docker run -p 8080:8080 ghcr.io/hlop3z/dinja:latest
```

## 2. Install a Client

=== "Python"

    ```bash
    pip install dinja
    ```

=== "TypeScript"

    ```bash
    npm install @dinja/core
    ```

## 3. Render Your First MDX

=== "Python"

    ```python
    from dinja import Renderer

    renderer = Renderer("http://localhost:8080")

    result = renderer.html(
        views={"hello.mdx": "# Hello World\n\nThis is my first MDX file!"}
    )

    print(result.get_output("hello.mdx"))
    # <h1>Hello World</h1><p>This is my first MDX file!</p>
    ```

=== "TypeScript"

    ```typescript
    import { Renderer, getOutput } from '@dinja/core';

    const renderer = new Renderer({ baseUrl: 'http://localhost:8080' });

    const result = await renderer.html({
        views: { 'hello.mdx': '# Hello World\n\nThis is my first MDX file!' }
    });

    console.log(getOutput(result, 'hello.mdx'));
    // <h1>Hello World</h1><p>This is my first MDX file!</p>
    ```

## Complete Example

=== "Python"

    ```python
    from dinja import Renderer

    renderer = Renderer("http://localhost:8080")

    # Define MDX content with frontmatter
    mdx_content = """---
    title: My First Page
    author: John Doe
    ---
    # Welcome to Dinja

    This is **markdown** with *formatting*.
    """

    # Render
    result = renderer.html(views={"index.mdx": mdx_content})

    # Check success
    if result.is_all_success():
        output = result.get_output("index.mdx")
        metadata = result.get_metadata("index.mdx")

        print(f"Title: {metadata.get('title')}")
        print(f"Author: {metadata.get('author')}")
        print(f"HTML: {output}")
    ```

=== "TypeScript"

    ```typescript
    import { Renderer, isAllSuccess, getOutput, getMetadata } from '@dinja/core';

    const renderer = new Renderer({ baseUrl: 'http://localhost:8080' });

    const mdxContent = `---
    title: My First Page
    author: John Doe
    ---
    # Welcome to Dinja

    This is **markdown** with *formatting*.
    `;

    const result = await renderer.html({
        views: { 'index.mdx': mdxContent }
    });

    if (isAllSuccess(result)) {
        const output = getOutput(result, 'index.mdx');
        const metadata = getMetadata(result, 'index.mdx');

        console.log(`Title: ${metadata.title}`);
        console.log(`Author: ${metadata.author}`);
        console.log(`HTML: ${output}`);
    }
    ```

## Rust Quick Start

For direct Rust integration:

```rust
use dinja_core::service::{RenderService, RenderServiceConfig};
use dinja_core::models::{NamedMdxBatchInput, RenderSettings, OutputFormat};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    let service = RenderService::new(RenderServiceConfig::default())?;

    let mut mdx = HashMap::new();
    mdx.insert("hello.mdx".to_string(), "# Hello World".to_string());

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

    if let Some(entry) = outcome.files.get("hello.mdx") {
        if let Some(ref result) = entry.result {
            println!("{}", result.output);
        }
    }

    Ok(())
}
```

## Next Steps

- Learn about [Output Formats](../guides/output-formats.md)
- Explore [Component Support](../guides/components.md)
- Check out the [Python API Reference](../python/overview.md)
