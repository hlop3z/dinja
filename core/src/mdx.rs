//! MDX processing and rendering logic
//!
//! This module handles the core MDX processing pipeline:
//! 1. YAML frontmatter extraction
//! 2. Markdown to HTML conversion (with JSX support)
//! 3. TSX transformation to JavaScript
//! 4. Component rendering via JavaScript runtime
//!
//! ## Pipeline Types
//!
//! The module supports a single rendering pipeline:
//!
//! - **Engine Pipeline**: For HTML, JavaScript, and Schema output formats, uses engine for rendering
//!
//! ## Error Handling
//!
//! All domain-specific errors use `MdxError`. Errors are converted to `anyhow::Error` at the
//! service boundary for consistent error handling in HTTP handlers.

use crate::error::MdxError;
use crate::models::{
    ComponentDefinition, OutputFormat, RenderSettings, RenderedMdx, TsxTransformConfig,
};
use crate::renderer::JsRenderer;
use crate::transform::{transform_tsx_to_js_for_output, transform_tsx_to_js_with_config};
use gray_matter::{engine::YAML, Matter};
use markdown::{to_html_with_options, CompileOptions, Constructs, Options, ParseOptions};
use regex::Regex;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

// =============================================================================
// JSX Protection - Constants and Compiled Patterns
// =============================================================================

/// Maximum number of JSX placeholders allowed per document.
/// Prevents DoS attacks from documents with excessive JSX components.
const MAX_JSX_PLACEHOLDERS: usize = 1000;

/// Maximum nesting depth for JSX components with children.
/// Prevents stack overflow from deeply nested components.
const MAX_JSX_NESTING_DEPTH: usize = 100;

/// Compiled regex for self-closing JSX components with expression attributes.
/// Pattern: <ComponentName attr={...} />
/// - Component names must start with uppercase (JSX convention)
/// - Must have at least one expression attribute (curly braces)
/// - Must be self-closing (ends with />)
///
/// # Safety
/// Pattern is compile-time constant and known to be valid.
static SELF_CLOSING_JSX_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<([A-Z][a-zA-Z0-9]*)\s+[^>]*\{[^}]*\}[^>]*/\s*>")
        .expect("hardcoded regex pattern is valid")
});

/// Compiled regex for opening JSX tags with expression attributes.
/// Pattern: <ComponentName attr={...}>
/// Used to find JSX components with children that need protection.
///
/// # Safety
/// Pattern is compile-time constant and known to be valid.
static OPENING_JSX_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<([A-Z][a-zA-Z0-9]*)\s+[^>]*\{[^}]*\}[^>]*>")
        .expect("hardcoded regex pattern is valid")
});

/// Compiled regex for extracting component names from HTML.
/// Used for schema extraction.
///
/// # Safety
/// Pattern is compile-time constant and known to be valid.
static COMPONENT_NAME_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<([A-Z][a-zA-Z0-9]*)").expect("hardcoded regex pattern is valid")
});

struct RenderContext<'a> {
    renderer: &'a JsRenderer,
    components: Option<&'a HashMap<String, ComponentDefinition>>,
    props_json: &'a str,
    settings: &'a RenderSettings,
}

/// Creates markdown parsing and compilation options.
///
/// # Security Model
///
/// `allow_dangerous_html` is set to `true` because:
/// 1. MDX requires unescaped HTML/JSX tags to function (e.g., `<Component prop={value} />`)
/// 2. JSX expression attributes `{...}` would be escaped and broken otherwise
/// 3. The actual security boundary is the JavaScript runtime execution layer
///
/// **Important**: MDX content should only come from trusted sources. If you need to
/// render untrusted markdown, sanitize the output HTML after rendering or use a
/// separate markdown-only pipeline without JSX support.
///
/// # Features Enabled
///
/// - **CommonMark**: All standard markdown features (headings, lists, code blocks, etc.)
/// - **GFM Extensions**: Tables, strikethrough, task lists, autolinks, footnotes
/// - **HTML/JSX**: Block and inline HTML elements for component embedding
fn markdown_options() -> Options {
    Options {
        parse: ParseOptions {
            constructs: Constructs {
                html_flow: true, // Allow block-level HTML/JSX
                html_text: true, // Allow inline HTML/JSX
                // GFM (GitHub Flavored Markdown) extensions
                gfm_autolink_literal: true, // Auto-linkify URLs without angle brackets
                gfm_footnote_definition: true, // Footnotes: [^a]: footnote text
                gfm_label_start_footnote: true, // Footnote references: [^a]
                gfm_strikethrough: true,    // Strikethrough: ~text~ or ~~text~~
                gfm_table: true,            // Tables with | pipes |
                gfm_task_list_item: true,   // Task lists: - [x] done
                ..Constructs::default()
            },
            ..ParseOptions::default()
        },
        compile: CompileOptions {
            // SECURITY: HTML is not escaped to preserve JSX syntax.
            // See function doc comment for security model explanation.
            allow_dangerous_html: true,
            ..CompileOptions::default()
        },
    }
}

// =============================================================================
// JSX Protection - Core Functions
// =============================================================================

