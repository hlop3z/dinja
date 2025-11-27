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
              uv run release.py bump --version 0.3.0          # update both (use --version flag)
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
from typing import Annotated, Dict, Iterable, Optional

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
            relevant_env = {
                k: v
                for k, v in env.items()
                if k.startswith("PYO3_") or k == "VIRTUAL_ENV"
            }
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

    # Check formatting first, and auto-fix if needed
    try:
        run_cmd(["cargo", "fmt", "--all", "--check"], env=env, debug=debug)
    except subprocess.CalledProcessError:
        print("Code formatting check failed. Auto-formatting code...")
        run_cmd(["cargo", "fmt", "--all"], env=env, debug=debug)
        print("Code formatted. Committing formatting changes...")
        run_cmd(["git", "add", "-u"], env=env, debug=debug)
        run_cmd(
            ["git", "commit", "-m", "style: auto-format code"], env=env, debug=debug
        )
        print("Formatting changes committed. Re-running format check...")
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


app = App(help=__doc__, version_flags="")


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
    target_version: Annotated[
        str | None,
        Parameter(
            name="--version",
            help="Version applied to both the Rust workspace and Python bindings.",
        ),
    ] = None,
    rust_version: Annotated[
        str | None, Parameter(help="Version applied only to the Rust workspace.")
    ] = None,
    python_version: Annotated[
        str | None, Parameter(help="Version applied only to the Python bindings.")
    ] = None,
    dry_run: Annotated[
        bool, Parameter(help="Show planned edits without touching the files.")
    ] = False,
    commit: Annotated[
        bool, Parameter(help="Automatically commit the version bump.")
    ] = True,
    commit_message: Annotated[
        str | None,
        Parameter(help="Custom commit message (defaults to 'chore: bump ...')."),
    ] = None,
    debug: Annotated[
        bool, Parameter(help="Enable debug output with verbose logging.")
    ] = False,
) -> None:
    if debug:
        print("[DEBUG] Version bump operation starting...")
        print(f"[DEBUG] target_version: {target_version!r}")
        print(f"[DEBUG] rust_version: {rust_version!r}")
        print(f"[DEBUG] python_version: {python_version!r}")
        print(f"[DEBUG] dry_run: {dry_run}")
        print(f"[DEBUG] commit: {commit}")

    rust_version = rust_version or target_version
    python_version = python_version or target_version
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
    skip_tests: Annotated[
        bool,
        Parameter(help="Skip Rust and Python tests (still runs fmt/clippy/uv sync)."),
    ] = False,
    no_push: Annotated[
        bool,
        Parameter(help="Create the tag locally without pushing HEAD/tag to origin."),
    ] = False,
    dry_run: Annotated[
        bool, Parameter(help="Run checks but do not create or push the git tag.")
    ] = False,
    debug: Annotated[
        bool, Parameter(help="Enable debug output with verbose logging.")
    ] = False,
) -> None:
    if debug:
        print("[DEBUG] Release operation starting...")
        print(f"[DEBUG] Target version: {version!r}")
        print(f"[DEBUG] skip_tests: {skip_tests}")
        print(f"[DEBUG] no_push: {no_push}")
        print(f"[DEBUG] dry_run: {dry_run}")

    # Check if we need to update versions first
    versions = read_current_versions()
    if debug:
        print(f"[DEBUG] Current versions: {versions}")

    expected = version
    mismatches = {
        component: value for component, value in versions.items() if value != expected
    }
    version_updated = False
    if mismatches:
        mismatch_lines = ", ".join(
            f"{comp}={value}" for comp, value in mismatches.items()
        )
        if debug:
            print(f"[DEBUG] Version mismatch detected: {mismatch_lines}")
            print(f"[DEBUG] Automatically updating all versions to {expected}...")

        # Automatically update versions (both rust and python)
        changed = update_versions(rust=expected, python=expected, dry_run=dry_run)
        if changed:
            version_updated = True
            if dry_run:
                print(f"[dry-run] Would update versions to {expected}")
            else:
                print(f"Updated all versions to {expected}")
                # Commit the version bump automatically
                msg = _default_commit_message(
                    rust_version=expected, python_version=expected
                )
                if debug:
                    print(f"[DEBUG] Committing version bump: {msg}")
                run_cmd(
                    [
                        "git",
                        "add",
                        "Cargo.toml",
                        "python-bindings/pyproject.toml",
                        "python-bindings/dinja/__about__.py",
                    ],
                    debug=debug,
                )
                run_cmd(["git", "commit", "-m", msg], debug=debug)
                # Re-read versions to confirm
                versions = read_current_versions()
                if debug:
                    print(f"[DEBUG] Updated versions: {versions}")
        else:
            if debug:
                print("[DEBUG] No version files needed updating")

    # Check tree is clean (after version update/commit if it happened)
    if not version_updated:
        # Normal case: check tree is clean before proceeding
        ensure_clean_tree(debug=debug)
    elif dry_run:
        # In dry-run, we didn't actually modify files, so just check normally
        ensure_clean_tree(debug=debug)
    else:
        # We updated versions and committed, so tree should be clean now
        try:
            run_cmd(["git", "diff", "--quiet"], debug=debug)
            run_cmd(["git", "diff", "--cached", "--quiet"], debug=debug)
            if debug:
                print("[DEBUG] Working tree is clean after version update")
        except subprocess.CalledProcessError as exc:
            # Get list of uncommitted files for better error message
            try:
                result = subprocess.check_output(
                    ["git", "status", "--short"], text=True, stderr=subprocess.DEVNULL
                ).strip()
                if result:
                    files = "\n  ".join(result.split("\n"))
                    raise ReleaseError(
                        f"Working tree has uncommitted changes after version update:\n  {files}\n"
                        f"Please commit or stash these changes first."
                    ) from exc
            except subprocess.CalledProcessError:
                pass
            raise ReleaseError("Working tree must be clean before releasing.") from exc

    # Final check - ensure versions match after update (skip in dry-run if we showed updates)
    if not (dry_run and version_updated):
        versions = read_current_versions()
        final_mismatches = {
            component: value
            for component, value in versions.items()
            if value != expected
        }
        if final_mismatches:
            mismatch_lines = ", ".join(
                f"{comp}={value}" for comp, value in final_mismatches.items()
            )
            raise ReleaseError(
                f"Version mismatch after update. Expected {expected} everywhere but found {mismatch_lines}."
            )

        if debug:
            print(f"[DEBUG] Version check passed: all components at {expected}")
    elif debug:
        print(
            f"[DEBUG] Skipping final version check (dry-run mode, versions would be updated to {expected})"
        )

    env = ensure_uv_python(debug=debug)

    run_release_checks(skip_tests=skip_tests, env=env, debug=debug)

    tag = f"v{expected}"
    if dry_run:
        print(f"[dry-run] Would create git tag {tag} and push to origin.")
        if debug:
            print("[DEBUG] Dry run mode: skipping actual git operations")
        return

    # Copy README.md to python-bindings directory (before creating tag)
    readme_source = ROOT / "README.md"
    readme_dest = PYTHON_BINDINGS / "README.md"
    if debug:
        print("[DEBUG] Copying README.md to python-bindings directory...")
    if readme_source.exists():
        readme_source_content = readme_source.read_text(encoding="utf-8")
        readme_needs_update = True
        if readme_dest.exists():
            readme_dest_content = readme_dest.read_text(encoding="utf-8")
            readme_needs_update = readme_source_content != readme_dest_content
        if readme_needs_update:
            readme_dest.write_text(readme_source_content, encoding="utf-8")
            print("Updated python-bindings/README.md")
            if not dry_run:
                run_cmd(["git", "add", str(readme_dest)], debug=debug)
                # Check if there are actual changes to commit (file might be untracked)
                try:
                    run_cmd(["git", "diff", "--cached", "--quiet"], debug=debug)
                    # No staged changes, check if file is untracked
                    result = subprocess.run(
                        ["git", "status", "--porcelain", str(readme_dest)],
                        capture_output=True,
                        text=True,
                        check=False,
                    )
                    if result.stdout.strip().startswith("??"):
                        # File is untracked, commit it
                        run_cmd(
                            [
                                "git",
                                "commit",
                                "-m",
                                "chore: sync README.md to python-bindings",
                            ],
                            debug=debug,
                        )
                        print("Committed README.md update to python-bindings")
                    else:
                        if debug:
                            print("[DEBUG] No changes to commit for README.md")
                except subprocess.CalledProcessError:
                    # There are staged changes, commit them
                    run_cmd(
                        [
                            "git",
                            "commit",
                            "-m",
                            "chore: sync README.md to python-bindings",
                        ],
                        debug=debug,
                    )
                    print("Committed README.md update to python-bindings")
        elif debug:
            print("[DEBUG] python-bindings/README.md is already up to date")
    else:
        if debug:
            print("[DEBUG] README.md not found in root directory")

    # Write VERSION file with the released version (before creating tag)
    version_file = ROOT / "VERSION"
    if debug:
        print(f"[DEBUG] Writing VERSION file: {version_file}")
    old_content = (
        version_file.read_text(encoding="utf-8").strip()
        if version_file.exists()
        else None
    )
    version_file.write_text(f"{expected}\n", encoding="utf-8")

    # Commit VERSION file if it changed or is new
    version_file_changed = old_content != expected
    if version_file_changed:
        if debug:
            print("[DEBUG] VERSION file changed, committing...")
        run_cmd(["git", "add", str(version_file)], debug=debug)
        run_cmd(
            ["git", "commit", "-m", f"chore: update VERSION to {expected}"], debug=debug
        )
        print(f"Committed VERSION file with version {expected}")
    elif debug:
        print("[DEBUG] VERSION file unchanged, skipping commit")

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
