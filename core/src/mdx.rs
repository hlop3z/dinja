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
use serde_json::json;
use std::collections::HashMap;

struct RenderContext<'a> {
    renderer: &'a JsRenderer,
    components: Option<&'a HashMap<String, ComponentDefinition>>,
    props_json: &'a str,
    settings: &'a RenderSettings,
}

fn markdown_options() -> Options {
    Options {
        parse: ParseOptions {
            constructs: Constructs {
                html_flow: true, // Allow block-level HTML/JSX
                html_text: true, // Allow inline HTML/JSX
                ..Constructs::default()
            },
            ..ParseOptions::default()
        },
        compile: CompileOptions {
            allow_dangerous_html: true, // Don't escape HTML tags
            ..CompileOptions::default()
        },
    }
}

/// Unwraps the first Fragment wrapper from HTML output if present.
///
/// If the HTML starts with `<Fragment>` and ends with `</Fragment>`, this function
/// extracts only the children content, removing the Fragment wrapper.
///
/// This handles the case where MDX content is wrapped in a Fragment by default,
/// and we only want to return the actual content without the wrapper.
///
/// # Arguments
/// * `html` - HTML string that may be wrapped in a Fragment
///
/// # Returns
/// HTML string with Fragment wrapper removed if present, otherwise unchanged
fn unwrap_fragment(html: &str) -> String {
    let trimmed = html.trim();

    // Check if the HTML starts with <Fragment (case-insensitive, allowing attributes)
    let fragment_start_patterns = ["<Fragment", "<fragment"];
    let mut fragment_start: Option<usize> = None;

    for pattern in &fragment_start_patterns {
        if let Some(pos) = trimmed.find(pattern) {
            fragment_start = Some(pos);
            break;
        }
    }

    if let Some(start) = fragment_start {
        // Find the closing > of the opening tag (handle self-closing or with attributes)
        if let Some(tag_end) = trimmed[start..].find('>') {
            let tag_end = start + tag_end + 1;
            let content_start = tag_end;

            // Find the closing </Fragment> tag (case-insensitive)
            let remaining = &trimmed[content_start..];
            let fragment_end_patterns = ["</Fragment>", "</fragment>"];
            let mut fragment_end: Option<usize> = None;

            for pattern in &fragment_end_patterns {
                if let Some(pos) = remaining.rfind(pattern) {
                    fragment_end = Some(pos);
                    break;
                }
            }

            if let Some(end_pos) = fragment_end {
                // Extract just the content between the tags
                return remaining[..end_pos].trim().to_string();
            }
        }
    }

    // No Fragment wrapper found, return as-is
    html.to_string()
}

fn render_markdown(content: &str) -> Result<String, MdxError> {
    let options = markdown_options();
    to_html_with_options(content, &options).map_err(|e| MdxError::MarkdownRender(e.to_string()))
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

fn render_with_engine_pipeline(
    context: &RenderContext<'_>,
    html_output: &str,
) -> Result<String, MdxError> {
    // HOT PATH: TSX transformation - called for every MDX file with Html/Javascript output
    let mut transform_config = TsxTransformConfig::for_engine(false);

    // For schema/json output, convert component function references to strings
    // For HTML output, keep as function references so they can be rendered
    // For JavaScript output, keep Preact syntax with h() and Fragment
    if matches!(context.settings.output, OutputFormat::Schema | OutputFormat::Json) {
        if let Some(components) = context.components {
            let component_names: std::collections::HashSet<String> = components
                .iter()
                .map(|(key, comp_def)| {
                    comp_def
                        .name
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| key.clone())
                })
                .collect();
            if !component_names.is_empty() {
                transform_config.component_names = Some(component_names);
            }
        }
    }

    let javascript_output = transform_tsx_to_js_with_config(html_output, transform_config)
        .map_err(|e| {
            MdxError::TsxTransform(format!("Failed to transform TSX to JavaScript: {e}"))
        })?;
    eprintln!("[DEBUG] TSX: {}", javascript_output.chars().take(150).collect::<String>());

    // HOT PATH: Component rendering - executes JavaScript and renders to HTML
    let template_output = render_template(context, &javascript_output)?;
    eprintln!("[DEBUG] Result: {}", template_output.chars().take(150).collect::<String>());

    match context.settings.output {
        OutputFormat::Html => {
            // Unwrap Fragment wrapper if present - only return children of first Fragment
            Ok(unwrap_fragment(&template_output))
        }
        OutputFormat::Javascript => {
            transform_tsx_to_js_for_output(&template_output, context.settings.minify).map_err(|e| {
                MdxError::TsxTransform(format!("Failed to transform template to JavaScript: {e}"))
            })
        }
        OutputFormat::Schema | OutputFormat::Json => {
            // Render using core.js engine for schema/json output
            render_template_to_schema(context, &javascript_output)
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
        )
        .map_err(|e| {
            log_render_error(&e, javascript_output, "Component");
            MdxError::TsxTransform(format!("Failed to render component template: {:#}", e))
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
        )
        .map_err(|e| {
            log_render_error(&e, javascript_output, "Schema");
            MdxError::TsxTransform(format!("Failed to render component to schema: {:#}", e))
        })
}