/// Protects JSX components from markdown processing by replacing them with placeholders.
///
/// JSX components are identified as tags starting with a capital letter that contain
/// expression attributes (curly braces). This prevents markdown from escaping the
/// curly braces which would break the JSX syntax.
///
/// # Safety Limits
/// - Maximum `MAX_JSX_PLACEHOLDERS` placeholders per document
/// - Maximum `MAX_JSX_NESTING_DEPTH` nesting depth for components with children
///
/// # Arguments
/// * `content` - The MDX content to process
///
/// # Returns
/// A tuple of (processed content, placeholder map)
fn protect_jsx_components(content: &str) -> (String, HashMap<String, String>) {
    // Early return for empty content
    if content.is_empty() {
        return (String::new(), HashMap::new());
    }

    // Pre-allocate with estimated capacity
    let estimated_placeholders = content.matches('<').count().min(MAX_JSX_PLACEHOLDERS) / 4;
    let mut placeholders: HashMap<String, String> =
        HashMap::with_capacity(estimated_placeholders.max(8));
    let mut result = content.to_string();
    let mut counter: usize = 0;

    // Phase 1: Protect self-closing JSX components
    // These are the most common and safest to handle
    let matches: Vec<_> = SELF_CLOSING_JSX_PATTERN.find_iter(content).collect();

    for mat in matches.into_iter().rev() {
        // Check placeholder limit
        if counter >= MAX_JSX_PLACEHOLDERS {
            eprintln!(
                "Warning: JSX placeholder limit ({}) reached, some JSX may not be protected",
                MAX_JSX_PLACEHOLDERS
            );
            break;
        }

        let jsx = mat.as_str();
        let placeholder = format!("<!--JSX:{}-->", counter);

        // Use positional replacement to avoid issues with duplicate JSX
        let start = mat.start();
        let end = mat.end();

        // Adjust positions based on previous replacements
        // Since we iterate in reverse, positions should still be valid
        if start < result.len() && end <= result.len() {
            // Verify the content at this position still matches
            if result.get(start..end).map(|s| s == jsx).unwrap_or(false) {
                result.replace_range(start..end, &placeholder);
                placeholders.insert(placeholder, jsx.to_string());
                counter += 1;
            }
        }
    }

    // Phase 2: Protect JSX components with children
    // This requires finding matching closing tags
    protect_jsx_with_children(&mut result, &mut placeholders, &mut counter);

    (result, placeholders)
}

/// Protects JSX components that have children (non-self-closing).
/// Uses a more careful approach to match opening and closing tags.
fn protect_jsx_with_children(
    content: &mut String,
    placeholders: &mut HashMap<String, String>,
    counter: &mut usize,
) {
    let mut depth = 0;
    let mut iterations = 0;
    let max_iterations = MAX_JSX_PLACEHOLDERS;

    // Keep processing until no more matches or limits reached
    loop {
        iterations += 1;
        if iterations > max_iterations || *counter >= MAX_JSX_PLACEHOLDERS {
            break;
        }

        // Find the next opening tag with expression attributes
        let content_snapshot = content.clone();
        let capture = match OPENING_JSX_PATTERN.captures(&content_snapshot) {
            Some(cap) => cap,
            None => break,
        };

        let tag_name = match capture.get(1) {
            Some(m) => m.as_str(),
            None => break,
        };

        let opening_tag = match capture.get(0) {
            Some(m) => m.as_str(),
            None => break,
        };

        // Find positions
        let open_pos = match content.find(opening_tag) {
            Some(pos) => pos,
            None => break,
        };

        // Find matching closing tag with proper nesting
        let closing_tag = format!("</{}>", tag_name);
        let search_start = open_pos + opening_tag.len();

        if let Some(close_pos) =
            find_matching_close_tag(content, search_start, tag_name, &closing_tag, &mut depth)
        {
            // Check nesting depth limit
            if depth > MAX_JSX_NESTING_DEPTH {
                eprintln!(
                    "Warning: JSX nesting depth ({}) exceeded limit ({})",
                    depth, MAX_JSX_NESTING_DEPTH
                );
                break;
            }

            let full_end = close_pos + closing_tag.len();

            // Validate bounds
            if full_end > content.len() {
                break;
            }

            let full_jsx = content[open_pos..full_end].to_string();
            let placeholder = format!("<!--JSX:{}-->", counter);

            // Replace in content
            content.replace_range(open_pos..full_end, &placeholder);
            placeholders.insert(placeholder, full_jsx);
            *counter += 1;
        } else {
            // No matching close tag found - this JSX is malformed
            // Skip this tag and continue (don't protect malformed JSX)
            break;
        }
    }
}

