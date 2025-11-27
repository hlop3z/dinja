//! TSX to JavaScript transformation logic
//!
//! This module handles the transformation of TSX/JSX syntax to JavaScript using the Oxc compiler.
//! It supports various JSX pragmas (engine, React-compatible) and handles component wrapping.
//!
//! ## Transformation Process
//!
//! 1. Optionally wrap raw TSX in a component function
//! 2. Parse TSX into an AST using Oxc parser
//! 3. Build semantic information for better transformation
//! 4. Apply JSX transformations (convert to function calls)
//! 5. Generate JavaScript code with optional minification
//! 6. Clean up generated code (remove pure annotations)
//!
//! ## Error Handling
//!
//! All transformation errors use `MdxError` for domain-specific error reporting.

use crate::error::MdxError;
use crate::models::TsxTransformConfig;
use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{JsxRuntime, TransformOptions, Transformer};
use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::path::Path;

/// Base overhead for component wrapper (function declaration, JSX wrapper, etc.)
const COMPONENT_WRAPPER_OVERHEAD: usize = 100;

/// Wraps TSX content in a React component structure
///
/// This function wraps raw TSX/HTML content in a function component that returns
/// the content wrapped in a Fragment. This is necessary for proper JSX transformation.
///
/// # Arguments
/// * `tsx_content` - Raw TSX/HTML content to wrap
///
/// # Returns
/// A properly formatted React component string
pub fn wrap_in_component(tsx_content: &str) -> String {
    // Pre-allocate with estimated capacity to reduce reallocations
    let estimated_capacity = tsx_content.len() + COMPONENT_WRAPPER_OVERHEAD;
    let mut result = String::with_capacity(estimated_capacity);

    result.push_str("function View(context = {}) {\n  return (\n    <>\n");

    // Use iterator combinators for more idiomatic Rust
    for line in tsx_content.lines() {
        result.push_str("      ");
        result.push_str(line);
        result.push('\n');
    }

    result.push_str("    </>\n  );\n}");
    result
}

/// Creates transformation options for JSX processing
///
/// # Arguments
/// * `config` - Configuration for JSX transformation
///
/// # Returns
/// Configured `TransformOptions` for Oxc transformer
pub fn create_transform_options(config: &TsxTransformConfig) -> TransformOptions {
    // Start with ES5 target preset
    let mut options =
        TransformOptions::from_target("es5").unwrap_or_else(|_| TransformOptions::enable_all());
    // Clone is necessary here as TransformOptions requires owned String values
    options.jsx.pragma = Some(config.jsx_pragma.clone());
    options.jsx.pragma_frag = Some(config.jsx_pragma_frag.clone());
    options.jsx.runtime = JsxRuntime::Classic;
    options.jsx.development = false;
    options.jsx.refresh = None;
    options
}

/// Transforms TSX content to JavaScript using the Oxc compiler
///
/// This function performs the following steps:
/// 1. Wraps the TSX content in a component structure
/// 2. Parses the TSX into an AST
/// 3. Builds semantic information for the code
/// 4. Applies JSX transformations (e.g., converting to engine's `h` function)
/// 5. Generates JavaScript code from the transformed AST
///
/// # Arguments
/// * `tsx_content` - TSX source code to transform
/// * `config` - Optional transformation configuration (defaults to standard config)
///
/// # Returns
/// Generated JavaScript code or an error
pub fn transform_tsx_to_js_with_config(
    tsx_content: &str,
    config: TsxTransformConfig,
) -> Result<String, MdxError> {
    transform_tsx_internal(tsx_content, &config, true)
}

/// Transforms TSX content to JavaScript using the default configuration
///
/// # Arguments
/// * `tsx_content` - TSX source code to transform
///
/// # Returns
/// Generated JavaScript code or an error
pub fn transform_tsx_to_js(tsx_content: &str) -> Result<String, MdxError> {
    transform_tsx_to_js_with_config(tsx_content, TsxTransformConfig::default())
}

/// Transforms TSX content to JavaScript for final output (uses `h` instead of `engine.h`)
///
/// # Arguments
/// * `tsx_content` - TSX source code to transform
///
/// # Returns
/// Generated JavaScript code or an error
pub fn transform_tsx_to_js_for_output(tsx_content: &str, minify: bool) -> Result<String, MdxError> {
    transform_tsx_to_js_with_config(tsx_content, TsxTransformConfig::for_output(minify))
}

/// Estimated characters per error message including separators
const ESTIMATED_CHARS_PER_ERROR: usize = 60;

