#!/usr/bin/env python3
# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "cyclopts>=2.9",
# ]
# ///
"""
Release utilities for the dinja workspace, implemented with Cyclopts.

Commands:

  * bump: update version strings for the Rust workspace and/or Python bindings.
          Examples:
              uv run release.py bump --version 0.3.0          # update both
              uv run release.py bump --python-version 0.2.5   # python only
              uv run release.py bump --rust-version 0.2.1     # rust only

  * release: run the validation pipeline and create a git tag that triggers the
             GitHub Actions release workflow (same behavior as the former shell
             script). Versions must already be updated and committed.
              uv run release.py release 0.3.0
"""
from __future__ import annotations

import importlib
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, Optional

cyclopts_module = importlib.import_module("cyclopts")
App = cyclopts_module.App
Parameter = cyclopts_module.Parameter

ROOT = Path(__file__).resolve().parent
PYTHON_BINDINGS = ROOT / "python-bindings"


class ReleaseError(RuntimeError):
    """Custom error surfaced to the CLI with a friendly message."""


@dataclass(frozen=True)
class VersionField:
    path: Path
    pattern: re.Pattern[str]
    label: str


def _compile_version_pattern(lhs: str) -> re.Pattern[str]:
    return re.compile(
        rf'(?m)^(?P<prefix>\s*{re.escape(lhs)}\s*=\s*")(?P<value>[^"]+)(?P<suffix>")'
    )


VERSION_FIELDS: Dict[str, tuple[VersionField, ...]] = {
    "rust": (
        VersionField(
            path=ROOT / "Cargo.toml",
            pattern=_compile_version_pattern("version"),
            label="workspace Cargo.toml",
        ),
    ),
    "python": (
        VersionField(
            path=PYTHON_BINDINGS / "pyproject.toml",
            pattern=_compile_version_pattern("version"),
            label="python-bindings/pyproject.toml",
        ),
        VersionField(
            path=PYTHON_BINDINGS / "dinja" / "__about__.py",
            pattern=_compile_version_pattern("__version__"),
            label="python-bindings/dinja/__about__.py",
        ),
    ),
}


def _replace_in_field(field: VersionField, new_version: str, dry_run: bool) -> bool:
    text = field.path.read_text(encoding="utf-8")
    match = field.pattern.search(text)
    if not match:
        raise ReleaseError(f"Could not locate a version field in {field.path}")

    current = match.group("value")
    if current == new_version:
        print(f"{field.label} already set to {new_version}")
        return False

    if dry_run:
        print(f"[dry-run] {field.label}: {current} -> {new_version}")
        return True

    updated = "".join(
        (
            text[: match.start("value")],
            new_version,
            text[match.end("value") :],
        )
    )
    field.path.write_text(updated, encoding="utf-8")
    print(f"Updated {field.label}: {current} -> {new_version}")
    return True


def update_versions(
    *, rust: Optional[str], python: Optional[str], dry_run: bool
) -> bool:
    if not rust and not python:
        raise ReleaseError(
            "At least one of --rust-version or --python-version is required."
        )

    changed = False
    if rust:
        for field in VERSION_FIELDS["rust"]:
            changed |= _replace_in_field(field, rust, dry_run)
    if python:
        for field in VERSION_FIELDS["python"]:
            changed |= _replace_in_field(field, python, dry_run)
    return changed


def read_current_versions() -> Dict[str, str]:
    versions: Dict[str, str] = {}
    for key, fields in VERSION_FIELDS.items():
        # assume the first field is the canonical source of truth per component
        field = fields[0]
        text = field.path.read_text(encoding="utf-8")
        match = field.pattern.search(text)
        if not match:
            raise ReleaseError(f"Could not read version from {field.path}")
        versions[key] = match.group("value")
    return versions


def run_cmd(
    cmd: list[str],
    *,
    cwd: Optional[Path] = None,
    env: Optional[dict] = None,
    debug: bool = False,
) -> None:
    display_cwd = f"(cd {cwd} && " if cwd else ""
    close = ")" if cwd else ""
    print(f"$ {display_cwd}{' '.join(cmd)}{close}")
    if debug:
        print(f"[DEBUG] Working directory: {cwd or Path.cwd()}")
        if env:
            relevant_env = {k: v for k, v in env.items() if k.startswith("PYO3_") or k == "VIRTUAL_ENV"}
            if relevant_env:
                print(f"[DEBUG] Environment: {relevant_env}")
    subprocess.run(cmd, cwd=cwd, env=env, check=True)


