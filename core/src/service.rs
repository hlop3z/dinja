//! High-level rendering service with pooling, resource limits, and batch reporting.
//!
//! This module provides the main entry point for MDX rendering operations. It manages
//! a pool of JavaScript renderers, enforces resource limits, and provides batch processing
//! capabilities.
//!
//! ## Module Size Note
//!
//! This module is currently ~593 lines. While slightly over the ~500 line guideline,
//! the code is well-organized into cohesive sections (configuration, service, errors, outcomes).
//! Consider splitting into submodules if it grows beyond ~700 lines or if new major features
//! are added that don't fit the current structure.
//!
//! ## Architecture
//!
//! The `RenderService` coordinates between several components:
//! - **Renderer Pool**: Thread-local cache of initialized JavaScript runtimes
//! - **Resource Limits**: Prevents memory exhaustion from large batches or content
//! - **Batch Processing**: Renders multiple MDX files in a single operation
//!
//! ## Thread Safety
//!
//! `RenderService` is `Clone` and can be shared across threads. However, the underlying
//! renderer pool uses thread-local storage, so each thread maintains its own cache of renderers.
//!
//! ## Configuration
//!
//! Configuration can be loaded from environment variables or provided programmatically.
//! Use `RenderServiceConfig::from_env()` for environment-based configuration or construct
//! `RenderServiceConfig` directly for programmatic configuration.
//!
//! ## Example
//!
//! ```no_run
//! use dinja_core::service::{RenderService, RenderServiceConfig};
//! use dinja_core::models::{NamedMdxBatchInput, RenderSettings, OutputFormat};
//! use std::collections::HashMap;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RenderServiceConfig::from_env();
//! let service = RenderService::new(config)?;
//!
//! let mut mdx_files = HashMap::new();
//! mdx_files.insert("page1.mdx".to_string(), "# Hello World".to_string());
//!
//! let input = NamedMdxBatchInput {
//!     settings: RenderSettings {
//!         output: OutputFormat::Html,
//!         minify: true,
//!     },
//!     mdx: mdx_files,
//!     components: None,
//! };
//!
//! let outcome = service.render_batch(&input)?;
//! # Ok(())
//! # }
//! ```
use crate::mdx::{create_error_response, mdx_to_html_with_frontmatter};
use crate::models::{
    ComponentDefinition, NamedMdxBatchInput, OutputFormat, RenderedMdx,
    ResourceLimits,
};
use crate::renderer::pool::{RendererPool, RendererProfile};
use anyhow::Error as AnyhowError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
#[cfg(feature = "http")]
use std::fs;
#[cfg(feature = "http")]
use std::path::Path;
use std::path::PathBuf;

const ENV_STATIC_DIR: &str = "RUST_CMS_STATIC_DIR";

/// Configuration for the rendering service.
#[derive(Clone, Debug)]
pub struct RenderServiceConfig {
    /// Directory containing static JavaScript files (e.g., engine.min.js)
    pub static_dir: PathBuf,
    /// Maximum number of cached renderers per profile
    pub max_cached_renderers: usize,
    /// Resource limits for preventing resource exhaustion
    pub resource_limits: ResourceLimits,
}

impl Default for RenderServiceConfig {
    fn default() -> Self {
        Self {
            static_dir: PathBuf::from("static"),
            max_cached_renderers: 4,
            resource_limits: ResourceLimits::default(),
        }
    }
}

/// TOML configuration structure for file-based configuration
#[cfg(feature = "http")]
#[derive(Deserialize, Debug)]
struct TomlConfig {
    static_dir: Option<String>,
    max_cached_renderers: Option<usize>,
    resource_limits: Option<TomlResourceLimits>,
}

#[cfg(feature = "http")]
#[derive(Deserialize, Debug)]
struct TomlResourceLimits {
    max_batch_size: Option<usize>,
    max_mdx_content_size: Option<usize>,
    max_component_code_size: Option<usize>,
}

impl RenderServiceConfig {
    /// Loads configuration from environment variables, falling back to defaults.
    pub fn from_env() -> Self {
        let mut config = Self::default();
        if let Ok(path) = env::var(ENV_STATIC_DIR) {
            config.static_dir = PathBuf::from(path);
        }
        config
    }

