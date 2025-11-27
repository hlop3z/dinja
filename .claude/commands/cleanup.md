# 🦀 **Codebase Cleanup Guidelines — Rust 2026**

## ✅ **Ensure the Code Is**0

- **Up to Date:** Leverages modern **Rust 2024+ edition** features — pattern matching, `let-else`, const generics, Generic Associated Types (GATs), `async`/`await`, and the latest standard library improvements.
- **DRY:** Remove duplicated logic. Abstract shared functionality into well-named functions, traits, or modules. Use macros judiciously for unavoidable repetition.
- **KISS & YAGNI:** Keep solutions simple; avoid over-engineering with complex type hierarchies, unnecessary lifetimes, or speculative trait bounds.
- **Separated by Concern (SoC):** Each module, struct, trait, and function should have a single, clear responsibility. Use Rust's module system to enforce boundaries.
- **Convention over Configuration:** Align with Rust ecosystem norms — standard project structure (`src/`, `tests/`, `examples/`), idiomatic naming (snake_case for functions/variables, PascalCase for types), and Cargo conventions.
- **Optimized for Performance:** Leverage Rust's zero-cost abstractions. Minimize allocations, avoid unnecessary clones, use iterators over explicit loops, prefer stack over heap, and profile with `cargo flamegraph` or `perf`.
- **Readable & Self-Documenting:** Prefer expressive naming and clear control flow over excessive comments. Use doc comments (`///`) for public APIs. Code should read like a well-written specification.
- **Loosely Coupled, Highly Cohesive:** Reduce inter-module dependencies via traits and dependency injection. Avoid circular dependencies and excessive `pub` exposure.
- **Composable:** Write functions and traits that can be combined predictably. Embrace functional patterns with iterators, `Option`, `Result`, and combinators (`map`, `and_then`, `filter`, etc.).
- **Testable by Design:** Structure code for unit, integration, and doc tests. Isolate side effects behind traits for easy mocking. Use `#[cfg(test)]` modules effectively.
- **Refactorable & Evolvable:** Maintain clear module boundaries and stable public APIs. Use semantic versioning. The compiler is your refactoring ally—lean on it.
- **Idiomatic:** Follow **Rust API Guidelines**, embrace the **Rust Book** principles, and use `clippy` aggressively. Write "**Rustic**" code — safe, explicit, and leveraging ownership semantics.

---

## 🔍 **Look For**

### **Dead Code & Unused Items**

- Unused imports, functions, types, or feature flags. Run `cargo check` with warnings enabled.
- Unused dependencies in `Cargo.toml`. Use `cargo-machete` or `cargo-udeps` to detect orphaned crates.
- Dead code paths unreachable due to type system constraints or configuration flags.

### **Code Smells**

- **Excessive `.clone()` or `.to_owned()`:** Often indicates misunderstanding of ownership or borrowing. Redesign to use references or `Cow<'_, T>`.
- **Overly Complex Lifetimes:** Simplify with owned types, refactor to smaller scopes, or use `Arc`/`Rc` judiciously when sharing is essential.
- **Large `match` or `if-else` Chains:** Refactor using pattern matching, trait polymorphism, or lookup tables (e.g., `HashMap`, `phf`).
- **Long Functions:** Break down into smaller, composable pieces. Each function should do one thing well.
- **Unclear Variable Naming:** Use descriptive names that convey intent. Avoid single-letter variables except in very short scopes or well-known idioms (e.g., `i` for index).
- **Nested Indentation:** Use early returns, `let-else`, `?` operator, and combinators to flatten control flow.

### **Anti-Patterns**

- **Inappropriate `unwrap()` or `expect()` Usage:** Replace with proper error propagation using `?`, or handle explicitly with `match`/`if let`.
- **String Abuse:** Avoid excessive `String` allocations. Use `&str` where possible, and `Cow<'_, str>` for conditional ownership.
- **Ignoring Results:** Never ignore `Result` or `Option` without explicitly handling with `let _ = ...` or `.ok()` when intentional.
- **Manual Memory Management:** Avoid `unsafe` unless absolutely necessary and well-documented. Prefer safe abstractions.
- **Over-reliance on Mutability:** Default to immutability. Use `&mut` only when state change is essential.
- **Tight Coupling via Concrete Types:** Depend on traits (`impl Trait`, `dyn Trait`) rather than concrete implementations.

### **Inconsistent Style**

- Violations of `rustfmt` and `clippy` standards. Enforce via CI/CD (`cargo fmt --check`, `cargo clippy -- -D warnings`).
- Inconsistent error types across modules. Consider a unified error type using `thiserror` or `anyhow`.
- Mixed async runtimes (`tokio`, `async-std`, `smol`). Standardize on one unless there's a compelling reason.

### **Inefficient Patterns**

- **Iterators Not Consumed Efficiently:** Use `.collect()`, `.fold()`, or `.for_each()` appropriately. Avoid intermediate allocations.
- **Blocking I/O in Async Contexts:** Use `tokio::fs`, `async-std::fs`, or `spawn_blocking` for CPU-bound work.
- **Repeated Allocations in Loops:** Pre-allocate with `Vec::with_capacity()`, reuse buffers, or use `SmallVec`.
- **Inefficient Data Structures:** Choose the right container (`Vec`, `VecDeque`, `BTreeMap`, `HashMap`, `HashSet`, `BinaryHeap`).
- **Unnecessary Boxing:** Avoid `Box<T>` unless dynamic dispatch (`Box<dyn Trait>`) or recursive types are required.

### **Outdated Dependencies**

- Audit with `cargo outdated` or `cargo audit` for CVEs, deprecations, or unmaintained crates.
- Replace deprecated crates with actively maintained alternatives (e.g., `failure` → `thiserror`/`anyhow`).

