//! Data structures for MDX processing
//!
//! This module defines the core data structures used throughout the library:
//!
//! - **Component Definitions**: Component code and metadata
//! - **Render Settings**: Output format and minification options
//! - **Resource Limits**: Configuration for preventing resource exhaustion
//! - **Batch Input/Output**: Structures for batch rendering operations
//!
//! ## Resource Limits
//!
//! Resource limits are enforced at the library level to prevent memory exhaustion.
//! These are reliability measures, not security controls (security is handled at the web layer).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Component definition with code and metadata
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComponentDefinition {
    /// Component name (optional, defaults to HashMap key)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Component documentation (metadata)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
    /// Component arguments/props types (metadata)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
    /// Component code (JSX/TSX)
    pub code: String,
}

/// Output format options
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Return HTML template
    #[default]
    Html,
    /// Return JavaScript (transform template back to JS)
    Javascript,
    /// Return JSON schema representation (same as Json)
    Schema,
    /// Return JSON schema representation (alias for Schema)
    #[serde(alias = "json")]
    Json,
}

/// Rendering settings
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RenderSettings {
    /// Output format (html, javascript, schema, or json)
    #[serde(default)]
    pub output: OutputFormat,
    /// Enable minification
    #[serde(default = "default_minify_true")]
    pub minify: bool,
}

const fn default_minify_true() -> bool {
    true
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            output: OutputFormat::default(),
            minify: true,
        }
    }
}

/// Input structure for batch MDX rendering requests
#[derive(Deserialize, Serialize)]
pub struct NamedMdxBatchInput {
    /// Rendering settings
    #[serde(default)]
    pub settings: RenderSettings,
    /// Map of file names to MDX content strings
    pub mdx: HashMap<String, String>,
    /// Optional map of component names to their definitions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<HashMap<String, ComponentDefinition>>,
}

/// Output structure containing rendered output and metadata
#[derive(Serialize, Deserialize, Debug)]
pub struct RenderedMdx {
    /// Parsed YAML frontmatter metadata
    pub metadata: serde_json::Value,
    /// Rendered output content (HTML or JavaScript depending on output format)
    ///
    /// This field always contains the rendered result, regardless of output format.
    /// The format is determined by the `output` setting in the render request:
    /// - `OutputFormat::Html` → HTML string
    /// - `OutputFormat::Javascript` → JavaScript code
    /// - `OutputFormat::Schema` → JavaScript code after TSX transformation (before rendering)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// Resource limits for preventing resource exhaustion.
///
/// These limits are enforced at the library level to prevent memory exhaustion
/// and ensure reliable operation. They are not HTTP security controls, but rather
/// internal reliability measures.
#[derive(Clone, Debug)]
pub struct ResourceLimits {
    /// Maximum number of files in a batch request
    pub max_batch_size: usize,
    /// Maximum MDX content size per file (in bytes)
    pub max_mdx_content_size: usize,
    /// Maximum component code size (in bytes)
    pub max_component_code_size: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            max_mdx_content_size: 10 * 1024 * 1024, // 10 MB
            max_component_code_size: 1024 * 1024,   // 1 MB
        }
    }
}

impl ResourceLimits {
    /// Validates resource limits and returns an error if invalid.
    ///
    /// # Returns
    /// `Ok(())` if limits are valid, `Err` with a descriptive message if invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_batch_size == 0 {
            return Err("max_batch_size must be greater than 0".to_string());
        }

        if self.max_mdx_content_size == 0 {
            return Err("max_mdx_content_size must be greater than 0".to_string());
        }

        if self.max_component_code_size == 0 {
            return Err("max_component_code_size must be greater than 0".to_string());
        }

        // Enforce maximum recommended limits to prevent memory exhaustion
        const MAX_RECOMMENDED_BATCH_SIZE: usize = 100_000;
        if self.max_batch_size > MAX_RECOMMENDED_BATCH_SIZE {
            return Err(format!(
                "max_batch_size ({}) exceeds recommended maximum of {}",
                self.max_batch_size, MAX_RECOMMENDED_BATCH_SIZE
            ));
        }

        const MAX_RECOMMENDED_MDX_CONTENT_SIZE: usize = 100 * 1024 * 1024; // 100 MB
        if self.max_mdx_content_size > MAX_RECOMMENDED_MDX_CONTENT_SIZE {
            return Err(format!(
                "max_mdx_content_size ({}) exceeds recommended maximum of {} bytes (100 MB)",
                self.max_mdx_content_size, MAX_RECOMMENDED_MDX_CONTENT_SIZE
            ));
        }

        Ok(())
    }
}

/// Configuration for TSX transformation
pub struct TsxTransformConfig {
    /// JSX pragma function name (e.g., "engine.h" or "h")
    pub jsx_pragma: String,
    /// JSX fragment pragma (e.g., "engine.Fragment" or "Fragment")
    pub jsx_pragma_frag: String,
    /// Whether to minify the output JavaScript
    pub minify: bool,
    /// Component names to convert from function references to strings (for schema output)
    pub component_names: Option<std::collections::HashSet<String>>,
}

impl Default for TsxTransformConfig {
    fn default() -> Self {
        Self {
            jsx_pragma: "engine.h".to_string(),
            jsx_pragma_frag: "engine.Fragment".to_string(),
            minify: false,
            component_names: None,
        }
    }
}

impl TsxTransformConfig {
    /// Configuration for final output transformation (uses `h` instead of `engine.h`)
    pub fn for_output(minify: bool) -> Self {
        Self {
            jsx_pragma: "h".to_string(),
            jsx_pragma_frag: "Fragment".to_string(),
            minify,
            component_names: None,
        }
    }

    /// Helper that clones the default config but toggles minification.
    pub fn for_engine(minify: bool) -> Self {
        Self {
            minify,
            component_names: None,
            ..Self::default()
        }
    }
}
