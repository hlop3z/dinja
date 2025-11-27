# Lint and Format Dinja Code

Run linting and formatting checks on the dinja codebase.

## Arguments
- `$ARGUMENTS` - What to run: `check` (default), `fix`, or `all`

## Instructions

### For `check` (default):
Run all checks without making changes:
```bash
cargo fmt --all -- --check
cargo clippy -p dinja-core -- -D warnings
```

### For `fix`:
Auto-fix formatting and apply clippy suggestions:
```bash
cargo fmt --all
cargo clippy -p dinja-core --fix --allow-dirty
```

### For `all`:
Run comprehensive checks including:
1. Rust formatting (cargo fmt)
2. Rust linting (cargo clippy)
3. Check for unused dependencies:
   ```bash
   cargo machete --skip-target-dir 2>/dev/null || echo "cargo-machete not installed"
   ```

## Integration with build.sh
After fixing lint issues, verify the build and tests pass:
```bash
./utils/build.sh test-core
```

## Notes
- Use `-p dinja-core` to avoid Python environment issues when running clippy
- The full workspace clippy requires Python 3.13 due to `abi3-py313` feature

Report any issues found and suggest fixes. For clippy warnings, explain what each warning means and how to resolve it.
