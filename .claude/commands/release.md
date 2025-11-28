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
