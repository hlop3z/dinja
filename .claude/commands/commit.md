# Git Commit via release.py

Stage all changes and create a commit using the project's release.py script.

## Arguments
- `$ARGUMENTS` - Commit message (optional). Use `--uuid` for auto-generated UUID.

## Instructions

Use `release.py commit` to stage and commit changes. This ensures consistent commit workflow.

### Usage Patterns:

1. **With a message:**
   ```bash
   uv run release.py commit -m "fix: description"
   ```

2. **With UUID (for WIP commits):**
   ```bash
   uv run release.py commit --uuid
   ```

3. **Interactive (no args):**
   ```bash
   uv run release.py commit
   ```

### Commit Message Conventions:

Follow conventional commits format:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `style:` - Code style (formatting, whitespace)
- `refactor:` - Code change that neither fixes nor adds
- `perf:` - Performance improvement
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks
- `build:` - Build system or dependencies
- `ci:` - CI/CD changes

### Process:

1. First check git status to see what will be committed
2. If `$ARGUMENTS` is provided:
   - If it's `--uuid` or `-u`, use: `uv run release.py commit --uuid`
   - Otherwise use: `uv run release.py commit -m "$ARGUMENTS"`
3. If no arguments, ask user for commit message or offer UUID option
4. Run the commit command
5. Show the result
