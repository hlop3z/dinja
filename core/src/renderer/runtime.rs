//! Runtime management for JavaScript execution
//!
//! This module handles JsRuntime lifecycle, cleanup, and value extraction.

use crate::error::MdxError;
use anyhow::Result as AnyhowResult;
use deno_core::JsRuntime;
use std::cell::RefCell;
use std::rc::Rc;

use super::constants::script_tags;

/// Estimated overhead for context setup script (wrapper code, try-catch, etc.)
/// This is used to pre-allocate string capacity for better performance.
const CONTEXT_SETUP_OVERHEAD: usize = 150;

/// Safe guard that ensures runtime cleanup happens even on panic.
///
/// ## Why Rc<RefCell<JsRuntime>>?
///
/// The `RuntimeCleanupGuard` holds an `Rc<RefCell<JsRuntime>>` (reference-counted, interior-mutable)
/// rather than owning the runtime directly. This design is necessary because:
///
/// 1. **Shared Ownership**: The runtime is shared between the guard and the `with_runtime` function.
///    The guard needs access to the runtime for cleanup, but `with_runtime` also needs mutable access
///    to execute JavaScript code.
///
/// 2. **Rc Clone Requirement**: We clone the `Rc` (not the runtime itself) to create a new reference
///    to the same runtime. This is cheap (just increments a reference count) and allows the guard
///    to outlive the original borrow. When `with_runtime` returns and releases its borrow, the guard
///    still has a reference and can perform cleanup.
///
/// 3. **Drop Order Independence**: Without the `Rc` clone, if the guard were to take ownership of
///    the runtime directly, we'd have a borrow checker issue: `with_runtime` borrows the runtime
///    mutably, but the guard (created inside `with_runtime`) would need to own it. The `Rc` clone
///    solves this by allowing shared ownership without violating borrow rules.
///
/// 4. **Panic Safety**: If `with_runtime` panics, the guard's `Drop` implementation will still run
///    and attempt cleanup, preventing resource leaks. The `Rc` ensures the runtime isn't dropped
///    until all references (including the guard) are dropped.
///
/// ## Cleanup Strategy
///
/// The guard uses `try_borrow_mut()` instead of `borrow_mut()` to avoid panicking if the runtime
/// is already borrowed. This makes cleanup best-effort: if cleanup can't happen immediately,
/// it will happen on the next successful borrow or when the runtime is dropped.
pub(super) struct RuntimeCleanupGuard {
    runtime: Rc<RefCell<JsRuntime>>,
}

impl RuntimeCleanupGuard {
    pub(super) fn new(runtime: Rc<RefCell<JsRuntime>>) -> Self {
        Self { runtime }
    }
}

impl Drop for RuntimeCleanupGuard {
    fn drop(&mut self) {
        // Attempt to borrow mutably for cleanup. If we can't (e.g., already borrowed),
        // we skip cleanup rather than panicking. This is safe because cleanup is
        // best-effort and the runtime will be cleaned up on the next successful borrow.
        if let Ok(mut runtime) = self.runtime.try_borrow_mut() {
            if let Err(err) = cleanup_runtime(&mut runtime) {
                eprintln!("Renderer runtime cleanup failed: {err:?}");
            }
        }
    }
}

/// Executes a function with mutable access to the runtime, ensuring cleanup happens
///
/// Performance optimization: Cleanup is deferred to the guard's drop implementation
/// to avoid double cleanup. The guard ensures cleanup happens even on panic.
///
/// # Errors
/// Returns an error if the runtime cannot be borrowed mutably. This can happen if:
/// - Another operation is currently using the runtime (concurrent access within the same thread)
/// - Runtime cleanup is in progress
/// - The runtime is being accessed from multiple places simultaneously
pub(super) fn with_runtime<R>(
    runtime: Rc<RefCell<JsRuntime>>,
    f: impl FnOnce(&mut JsRuntime) -> AnyhowResult<R>,
) -> AnyhowResult<R> {
    // Use try_borrow_mut for graceful error handling instead of panicking
    // This can fail if the runtime is already borrowed (e.g., concurrent access,
    // cleanup in progress, or recursive calls)
    let mut rt = runtime.try_borrow_mut().map_err(|e| {
        anyhow::anyhow!(
            "Failed to borrow runtime mutably: {e}. This may indicate concurrent access, \
             cleanup in progress, or recursive runtime operations within the same thread."
        )
    })?;

    // Create guard that will cleanup on drop (even if f panics)
    // This avoids double cleanup - we rely on the guard for cleanup
    let _cleanup = RuntimeCleanupGuard::new(Rc::clone(&runtime));

    // Execute the function
    f(&mut rt)
}

