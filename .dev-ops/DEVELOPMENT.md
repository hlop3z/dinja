# Development Guide

## Project layout

- `core/` – Rust crate (`dinja-core`) with the rendering engine.
- `python-bindings/` – PyO3 wrapper that exposes the engine as the `dinja` Python package.
- `docs/dev-docs/` – contributor docs (release playbooks, etc.).

## Local workflow

Requirements: Rust (stable) and [`uv`](https://docs.astral.sh/uv/latest/).

```sh
cargo fmt --all
cargo clippy --all-targets --all-features
cargo test --all-features

cd python-bindings
uv sync --dev
uv run pytest
```

## Releases

Use the Cyclopts helper:

```sh
uv run release.py bump --version 0.3.0   # updates files and commits
uv run release.py release 0.3.0          # runs checks, tags, pushes
```

For the detailed workflow (selective bumps, flags, troubleshooting) see `.dev-ops/RELEASE.md`.
