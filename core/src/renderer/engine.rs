//! Engine loading and initialization
//!
//! This module handles loading JavaScript libraries, setting up globals,
//! and initializing custom engines.

use crate::error::MdxError;
use anyhow::{Context, Result as AnyhowResult};
use deno_core::JsRuntime;
use std::fs;
use std::path::Path;

use super::constants::{script_tags, static_files};

/// Sets up global JavaScript objects needed by the libraries
pub(super) fn setup_globals(runtime: &mut JsRuntime) -> Result<(), MdxError> {
    const SETUP_GLOBALS: &str = r#"
        // Create global object if it doesn't exist
        if (typeof global === 'undefined') {
            globalThis.global = globalThis;
        }
        if (typeof window === 'undefined') {
            globalThis.window = globalThis;
        }
        if (typeof self === 'undefined') {
            globalThis.self = globalThis;
        }
        
        // Add minimal timer functions for engine
        if (typeof setTimeout === 'undefined') {
            globalThis.setTimeout = function(fn, delay) {
                // For SSR, execute immediately (we don't need real timers)
                if (delay === 0 || delay === undefined) {
                    fn();
                }
                return 0;
            };
        }
        if (typeof clearTimeout === 'undefined') {
            globalThis.clearTimeout = function() {};
        }
        if (typeof setInterval === 'undefined') {
            globalThis.setInterval = function(fn, delay) {
                return 0;
            };
        }
        if (typeof clearInterval === 'undefined') {
            globalThis.clearInterval = function() {};
        }
        if (typeof requestAnimationFrame === 'undefined') {
            globalThis.requestAnimationFrame = function(fn) {
                return 0;
            };
        }
        if (typeof cancelAnimationFrame === 'undefined') {
            globalThis.cancelAnimationFrame = function() {};
        }
        
        // Create a minimal document mock for server-side rendering
        // The actual DOM operations will be handled by engine-render-to-string
        if (typeof document === 'undefined') {
            globalThis.document = {
                createElementNS: function(ns, tag) { 
                    var el = { 
                        localName: tag,
                        nodeType: 1,
                        setAttribute: function() {},
                        removeAttribute: function() {},
                        appendChild: function() {},
                        insertBefore: function() {},
                        removeChild: function() {},
                        childNodes: [],
                        attributes: [],
                        style: {}
                    };
                    return el;
                },
                createTextNode: function(text) { 
                    return { 
                        nodeType: 3,
                        data: text || '',
                        parentNode: null
                    }; 
                },
                documentElement: {
                    appendChild: function() {},
                    insertBefore: function() {},
                    childNodes: [],
                    firstChild: null
                },
                createElement: function(tag) {
                    return document.createElementNS(null, tag);
                }
            };
        }

        // Helper to extract props with a given prefix (e.g. $attrs(props, "x-"))
        if (typeof globalThis.$attrs === 'undefined') {
            globalThis.$attrs = function(props, prefix) {
                if (!props || typeof props !== 'object') {
                    return {};
                }
                if (typeof prefix !== 'string' || !prefix.length) {
                    prefix = 'x-';
                }
                return Object.fromEntries(
                    Object.entries(props).filter(function(entry) {
                        return typeof entry[0] === 'string' && entry[0].startsWith(prefix);
                    })
                );
            };
        }

        // Object spread helper for ES5 compatibility (used by Oxc transformer)
        if (typeof globalThis._objectSpread === 'undefined') {
            globalThis._objectSpread = function(target) {
                for (var i = 1; i < arguments.length; i++) {
                    var source = arguments[i];
                    if (source != null) {
                        for (var key in source) {
                            if (Object.prototype.hasOwnProperty.call(source, key)) {
                                target[key] = source[key];
                            }
                        }
                    }
                }
                return target;
            };
        }
    "#;

    runtime
        .execute_script(script_tags::SETUP, SETUP_GLOBALS)
        .map_err(|e| MdxError::TsxTransform(format!("Failed to setup globals: {e:?}")))?;
    Ok(())
}

/// Wraps JavaScript code in a try-catch block for better error reporting
pub(super) fn wrap_js_code(code: &str, file_name: &str) -> String {
    format!(
        r#"
        try {{
            {code}
        }} catch (e) {{
            var errorMsg = 'JavaScript Error in {file_name}: ';
            if (e.message) errorMsg += e.message;
            else errorMsg += String(e);
            if (e.stack) errorMsg += '\nStack: ' + e.stack;
            throw new Error(errorMsg);
        }}
        "#,
        code = code,
        file_name = file_name
    )
}