/// Formats a collection of errors into a single error message
fn format_errors(errors: &[impl std::fmt::Debug]) -> String {
    if errors.is_empty() {
        return String::new();
    }

    // Pre-allocate with estimated capacity to reduce reallocations
    let estimated_capacity = errors.len() * ESTIMATED_CHARS_PER_ERROR;
    errors.iter().map(|e| format!("{e:?}")).fold(
        String::with_capacity(estimated_capacity),
        |mut acc, e| {
            if !acc.is_empty() {
                acc.push_str(", ");
            }
            acc.push_str(&e);
            acc
        },
    )
}

/// Validates and parses TSX content, returning an error if parsing fails
fn validate_parse_result(parser_return: &oxc_parser::ParserReturn) -> Result<(), MdxError> {
    if !parser_return.errors.is_empty() {
        return Err(MdxError::TsxParse(format_errors(&parser_return.errors)));
    }
    Ok(())
}

/// Validates JSX transformation result, returning an error if transformation fails
fn validate_transform_result(
    transform_return: &oxc_transformer::TransformerReturn,
) -> Result<(), MdxError> {
    if !transform_return.errors.is_empty() {
        return Err(MdxError::TsxTransform(format_errors(
            &transform_return.errors,
        )));
    }
    Ok(())
}

/// Transform that converts component function references to string names in AST
///
/// This uses a simple post-processing approach on the generated code since
/// AST traversal with Oxc requires more complex setup. The string replacement
/// is safe because we only replace known component names in specific patterns.
fn convert_component_refs_in_ast(code: &str, component_names: &HashSet<&str>) -> String {
    if component_names.is_empty() {
        return code.to_string();
    }

    let mut result = code.to_string();

    // Sort by length (longest first) to avoid partial matches
    let mut sorted_names: Vec<&str> = component_names.iter().copied().collect();
    sorted_names.sort_by_key(|name| Reverse(name.len()));

    for component_name in sorted_names {
        // Pattern 1: h(ComponentName, -> h('ComponentName',
        let pattern1 = format!("h({},", component_name);
        let replacement1 = format!("h('{}',", component_name);
        result = result.replace(&pattern1, &replacement1);

        // Pattern 2: h(ComponentName) -> h('ComponentName')
        let pattern2 = format!("h({})", component_name);
        let replacement2 = format!("h('{}')", component_name);
        result = result.replace(&pattern2, &replacement2);

        // Pattern 3: engine.h(ComponentName, -> engine.h('ComponentName',
        let pattern3 = format!("engine.h({},", component_name);
        let replacement3 = format!("engine.h('{}',", component_name);
        result = result.replace(&pattern3, &replacement3);

        // Pattern 4: engine.h(ComponentName) -> engine.h('ComponentName')
        let pattern4 = format!("engine.h({})", component_name);
        let replacement4 = format!("engine.h('{}')", component_name);
        result = result.replace(&pattern4, &replacement4);
    }

    result
}

/// Cleans up the generated code by removing pure annotations, ES module imports, and export statements
fn cleanup_generated_code(code: &str) -> String {
    let mut cleaned = code.to_string();
    // Replace pure annotations with a space
    cleaned = cleaned.replace("/* @__PURE__ */ ", " ");
    // Remove ES module constructs (import/export) that aren't valid in script context
    let lines: Vec<&str> = cleaned
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Filter out import and export statements
            !trimmed.starts_with("import ")
                && !trimmed.starts_with("export default ")
                && !trimmed.starts_with("export ")
        })
        .collect();
    cleaned = lines.join("\n");
    cleaned
}

