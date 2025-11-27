# Cleanup Dinja Codebase

Run cleanup checks and optimizations on the dinja codebase.

## Arguments
- `$ARGUMENTS` - Scope: `core`, `python`, `js`, `all`, or `check` (default: check)

## Instructions

Based on the argument provided, perform cleanup operations:

### For `check` (default):
Run analysis without making changes:
```bash
cargo fmt --all -- --check
cargo clippy -p dinja-core -- -D warnings
cargo machete --skip-target-dir 2>/dev/null || echo "cargo-machete not installed"
```

### For `core` or `all`:
Clean and optimize Rust core:
1. Format code: `cargo fmt --all`
2. Fix clippy warnings: `cargo clippy -p dinja-core --fix --allow-dirty`
3. Check for unused dependencies with cargo-machete
4. Look for: excessive `.clone()`, unnecessary `unwrap()`, dead code
5. Run tests to verify: `./utils/build.sh test-core`

### For `python` or `all`:
Clean Python bindings:
1. Check for unused imports
2. Verify type hints are consistent
3. Look for debug statements (print, breakpoint)
4. Run tests: `./utils/build.sh test` (runs both Rust and Python tests)

### For `js` or `all`:
Clean JavaScript bindings:
1. Check for console.log statements
2. Verify TypeScript definitions match Rust API

## Using build.sh
The project includes `./utils/build.sh` for consistent builds and tests:
- `./utils/build.sh test-core` - Run core tests only (no Python required)
- `./utils/build.sh test` - Run all tests (Rust + Python)
- `./utils/build.sh build-core` - Build core only
- `./utils/build.sh build` - Build full workspace

## Cleanup Checklist
When cleaning code, ensure:
- [ ] All clippy warnings resolved
- [ ] Code formatted with rustfmt
- [ ] No unused dependencies
- [ ] No debug statements (dbg!, println! for DEBUG, console.log, breakpoint)
- [ ] Tests still pass after cleanup
- [ ] No unnecessary `.clone()` or allocations in hot paths

## Key Patterns to Fix
- **Excessive `.clone()`** - Use references or `Cow<'_, T>`
- **Unnecessary `unwrap()`** - Use `?` operator or proper error handling
- **Dead code** - Remove unused functions, imports, types
- **Large match chains** - Refactor with traits or lookup tables
- **Nested indentation** - Use early returns and `?` operator
