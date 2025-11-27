# Lint and Format Dinja Code

Run linting and formatting checks on the dinja codebase.

## Arguments
- `$ARGUMENTS` - What to run: `check` (default), `fix`, or `all`

## Instructions

### For `check` (default):
Run all checks without making changes:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

### For `fix`:
Auto-fix formatting and apply clippy suggestions:
```bash
cargo fmt --all
cargo clippy --all-targets --all-features --fix --allow-dirty
```

### For `all`:
Run comprehensive checks including:
1. Rust formatting (cargo fmt)
2. Rust linting (cargo clippy)
3. Check for unused dependencies

Report any issues found and suggest fixes. For clippy warnings, explain what each warning means and how to resolve it.
