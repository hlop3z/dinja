#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./commit.sh <version> [commit-message]

Runs release checks, tags the repo, and pushes a release tag that triggers
the GitHub Actions trusted publisher workflow for crates.io and PyPI.
- <version>: semantic version that must already match Cargo.toml and python-bindings/pyproject.toml
- [commit-message]: optional git commit message (defaults to "release: v<version>")

Environment variables:
  (none required locally; GitHub Actions will use repo secrets / trusted publisher)
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

TAG="v${VERSION}"

echo "==> Creating git tag ${TAG}"
git tag -a "${TAG}" -m "${COMMIT_MSG}"
git push origin HEAD
git push origin "${TAG}"

cat <<'MSG'
Release tag pushed. GitHub Actions will now:
  - Publish the dinja-core crate using the repository secret CRATES_IO_TOKEN
  - Publish the dinja Python package via the configured Trusted Publisher
MSG
