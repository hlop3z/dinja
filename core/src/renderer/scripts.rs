//! Script generation for component rendering
//!
//! This module handles the generation of JavaScript code for rendering components,
//! component registration, and script wrappers.

use crate::error::MdxError;
use crate::models::ComponentDefinition;
use std::collections::HashMap;

/// Generates JavaScript code to resolve a component from various export patterns
///
/// # Arguments
/// * `var_name` - Name of the variable to assign the resolved component to
/// * `throw_on_not_found` - If true, throws an error if component is not found
///
/// # Returns
/// JavaScript code that resolves View, Component (for backwards compatibility), module.exports.default, or module.exports
/// Generates JavaScript code to resolve the main View component for rendering
/// Throws an error if component is not found.
pub(super) fn component_resolution_code() -> String {
    format!(
        r#"
            let ComponentToRender = typeof View !== 'undefined' ? View : (typeof Component !== 'undefined' ? Component : null);
            if (!ComponentToRender && module && module.exports) {{
                ComponentToRender = module.exports.default || module.exports;
            }}
            if (!ComponentToRender && exports) {{
                ComponentToRender = exports.default || exports;
            }}
            if (!ComponentToRender) {{
                throw new Error('Component not found. Expected View, Component or default export.');
            }}
    "#
    )
}

/// Helper function to resolve a component being registered (NOT the main View)
/// Returns JavaScript code that resolves Component, module.exports.default, or module.exports
/// Does NOT look for View - View is the MDX content being rendered, not a component to register
/// Does not throw an error if component is not found (caller should check).
pub(super) fn resolve_component_code() -> String {
    format!(
        r#"
            let resolved = typeof Component !== 'undefined' ? Component : null;
            if (!resolved && module && module.exports) {{
                resolved = module.exports.default || module.exports;
            }}
            if (!resolved && exports) {{
                resolved = exports.default || exports;
            }}
    "#
    )
}

/// Builds a render script wrapper with common component resolution logic
///
/// ## Performance Optimizations
///
/// This function is a hot path called for every component render. Key optimizations:
/// - **Pre-allocation**: Estimates total capacity and pre-allocates to avoid multiple reallocations
/// - **`write!` macro**: More efficient than `format!` for building strings incrementally
/// - **Capacity estimation**: Adds ~200 bytes overhead for wrapper code (function declaration, etc.)
///
/// ## String Allocation Strategy
///
/// We pre-allocate with `String::with_capacity()` based on the sum of input lengths plus overhead.
/// This strategy works well because:
/// - Input sizes are known at call time
/// - The final string size is predictable (inputs + fixed wrapper)
/// - Avoids multiple reallocations during string building
/// - Reduces memory fragmentation
pub(super) fn build_render_script_wrapper(
    component_bootstrap: &str,
    component_code: &str,
    props_json: &str,
    render_body: &str,
) -> Result<String, MdxError> {
    // Pre-allocate with estimated capacity for better performance
    // Strategy: Sum all input lengths + fixed overhead to avoid reallocations
    let estimated_capacity = component_bootstrap.len()
        + component_code.len()
        + props_json.len()
        + render_body.len()
        + 200; // Base script overhead (function wrapper, etc.)
    let mut script = String::with_capacity(estimated_capacity);

    // Use write! macro for better performance in hot path
    use std::fmt::Write;
    let component_resolution = component_resolution_code();
    write!(
        script,
        r#"
        (function() {{
            {component_bootstrap}

            // Execute the component code
            {component_code}
            
            {component_resolution}
            
            // Context originates from trusted serde_json serialization
            // Create context function using reducer pattern for dotted path access
            const contextData = {props_json};
            const context = (function(options) {{
                return function(key) {{
                    return key.split(".").reduce((o, i) => {{
                        if (o) return o[i];
                        return undefined;
                    }}, options);
                }};
            }})(contextData);
            
            {render_body}
        }})()
        "#,
        component_bootstrap = component_bootstrap,
        component_code = component_code,
        component_resolution = component_resolution,
        props_json = props_json,
        render_body = render_body
    )
    .map_err(|e| MdxError::TsxTransform(format!("Failed to write script wrapper: {e}")))?;

    Ok(script)
}

/// Generates a render script for standard engine components
pub(super) fn component_render_script(
    component_code: &str,
    props_json: &str,
) -> Result<String, MdxError> {
    const RENDER_BODY: &str = r#"
            // Render using engine-render-to-string
            if (typeof engine_to_string !== 'undefined' && engine_to_string) {
                return engine_to_string(ComponentToRender(context));
            } else if (typeof engine_to_string !== 'undefined' && engine_to_string.renderToString) {
                return engine_to_string.renderToString(ComponentToRender(context));
            } else {
                throw new Error('engine_to_string not available');
            }
    "#;

    build_render_script_wrapper("", component_code, props_json, RENDER_BODY)
}