def ensure_clean_tree(debug: bool = False) -> None:
    if debug:
        print("[DEBUG] Checking git working tree status...")
    try:
        run_cmd(["git", "diff", "--quiet"], debug=debug)
        run_cmd(["git", "diff", "--cached", "--quiet"], debug=debug)
        if debug:
            print("[DEBUG] Working tree is clean")
    except subprocess.CalledProcessError as exc:
        # Get list of uncommitted files for better error message
        try:
            result = subprocess.check_output(
                ["git", "status", "--short"], text=True, stderr=subprocess.DEVNULL
            ).strip()
            if result:
                files = "\n  ".join(result.split("\n"))
                raise ReleaseError(
                    f"Working tree must be clean before releasing.\n"
                    f"Uncommitted changes:\n  {files}\n"
                    f"Commit or stash these changes first."
                ) from exc
        except subprocess.CalledProcessError:
            pass
        raise ReleaseError("Working tree must be clean before releasing.") from exc


def ensure_uv_python(debug: bool = False) -> dict:
    if debug:
        print("[DEBUG] Checking for uv and Python interpreter...")
    
    if shutil.which("uv") is None:
        raise ReleaseError(
            "uv is required (install it from https://docs.astral.sh/uv/)."
        )

    def find_python() -> str:
        try:
            result = subprocess.check_output(
                ["uv", "python", "find"], text=True
            ).strip()
            return result
        except subprocess.CalledProcessError as exc:
            raise ReleaseError("Failed to locate a Python interpreter via uv.") from exc

    python_path = find_python()
    if not python_path:
        print("uv did not report a Python interpreter; attempting installation...")
        if debug:
            print("[DEBUG] Installing Python via uv...")
        run_cmd(["uv", "python", "install"], debug=debug)
        python_path = find_python()
        if not python_path:
            raise ReleaseError("uv could not provide a usable Python interpreter.")

    env = os.environ.copy()
    env["PYO3_PYTHON"] = python_path
    print(f"Using PYO3_PYTHON={python_path}")
    if debug:
        print(f"[DEBUG] Python environment configured: {python_path}")
    return env


