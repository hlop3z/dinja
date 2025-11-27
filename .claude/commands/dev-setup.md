# Development Environment Setup

Set up or verify the development environment for dinja.

## Arguments
- `$ARGUMENTS` - Action: `check` (default), `install`, or `full`

## Instructions

### For `check` (default):
Verify all required tools are installed:
```bash
rustc --version
cargo --version
uv --version
node --version
npm --version
```

Also check:
- Python virtual environment exists in `python-bindings/.venv`
- Node modules installed in `js-bindings/node_modules`

### For `install`:
Install/update development dependencies:

1. **Rust toolchain:**
   ```bash
   rustup update
   ```

2. **Python bindings:**
   ```bash
   cd python-bindings && uv sync --dev
   ```

3. **JavaScript bindings:**
   ```bash
   cd js-bindings && npm install
   ```

### For `full`:
Complete setup including building all components:
1. Run all `install` steps
2. Build core: `cargo build --release -p dinja`
3. Build Python: `cd python-bindings && maturin develop --release`
4. Build JS: `cd js-bindings && npm run build`
5. Run all tests to verify

## Required Tools
- Rust 1.75+ (with cargo)
- Python 3.13+ (with uv package manager)
- Node.js 18+ (with npm)
- maturin (for Python bindings)
