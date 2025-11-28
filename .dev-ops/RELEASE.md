# Release Guide

This document explains how to cut a release of dinja across crates.io and PyPI
using the Cyclopts-based `release.py` helper plus GitHub Actions.

```sh
uv run release.py release --version 0.x.x
```

## Prerequisites

- Clean `main` branch with all changes merged.
- `uv` installed and on PATH.
- Rust toolchain (stable) available.
- Permission to push tags to the origin repository.

## Step 1: Decide the version

- Determine the semantic version to release (e.g., `0.3.0`).
- Ensure changelog/README updates are merged before proceeding.

## Step 2: Bump versions

To bump the Rust workspace, Python client, and JavaScript client in one go, run:

```sh
uv run release.py bump --version 0.3.0
```

This updates all version fields, auto-commits the change (message like
`chore: bump rust+python+javascript to v0.3.0`), and leaves the tree ready for a release tag.
To skip committing use `--no-commit`; to customize the message pass
`--commit-message "custom text"`.

Use the targeted options below if you only need to bump one side. Always run
through `uv run` so Cyclopts (inline dependency) is resolved automatically.

### Python only

```sh
uv run release.py bump --python-version 0.2.5
```

### Rust only

```sh
uv run release.py bump --rust-version 0.2.1
```

After bumping push to `main` and ensure CI passes.

## Step 3: Run the release pipeline

```sh
uv run release.py release 0.3.0
```

What this does:

1. Verifies the working tree is clean.
2. Confirms all known version fields match `0.3.0`.
3. Finds/installs a Python interpreter via `uv` and exports `PYO3_PYTHON`.
4. Runs:
   - `cargo fmt --all --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test --all-features`
   - `uv sync --dev` in `clients/py`
5. Tags the repo (`v0.3.0`) and pushes both `HEAD` and the tag to origin.

### Useful flags

- `--skip-tests`: skip Rust + Python test suites (still runs fmt/clippy/uv sync).
- `--no-push`: tag locally without pushing (useful for rehearsals).
- `--dry-run`: run validations but do not tag/push.

## Step 4: Monitor GitHub Actions

Pushing the tag triggers `.github/workflows/release.yml`, which:

1. Publishes `dinja-core` to crates.io.
2. Builds and publishes the Python `dinja` package to PyPI via Trusted Publisher.
3. Builds and publishes the JavaScript `@dinja/core` package to npm.
4. Builds and publishes the Docker image to GitHub Container Registry.

Watch the workflow for failures. If something breaks, fix and re-run the job or
delete/recreate the tag as needed.

## Step 5: Post-release

- Verify the crate exists on crates.io and the package on PyPI.
- Update release notes / changelog if you have an external announcement.
- Consider drafting a GitHub Release using the tag.

## Troubleshooting

- **Version mismatch error**: run a targeted bump (`release.py bump ...`) and re-run.
- **Dirty tree**: commit or stash changes before invoking `release.py release`.
- **PyO3 Python missing**: ensure `uv` is installed; the script auto-installs a
  Python interpreter if one isn't found.
- **CI wheel failures**: rerun the affected matrix job after fixing the issue.
