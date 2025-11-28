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
//! Errors include source location information when available from OXC.

use crate::error::{byte_offset_to_line_col, MdxError, ParseError, SourceLocation};
use crate::models::TsxTransformConfig;
use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{DecoratorOptions, JsxRuntime, TransformOptions, Transformer};
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
    // Target esnext since Deno Core uses V8 which supports all modern ES features.
    // This preserves arrow functions, classes, async/await, optional chaining, etc.
    // instead of downleveling them to ES5 which creates larger, slower code.
    let mut options =
        TransformOptions::from_target("esnext").unwrap_or_else(|_| TransformOptions::default());

    // JSX configuration
    options.jsx.pragma = Some(config.jsx_pragma.clone());
    options.jsx.pragma_frag = Some(config.jsx_pragma_frag.clone());
    options.jsx.runtime = JsxRuntime::Classic;
    options.jsx.development = false;
    options.jsx.refresh = None;

    // Enable TypeScript legacy decorators (experimentalDecorators + emitDecoratorMetadata).
    // This supports the decorator syntax used by Angular, NestJS, TypeORM, MobX, etc.
    //
    // NOTE: TC39 Stage 3 decorators (2023 standard) are NOT yet supported by OXC transformer.
    // See: https://github.com/oxc-project/oxc/issues/9170
    // The parser can parse TC39 syntax, but transformation is not implemented.
    // Most frameworks still use legacy decorators, so this should cover common use cases.
    options.decorator = DecoratorOptions {
        legacy: true,
        emit_decorator_metadata: true,
    };

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

/// Extracts parse errors from OXC diagnostics with location info
///
/// This function converts OXC's `OxcDiagnostic` errors into our `ParseError` type,
/// extracting source location information when available from the diagnostic labels.
fn extract_parse_errors(diagnostics: &[OxcDiagnostic], source: &str) -> Vec<ParseError> {
    diagnostics
        .iter()
        .map(|diag| {
            // Get the error message
            let message = diag.to_string();

            // Try to extract location from diagnostic labels
            // OxcDiagnostic derefs to OxcDiagnosticInner which has public labels field
            let location = diag.labels.as_ref().and_then(|labels| {
                labels.first().map(|label| {
                    let offset = label.offset() as u32;
                    let length = label.len() as u32;
                    let (line, column) = byte_offset_to_line_col(source, offset);
                    SourceLocation::new(line, column, offset, length)
                })
            });

            // Try to get help text from the public help field
            let help = diag.help.as_ref().map(|h| h.to_string());

            ParseError {
                message,
                location,
                help,
            }
        })
        .collect()
}

/// Validates and parses TSX content, returning an error if parsing fails
fn validate_parse_result(
    parser_return: &oxc_parser::ParserReturn,
    source: &str,
) -> Result<(), MdxError> {
    if !parser_return.errors.is_empty() {
        let errors = extract_parse_errors(&parser_return.errors, source);
        return Err(MdxError::TsxParse(errors));
    }
    Ok(())
}

/// Validates JSX transformation result, returning an error if transformation fails
fn validate_transform_result(
    transform_return: &oxc_transformer::TransformerReturn,
    source: &str,
) -> Result<(), MdxError> {
    if !transform_return.errors.is_empty() {
        let errors = extract_parse_errors(&transform_return.errors, source);
        return Err(MdxError::TsxTransform(errors));
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

    // Convert `export default function` to just `function` (preserves function declaration)
    // This handles cases like `export default function Component() { ... }`
    cleaned = cleaned.replace("export default function ", "function ");

    // Convert `export default class` to just `class` (preserves class declaration)
    cleaned = cleaned.replace("export default class ", "class ");

    // Remove remaining ES module constructs (import/export) that aren't valid in script context
    let lines: Vec<&str> = cleaned
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Filter out import statements and export-only statements
            // Note: `export default function/class` already converted above
            !is_import_or_pure_export(trimmed)
        })
        .collect();
    cleaned = lines.join("\n");
    cleaned
}