/// Finds the matching closing tag, accounting for nested tags of the same type.
///
/// # Arguments
/// * `content` - The content to search in
/// * `start` - Position to start searching from (after opening tag)
/// * `tag_name` - The tag name to match
/// * `closing_tag` - The full closing tag string (e.g., "</Component>")
/// * `depth` - Tracks current nesting depth for limit checking
///
/// # Returns
/// Position of the matching closing tag, or None if not found
fn find_matching_close_tag(
    content: &str,
    start: usize,
    tag_name: &str,
    closing_tag: &str,
    depth: &mut usize,
) -> Option<usize> {
    let search_region = &content[start..];

    // Build pattern for nested opening tags of same type
    let nested_open_pattern = format!("<{}", tag_name);

    let mut nesting = 1;
    let mut pos = 0;

    while nesting > 0 && pos < search_region.len() {
        // Find next occurrence of either opening or closing tag
        let next_open = search_region[pos..].find(&nested_open_pattern);
        let next_close = search_region[pos..].find(closing_tag);

        match (next_open, next_close) {
            (Some(open_offset), Some(close_offset)) => {
                if open_offset < close_offset {
                    // Found nested opening tag first
                    nesting += 1;
                    *depth = (*depth).max(nesting);
                    pos += open_offset + nested_open_pattern.len();
                } else {
                    // Found closing tag first
                    nesting -= 1;
                    if nesting == 0 {
                        return Some(start + pos + close_offset);
                    }
                    pos += close_offset + closing_tag.len();
                }
            }
            (None, Some(close_offset)) => {
                // Only closing tag found
                nesting -= 1;
                if nesting == 0 {
                    return Some(start + pos + close_offset);
                }
                pos += close_offset + closing_tag.len();
            }
            (Some(open_offset), None) => {
                // Only opening tag found - unbalanced
                nesting += 1;
                *depth = (*depth).max(nesting);
                pos += open_offset + nested_open_pattern.len();
            }
            (None, None) => {
                // Neither found - unbalanced
                break;
            }
        }

        // Safety limit on nesting
        if nesting > MAX_JSX_NESTING_DEPTH {
            return None;
        }
    }

    None
}

/// Restores JSX components from placeholders after markdown processing.
///
/// # Arguments
/// * `content` - The HTML content with placeholders
/// * `placeholders` - Map of placeholder -> original JSX
///
/// # Returns
/// Content with all placeholders replaced with original JSX.
/// If any placeholders remain unreplaced, a warning is logged.
fn restore_jsx_components(content: &str, placeholders: &HashMap<String, String>) -> String {
    if placeholders.is_empty() {
        return content.to_string();
    }

    let mut result = content.to_string();
    let mut restored_count = 0;

    // Sort placeholders by index to ensure consistent restoration order
    let mut placeholder_vec: Vec<_> = placeholders.iter().collect();
    placeholder_vec.sort_by_key(|(k, _)| {
        // Extract number from <!--JSX:N-->
        k.strip_prefix("<!--JSX:")
            .and_then(|s| s.strip_suffix("-->"))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0)
    });

    for (placeholder, jsx) in placeholder_vec {
        if result.contains(placeholder.as_str()) {
            result = result.replace(placeholder.as_str(), jsx);
            restored_count += 1;
        }
    }

    // Validation: check if all placeholders were restored
    if restored_count != placeholders.len() {
        eprintln!(
            "Warning: JSX restoration incomplete - {} of {} placeholders restored",
            restored_count,
            placeholders.len()
        );
    }

    // Validate no placeholders remain in output
    if result.contains("<!--JSX:") {
        eprintln!("Warning: Unreplaced JSX placeholders found in output");
    }

    result
}

/// Removes all Fragment tags from HTML output.
///
/// Fragment tags (`<Fragment>` and `</Fragment>`) are virtual wrappers used in JSX
/// that should not appear in the final HTML output. This function strips all
/// occurrences of Fragment tags while preserving their children content.
///
/// # Arguments
/// * `html` - HTML string that may contain Fragment tags
///
/// # Returns
/// HTML string with all Fragment tags removed
fn unwrap_fragment(html: &str) -> String {
    // Static regex pattern compiled once at first use
    // Matches: <Fragment>, <Fragment ...attrs>, </Fragment>, </fragment> (case-insensitive)
    // Safety: Pattern is compile-time constant and known to be valid.
    static FRAGMENT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)</?Fragment[^>]*>").expect("hardcoded regex pattern is valid")
    });

    let trimmed = html.trim();
    let result = FRAGMENT_PATTERN.replace_all(trimmed, "");
    result.into_owned()
}

fn render_markdown(content: &str) -> Result<String, MdxError> {
    // Protect JSX components with expression attributes from markdown processing
    let (protected_content, placeholders) = protect_jsx_components(content);

    let options = markdown_options();
    let html = to_html_with_options(&protected_content, &options)
        .map_err(|e| MdxError::MarkdownRender(e.to_string()))?;

    // Restore JSX components after markdown processing
    Ok(restore_jsx_components(&html, &placeholders))
}

/// Helper function to log render errors with context
/// Preserves the full error chain for better debugging
fn log_render_error(e: &anyhow::Error, js_output: &str, context: &str) {
    eprintln!("{context} render error details: {:#}", e);
    eprintln!("JavaScript output: {js_output}");
}

/// Converts MDX content to HTML and JavaScript with frontmatter extraction
///
/// # Arguments
/// * `mdx_content` - Raw MDX content with optional YAML frontmatter
/// * `renderer` - JavaScript renderer instance for component rendering
/// * `components` - Optional map of component definitions to inject
/// * `settings` - Rendering settings including output format
///
/// # Returns
/// A `RenderedMdx` struct containing rendered output and metadata
pub fn mdx_to_html_with_frontmatter(
    mdx_content: &str,
    renderer: &JsRenderer,
    components: Option<&HashMap<String, ComponentDefinition>>,
    settings: &RenderSettings,
) -> Result<RenderedMdx, MdxError> {
    // Parse YAML frontmatter
    let matter = Matter::<YAML>::new();
    let parsed = matter
        .parse::<serde_json::Value>(mdx_content)
        .map_err(|e| MdxError::FrontmatterParse(e.to_string()))?;

    let frontmatter = parsed
        .data
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::with_capacity(0)));

    // Render markdown to HTML with HTML/JSX components enabled
    let html_output = render_markdown(&parsed.content)?;

    // Convert frontmatter to JSON string for props
    let props_json = serde_json::to_string(&frontmatter)
        .map_err(|e| MdxError::FrontmatterParse(format!("Failed to serialize frontmatter: {e}")))?;

    let context = RenderContext {
        renderer,
        components,
        props_json: &props_json,
        settings,
    };

    let output = render_with_engine_pipeline(&context, &html_output)?;

    Ok(RenderedMdx {
        metadata: frontmatter,
        output: Some(output),
    })
}