/// Generates a render script for schema output using core.js engine
pub(super) fn schema_render_script(
    component_code: &str,
    props_json: &str,
) -> Result<String, MdxError> {
    const RENDER_BODY: &str = r#"
            // Render using core.js engine.render() which returns JSON string
            // Use coreEngine (saved from core.js) for schema rendering
            if (typeof coreEngine !== 'undefined' && coreEngine && typeof coreEngine.render === 'function') {
                return coreEngine.render(ComponentToRender, context);
            } else if (typeof engine !== 'undefined' && engine && typeof engine.render === 'function') {
                // Fallback to engine if coreEngine not available
                return engine.render(ComponentToRender, context);
            } else {
                throw new Error('core.js engine not available. Expected coreEngine or engine with render method.');
            }
    "#;

    build_render_script_wrapper("", component_code, props_json, RENDER_BODY)
}

/// Wraps transformed component code with bootstrap
pub(super) fn wrap_transformed_component(
    component_bootstrap: &str,
    transformed_js: &str,
    component_names: &[String],
) -> String {
    // Generate variable declarations for components
    let mut component_vars = String::new();
    for name in component_names {
        component_vars.push_str(&format!("        const {name} = globalThis.{name};\n"));
    }

    // Wrap component bootstrap in its own scope to prevent View from being visible during registration
    // This avoids the hoisting issue where function View() {...} would be visible to component resolution code
    let wrapped_bootstrap = if !component_bootstrap.trim().is_empty() {
        format!("(function() {{\n{}\n}})();", component_bootstrap)
    } else {
        String::new()
    };

    format!(
        r#"
        {component_bootstrap}

        // Make registered components available as variables
{component_vars}
        // Transformed component code
        {transformed_js}
        "#,
        component_bootstrap = wrapped_bootstrap,
        component_vars = component_vars,
        transformed_js = transformed_js
    )
}

/// Builds the registration script for a single component
pub(super) fn build_single_component_registration(
    registration_name: &str,
    component_js: &str,
) -> Result<String, MdxError> {
    let name_literal = serde_json::to_string(registration_name).map_err(|e| {
        MdxError::TsxTransform(format!(
            "Failed to serialize component name {registration_name}: {e}"
        ))
    })?;

    // Use write! for better performance in hot path
    use std::fmt::Write;
    let mut script = String::with_capacity(200 + component_js.len());
    let resolve_component = resolve_component_code();
    write!(
        script,
        r#"
            (function() {{
                const module = {{ exports: {{}} }};
                const exports = module.exports;
                {component_js}

                {resolve_component}

                if (!resolved) {{
                    throw new Error('Component {name_literal} did not export a value');
                }}

                globalThis[{name_literal}] = resolved;
                if (Array.isArray(globalThis.__registered_component_names)) {{
                    globalThis.__registered_component_names.push({name_literal});
                }}
            }})();
            "#,
        resolve_component = resolve_component
    )
    .map_err(|e| {
        MdxError::TsxTransform(format!(
            "Failed to build component registration script for {registration_name}: {e}"
        ))
    })?;

    Ok(script)
}

/// Builds the component registration script for multiple components
pub(super) fn build_component_registration_script(
    components: &HashMap<String, ComponentDefinition>,
) -> Result<String, MdxError> {
    if components.is_empty() {
        return Ok(String::new());
    }

    // Pre-allocate with estimated capacity: base script + ~500 chars per component
    const BASE_SCRIPT_CAPACITY: usize = 200;
    const ESTIMATED_CHARS_PER_COMPONENT: usize = 500;
    let estimated_capacity =
        BASE_SCRIPT_CAPACITY + (components.len() * ESTIMATED_CHARS_PER_COMPONENT);
    let mut script = String::with_capacity(estimated_capacity);
    script.push_str(
        r#"
        if (!Array.isArray(globalThis.__registered_component_names)) {
            globalThis.__registered_component_names = [];
        }
        "#,
    );

    for (map_key, comp_def) in components {
        let registration_name = comp_def.name.as_deref().unwrap_or(map_key.as_str());
        let component_js =
            crate::transform::transform_component_code(&comp_def.code).map_err(|e| {
                MdxError::TsxTransform(format!(
                    "Failed to transform component {registration_name} code: {e:?}"
                ))
            })?;

        let component_registration =
            build_single_component_registration(registration_name, &component_js)?;
        script.push_str(&component_registration);
    }

    Ok(script)
}

/// Generates bootstrap script for components
pub(super) fn component_bootstrap_script(
    components: Option<&HashMap<String, ComponentDefinition>>,
) -> Result<String, MdxError> {
    components
        .filter(|map| !map.is_empty())
        .map(build_component_registration_script)
        .transpose()
        .map(|maybe| maybe.unwrap_or_default())
}