/// Loads a JavaScript file and executes it in the runtime
pub(super) fn load_js_file(
    runtime: &mut JsRuntime,
    static_path: &Path,
    file_name: &str,
    script_tag: &'static str,
) -> AnyhowResult<()> {
    let file_path = static_path.join(file_name);
    let code = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    let wrapped_code = wrap_js_code(&code, file_name);

    runtime
        .execute_script(script_tag, wrapped_code)
        .map_err(|e| {
            let error_details = format!("{e:?}");
            MdxError::TsxTransform(format!(
                "Failed to load {}: {}. \
                This might be due to missing browser APIs or incompatible JavaScript code.",
                file_name, error_details
            ))
        })?;

    Ok(())
}

/// Verifies that a JavaScript variable is available in the global scope
pub(super) fn verify_global_var(
    runtime: &mut JsRuntime,
    var_name: &str,
    check_script_tag: &'static str,
    verify_script: &'static str,
) -> Result<(), MdxError> {
    runtime
        .execute_script(check_script_tag, verify_script)
        .map_err(|e| MdxError::TsxTransform(format!("Failed to verify {}: {e:?}", var_name)))?;
    Ok(())
}

/// Loads engine library and verifies it's available
pub(super) fn load_engine_library(
    runtime: &mut JsRuntime,
    static_path: &Path,
) -> Result<(), MdxError> {
    load_js_file(
        runtime,
        static_path,
        static_files::ENGINE_MIN_JS,
        script_tags::ENGINE,
    )
    .map_err(|e| MdxError::TsxTransform(format!("Failed to load engine: {e:?}")))?;

    verify_global_var(
        runtime,
        "engine",
        script_tags::CHECK_ENGINE,
        r#"
        if (typeof engine === 'undefined') {
            throw new Error('engine.min.js loaded but engine variable is not available in global scope');
        }
        "#,
    )?;

    // Wrap h function to convert component function references to string names
    const WRAP_H_FUNCTION: &str = r#"
        // Helper function to find component name from function reference
        function findComponentName(componentFunc) {
            if (typeof componentFunc !== 'function') {
                return null;
            }
            
            // Check all properties in globalThis to find which one points to this function
            for (let key in globalThis) {
                try {
                    if (globalThis[key] === componentFunc) {
                        return key;
                    }
                } catch (e) {
                    // Skip properties that can't be accessed
                    continue;
                }
            }
            
            // Fallback: try to get function name if available
            if (componentFunc.name) {
                return componentFunc.name;
            }
            
            return null;
        }
        
        // Wrap engine.h to convert function references to string names
        if (typeof engine !== 'undefined' && engine.h) {
            const originalH = engine.h;
            engine.h = function(tag, props, ...children) {
                // If tag is a function (component), convert it to a string name
                if (typeof tag === 'function') {
                    const componentName = findComponentName(tag);
                    if (componentName) {
                        tag = componentName;
                    } else {
                        // Fallback: use 'Component' if we can't find the name
                        tag = 'Component';
                    }
                }
                // Handle Fragment/null/undefined
                else if (typeof engine !== 'undefined' && (tag === engine.Fragment || tag === null || tag === undefined)) {
                    tag = 'Fragment';
                }
                return originalH(tag, props || {}, ...children);
            };
        }
        
        // Also wrap global h function if it exists
        if (typeof h !== 'undefined' && typeof h === 'function') {
            const originalGlobalH = h;
            globalThis.h = function(tag, props, ...children) {
                // If tag is a function (component), convert it to a string name
                if (typeof tag === 'function') {
                    const componentName = findComponentName(tag);
                    if (componentName) {
                        tag = componentName;
                    } else {
                        // Fallback: use 'Component' if we can't find the name
                        tag = 'Component';
                    }
                }
                // Handle Fragment/null/undefined
                else if (typeof engine !== 'undefined' && (tag === engine.Fragment || tag === null || tag === undefined)) {
                    tag = 'Fragment';
                }
                return originalGlobalH(tag, props || {}, ...children);
            };
        }
    "#;

    runtime
        .execute_script(script_tags::WRAP_H_FUNCTION, WRAP_H_FUNCTION)
        .map_err(|e| MdxError::TsxTransform(format!("Failed to wrap h function: {e:?}")))?;

    Ok(())
}