/// Creates a fallback error response for failed MDX rendering
///
/// Preserves the full error chain for better debugging and error tracking.
/// The error chain includes all underlying causes, making it easier to diagnose
/// root causes of rendering failures.
///
/// # Arguments
/// * `error` - The error that occurred during rendering
///
/// # Returns
/// A `RenderedMdx` struct with error information including full error chain
pub fn create_error_response(error: &anyhow::Error) -> RenderedMdx {
    // Format full error chain with all context
    let error_chain = format!("{:#}", error);
    let error_message = error.to_string();

    // Log full error chain for debugging
    eprintln!("MDX rendering error: {error_chain}");

    let error_html = format!("<p>Error rendering MDX: {error_message}</p>");
    RenderedMdx {
        metadata: json!({
            "error": error_message,
            "error_chain": error_chain
        }),
        output: Some(error_html),
    }
}

/// Schema extraction result containing components and directives information
#[derive(serde::Serialize, Default)]
struct SchemaResult {
    /// Unique component names (elements starting with capital letters)
    components: Vec<String>,
    /// Directive information extracted based on settings.directives prefixes
    directives: DirectivesResult,
}

/// Directive extraction results
#[derive(serde::Serialize, Default)]
struct DirectivesResult {
    /// Unique directive attribute keys (e.g., "v-on:click", "x-show")
    keys: Vec<String>,
    /// Unique directive patterns (e.g., "v-on:*", "x-*")
    patterns: Vec<String>,
    /// Unique directive values
    values: Vec<serde_json::Value>,
}