def run_release_checks(*, skip_tests: bool, env: dict, debug: bool = False) -> None:
    if debug:
        print("[DEBUG] Starting release checks...")
        print(f"[DEBUG] Skip tests: {skip_tests}")
    
    run_cmd(["cargo", "fmt", "--all", "--check"], env=env, debug=debug)
    run_cmd(
        [
            "cargo",
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
        env=env,
        debug=debug,
    )
    if not skip_tests:
        run_cmd(["cargo", "test", "--all-features"], env=env, debug=debug)

    run_cmd(["uv", "sync", "--dev"], cwd=PYTHON_BINDINGS, env=env, debug=debug)
    if not skip_tests:
        run_cmd(["uv", "run", "pytest"], cwd=PYTHON_BINDINGS, env=env, debug=debug)
    
    if debug:
        print("[DEBUG] All release checks completed successfully")


app = App(help=__doc__)


def _default_commit_message(
    *, rust_version: str | None, python_version: str | None
) -> str:
    components = []
    if rust_version:
        components.append(("rust", rust_version))
    if python_version:
        components.append(("python", python_version))

    unique_versions = {version for _, version in components}
    component_labels = "/".join(name for name, _ in components)

    if len(unique_versions) == 1:
        version = next(iter(unique_versions))
        if component_labels in {"rust/python", "python/rust"}:
            label = "rust+python"
        else:
            label = component_labels
        return f"chore: bump {label} to v{version}"

    bumped_pairs = ", ".join(f"{name}=v{version}" for name, version in components)
    return f"chore: bump versions ({bumped_pairs})"


@app.command(help="Update version strings in project files.")
def bump(
    version: str | None = Parameter(
        None, help="Version applied to both the Rust workspace and Python bindings."
    ),
    rust_version: str | None = Parameter(
        None, help="Version applied only to the Rust workspace."
    ),
    python_version: str | None = Parameter(
        None, help="Version applied only to the Python bindings."
    ),
    dry_run: bool = Parameter(
        False, help="Show planned edits without touching the files."
    ),
    commit: bool = Parameter(True, help="Automatically commit the version bump."),
    commit_message: str | None = Parameter(
        None,
        help="Custom commit message (defaults to 'chore: bump ...').",
    ),
    debug: bool = Parameter(
        False, help="Enable debug output with verbose logging."
    ),
) -> None:
    if debug:
        print("[DEBUG] Version bump operation starting...")
        print(f"[DEBUG] version: {version!r}")
        print(f"[DEBUG] rust_version: {rust_version!r}")
        print(f"[DEBUG] python_version: {python_version!r}")
        print(f"[DEBUG] dry_run: {dry_run}")
        print(f"[DEBUG] commit: {commit}")
    
    rust_version = rust_version or version
    python_version = python_version or version
    if not rust_version and not python_version:
        raise ReleaseError(
            "Specify --version for both components or at least one of "
            "--rust-version / --python-version."
        )

    changed = update_versions(rust=rust_version, python=python_version, dry_run=dry_run)
    if not changed:
        print("No files were updated.")
        if debug:
            print("[DEBUG] All version fields already match target versions")
        return

    if dry_run or not commit:
        if dry_run and commit:
            print("[dry-run] Skipping git commit.")
        if debug:
            print("[DEBUG] Skipping commit (dry_run or commit disabled)")
        return

    msg = commit_message or _default_commit_message(
        rust_version=rust_version, python_version=python_version
    )
    if debug:
        print(f"[DEBUG] Commit message: {msg}")
    run_cmd(["git", "commit", "-am", msg], debug=debug)


@app.command(
    help="Run formatting, linting, tests, and create the git tag that triggers publishing."
)
def release(
    version: str = Parameter(
        help="Semantic version that must already match Cargo.toml and pyproject.toml."
    ),
    skip_tests: bool = Parameter(
        False,
        help="Skip Rust and Python tests (still runs fmt/clippy/uv sync).",
    ),
    no_push: bool = Parameter(
        False, help="Create the tag locally without pushing HEAD/tag to origin."
    ),
    dry_run: bool = Parameter(
        False, help="Run checks but do not create or push the git tag."
    ),
    debug: bool = Parameter(
        False, help="Enable debug output with verbose logging."
    ),
) -> None:
    if debug:
        print("[DEBUG] Release operation starting...")
        print(f"[DEBUG] Target version: {version!r}")
        print(f"[DEBUG] skip_tests: {skip_tests}")
        print(f"[DEBUG] no_push: {no_push}")
        print(f"[DEBUG] dry_run: {dry_run}")
    
    ensure_clean_tree(debug=debug)
    versions = read_current_versions()
    if debug:
        print(f"[DEBUG] Current versions: {versions}")
    
    expected = version
    mismatches = {
        component: value for component, value in versions.items() if value != expected
    }
    if mismatches:
        mismatch_lines = ", ".join(
            f"{comp}={value}" for comp, value in mismatches.items()
        )
        raise ReleaseError(
            f"Version mismatch. Expected {expected} everywhere but found {mismatch_lines}. "
            "Use `python release.py bump --version ...` first."
        )
    
    if debug:
        print(f"[DEBUG] Version check passed: all components at {expected}")

    env = ensure_uv_python(debug=debug)
    
    run_release_checks(skip_tests=skip_tests, env=env, debug=debug)

    tag = f"v{expected}"
    if dry_run:
        print(f"[dry-run] Would create git tag {tag} and push to origin.")
        if debug:
            print(f"[DEBUG] Dry run mode: skipping actual git operations")
        return

    if debug:
        print(f"[DEBUG] Creating git tag: {tag}")
    run_cmd(["git", "tag", "-a", tag, "-m", f"release: v{expected}"], debug=debug)
    if no_push:
        print("Skipping git push (--no-push flag provided).")
        if debug:
            print("[DEBUG] Tag created locally, not pushing to origin")
        return

    if debug:
        print("[DEBUG] Pushing HEAD and tag to origin...")
    run_cmd(["git", "push", "origin", "HEAD"], debug=debug)
    run_cmd(["git", "push", "origin", tag], debug=debug)
    if debug:
        print("[DEBUG] Release completed successfully")


def main(argv: Optional[Iterable[str]] = None) -> None:
    try:
        if argv is None:
            app()
        else:
            app(argv)
    except ReleaseError as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
    except subprocess.CalledProcessError as exc:
        print(
            f"error: command {' '.join(map(str, exc.cmd))} failed",
            file=sys.stderr,
        )
        raise SystemExit(exc.returncode) from exc


if __name__ == "__main__":
    main()