/// Loads engine render-to-string library and verifies it's available
pub(super) fn load_engine_render_library(
    runtime: &mut JsRuntime,
    static_path: &Path,
) -> Result<(), MdxError> {
    load_js_file(
        runtime,
        static_path,
        static_files::ENGINE_TO_STRING_MIN_JS,
        script_tags::ENGINE_TO_STRING,
    )
    .map_err(|e| MdxError::TsxTransform(format!("Failed to load engine_to_string: {e:?}")))?;

    verify_global_var(
        runtime,
        "engine_to_string",
        script_tags::CHECK_ENGINE_TO_STRING,
        r#"
        if (typeof engine_to_string === 'undefined') {
            throw new Error('engine_to_string.min.js loaded but engine_to_string variable is not available in global scope');
        }
        "#,
    )?;
    Ok(())
}

/// Loads core.js engine library and verifies it's available
pub(super) fn load_core_engine_library(
    runtime: &mut JsRuntime,
    static_path: &Path,
) -> Result<(), MdxError> {
    load_js_file(
        runtime,
        static_path,
        static_files::CORE_JS,
        script_tags::CORE_ENGINE,
    )
    .map_err(|e| MdxError::TsxTransform(format!("Failed to load core.js: {e:?}")))?;

    verify_global_var(
        runtime,
        "engine",
        script_tags::CHECK_CORE_ENGINE,
        r#"
        if (typeof engine === 'undefined' || !engine || typeof engine.render !== 'function') {
            throw new Error('core.js loaded but engine variable is not available or engine.render is not a function');
        }
        "#,
    )?;
    Ok(())
}

/// Loads static JavaScript files from the static directory into the engine context
///
/// # Arguments
/// * `runtime` - Mutable reference to the JsRuntime
/// * `static_dir` - Path to the directory containing static JavaScript files
/// * `engine_code` - Optional custom engine code to inject
pub(super) fn load_static_files_internal(
    runtime: &mut JsRuntime,
    static_dir: impl AsRef<Path>,
) -> AnyhowResult<()> {
    let static_path = static_dir.as_ref();

    // Set up global objects that might be needed by the JS libraries
    setup_globals(runtime)?;

    // Load engine libraries (preact, used for HTML/JavaScript rendering)
    load_engine_library(runtime, static_path).map_err(anyhow::Error::from)?;
    load_engine_render_library(runtime, static_path).map_err(anyhow::Error::from)?;

    // Save preact engine reference before loading core.js (which overwrites engine)
    const SAVE_PREACT_ENGINE: &str = r#"
        if (typeof engine !== 'undefined') {
            globalThis.__preactEngine = engine;
            globalThis.__preactEngine_to_string = engine_to_string;
        }
    "#;
    runtime
        .execute_script(script_tags::SETUP, SAVE_PREACT_ENGINE)
        .map_err(|e| {
            anyhow::Error::from(MdxError::TsxTransform(format!(
                "Failed to save preact engine reference: {e:?}"
            )))
        })?;

    // Load core.js engine (used for schema rendering, overwrites engine variable)
    load_core_engine_library(runtime, static_path).map_err(anyhow::Error::from)?;

    // Save core engine and restore preact engine
    const SETUP_DUAL_ENGINES: &str = r#"
        // Save core.js engine as coreEngine for schema rendering
        if (typeof engine !== 'undefined' && engine.render) {
            globalThis.coreEngine = engine;
        }
        // Restore preact engine for HTML/JavaScript rendering
        if (typeof globalThis.__preactEngine !== 'undefined') {
            globalThis.engine = globalThis.__preactEngine;
            if (typeof globalThis.__preactEngine_to_string !== 'undefined') {
                globalThis.engine_to_string = globalThis.__preactEngine_to_string;
            }
        }
    "#;
    runtime
        .execute_script(script_tags::SETUP, SETUP_DUAL_ENGINES)
        .map_err(|e| {
            anyhow::Error::from(MdxError::TsxTransform(format!(
                "Failed to setup dual engines: {e:?}"
            )))
        })?;

    Ok(())
}