/// Checks if a line is an import statement or a pure export (not function/class declaration)
fn is_import_or_pure_export(trimmed: &str) -> bool {
    if trimmed.starts_with("import ") {
        return true;
    }
    if trimmed.starts_with("export default ") || trimmed.starts_with("export {") {
        return true;
    }
    // Export of function or class declaration should be kept
    if trimmed.starts_with("export ") {
        return !trimmed.contains("function ") && !trimmed.contains("class ");
    }
    false
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
    validate_parse_result(&parser_return, &content_to_parse)?;

    let mut program = parser_return.program;

    // Build semantic information for better transformation
    let semantic_return = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .build(&program);

    // Configure and apply JSX transformations
    let transform_options = create_transform_options(config);
    let transform_return = Transformer::new(&allocator, path, &transform_options)
        .build_with_scoping(semantic_return.semantic.into_scoping(), &mut program);
    validate_transform_result(&transform_return, &content_to_parse)?;

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

/// Validates that an export default statement uses the required pattern
///
/// The only valid pattern is:
/// - `export default function Component() { ... }` (sync function named Component)
///
/// Invalid exports (will return error):
/// - `export default function Button() { ... }` (wrong name - must be Component)
/// - `export default () => { ... }` (arrow functions not supported)
/// - `export default async function Component() { ... }` (async not supported)
/// - `export default class Component { ... }` (classes not supported)
/// - `export default SomeVariable` (identifier reference)
///
/// Note: Arrow functions, async functions, and classes are rejected because
/// the renderer requires a synchronous function named `Component` that can
/// be called directly.
fn validate_export_default(rest: &str) -> Result<(), MdxError> {
    let trimmed = rest.trim();

    // Arrow functions are NOT supported - they don't define a named Component
    if trimmed.starts_with('(') {
        return Err(MdxError::InvalidExportDefault("arrow function".to_string()));
    }

    // Async arrow functions are NOT supported
    if trimmed.starts_with("async (") {
        return Err(MdxError::InvalidExportDefault(
            "async arrow function".to_string(),
        ));
    }

    // Async functions are NOT supported - they return Promises
    if trimmed.starts_with("async function") {
        return Err(MdxError::InvalidExportDefault("async function".to_string()));
    }

    // Classes are NOT supported - they require `new` to instantiate
    if trimmed.starts_with("class ") {
        let class_name = trimmed
            .strip_prefix("class ")
            .unwrap_or("")
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
            .next()
            .unwrap_or("unknown");
        return Err(MdxError::InvalidExportDefault(format!(
            "class {class_name}"
        )));
    }

    // Check for named function: must be "function Component"
    if let Some(after_fn) = trimmed.strip_prefix("function ") {
        let fn_name = after_fn
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
            .next()
            .unwrap_or("");

        if fn_name == "Component" {
            return Ok(());
        }
        return Err(MdxError::InvalidExportDefault(format!(
            "function {fn_name}"
        )));
    }

    // If we get here, it's likely an identifier export like `export default MyComponent`
    // Extract the identifier name for the error message
    let identifier = trimmed
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
        .next()
        .unwrap_or("unknown");

    // Check if it looks like an identifier (starts with letter or underscore)
    if !identifier.is_empty()
        && identifier
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_' || c == '$')
    {
        return Err(MdxError::InvalidExportDefault(identifier.to_string()));
    }

    // For other invalid patterns, use a generic message
    Err(MdxError::InvalidExportDefault("expression".to_string()))
}

/// Strips export statements from component code
///
/// Removes `export default` and `export` from the beginning of component code
/// to make it compatible with the TSX parser.
///
/// # Errors
///
/// Returns `MdxError::InvalidExportDefault` if `export default` is followed by
/// an identifier reference instead of a component definition.
fn strip_export_statements(code: &str) -> Result<String, MdxError> {
    let trimmed = code.trim();

    // Handle "export default function" or "export default ..."
    if let Some(rest) = trimmed.strip_prefix("export default ") {
        // Validate that this is a proper component export
        validate_export_default(rest)?;
        return Ok(rest.to_string());
    }

    // Handle "export function" or "export const/let/var"
    if let Some(rest) = trimmed.strip_prefix("export ") {
        return Ok(rest.to_string());
    }

    Ok(code.to_string())
}

/// Intelligently transforms component code (detects if it's raw JSX or a function)
///
/// # Arguments
/// * `code` - Component code (either raw JSX or a complete function)
///
/// # Returns
/// Generated JavaScript code or an error
///
/// # Errors
///
/// Returns `MdxError::InvalidExportDefault` if the code contains an invalid
/// `export default` statement (e.g., `export default SomeVariable` instead of
/// a proper component definition).
pub fn transform_component_code(code: &str) -> Result<String, MdxError> {
    // First, strip any export statements (validates export default)
    let code_without_exports = strip_export_statements(code)?;
    let trimmed = code_without_exports.trim();

    // Check if it's already a function definition
    let is_function = trimmed.starts_with("function")
        || trimmed.starts_with("async function")
        || trimmed.starts_with("async (")
        || (trimmed.starts_with('(') && trimmed.contains("=>"))
        || trimmed.starts_with("const ")
        || trimmed.starts_with("let ")
        || trimmed.starts_with("var ")
        || trimmed.starts_with("class ");

    if is_function {
        // It's a function, transform without wrapping
        transform_component_function(&code_without_exports)
    } else {
        // It's raw JSX, use the normal transformer that wraps it
        transform_tsx_to_js(&code_without_exports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Valid export default test ====================
    // Only `export default function Component` is valid

    #[test]
    fn test_valid_export_default_function_component() {
        let code = "export default function Component() { return <button>Click</button>; }";
        let result = transform_component_code(code);
        assert!(
            result.is_ok(),
            "function Component should be valid, got: {:?}",
            result.err()
        );
        let output = result.unwrap();
        assert!(
            output.contains("function Component()"),
            "Output should contain function Component, got: {output}"
        );
        assert!(
            output.contains("engine.h("),
            "Output should have transformed JSX to engine.h calls, got: {output}"
        );
    }

    #[test]
    fn test_valid_export_default_function_component_with_props() {
        let code = "export default function Component(props) { return <div>{props.name}</div>; }";
        let result = transform_component_code(code);
        assert!(
            result.is_ok(),
            "function Component with props should be valid, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_valid_export_default_function_component_typescript() {
        // Test TypeScript syntax with type annotations
        let code =
            "export default function Component(props: any) { return <div>{props.name}</div>; }";
        let result = transform_component_code(code);
        assert!(
            result.is_ok(),
            "function Component with TypeScript props: any should be valid, got: {:?}",
            result.err()
        );
        let output = result.unwrap();
        assert!(
            output.contains("function Component(props)"),
            "TypeScript types should be stripped, got: {output}"
        );
    }

    #[test]
    fn test_valid_export_default_function_component_typescript_interface() {
        // Test TypeScript with interface-like type
        let code = "export default function Component(props: { name: string; age: number }) { return <div>{props.name}</div>; }";
        let result = transform_component_code(code);
        assert!(
            result.is_ok(),
            "function Component with TypeScript inline type should be valid, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_decorator_before_export_default_fails() {
        // Decorators on function declarations are not valid JS/TS syntax
        // They only work on classes (which we already reject)
        let code = r#"@logged
export default function Component() { return <div>Decorated</div>; }"#;
        let result = transform_component_code(code);
        // Should fail - either at validation (doesn't start with export default)
        // or at parse time (invalid syntax)
        assert!(
            result.is_err(),
            "Decorator on function should fail: {:?}",
            result
        );
    }

    #[test]
    fn test_decorator_inside_component_on_class() {
        // Decorators work INSIDE the component on helper classes
        let code = r#"export default function Component(props: any) {
    function logged(target: any) { return target; }

    @logged
    class Helper {
        getValue() { return "helper"; }
    }

    const h = new Helper();
    return <div>{h.getValue()}</div>;
}"#;
        let result = transform_component_code(code);
        assert!(
            result.is_ok(),
            "Decorator on class inside component should work: {:?}",
            result.err()
        );
        let output = result.unwrap();
        assert!(
            output.contains("_decorate"),
            "Should transform decorator: {output}"
        );
    }

    #[test]
    fn test_decorator_on_class_method() {
        // Method decorators also work
        let code = r#"export default function Component(props: any) {
    function log(target: any, key: string) { return target; }

    class Utils {
        @log
        format(value: string) { return value.toUpperCase(); }
    }

    const u = new Utils();
    return <div>{u.format(props.text)}</div>;
}"#;
        let result = transform_component_code(code);
        assert!(
            result.is_ok(),
            "Method decorator should work: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_decorator_outside_component_at_module_level() {
        // Decorators on classes defined at module level (outside Component)
        let code = r#"function logged(target: any) { console.log('decorated'); return target; }

@logged
class Utils {
    format(value: string) { return value.toUpperCase(); }
}

export default function Component(props: any) {
    const u = new Utils();
    return <div>{u.format(props.text)}</div>;
}"#;
        let result = transform_component_code(code);
        assert!(
            result.is_ok(),
            "Module-level decorator should work: {:?}",
            result.err()
        );
        let output = result.unwrap();
        assert!(
            output.contains("_decorate"),
            "Should transform decorator: {output}"
        );
    }

    #[test]
    fn test_decorator_on_standalone_function_fails() {
        // Decorators on standalone functions are NOT valid in TypeScript/JavaScript
        // This is a language limitation - decorators only work on classes and class members
        let code = r#"function log(fn: any) { return fn; }

@log
function myUtil() { return "hello"; }

export default function Component() {
    return <div>{myUtil()}</div>;
}"#;
        let result = transform_component_code(code);
        // Should fail at parse time - decorators on functions are invalid syntax
        assert!(
            result.is_err(),
            "Decorator on standalone function should fail (invalid syntax)"
        );
    }

    // ==================== Invalid: wrong function name ====================

    #[test]
    fn test_invalid_export_default_function_wrong_name() {
        let code = "export default function Button() { return <button>Click</button>; }";
        let result = transform_component_code(code);

        assert!(
            result.is_err(),
            "function Button should be invalid (must be Component)"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, MdxError::InvalidExportDefault(ref name) if name == "function Button"),
            "Error should mention 'function Button', got: {err:?}"
        );
    }

    // ==================== Invalid: arrow functions not supported ====================

    #[test]
    fn test_invalid_export_default_arrow_function() {
        let code = "export default () => <div>Hello</div>";
        let result = transform_component_code(code);

        assert!(result.is_err(), "Arrow function should be invalid");
        let err = result.unwrap_err();
        assert!(
            matches!(err, MdxError::InvalidExportDefault(ref name) if name == "arrow function"),
            "Error should mention 'arrow function', got: {err:?}"
        );
    }

    #[test]
    fn test_invalid_export_default_arrow_with_props() {
        let code = "export default (props) => <div>{props.name}</div>";
        let result = transform_component_code(code);

        assert!(
            result.is_err(),
            "Arrow function with props should be invalid"
        );
    }

    #[test]
    fn test_invalid_export_default_async_arrow() {
        let code = "export default async () => <div>Async</div>";
        let result = transform_component_code(code);

        assert!(result.is_err(), "Async arrow function should be invalid");
        let err = result.unwrap_err();
        assert!(
            matches!(err, MdxError::InvalidExportDefault(ref name) if name == "async arrow function"),
            "Error should mention 'async arrow function', got: {err:?}"
        );
    }

    // ==================== Invalid: async functions not supported ====================

    #[test]
    fn test_invalid_export_default_async_function() {
        let code = "export default async function Component() { return <div>Data</div>; }";
        let result = transform_component_code(code);

        assert!(result.is_err(), "Async function should be invalid");
        let err = result.unwrap_err();
        assert!(
            matches!(err, MdxError::InvalidExportDefault(ref name) if name == "async function"),
            "Error should mention 'async function', got: {err:?}"
        );
    }

    // ==================== Invalid: classes not supported ====================

    #[test]
    fn test_invalid_export_default_class() {
        let code = "export default class Component { render() { return <button />; } }";
        let result = transform_component_code(code);

        assert!(result.is_err(), "Class should be invalid");
        let err = result.unwrap_err();
        assert!(
            matches!(err, MdxError::InvalidExportDefault(ref name) if name == "class Component"),
            "Error should mention 'class Component', got: {err:?}"
        );
    }

    #[test]
    fn test_invalid_export_default_class_wrong_name() {
        let code = "export default class MyWidget { render() { return <div />; } }";
        let result = transform_component_code(code);

        assert!(result.is_err(), "Class MyWidget should be invalid");
        let err = result.unwrap_err();
        assert!(
            matches!(err, MdxError::InvalidExportDefault(ref name) if name == "class MyWidget"),
            "Error should mention 'class MyWidget', got: {err:?}"
        );
    }

    // ==================== Invalid: identifier reference ====================

    #[test]
    fn test_invalid_export_default_identifier() {
        let code = "export default MyComponent";
        let result = transform_component_code(code);

        assert!(result.is_err(), "Identifier export should be invalid");
        let err = result.unwrap_err();
        assert!(
            matches!(err, MdxError::InvalidExportDefault(ref name) if name == "MyComponent"),
            "Error should contain 'MyComponent', got: {err:?}"
        );
    }

    #[test]
    fn test_invalid_export_default_identifier_with_semicolon() {
        let code = "export default Button;";
        let result = transform_component_code(code);

        assert!(
            result.is_err(),
            "Identifier export with semicolon should be invalid"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, MdxError::InvalidExportDefault(ref name) if name == "Button"),
            "Error should contain 'Button', got: {err:?}"
        );
    }

    // ==================== Error message format ====================

    #[test]
    fn test_error_message_suggests_component() {
        let code = "export default function Widget() { return <div />; }";
        let result = transform_component_code(code);

        let err = result.unwrap_err();
        let error_message = format!("{err}");
        assert!(
            error_message.contains("function Widget"),
            "Error message should contain the invalid name"
        );
        assert!(
            error_message.contains("export default function Component()"),
            "Error message should suggest using 'Component'"
        );
    }

    // ==================== Non-export default tests (should still work) ====================

    #[test]
    fn test_no_export_raw_jsx() {
        let code = "<div>Hello World</div>";
        let result = transform_component_code(code);
        assert!(result.is_ok(), "Raw JSX without export should work");
    }

    #[test]
    fn test_export_const_function() {
        // Named exports (not default) don't have this restriction
        let code = "export const Button = () => <button>Click</button>";
        let result = transform_component_code(code);
        assert!(result.is_ok(), "export const should be valid");
    }

    #[test]
    fn test_export_function() {
        // Named exports (not default) don't have this restriction
        let code = "export function Button() { return <button>Click</button>; }";
        let result = transform_component_code(code);
        assert!(result.is_ok(), "export function should be valid");
    }
}
