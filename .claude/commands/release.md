# Dinja Release Process

Help with the release process for dinja.

## Arguments
- `$ARGUMENTS` - Release type: `patch`, `minor`, `major`, or `check`

## Instructions

### For `check`:
1. Read the current version from `VERSION` file
2. Check git status for uncommitted changes
3. Review recent commits since last tag
4. List what would be included in the release

### For `patch`, `minor`, or `major`:

**Important:** Only guide the user through the release process. Do NOT execute the release without explicit confirmation.

1. Read current version from `VERSION`
2. Calculate the new version based on semver:
   - patch: 0.3.1 -> 0.3.2
   - minor: 0.3.1 -> 0.4.0
   - major: 0.3.1 -> 1.0.0
3. Show what changes will be made
4. Guide through running: `uv run python release.py <new_version>`

## Release Checklist
Before releasing, verify:
- [ ] All tests pass (`cargo test --all-features`)
- [ ] No uncommitted changes
- [ ] CHANGELOG is updated (if exists)
- [ ] Version is bumped correctly across all crates

The release.py script handles:
- Updating VERSION file
- Updating Cargo.toml versions (workspace)
- Creating git tag
- Pushing to GitHub (triggers CI/CD for PyPI and npm)