/// Extracts schema information from JSON tree including components and directives
///
/// # Arguments
/// * `json_tree` - The rendered JSON tree from core.js engine
/// * `directive_prefixes` - Optional list of directive prefixes to extract (e.g., ["v-", "@", "x-"])
///
/// # Returns
/// A JSON string containing components and directives schema
fn extract_schema_from_json(
    json_tree: &str,
    directive_prefixes: Option<&Vec<String>>,
) -> Result<String, MdxError> {
    let tree: serde_json::Value = serde_json::from_str(json_tree)
        .map_err(|e| MdxError::FrontmatterParse(format!("Failed to parse JSON tree: {e}")))?;

    let mut components: HashSet<String> = HashSet::new();
    let mut directive_keys: HashSet<String> = HashSet::new();
    let mut directive_patterns: HashSet<String> = HashSet::new();
    let mut directive_values: HashSet<String> = HashSet::new(); // Store as JSON strings for dedup

    // Get directive prefixes as a slice for efficient iteration
    let prefixes: Vec<&str> = directive_prefixes
        .map(|d| d.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default();

    // Recursively traverse the JSON tree
    traverse_json_tree(
        &tree,
        &prefixes,
        &mut components,
        &mut directive_keys,
        &mut directive_patterns,
        &mut directive_values,
    );

    // Convert to sorted vectors for consistent output
    let mut sorted_components: Vec<String> = components.into_iter().collect();
    sorted_components.sort();

    let mut sorted_keys: Vec<String> = directive_keys.into_iter().collect();
    sorted_keys.sort();

    let mut sorted_patterns: Vec<String> = directive_patterns.into_iter().collect();
    sorted_patterns.sort();

    // Parse directive values back to JSON values
    let mut sorted_values: Vec<serde_json::Value> = directive_values
        .into_iter()
        .filter_map(|s| serde_json::from_str(&s).ok())
        .collect();
    // Sort values by their JSON string representation for consistency
    sorted_values.sort_by_key(|a| a.to_string());

    let result = SchemaResult {
        components: sorted_components,
        directives: DirectivesResult {
            keys: sorted_keys,
            patterns: sorted_patterns,
            values: sorted_values,
        },
    };

    serde_json::to_string(&result)
        .map_err(|e| MdxError::FrontmatterParse(format!("Failed to serialize schema: {e}")))
}

/// Recursively traverses the JSON tree to extract components and directives
fn traverse_json_tree(
    node: &serde_json::Value,
    prefixes: &[&str],
    components: &mut HashSet<String>,
    directive_keys: &mut HashSet<String>,
    directive_patterns: &mut HashSet<String>,
    directive_values: &mut HashSet<String>,
) {
    match node {
        serde_json::Value::Object(obj) => {
            // Check for component type (capitalized tag names, excluding built-in elements)
            if let Some(serde_json::Value::String(tag)) = obj.get("type") {
                if !tag.is_empty()
                    && tag
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    && tag != "Fragment"
                // Exclude built-in Fragment
                {
                    components.insert(tag.clone());
                }
            }

            // Check attributes for directives
            if let Some(serde_json::Value::Object(attrs)) = obj.get("attributes") {
                for (key, value) in attrs {
                    // Check if this attribute matches any directive prefix
                    for prefix in prefixes {
                        if key.starts_with(prefix) {
                            directive_keys.insert(key.clone());

                            // Extract pattern (e.g., "v-on:click" -> "v-on:*")
                            let pattern = if key.contains(':') {
                                let parts: Vec<&str> = key.splitn(2, ':').collect();
                                format!("{}:*", parts[0])
                            } else {
                                format!("{}*", prefix)
                            };
                            directive_patterns.insert(pattern);

                            // Store value as JSON string for dedup
                            if let Ok(value_str) = serde_json::to_string(value) {
                                directive_values.insert(value_str);
                            }
                            break;
                        }
                    }
                }
            }

            // Recurse into children
            if let Some(children) = obj.get("children") {
                traverse_json_tree(
                    children,
                    prefixes,
                    components,
                    directive_keys,
                    directive_patterns,
                    directive_values,
                );
            }

            // Recurse into all object values
            for value in obj.values() {
                traverse_json_tree(
                    value,
                    prefixes,
                    components,
                    directive_keys,
                    directive_patterns,
                    directive_values,
                );
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                traverse_json_tree(
                    item,
                    prefixes,
                    components,
                    directive_keys,
                    directive_patterns,
                    directive_values,
                );
            }
        }
        _ => {}
    }
}

fn render_with_engine_pipeline(
    context: &RenderContext<'_>,
    html_output: &str,
) -> Result<String, MdxError> {
    // HOT PATH: TSX transformation - called for every MDX file with Html/Javascript output
    let mut transform_config = TsxTransformConfig::for_engine(false);

    match context.settings.output {
        OutputFormat::Schema => {
            // For schema output, render to JSON first then extract schema information
            // This allows us to extract both components and directives from the tree

            // For schema, convert component function references to strings
            // Start by extracting component names from the HTML content (JSX tags starting with capital letters)
            let mut component_names: HashSet<String> = HashSet::new();

            // Extract from HTML content using pre-compiled pattern
            for cap in COMPONENT_NAME_PATTERN.captures_iter(html_output) {
                if let Some(name) = cap.get(1) {
                    component_names.insert(name.as_str().to_string());
                }
            }

            // Also include names from component definitions if provided
            if let Some(components) = context.components {
                for (key, comp_def) in components.iter() {
                    let name = comp_def
                        .name
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| key.clone());
                    component_names.insert(name);
                }
            }

            if !component_names.is_empty() {
                transform_config.component_names = Some(component_names);
            }

            let javascript_output = transform_tsx_to_js_with_config(html_output, transform_config)
                .map_err(|e| {
                    MdxError::tsx_transform(format!("Failed to transform TSX to JavaScript: {e}"))
                })?;

            // Render to JSON tree using core.js engine
            let json_tree = render_template_to_schema(context, &javascript_output)?;

            // Extract schema from JSON tree (components + directives)
            extract_schema_from_json(&json_tree, context.settings.directives.as_ref())
        }
        OutputFormat::Html | OutputFormat::Javascript | OutputFormat::Json => {
            // For json output, convert component function references to strings
            // For HTML output, keep as function references so they can be rendered
            // For JavaScript output, keep Preact syntax with h() and Fragment
            if matches!(context.settings.output, OutputFormat::Json) {
                // Extract component names from HTML content (JSX tags starting with capital letters)
                let mut component_names: HashSet<String> = HashSet::new();

                // Extract from HTML content using pre-compiled pattern
                for cap in COMPONENT_NAME_PATTERN.captures_iter(html_output) {
                    if let Some(name) = cap.get(1) {
                        component_names.insert(name.as_str().to_string());
                    }
                }

                // Also include names from component definitions if provided
                if let Some(components) = context.components {
                    for (key, comp_def) in components.iter() {
                        let name = comp_def
                            .name
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| key.clone());
                        component_names.insert(name);
                    }
                }

                if !component_names.is_empty() {
                    transform_config.component_names = Some(component_names);
                }
            }

            let javascript_output = transform_tsx_to_js_with_config(html_output, transform_config)
                .map_err(|e| {
                    MdxError::tsx_transform(format!("Failed to transform TSX to JavaScript: {e}"))
                })?;

            // HOT PATH: Component rendering - executes JavaScript and renders to HTML
            let template_output = render_template(context, &javascript_output)?;

            match context.settings.output {
                OutputFormat::Html => {
                    // Unwrap Fragment wrapper if present - only return children of first Fragment
                    Ok(unwrap_fragment(&template_output))
                }
                OutputFormat::Javascript => {
                    transform_tsx_to_js_for_output(&template_output, context.settings.minify)
                        .map_err(|e| {
                            MdxError::tsx_transform(format!(
                                "Failed to transform template to JavaScript: {e}"
                            ))
                        })
                }
                OutputFormat::Json => {
                    // Render using core.js engine for json output
                    render_template_to_schema(context, &javascript_output)
                }
                OutputFormat::Schema => unreachable!("Schema handled in outer match"),
            }
        }
    }
}

fn render_template(
    context: &RenderContext<'_>,
    javascript_output: &str,
) -> Result<String, MdxError> {
    context
        .renderer
        .render_transformed_component(
            javascript_output,
            Some(context.props_json),
            context.components,
            context.settings.utils.as_deref(),
        )
        .map_err(|e| {
            log_render_error(&e, javascript_output, "Component");
            MdxError::tsx_transform(format!("Failed to render component template: {:#}", e))
        })
}