    /// Loads configuration from a TOML file.
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    /// `Ok(RenderServiceConfig)` if the file was successfully loaded and parsed,
    /// `Err` with a descriptive error message if the file cannot be read or parsed.
    ///
    /// # Note
    /// This method requires the `http` feature to be enabled.
    #[cfg(feature = "http")]
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|e| {
            format!(
                "Failed to read configuration file {}: {}",
                path.display(),
                e
            )
        })?;

        let toml_config: TomlConfig = toml::from_str(&contents).map_err(|e| {
            format!(
                "Failed to parse TOML configuration file {}: {}",
                path.display(),
                e
            )
        })?;

        let mut config = Self::default();

        if let Some(static_dir) = toml_config.static_dir {
            config.static_dir = PathBuf::from(static_dir);
        }

        if let Some(max_cached) = toml_config.max_cached_renderers {
            config.max_cached_renderers = max_cached;
        }

        if let Some(limits) = toml_config.resource_limits {
            if let Some(max_batch_size) = limits.max_batch_size {
                config.resource_limits.max_batch_size = max_batch_size;
            }
            if let Some(max_mdx_content_size) = limits.max_mdx_content_size {
                config.resource_limits.max_mdx_content_size = max_mdx_content_size;
            }
            if let Some(max_component_code_size) = limits.max_component_code_size {
                config.resource_limits.max_component_code_size = max_component_code_size;
            }
        }

        Ok(config)
    }

    /// Loads configuration from a TOML file and merges with environment variables.
    ///
    /// Environment variables override file settings. This allows for deployment-specific
    /// overrides while maintaining a base configuration in a file.
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    /// `Ok(RenderServiceConfig)` if the file was successfully loaded and parsed,
    /// `Err` with a descriptive error message if the file cannot be read or parsed.
    ///
    /// # Note
    /// This method requires the `http` feature to be enabled.
    #[cfg(feature = "http")]
    pub fn from_file_and_env(path: impl AsRef<Path>) -> Result<Self, String> {
        let mut config = Self::from_file(path)?;

        // Environment variables override file settings
        if let Ok(path) = env::var(ENV_STATIC_DIR) {
            config.static_dir = PathBuf::from(path);
        }

        Ok(config)
    }

    /// Validates the configuration and returns an error if invalid.
    ///
    /// # Returns
    /// `Ok(())` if configuration is valid, `Err` with a descriptive message if invalid.
    pub fn validate(&self) -> Result<(), String> {
        // Validate static directory exists
        if !self.static_dir.exists() {
            return Err(format!(
                "Static directory does not exist: {}",
                self.static_dir.display()
            ));
        }
        if !self.static_dir.is_dir() {
            return Err(format!(
                "Static directory path is not a directory: {}",
                self.static_dir.display()
            ));
        }

        // Validate max_cached_renderers is reasonable
        if self.max_cached_renderers == 0 {
            return Err("max_cached_renderers must be greater than 0".to_string());
        }
        if self.max_cached_renderers > 1000 {
            return Err(format!(
                "max_cached_renderers ({}) is unreasonably large, maximum recommended is 1000",
                self.max_cached_renderers
            ));
        }

        // Validate resource limits
        self.resource_limits.validate()?;

        Ok(())
    }
}

/// Top-level service that batches MDX rendering requests.
///
/// This service manages a pool of JavaScript renderers and provides batch rendering
/// capabilities for MDX content. It handles resource limits, error recovery, and
/// renderer lifecycle management.
#[derive(Clone)]
pub struct RenderService {
    config: RenderServiceConfig,
    pool: RendererPool,
}

impl RenderService {
    /// Creates a new render service with the given configuration.
    ///
    /// Configuration is always validated, even in release builds.
    ///
    /// # Arguments
    /// * `config` - Service configuration including static directory and resource limits
    ///
    /// # Returns
    /// `Ok(RenderService)` if configuration is valid, `Err` with validation error if invalid
    pub fn new(config: RenderServiceConfig) -> Result<Self, String> {
        config.validate()?;
        let pool = RendererPool::new(config.static_dir.clone(), config.max_cached_renderers);
        // Warm up the pool with one renderer per common profile to reduce first-request latency
        // Skip warming when RUST_CMS_SKIP_POOL_WARMING is set (useful for tests)
        if env::var("RUST_CMS_SKIP_POOL_WARMING").is_err() {
            pool.warm(1);
        }
        Ok(Self { config, pool })
    }

    /// Creates a new render service with configuration validation.
    ///
    /// This is an alias for `new()` which always validates configuration.
    /// Kept for backward compatibility.
    ///
    /// # Arguments
    /// * `config` - Service configuration including static directory and resource limits
    ///
    /// # Returns
    /// `Ok(RenderService)` if configuration is valid, `Err` with validation error if invalid
    pub fn new_with_validation(config: RenderServiceConfig) -> Result<Self, String> {
        Self::new(config)
    }

