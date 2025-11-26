# TODO: Establish project structure

- Create workspace root with shared Cargo.toml
- Add `core` crate (pure Rust)
- Add `python-bindings` crate (PyO3 + maturin)

# TODO: Configure workspace

- Define `[workspace]` with members = ["core", "python-bindings"]

# TODO: Implement `core` crate

- Set up Cargo.toml for pure Rust library (crate-type = ["rlib"])
- Implement core logic, algorithms, and models
- Ensure no Python or FFI dependencies
- Add Rust unit tests

# TODO: Implement Python bindings crate

- Configure Cargo.toml with crate-type = ["cdylib"]
- Add PyO3 dependency with "extension-module"
- Add dependency on `core` crate
- Create pyproject.toml (maturin build system)
- Implement PyO3 wrapper functions, modules, conversions
- Add Python integration tests

# TODO: Improve binding surface

- Add #[pyclass] wrappers for Rust structs (optional)
- Map Rust enums ↔ Python enums or strings
- Map Rust errors ↔ PyErr
- Add feature flags if needed

# TODO: Build & publish

- Use `maturin develop` for local development
- Use `maturin build --release` for publishing wheels

# TODO: Optimize workflow

- Keep binding crate minimal and thin
- Maintain strict separation between Rust core and Python layer
- Use workspace for simplified dependency management
- Maintain fast compile times by isolating PyO3 to bindings crate
