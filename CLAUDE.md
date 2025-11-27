# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Dinja is a safe, deterministic MDX (Markdown with JSX) renderer. It has a Rust core (`dinja-core`) with Python bindings (PyO3) and JavaScript bindings (NAPI-rs).

## Build Commands

All builds use the centralized script `./utils/build.sh`:

```bash
./utils/build.sh build           # Build entire Rust workspace (debug)
./utils/build.sh build-core      # Build core crate only (no Python needed)
./utils/build.sh build-release   # Build workspace (release mode)
./utils/build.sh build-python    # Build Python wheels via maturin
./utils/build.sh dev             # Install Python bindings in development mode
```

JavaScript bindings (not integrated with build.sh):

```bash
cd js-bindings && npm run build
```

## Test Commands

```bash
./utils/build.sh test            # Run Rust + Python tests
./utils/build.sh test-core       # Run core crate tests only (no Python needed)
cd js-bindings && npm test       # Run JavaScript tests
```

Run a single Rust test:

```bash
cargo test -p dinja-core test_name
```

Run a single Python test:

```bash
uv run pytest python-bindings/tests/test_render.py::test_name -v
```

## Lint and Format

```bash
cargo fmt --all -- --check                    # Check formatting
cargo fmt --all                               # Fix formatting
cargo clippy -p dinja-core -- -D warnings     # Lint (use -p to avoid Python env issues)
cargo machete --skip-target-dir               # Check for unused dependencies
```

## Architecture

### Workspace Structure

- `core/` - Rust core library (`dinja-core` crate)
- `python-bindings/` - Python bindings via PyO3
- `js-bindings/` - JavaScript bindings via NAPI-rs
- `utils/build.sh` - Centralized build orchestration

### Rendering Pipeline

```
MDX Content → Extract YAML Frontmatter (gray_matter) → Markdown to HTML (markdown crate)
            → Transform JSX to JavaScript (oxc) → Execute in Deno Core → Output
```

### Key Patterns

**Thread-Local Renderer Pool**: Each thread maintains its own cache of JavaScript runtimes because V8 isolates are not Send/Sync. This is managed in `core/src/renderer/pool.rs`.

**Reusable Renderer Instance**: Python and JS bindings expose a `Renderer` class that creates the service once and reuses it across multiple renders, solving V8 isolate cleanup ordering issues.

**Embedded Static Assets**: JavaScript engine code (`engine.min.js`, etc.) is embedded via `include_str!` and extracted to a temp directory on first use. See `core/static/`.

**Resource Limits**: Prevents memory exhaustion with configurable limits (all configurable via `Renderer` constructor in Python/JS):

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

## Requirements

- Rust (stable)
- Python 3.13+ (for bindings, uses `abi3-py313` feature)
- Node.js 18+ (for JS bindings)
- `uv` package manager (Python environment management)
