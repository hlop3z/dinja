# Dinja Release Process

Create a release using semver bump type.

## Arguments
- `$ARGUMENTS` - Release type: `patch`, `minor`, `major`, or `check`

## Instructions

### For `check`:
1. Read current version from `VERSION` file
2. Run `git status` to check for uncommitted changes
3. Run `git log --oneline $(git describe --tags --abbrev=0)..HEAD` to show commits since last tag
4. Report findings to user

### For `patch`, `minor`, or `major`:

1. Read current version from `VERSION` file (format: MAJOR.MINOR.PATCH)
2. Calculate new version:
   - `patch`: increment PATCH (0.4.3 -> 0.4.4)
   - `minor`: increment MINOR, reset PATCH (0.4.3 -> 0.5.0)
   - `major`: increment MAJOR, reset MINOR and PATCH (0.4.3 -> 1.0.0)
3. Show user: "Releasing X.Y.Z -> A.B.C"
4. Ask user to confirm before proceeding
5. If confirmed, run:
   ```bash
   uv run release.py release <new_version>
   ```

### What release.py does:
- Updates VERSION file
- Updates Cargo.toml (workspace)
- Updates pyproject.toml and __about__.py (Python)
- Updates package.json (JavaScript)
- Runs formatting, linting, and tests
- Creates git tag `vX.Y.Z`
- Pushes to GitHub (triggers CI/CD for PyPI, npm, crates.io, Docker)

### Useful flags (release.py release):
- `--dry-run` - run all validations but do not tag/push
- `--no-push` - tag locally without pushing (rehearsals)
- `--skip-tests` - skip Rust + Python test suites (still runs fmt/clippy/uv sync)

### Targeted version bumps (without releasing):
```bash
uv run release.py bump --version 0.3.0            # bump everything, auto-commits
uv run release.py bump --python-version 0.2.5     # python client only
uv run release.py bump --rust-version 0.2.1       # rust workspace only
```
Use `--no-commit` to skip the auto-commit, `--commit-message "..."` to customize it.

### Troubleshooting:
- **Version mismatch error**: run a targeted `release.py bump ...` and re-run
- **Dirty tree**: commit or stash before `release.py release`
- **PyO3 Python missing**: ensure `uv` is on PATH; the script auto-installs an interpreter
- **CI failures after tag push**: fix, then re-run the failed job (`gh run rerun <id> --failed`) or delete/recreate the tag