    /// Returns a reference to the service configuration.
    pub fn config(&self) -> &RenderServiceConfig {
        &self.config
    }

    /// Returns a reference to the renderer pool.
    ///
    /// This is primarily useful for testing and advanced use cases.
    pub fn pool(&self) -> &RendererPool {
        &self.pool
    }

    /// Renders a batch of MDX files.
    ///
    /// ## Error Recovery Strategy
    ///
    /// This function implements graceful error recovery: individual file failures don't stop
    /// the batch processing. Errors are collected and returned in the `BatchRenderOutcome`,
    /// allowing partial success scenarios. This is important for batch operations where
    /// some files may be invalid while others are valid.
    ///
    /// ## Public API Error Type Consistency
    ///
    /// This function returns `Result<BatchRenderOutcome, RenderBatchError>`, which is consistent
    /// with the service boundary pattern:
    /// - Domain functions return `Result<T, MdxError>` (domain-specific errors)
    /// - Service functions return `Result<T, RenderBatchError>` (service-level errors)
    /// - HTTP handlers convert to `anyhow::Error` (framework-level errors)
    ///
    /// # Arguments
    /// * `input` - Batch input containing MDX content, components, and settings
    ///
    /// # Returns
    /// A `BatchRenderOutcome` containing rendered files and any errors
    ///
    /// # Errors
    /// Returns `RenderBatchError` if resource limits are exceeded, custom engines are
    /// disabled but requested, or internal errors occur during rendering.
    pub fn render_batch(
        &self,
        input: &NamedMdxBatchInput,
    ) -> Result<BatchRenderOutcome, RenderBatchError> {
        // Use components from input directly
        let resolved_components = input.components.as_ref();

        // Validate resource limits
        self.validate_resource_limits(input, resolved_components)?;

        let profile = self.profile_for_request(&input.settings.output)?;

        if input.mdx.is_empty() {
            return Ok(BatchRenderOutcome::empty());
        }

        let renderer = self
            .pool
            .checkout(profile)
            .map_err(RenderBatchError::Internal)?;

        let mut files = HashMap::with_capacity(input.mdx.len());
        // Pre-allocate errors Vec with estimated capacity (assume ~10% failure rate)
        // This denominator represents the expected success rate: 1/10 = 10% failure rate
        // Pre-allocating prevents multiple reallocations during batch processing
        const ESTIMATED_ERROR_RATE_DENOMINATOR: usize = 10;
        let mut errors = Vec::with_capacity(input.mdx.len() / ESTIMATED_ERROR_RATE_DENOMINATOR);
        let mut succeeded = 0usize;
        let mut failed = 0usize;

        // HOT PATH: Batch processing loop - processes multiple MDX files sequentially
        // Error recovery: Individual file failures don't stop the batch; errors are collected
        // and returned in the outcome. This allows partial success scenarios.
        for (name, mdx_source) in &input.mdx {
            let renderer_ref = renderer
                .renderer()
                .map_err(|e| RenderBatchError::Internal(anyhow::Error::from(e)))?;
            match mdx_to_html_with_frontmatter(
                mdx_source,
                renderer_ref,
                resolved_components,
                &input.settings,
            ) {
                Ok(rendered) => {
                    succeeded += 1;
                    files.insert(name.clone(), FileRenderOutcome::success(rendered));
                }
                Err(err) => {
                    failed += 1;
                    // Convert MdxError to anyhow::Error for error response creation
                    // Using `anyhow::Error::from()` preserves the error chain automatically
                    // since MdxError implements std::error::Error via thiserror
                    let anyhow_err = anyhow::Error::from(err);
                    // Preserve full error context including chain using {:#} format
                    // This includes all underlying causes in the error chain
                    let message = format!("{:#}", anyhow_err);
                    let fallback = create_error_response(&anyhow_err);
                    errors.push(BatchError {
                        file: name.clone(),
                        message: message.clone(),
                    });
                    files.insert(name.clone(), FileRenderOutcome::failure(message, fallback));
                }
            }
        }

        Ok(BatchRenderOutcome::new(files, errors, succeeded, failed))
    }

