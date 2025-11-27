# Build Dinja Components

Build the specified component(s) of the dinja project.

## Arguments
- `$ARGUMENTS` - Component to build: `core`, `python`, `js`, or `all` (default: all)

## Instructions

Based on the argument provided, build the appropriate component(s):

### For `core` or `all`:
```bash
cargo build --release -p dinja
```

### For `python` or `all`:
```bash
cd python-bindings && maturin develop --release
```

### For `js` or `all`:
```bash
cd js-bindings && npm run build
```

## Build Order
When building `all`, build in this order:
1. Core (Rust library)
2. Python bindings
3. JavaScript bindings

Report any build errors clearly and suggest fixes based on the error messages.