/// Internal transformation function that performs the complete TSX-to-JavaScript pipeline.
///
/// This is the core algorithm that transforms TSX/JSX syntax into executable JavaScript code.
/// The transformation follows these steps:
///
/// 1. **Source Type Detection**: Determines the file type (TSX) from a virtual path.
///    This is necessary because Oxc requires a path to infer source type features.
///
/// 2. **Content Wrapping** (optional): If `wrap_content` is true, wraps raw TSX content
///    in a React component function structure. This is needed when transforming standalone
///    JSX fragments that aren't already wrapped in a function.
///
/// 3. **Parsing**: Uses Oxc parser to convert TSX source code into an Abstract Syntax Tree (AST).
///    The parser handles TypeScript syntax, JSX elements, and all ECMAScript features.
///
/// 4. **Semantic Analysis**: Builds semantic information (symbol tables, scopes, etc.) from the AST.
///    This enables better transformations by understanding variable bindings and scopes.
///    Uses `with_excess_capacity(2.0)` to pre-allocate memory and reduce reallocations.
///
/// 5. **JSX Transformation**: Applies JSX-to-function-call transformations based on the config.
///    Converts JSX elements like `<div>Hello</div>` into function calls like `h('div', null, 'Hello')`.
///    The pragma (e.g., `engine.h` or `h`) is determined by the config.
///
/// 6. **Code Generation**: Converts the transformed AST back into JavaScript source code.
///    Optionally minifies the output if `config.minify` is true.
///
/// 7. **Code Cleanup**: Replaces pure annotations (`/* @__PURE__ */`) with a space and removes ES module import statements
///    (not valid in script execution context).
///
/// # Performance Considerations
///
/// This function is a hot path in the rendering pipeline. Key optimizations:
/// - Uses `Cow<str>` to avoid unnecessary allocations when content doesn't need wrapping
/// - Pre-allocates semantic builder with excess capacity to reduce reallocations
/// - Reuses the allocator across the transformation pipeline
///
/// # Arguments
/// * `tsx_content` - TSX source code to transform
/// * `config` - Transformation configuration (JSX pragma, minification, etc.)
/// * `wrap_content` - If true, wraps content in a component function; if false, assumes it's already a function
///
/// # Returns
/// Transformed JavaScript code or an error if parsing/transformation fails
fn transform_tsx_internal(
    tsx_content: &str,
    config: &TsxTransformConfig,
    wrap_content: bool,
) -> Result<String, MdxError> {
    let allocator = Allocator::default();

    // Determine source type from file path and configure for module mode with decorators
    const COMPONENT_PATH: &str = "component.tsx";
    let mut source_type = SourceType::from_path(Path::new(COMPONENT_PATH))
        .map_err(|e| MdxError::SourceType(e.to_string()))?;

    // Enable module mode to better handle export statements and enable decorators
    source_type = source_type.with_module(true);

    let path = Path::new(COMPONENT_PATH);

    // Optionally wrap content in a proper React component
    let content_to_parse: Cow<'_, str> = if wrap_content {
        Cow::Owned(wrap_in_component(tsx_content))
    } else {
        Cow::Borrowed(tsx_content)
    };

    // Parse TSX source into AST
    let parser_return = Parser::new(&allocator, &content_to_parse, source_type).parse();
    validate_parse_result(&parser_return)?;

    let mut program = parser_return.program;

    // Build semantic information for better transformation
    let semantic_return = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .build(&program);

    // Configure and apply JSX transformations
    let transform_options = create_transform_options(config);
    let transform_return = Transformer::new(&allocator, path, &transform_options)
        .build_with_scoping(semantic_return.semantic.into_scoping(), &mut program);
    validate_transform_result(&transform_return)?;

    // Generate JavaScript code from transformed AST
    let codegen_options = CodegenOptions {
        minify: config.minify,
        ..Default::default()
    };

    let code = Codegen::new()
        .with_options(codegen_options)
        .build(&program)
        .code;

    // Clean up the generated code
    let mut cleaned = cleanup_generated_code(&code);

    // Apply component-to-string transformation if component names are provided
    // This converts h(ComponentName, ...) to h('ComponentName', ...) in the generated code
    if let Some(component_names) = config.component_names.as_ref() {
        if !component_names.is_empty() {
            let names_set: HashSet<&str> = component_names.iter().map(|s| s.as_str()).collect();
            cleaned = convert_component_refs_in_ast(&cleaned, &names_set);
        }
    }

    Ok(cleaned)
}

/// Transforms a full component function definition (already wrapped)
///
/// # Arguments
/// * `component_code` - Complete component function code with JSX
///
/// # Returns
/// Generated JavaScript code or an error
pub fn transform_component_function(component_code: &str) -> Result<String, MdxError> {
    transform_tsx_internal(component_code, &TsxTransformConfig::default(), false)
}

/// Strips export statements from component code
///
/// Removes `export default` and `export` from the beginning of component code
/// to make it compatible with the TSX parser
fn strip_export_statements(code: &str) -> String {
    let trimmed = code.trim();

    // Handle "export default function" or "export default ..."
    if let Some(rest) = trimmed.strip_prefix("export default ") {
        return rest.to_string();
    }

    // Handle "export function" or "export const/let/var"
    if let Some(rest) = trimmed.strip_prefix("export ") {
        return rest.to_string();
    }

    code.to_string()
}

/// Intelligently transforms component code (detects if it's raw JSX or a function)
///
/// # Arguments
/// * `code` - Component code (either raw JSX or a complete function)
///
/// # Returns
/// Generated JavaScript code or an error
pub fn transform_component_code(code: &str) -> Result<String, MdxError> {
    // First, strip any export statements
    let code_without_exports = strip_export_statements(code);
    let trimmed = code_without_exports.trim();

    // Check if it's already a function definition
    let is_function = trimmed.starts_with("function")
        || (trimmed.starts_with('(') && trimmed.contains("=>"))
        || trimmed.starts_with("const ")
        || trimmed.starts_with("let ")
        || trimmed.starts_with("var ");

    if is_function {
        // It's a function, transform without wrapping
        transform_component_function(&code_without_exports)
    } else {
        // It's raw JSX, use the normal transformer that wraps it
        transform_tsx_to_js(&code_without_exports)
    }
}
