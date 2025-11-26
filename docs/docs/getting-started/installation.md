# Installation

Dinja can be installed for both Python and Rust projects.

## Python Installation

### Using uv (Recommended)

```bash
uv add dinja
```

### Using pip

```bash
pip install dinja
```

### Requirements

- Python 3.13 or higher
- Prebuilt wheels are available for:
  - Linux (x86_64, aarch64)
  - macOS (x86_64, arm64)
  - Windows (x86_64, abi3-py313+)

## Rust Installation

Add Dinja to your `Cargo.toml`:

```toml
[dependencies]
dinja-core = "0.2"
```

Or use cargo:

```bash
cargo add dinja-core
```

## Verification

=== "Python"

    ```python
    from dinja import Renderer
    
    renderer = Renderer()
    print("Dinja installed successfully!")
    ```

=== "Rust"

    ```rust
    use dinja_core::service::RenderService;
    
    fn main() {
        println!("Dinja installed successfully!");
    }
    ```

## Troubleshooting

### Python Import Errors

If you encounter import errors:

1. Ensure you're using Python 3.13 or higher
2. Verify the installation:
   ```bash
   pip show dinja
   ```
3. Check your Python environment:
   ```bash
   python --version
   ```

### Rust Compilation Issues

If you encounter compilation issues:

1. Ensure you have a recent Rust toolchain:
   ```bash
   rustc --version
   ```
2. Update your dependencies:
   ```bash
   cargo update
   ```

## Next Steps

Once installed, check out the [Quick Start Guide](quick-start.md) to render your first MDX file.