/// Cleans up the JavaScript runtime by removing registered components and globals
pub(super) fn cleanup_runtime(runtime: &mut JsRuntime) -> Result<(), MdxError> {
    const CLEANUP_SCRIPT: &str = r#"
        try {
            if (Array.isArray(globalThis.__registered_component_names)) {
                for (const name of globalThis.__registered_component_names) {
                    if (typeof name === 'string') {
                        delete globalThis[name];
                    }
                }
                globalThis.__registered_component_names.length = 0;
            }

            delete globalThis.Component;

            if (typeof module !== 'undefined' && module && module.exports) {
                module.exports = {};
            }

            if (typeof globalThis.exports !== 'undefined' && globalThis.exports) {
                globalThis.exports = {};
            }

            delete globalThis.context;
        } catch (cleanupError) {
            console.warn('Renderer cleanup failed', cleanupError);
        }
    "#;

    runtime
        .execute_script(script_tags::CLEANUP_RUNTIME, CLEANUP_SCRIPT)
        .map_err(|e| MdxError::TsxTransform(format!("Failed to cleanup runtime: {e:?}")))?;

    Ok(())
}

/// Sets up the global context variable in the JavaScript runtime as a function
///
/// The context is a function that accepts a dotted path string and uses a reducer
/// pattern to access nested values from the underlying metadata object.
///
/// Performance optimization: Uses `write!` macro for better performance in hot path
pub(super) fn setup_context(runtime: &mut JsRuntime, props_json: &str) -> Result<(), MdxError> {
    // Pre-allocate with estimated capacity (function wrapper adds more overhead)
    use std::fmt::Write;
    let mut setup_context = String::with_capacity(props_json.len() + CONTEXT_SETUP_OVERHEAD + 200);
    write!(
        setup_context,
        r#"
        try {{
            const contextData = {props_json};
            globalThis.context = (function(options) {{
                return function(key) {{
                    return key.split(".").reduce((o, i) => {{
                        if (o) return o[i];
                        return undefined;
                    }}, options);
                }};
            }})(contextData);
        }} catch (e) {{
            console.warn('Failed to parse metadata, using empty object for context');
            globalThis.context = (function(options) {{
                return function(key) {{
                    return key.split(".").reduce((o, i) => {{
                        if (o) return o[i];
                        return undefined;
                    }}, options);
                }};
            }})({{}});
        }}
        "#,
        props_json = props_json
    )
    .map_err(|e| MdxError::TsxTransform(format!("Failed to build context setup script: {e}")))?;

    runtime
        .execute_script(script_tags::SETUP_CONTEXT, setup_context)
        .map_err(|e| MdxError::TsxTransform(format!("Failed to setup context: {e:?}")))?;
    Ok(())
}

/// Extracts a string value from a V8 result handle
pub(super) fn extract_string_from_v8(
    result: deno_core::v8::Global<deno_core::v8::Value>,
    runtime: &mut JsRuntime,
    error_msg: &str,
) -> Result<String, MdxError> {
    let scope = &mut runtime.handle_scope();
    let local = deno_core::v8::Local::new(scope, result);

    if local.is_string() {
        local
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .ok_or_else(|| MdxError::TsxTransform(error_msg.to_string()))
    } else {
        Err(MdxError::TsxTransform(format!(
            "{error_msg}: result is not a string"
        )))
    }
}
