#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./commit.sh <version> [commit-message]

Publishes the Rust crate and Python package after verifying the workspace.
- <version>: semantic version that must already match Cargo.toml and python-bindings/pyproject.toml
- [commit-message]: optional git commit message (defaults to "release: v<version>")

Environment variables:
  CRATES_IO_TOKEN  crates.io API token with publish rights
  PYPI_API_TOKEN   PyPI API token for the dinja project
EOF
}

if [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 1
fi

VERSION="$1"
shift
COMMIT_MSG=${1:-"release: v${VERSION}"}

if ! command -v uv >/dev/null 2>&1; then
  echo "uv is required (see project rules for installation)." >&2
  exit 1
fi

if [[ -z "${CRATES_IO_TOKEN:-}" ]]; then
  echo "CRATES_IO_TOKEN must be set." >&2
  exit 1
fi

if [[ -z "${PYPI_API_TOKEN:-}" ]]; then
  echo "PYPI_API_TOKEN must be set." >&2
  exit 1
fi

ROOT_VERSION=$(grep -E '^version\s*=\s*"' Cargo.toml | head -n 1 | sed -E 's/.*"([^"]+)"/\1/')
PY_VERSION=$(grep -E '^version\s*=\s*"' python-bindings/pyproject.toml | head -n 1 | sed -E 's/.*"([^"]+)"/\1/')

if [[ "${ROOT_VERSION}" != "${VERSION}" ]]; then
  echo "Workspace Cargo.toml version (${ROOT_VERSION}) does not match ${VERSION}." >&2
  exit 1
fi

if [[ "${PY_VERSION}" != "${VERSION}" ]]; then
  echo "python-bindings/pyproject.toml version (${PY_VERSION}) does not match ${VERSION}." >&2
  exit 1
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Working tree must be clean before releasing." >&2
  exit 1
fi

echo "==> Checking Rust workspace"
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features

echo "==> Checking Python bindings"
pushd python-bindings >/dev/null
uv sync --dev
uv run pytest
popd >/dev/null

echo "==> Publishing dinja-core"
cargo publish --package dinja-core --locked --token "${CRATES_IO_TOKEN}"

echo "==> Publishing dinja PyPI package"
pushd python-bindings >/dev/null
uv run maturin publish --locked --skip-existing --username __token__ --password "${PYPI_API_TOKEN}"
popd >/dev/null

TAG="v${VERSION}"

echo "==> Creating git tag ${TAG}"
git tag -a "${TAG}" -m "${COMMIT_MSG}"
git push origin HEAD
git push origin "${TAG}"

echo "Release ${TAG} completed."
