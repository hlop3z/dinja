# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Dinja is a safe, deterministic MDX (Markdown with JSX) renderer. It has a Rust core (`dinja-core`) that runs as an HTTP service, with Python, JavaScript, and Go HTTP clients.

## Build Commands

All builds use the centralized script `./utils/build.sh`:

```bash
./utils/build.sh build           # Build entire Rust workspace (debug)
./utils/build.sh build-core      # Build core crate only (no Python needed)
./utils/build.sh build-release   # Build workspace (release mode)
```

Client builds:

```bash
cd clients/js && npm run build   # Build JavaScript client
cd clients/py && uv sync         # Sync Python client dependencies
cd clients/go && go build ./...  # Build Go client
```

## Test Commands

```bash
./utils/build.sh test            # Run Rust tests
./utils/build.sh test-core       # Run core crate tests only
cd clients/js && npm test        # Run JavaScript tests
cd clients/py && uv run pytest   # Run Python tests (requires service running)
cd clients/go && go test ./...   # Run Go tests
```

Run a single Rust test:

```bash
cargo test -p dinja-core test_name
```

Run a single Python test:

```bash
uv run pytest clients/py/tests/test_render.py::test_name -v
```

## Lint and Format

```bash
cargo fmt --all -- --check                    # Check formatting
cargo fmt --all                               # Fix formatting
cargo clippy -p dinja-core -- -D warnings     # Lint
cargo machete --skip-target-dir               # Check for unused dependencies
```

## Architecture

### Workspace Structure

- `core/` - Rust core library (`dinja-core` crate) and HTTP service
- `clients/py/` - Python HTTP client
- `clients/js/` - JavaScript/TypeScript HTTP client
- `clients/go/` - Go HTTP client
- `utils/build.sh` - Centralized build orchestration

### Rendering Pipeline

```
MDX Content → Extract YAML Frontmatter (gray_matter) → Markdown to HTML (markdown crate)
            → Transform JSX to JavaScript (oxc) → Execute in Deno Core → Output
```

### Key Patterns

**Thread-Local Renderer Pool**: Each thread maintains its own cache of JavaScript runtimes because V8 isolates are not Send/Sync. This is managed in `core/src/renderer/pool.rs`.

**HTTP Service Architecture**: The Rust core runs as an HTTP service. Python, JavaScript, and Go clients communicate with it via HTTP requests.

**Embedded Static Assets**: JavaScript engine code (`engine.min.js`, etc.) is embedded via `include_str!` and extracted to a temp directory on first use. See `core/static/`.

**Resource Limits**: Prevents memory exhaustion with configurable limits:

- `max_batch_size`: Default 1000
- `max_mdx_content_size`: Default 10 MB
- `max_component_code_size`: Default 1 MB
- `max_cached_renderers`: Default 4

### Output Formats

- `html` - Rendered HTML
- `javascript` - Transform back to JS
- `schema` - Extract custom component names
- `json` - JSON tree representation

## Key Files

- `core/src/service.rs` - Main `RenderService` implementation
- `core/src/mdx.rs` - MDX parsing and rendering
- `core/src/renderer/engine.rs` - Deno Core integration
- `core/src/transform.rs` - TSX/JSX transformation using oxc

## Release Process

Version is managed in `VERSION` file and synced to workspace `Cargo.toml`. The release script `release.py` handles version bumping and tagging. CI/CD publishes to PyPI, npm, crates.io, and Docker Hub on tag push.

## Git Hooks

Install pre-commit hooks for automatic checks before commits:

```bash
./hooks/install.sh
```

The pre-commit hook runs:
- Rust formatting check (`cargo fmt`)
- Clippy linting (`cargo clippy -p dinja-core`)
- Core tests (`cargo test -p dinja-core`)
- Unused dependency check (if `cargo-machete` installed)
- Python/JS/Go checks (if those clients are modified)

## Requirements

- Rust (stable)
- Python 3.13+ (for Python client)
- Node.js 18+ (for JS client)
- Go 1.21+ (for Go client)
- `uv` package manager (Python environment management)
