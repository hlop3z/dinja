# Installation

Dinja provides a Rust HTTP service with Python and TypeScript clients.

## Service Installation

### Using Docker (Recommended)

```bash
docker pull ghcr.io/hlop3z/dinja:latest
docker run -p 8080:8080 ghcr.io/hlop3z/dinja:latest
```

### Building from Source

```bash
git clone https://github.com/hlop3z/dinja.git
cd dinja
cargo build --release -p dinja-core
./target/release/dinja-core
```

## Client Installation

### Python

=== "Using uv (Recommended)"

    ```bash
    uv add dinja
    ```

=== "Using pip"

    ```bash
    pip install dinja
    ```

### TypeScript/JavaScript

```bash
npm install @dinja/core
```

### Rust (Direct Integration)

Add to your `Cargo.toml`:

```toml
[dependencies]
dinja-core = "0.2"
```

Or use cargo:

```bash
cargo add dinja-core
```

## Verification

### Python

```python
from dinja import Renderer

renderer = Renderer("http://localhost:8080")

if renderer.health():
    print("Dinja service is running!")
    result = renderer.html(views={"test.mdx": "# Hello"})
    print(result.get_output("test.mdx"))
```

### TypeScript

```typescript
import { Renderer } from '@dinja/core';

const renderer = new Renderer({ baseUrl: 'http://localhost:8080' });

const isHealthy = await renderer.health();
if (isHealthy) {
    console.log('Dinja service is running!');
    const result = await renderer.html({ views: { 'test.mdx': '# Hello' } });
    console.log(result);
}
```

### Rust

```rust
use dinja_core::service::{RenderService, RenderServiceConfig};

fn main() -> anyhow::Result<()> {
    let service = RenderService::new(RenderServiceConfig::default())?;
    println!("Dinja initialized successfully!");
    Ok(())
}
```

## Troubleshooting

### Service Not Running

If you get connection errors:

1. Ensure the Docker container is running:
   ```bash
   docker ps
   ```
2. Check if port 8080 is available:
   ```bash
   curl http://localhost:8080/health
   ```

### Python Import Errors

If you encounter import errors:

1. Verify the installation:
   ```bash
   pip show dinja
   ```
2. Check your Python version (3.8+ required):
   ```bash
   python --version
   ```

### TypeScript Import Errors

If you encounter import errors:

1. Verify the installation:
   ```bash
   npm list @dinja/core
   ```
2. Check your Node.js version (18+ recommended):
   ```bash
   node --version
   ```

## Next Steps

Once installed, check out the [Quick Start Guide](quick-start.md) to render your first MDX file.