    fn validate_resource_limits(
        &self,
        input: &NamedMdxBatchInput,
        components: Option<&HashMap<String, ComponentDefinition>>,
    ) -> Result<(), RenderBatchError> {
        let limits = &self.config.resource_limits;

        // Check batch size
        if input.mdx.len() > limits.max_batch_size {
            return Err(RenderBatchError::InvalidRequest(format!(
                "Batch size {} exceeds maximum allowed {}",
                input.mdx.len(),
                limits.max_batch_size
            )));
        }

        // Check MDX content sizes
        for (name, content) in &input.mdx {
            if content.len() > limits.max_mdx_content_size {
                return Err(RenderBatchError::InvalidRequest(format!(
                    "MDX content for '{}' is {} bytes, exceeds maximum allowed {} bytes",
                    name,
                    content.len(),
                    limits.max_mdx_content_size
                )));
            }
        }

        // Check component code sizes
        if let Some(component_map) = components {
            for (name, comp_def) in component_map {
                if comp_def.code.len() > limits.max_component_code_size {
                    return Err(RenderBatchError::InvalidRequest(format!(
                        "Component '{}' code is {} bytes, exceeds maximum allowed {} bytes",
                        name,
                        comp_def.code.len(),
                        limits.max_component_code_size
                    )));
                }
            }
        }

        Ok(())
    }

    fn profile_for_request(
        &self,
        format: &OutputFormat,
    ) -> Result<RendererProfile, RenderBatchError> {
        match format {
            OutputFormat::Html | OutputFormat::Javascript | OutputFormat::Schema | OutputFormat::Json => {
                Ok(RendererProfile::Engine)
            }
        }
    }

}

/// Errors surfaced by the batch renderer.
#[derive(Debug)]
pub enum RenderBatchError {
    /// Request is forbidden (e.g., custom engines disabled)
    Forbidden(String),
    /// Request is invalid (e.g., resource limits exceeded)
    InvalidRequest(String),
    /// Internal error during rendering
    Internal(AnyhowError),
}

impl std::error::Error for RenderBatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RenderBatchError::Internal(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl std::fmt::Display for RenderBatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderBatchError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            RenderBatchError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            RenderBatchError::Internal(err) => write!(f, "Internal error: {}", err),
        }
    }
}

impl From<anyhow::Error> for RenderBatchError {
    fn from(err: anyhow::Error) -> Self {
        RenderBatchError::Internal(err)
    }
}

impl From<crate::error::MdxError> for RenderBatchError {
    fn from(err: crate::error::MdxError) -> Self {
        RenderBatchError::Internal(anyhow::Error::from(err))
    }
}

/// Outcome of a batch rendering operation
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchRenderOutcome {
    /// Total number of files processed
    pub total: usize,
    /// Number of files that rendered successfully
    pub succeeded: usize,
    /// Number of files that failed to render
    pub failed: usize,
    /// List of errors encountered during rendering
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<BatchError>,
    /// Map of file names to their rendering outcomes
    #[serde(default)]
    pub files: HashMap<String, FileRenderOutcome>,
}

impl BatchRenderOutcome {
    /// Creates a new batch render outcome
    ///
    /// # Arguments
    /// * `files` - Map of file names to their render outcomes
    /// * `errors` - List of errors encountered
    /// * `succeeded` - Number of successful renders
    /// * `failed` - Number of failed renders
    pub fn new(
        files: HashMap<String, FileRenderOutcome>,
        errors: Vec<BatchError>,
        succeeded: usize,
        failed: usize,
    ) -> Self {
        let total = succeeded + failed;
        Self {
            total,
            succeeded,
            failed,
            errors,
            files,
        }
    }

    /// Creates an empty batch render outcome (no files processed)
    pub fn empty() -> Self {
        Self {
            total: 0,
            succeeded: 0,
            failed: 0,
            errors: Vec::new(),
            files: HashMap::new(),
        }
    }

    /// Returns true if all files rendered successfully
    pub fn is_all_success(&self) -> bool {
        self.failed == 0
    }

    /// Returns true if all files failed to render
    pub fn is_complete_failure(&self) -> bool {
        self.total > 0 && self.succeeded == 0
    }
}

/// Error information for a single file in a batch
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchError {
    /// Name of the file that failed
    pub file: String,
    /// Error message describing the failure
    pub message: String,
}

/// Status of a single file render operation
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileRenderStatus {
    /// File rendered successfully
    Success,
    /// File failed to render
    Failed,
}

/// Outcome of rendering a single file
#[derive(Debug, Serialize, Deserialize)]
pub struct FileRenderOutcome {
    /// Status of the render operation
    pub status: FileRenderStatus,
    /// Rendered result (present even on failure as fallback)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<RenderedMdx>,
    /// Error message (only present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl FileRenderOutcome {
    fn success(result: RenderedMdx) -> Self {
        Self {
            status: FileRenderStatus::Success,
            result: Some(result),
            error: None,
        }
    }

    fn failure(message: String, fallback: RenderedMdx) -> Self {
        Self {
            status: FileRenderStatus::Failed,
            result: Some(fallback),
            error: Some(message),
        }
    }
}