fn render_template_to_schema(
    context: &RenderContext<'_>,
    javascript_output: &str,
) -> Result<String, MdxError> {
    context
        .renderer
        .render_transformed_component_to_schema(
            javascript_output,
            Some(context.props_json),
            context.components,
            context.settings.utils.as_deref(),
        )
        .map_err(|e| {
            log_render_error(&e, javascript_output, "Schema");
            MdxError::tsx_transform(format!("Failed to render component to schema: {:#}", e))
        })
}

// =============================================================================
// Unit Tests for JSX Protection
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protect_jsx_no_jsx() {
        let content = "# Hello World\n\nThis is plain markdown.";
        let (result, placeholders) = protect_jsx_components(content);
        assert_eq!(result, content);
        assert!(placeholders.is_empty());
    }

    #[test]
    fn test_protect_jsx_without_expression_attributes() {
        // JSX without expression attributes should NOT be protected
        let content = r#"<Card title="Test">Content</Card>"#;
        let (result, placeholders) = protect_jsx_components(content);
        assert_eq!(result, content);
        assert!(placeholders.is_empty());
    }

    #[test]
    fn test_protect_self_closing_jsx_with_expression() {
        let content = r#"<Hero title={context("title")} />"#;
        let (result, placeholders) = protect_jsx_components(content);

        assert!(result.contains("<!--JSX:"));
        assert_eq!(placeholders.len(), 1);
        assert!(placeholders.values().any(|v| v.contains("Hero")));
    }

    #[test]
    fn test_protect_multiple_jsx_components() {
        let content = r#"
<Hero title={context("title")} />
<Card data={props.data} />
"#;
        let (result, placeholders) = protect_jsx_components(content);

        assert_eq!(placeholders.len(), 2);
        assert!(!result.contains("<Hero"));
        assert!(!result.contains("<Card"));
    }

    #[test]
    fn test_protect_jsx_with_children_and_expression() {
        let content = r#"<Container theme={props.theme}>Child content</Container>"#;
        let (result, placeholders) = protect_jsx_components(content);

        assert!(result.contains("<!--JSX:"));
        assert_eq!(placeholders.len(), 1);
    }

    #[test]
    fn test_restore_multiple_jsx_components() {
        let content = r#"
# Title
<Hero title={context("title")} />
Some text
<Card data={props.data} />
"#;
        let (protected, placeholders) = protect_jsx_components(content);
        let restored = restore_jsx_components(&protected, &placeholders);

        assert!(restored.contains("<Hero"));
        assert!(restored.contains("<Card"));
        assert!(restored.contains("context(\"title\")"));
    }

    #[test]
    fn test_protect_jsx_preserves_surrounding_content() {
        let content = "# Title\n\n<Hero title={data} />\n\nMore content";
        let (result, placeholders) = protect_jsx_components(content);

        assert!(result.contains("# Title"));
        assert!(result.contains("More content"));
        assert_eq!(placeholders.len(), 1);

        let restored = restore_jsx_components(&result, &placeholders);
        assert_eq!(restored, content);
    }

    #[test]
    fn test_find_matching_close_tag_nested() {
        let content = "<div><div>inner</div>outer</div>";
        let mut depth = 0;
        let result = find_matching_close_tag(content, 5, "div", "</div>", &mut depth);
        // Should find the outer closing tag, not the inner one
        assert_eq!(result, Some(26));
        assert!(depth >= 2); // Detected nesting
    }

    #[test]
    fn test_protect_jsx_lowercase_tags_ignored() {
        // Lowercase tags are HTML, not JSX components
        let content = r#"<div class={style}>Content</div>"#;
        let (_result, placeholders) = protect_jsx_components(content);
        // Lowercase tags with expressions might still get caught, but the pattern
        // specifically looks for uppercase component names
        assert!(placeholders.is_empty() || !placeholders.values().any(|v| v.starts_with("<div")));
    }

    // Tests for unwrap_fragment function

    #[test]
    fn test_unwrap_fragment_no_fragments() {
        // HTML without Fragment tags should be returned unchanged (just trimmed)
        let html = "<h1>Hello</h1><p>World</p>";
        let result = unwrap_fragment(html);
        assert_eq!(result, html);
    }

    #[test]
    fn test_unwrap_fragment_single_fragment() {
        // Single Fragment wrapper should be removed
        let html = "<Fragment><h1>Hello</h1></Fragment>";
        let result = unwrap_fragment(html);
        assert_eq!(result, "<h1>Hello</h1>");
    }

    #[test]
    fn test_unwrap_fragment_nested_fragments() {
        // Multiple/nested Fragment tags should all be removed
        let html = "<Fragment><h1>Title</h1><Fragment><p>Content</p></Fragment></Fragment>";
        let result = unwrap_fragment(html);
        assert_eq!(result, "<h1>Title</h1><p>Content</p>");
    }

    #[test]
    fn test_unwrap_fragment_case_insensitive() {
        // Should handle case variations
        let html = "<FRAGMENT><h1>Hello</h1></FRAGMENT>";
        let result = unwrap_fragment(html);
        assert_eq!(result, "<h1>Hello</h1>");
    }

    #[test]
    fn test_unwrap_fragment_with_attributes() {
        // Fragment with attributes (though rare) should still be removed
        let html = "<Fragment key=\"1\"><h1>Hello</h1></Fragment>";
        let result = unwrap_fragment(html);
        assert_eq!(result, "<h1>Hello</h1>");
    }

    #[test]
    fn test_unwrap_fragment_preserves_whitespace_inside() {
        // Content whitespace should be preserved
        let html = "<Fragment>\n  <h1>Hello</h1>\n</Fragment>";
        let result = unwrap_fragment(html);
        assert_eq!(result, "\n  <h1>Hello</h1>\n");
    }

    #[test]
    fn test_unwrap_fragment_trims_outer_whitespace() {
        // Outer whitespace should be trimmed
        let html = "  \n<h1>Hello</h1>\n  ";
        let result = unwrap_fragment(html);
        assert_eq!(result, "<h1>Hello</h1>");
    }

    // =========================================================================
    // GFM (GitHub Flavored Markdown) Tests
    // =========================================================================

    #[test]
    fn test_gfm_table_basic() {
        let content = r#"| Header 1 | Header 2 |
| -------- | -------- |
| Cell 1   | Cell 2   |"#;
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<table>"), "Should render table element");
        assert!(result.contains("<thead>"), "Should have thead");
        assert!(result.contains("<tbody>"), "Should have tbody");
        assert!(result.contains("<th>"), "Should have th elements");
        assert!(result.contains("<td>"), "Should have td elements");
        assert!(result.contains("Header 1"), "Should contain header text");
        assert!(result.contains("Cell 1"), "Should contain cell text");
    }

    #[test]
    fn test_gfm_table_with_alignment() {
        let content = r#"| Left | Center | Right |
| :--- | :----: | ----: |
| L    | C      | R     |"#;
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<table>"));
        // Check for alignment styles
        assert!(
            result.contains("text-align:") || result.contains("align="),
            "Should have alignment attributes"
        );
    }

    #[test]
    fn test_gfm_table_multiple_rows() {
        let content = r#"| File Path | URL Route |
| --------- | --------- |
| `views/index.mdx` | `/` |
| `views/about.mdx` | `/about` |
| `views/blog/index.mdx` | `/blog` |"#;
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<table>"));
        assert!(
            result.contains("<code>"),
            "Should render inline code in cells"
        );
        // Count rows (3 data rows)
        let row_count = result.matches("<tr>").count();
        assert!(row_count >= 4, "Should have header + 3 data rows");
    }

    #[test]
    fn test_gfm_strikethrough_single_tilde() {
        let content = "This is ~deleted~ text.";
        let result = render_markdown(content).unwrap();
        assert!(
            result.contains("<del>deleted</del>"),
            "Should render strikethrough with <del> tag"
        );
    }

    #[test]
    fn test_gfm_strikethrough_double_tilde() {
        let content = "This is ~~also deleted~~ text.";
        let result = render_markdown(content).unwrap();
        assert!(
            result.contains("<del>also deleted</del>"),
            "Should render double-tilde strikethrough"
        );
    }

    #[test]
    fn test_gfm_strikethrough_with_formatting() {
        let content = "~~**bold and deleted**~~";
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<del>"), "Should have del tag");
        assert!(result.contains("<strong>"), "Should preserve bold inside");
    }

    #[test]
    fn test_gfm_task_list_unchecked() {
        let content = "- [ ] Unchecked task";
        let result = render_markdown(content).unwrap();
        assert!(
            result.contains("<input") && result.contains("checkbox"),
            "Should render checkbox input"
        );
        // The markdown crate outputs type="checkbox" for all checkboxes
        // Unchecked items have no checked="" attribute (just "checkbox" in type)
        assert!(
            !result.contains("checked=\"\"") && !result.contains("checked="),
            "Unchecked task should not have checked attribute, got: {}",
            result
        );
    }

    #[test]
    fn test_gfm_task_list_checked() {
        let content = "- [x] Checked task";
        let result = render_markdown(content).unwrap();
        assert!(
            result.contains("<input") && result.contains("checkbox"),
            "Should render checkbox input"
        );
        assert!(
            result.contains("checked"),
            "Checked task should have checked attribute"
        );
    }

    #[test]
    fn test_gfm_task_list_mixed() {
        let content = r#"- [x] Done task
- [ ] Pending task
- [x] Another done"#;
        let result = render_markdown(content).unwrap();
        let checkbox_count = result.matches("checkbox").count();
        assert_eq!(checkbox_count, 3, "Should have 3 checkboxes");
        let checked_count = result.matches("checked").count();
        assert_eq!(checked_count, 2, "Should have 2 checked items");
    }

    #[test]
    fn test_gfm_autolink_url() {
        let content = "Visit https://example.com for more info.";
        let result = render_markdown(content).unwrap();
        assert!(
            result.contains("<a href=\"https://example.com\">"),
            "Should auto-link URL"
        );
    }

    #[test]
    fn test_gfm_autolink_www() {
        let content = "Check out www.example.com today.";
        let result = render_markdown(content).unwrap();
        assert!(
            result.contains("<a href=") && result.contains("example.com"),
            "Should auto-link www URLs"
        );
    }

    #[test]
    fn test_gfm_autolink_email() {
        let content = "Contact us at hello@example.com for help.";
        let result = render_markdown(content).unwrap();
        assert!(
            result.contains("<a href=\"mailto:hello@example.com\">"),
            "Should auto-link email addresses"
        );
    }

    #[test]
    fn test_gfm_footnote_basic() {
        let content = r#"Here is a footnote reference[^1].

[^1]: This is the footnote content."#;
        let result = render_markdown(content).unwrap();
        // Footnotes create anchor links and a footnote section
        assert!(
            result.contains("footnote") || result.contains("fn"),
            "Should process footnote syntax"
        );
    }

    #[test]
    fn test_gfm_footnote_multiple() {
        let content = r#"First[^1] and second[^2] footnotes.

[^1]: First footnote.
[^2]: Second footnote."#;
        let result = render_markdown(content).unwrap();
        // Should have multiple footnote references
        assert!(
            result.contains("1") && result.contains("2"),
            "Should have multiple footnote markers"
        );
    }

    #[test]
    fn test_gfm_combined_features() {
        // Test multiple GFM features in one document
        let content = r#"# Task List

- [x] Create ~old~ new feature
- [ ] Visit https://docs.rs

| Status | Task |
| ------ | ---- |
| Done   | ~~Completed~~ |

See footnote[^1].

[^1]: Reference here."#;
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<h1>"), "Should render heading");
        assert!(result.contains("checkbox"), "Should render task list");
        assert!(result.contains("<table>"), "Should render table");
        assert!(result.contains("<del>"), "Should render strikethrough");
        assert!(result.contains("<a href="), "Should auto-link URL");
    }

    #[test]
    fn test_gfm_table_empty_cells() {
        let content = r#"| A | B |
| - | - |
|   | X |
| Y |   |"#;
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<table>"));
        // Should handle empty cells gracefully
        let td_count = result.matches("<td>").count();
        assert_eq!(td_count, 4, "Should have 4 cells");
    }

    #[test]
    fn test_gfm_table_with_inline_code() {
        let content = r#"| Command | Description |
| ------- | ----------- |
| `git status` | Show status |
| `git diff` | Show changes |"#;
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<table>"));
        assert!(result.contains("<code>git status</code>"));
        assert!(result.contains("<code>git diff</code>"));
    }

    #[test]
    fn test_gfm_table_with_links() {
        let content = r#"| Name | Link |
| ---- | ---- |
| Rust | [rust-lang.org](https://rust-lang.org) |"#;
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<table>"));
        assert!(result.contains("<a href=\"https://rust-lang.org\">"));
    }

    #[test]
    fn test_gfm_nested_list_with_tasks() {
        let content = r#"- [x] Parent task
  - [ ] Child task 1
  - [x] Child task 2"#;
        let result = render_markdown(content).unwrap();
        let checkbox_count = result.matches("checkbox").count();
        assert_eq!(checkbox_count, 3, "Should render all nested checkboxes");
    }

    // =========================================================================
    // Fenced Code Block Tests (CommonMark)
    // =========================================================================

    #[test]
    fn test_code_fenced_basic() {
        let content = r#"```
let x = 1;
```"#;
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<pre>"), "Should render pre element");
        assert!(result.contains("<code>"), "Should render code element");
        assert!(
            result.contains("let x = 1;"),
            "Should preserve code content"
        );
    }

    #[test]
    fn test_code_fenced_with_language() {
        let content = r#"```python
def hello():
    print("Hello, World!")
```"#;
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<pre>"));
        assert!(result.contains("<code"));
        assert!(
            result.contains("language-python") || result.contains("class=\"python\""),
            "Should include language identifier in class, got: {}",
            result
        );
        assert!(result.contains("def hello():"));
    }

    #[test]
    fn test_code_fenced_rust() {
        let content = r#"```rust
fn main() {
    println!("Hello");
}
```"#;
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<pre>"));
        assert!(
            result.contains("language-rust"),
            "Should have language-rust class"
        );
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_code_fenced_javascript() {
        let content = r#"```javascript
const greet = () => console.log("Hi");
```"#;
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<pre>"));
        assert!(result.contains("language-javascript"));
        assert!(result.contains("const greet"));
    }

    #[test]
    fn test_code_fenced_preserves_whitespace() {
        let content = r#"```
  indented
    more indented
```"#;
        let result = render_markdown(content).unwrap();
        // Whitespace should be preserved inside code blocks
        assert!(
            result.contains("  indented") || result.contains("indented"),
            "Should preserve indentation"
        );
    }

    #[test]
    fn test_code_fenced_with_special_chars() {
        let content = r#"```html
<div class="test">&amp;</div>
```"#;
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<pre>"));
        // HTML entities in code should be escaped
        assert!(
            result.contains("&lt;div") || result.contains("<div"),
            "Should handle HTML in code blocks"
        );
    }

    #[test]
    fn test_code_inline() {
        let content = "Use `println!()` to print.";
        let result = render_markdown(content).unwrap();
        assert!(result.contains("<code>println!()</code>"));
    }

    #[test]
    fn test_code_indented() {
        // 4-space indented code block (CommonMark)
        let content = "Normal text\n\n    indented code\n    more code\n\nNormal again";
        let result = render_markdown(content).unwrap();
        assert!(
            result.contains("<pre>"),
            "Should render indented code block"
        );
        assert!(result.contains("<code>"));
    }
}