### **Unclear Error Handling**

- Vague error messages without context. Use `thiserror` for library errors and `anyhow` with `.context()` for application errors.
- Overuse of `panic!` or `unreachable!()` without justification. Reserve for truly impossible states.
- Inadequate logging. Use structured logging with `tracing` or `log` + `env_logger`.

---

## ⚙️ **Performance Often Depends More On**

### **Algorithmic Efficiency**

- Choose the right algorithm: O(n²) → O(n log n) → O(n).
- Leverage Rust's powerful iterator combinators for lazy, composable transformations.

### **Data Structures**

- Use the right container for the job:
  - **Sequential Access:** `Vec`, `VecDeque`, `LinkedList` (rarely).
  - **Key-Value Lookup:** `HashMap`, `BTreeMap`.
  - **Set Operations:** `HashSet`, `BTreeSet`.
  - **Priority Queue:** `BinaryHeap`.
  - **Small Collections:** `SmallVec`, `SmallString`, `ArrayVec` (from `smallvec` crate).

### **Memory Management**

- **Minimize Allocations:** Pre-allocate with `with_capacity()`, reuse buffers, use stack allocation where possible.
- **Avoid Unnecessary Clones:** Use references (`&T`, `&mut T`) or `Cow<'_, T>` for conditional ownership.
- **Smart Pointers:** Use `Rc`/`Arc` for shared ownership, but prefer unique ownership when feasible.
- **Memory Layout:** Be mindful of struct padding and alignment. Use `#[repr(C)]` or `#[repr(packed)]` when needed.

### **Zero-Cost Abstractions**

- **Iterators Over Loops:** Prefer `.iter()`, `.map()`, `.filter()`, etc. The compiler optimizes these aggressively.
- **Generics Over Trait Objects:** Monomorphization (`impl Trait`, `<T: Trait>`) generates specialized code. Use `dyn Trait` only when dynamic dispatch is required.
- **Inline Small Functions:** Use `#[inline]` or `#[inline(always)]` for hot paths, but profile first—LLVM is usually smart enough.

### **Concurrency & Parallelism**

- **Choose the Right Model:**
  - **Async I/O:** `tokio`, `async-std` for network/file operations.
  - **CPU-Bound Parallelism:** `rayon` for data parallelism.
  - **Shared State:** Prefer message passing (`mpsc`, `crossbeam`) over `Mutex`/`RwLock` when possible.
- **Minimize Lock Contention:** Use fine-grained locking, lock-free structures (`crossbeam`, `dashmap`), or actor models.
- **Avoid Blocking the Runtime:** Use `spawn_blocking` or dedicated thread pools for CPU-intensive work in async contexts.

### **I/O Optimization**

- **Buffered I/O:** Use `BufReader`, `BufWriter` for file operations.
- **Batching:** Reduce syscalls by batching reads/writes.
- **Async I/O:** Use async runtimes (`tokio`, `async-std`) for high-concurrency network operations.
- **Memory-Mapped Files:** Consider `memmap2` for large file processing.

### **Compiler Optimizations**

- **Release Builds:** Always profile with `--release`. Debug builds are intentionally slow.
- **Profile-Guided Optimization (PGO):** Use `cargo-pgo` for production builds.
- **Link-Time Optimization (LTO):** Enable `lto = "fat"` in `Cargo.toml` for final builds (trades compile time for performance).
- **Codegen Units:** Set `codegen-units = 1` for maximum optimization (slower compile, faster runtime).

### **Profiling & Benchmarking**

- **Always Measure:** Use `cargo bench` with `criterion` for micro-benchmarks.
- **Flame Graphs:** Use `cargo flamegraph` to identify hotspots.
- **System Profilers:** Leverage `perf`, `valgrind`, or `instruments` (macOS).
- **Avoid Premature Optimization:** Profile first, optimize second. Trust the compiler, but verify.

---

## 🛠️ **Recommended Tools**

- **`cargo fmt`** — Enforce consistent formatting.
- **`cargo clippy`** — Catch common mistakes and anti-patterns.
- **`cargo audit`** — Check for known security vulnerabilities.
- **`cargo outdated`** — Identify outdated dependencies.
- **`cargo-udeps`** / **`cargo-machete`** — Detect unused dependencies.
- **`cargo expand`** — Inspect macro expansions.
- **`cargo asm` / `cargo llvm-ir`** — Examine generated assembly or LLVM IR.
- **`cargo flamegraph`** — Generate flame graphs for profiling.
- **`cargo criterion`** — Advanced benchmarking framework.
- **`miri`** — Detect undefined behavior in unsafe code.
- **`cargo-deny`** — Enforce licensing, dependency policies, and security standards.

---

## 📚 **References**

- [The Rust Programming Language Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)
- [Clippy Lint List](https://rust-lang.github.io/rust-clippy/stable/index.html)

---

## 🎯 **Quick Checklist**

- All `clippy` warnings resolved (`cargo clippy -- -D warnings`)
- Code formatted with `rustfmt` (`cargo fmt --check`)
- No unused dependencies (`cargo-udeps` or `cargo-machete`)
- Security audit passed (`cargo audit`)
- Tests pass (`cargo test`)
- Benchmarks demonstrate acceptable performance (`cargo bench`)
- Documentation is complete and accurate (`cargo doc --open`)
- Error handling is explicit and idiomatic (`Result`, `Option`, proper error types)
- No unnecessary `clone()`, `unwrap()`, or allocations in hot paths
- Public APIs are stable and well-documented

---

**Remember:** Rust's compiler is one of the best refactoring tools available. Use it to guide your cleanup efforts. If the code compiles and passes `clippy`, you're already 80% of the way to production-quality Rust. 🦀
