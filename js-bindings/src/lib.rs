use dinja_core::service::{
    RenderBatchError, RenderService as CoreRenderService, RenderServiceConfig,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use once_cell::sync::OnceCell;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

// Embed static JavaScript files
const ENGINE_MIN_JS: &str = include_str!("../../core/static/engine.min.js");
const ENGINE_TO_STRING_MIN_JS: &str = include_str!("../../core/static/engine_to_string.min.js");
const CORE_JS: &str = include_str!("../../core/static/core.js");

// Global static directory path - created once on first use
static STATIC_DIR: OnceCell<PathBuf> = OnceCell::new();

/// Initialize the static directory with embedded files
fn init_static_dir() -> Result<PathBuf> {
    STATIC_DIR
        .get_or_try_init(|| -> Result<PathBuf> {
            // Create a temporary directory in the system temp location
            let temp_dir = std::env::temp_dir();
            let static_dir = temp_dir.join("dinja-static");

            // Create directory if it doesn't exist
            fs::create_dir_all(&static_dir).map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Failed to create static directory: {}", e),
                )
            })?;

            // Write embedded files
            fs::write(static_dir.join("engine.min.js"), ENGINE_MIN_JS).map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Failed to write engine.min.js: {}", e),
                )
            })?;

            fs::write(
                static_dir.join("engine_to_string.min.js"),
                ENGINE_TO_STRING_MIN_JS,
            )
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Failed to write engine_to_string.min.js: {}", e),
                )
            })?;

            fs::write(static_dir.join("core.js"), CORE_JS).map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Failed to write core.js: {}", e),
                )
            })?;
            Ok(static_dir)
        })
        .cloned()
}

/// A reusable renderer instance that maintains a single RenderService.
///
/// This class solves the v8 isolate issue by reusing the same service instance
/// across multiple renders, allowing the renderer pool to properly manage v8 isolates.
/// Use this instead of creating a new service for each render when you need to render
/// multiple times, especially with different output modes.
///
/// # Example
///
/// ```javascript
/// const { Renderer } = require('@dinja/core');
///
/// // Create a renderer instance (loads engine once)
/// const renderer = new Renderer();
///
/// // Reuse the same instance for multiple renders
/// const result1 = await renderer.render({
///   settings: { output: 'html', minify: false },
///   mdx: { 'file1.mdx': '# Hello' }
/// });
///
/// const result2 = await renderer.render({
///   settings: { output: 'schema', minify: false },
///   mdx: { 'file2.mdx': '# World' }
/// });
/// ```
#[napi]
pub struct Renderer {
    service: Mutex<CoreRenderService>,
}

#[napi]
impl Renderer {
    /// Creates a new Renderer instance.
    ///
    /// The engine is loaded once during initialization and reused for all subsequent renders.
    /// This prevents v8 isolate issues when rendering with different modes.
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        let static_dir = init_static_dir()?;
        let config = RenderServiceConfig {
            static_dir,
            max_cached_renderers: 4,
            resource_limits: dinja_core::models::ResourceLimits::default(),
        };
        let service = CoreRenderService::new(config).map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to create render service: {}", e),
            )
        })?;
        Ok(Self {
            service: Mutex::new(service),
        })
    }

    /// Renders MDX content using the reusable service instance.
    ///
    /// # Arguments
    /// * `input` - JSON string containing:
    ///   - `settings`: Object with `output` ("html", "javascript", "schema", or "json"),
    ///     `minify` (boolean)
    ///   - `mdx`: Object mapping file names to MDX content strings
    ///   - `components`: Optional object mapping component names to component definitions
    ///
    /// # Returns
    /// JSON string containing:
    /// - `total`: Total number of files processed
    /// - `succeeded`: Number of files that rendered successfully
    /// - `failed`: Number of files that failed to render
    /// - `errors`: Array of error objects with `file` and `message` keys
    /// - `files`: Object mapping file names to render outcomes
    ///
    /// # Errors
    /// - Throws `Error` if the request is invalid (e.g., resource limits exceeded, invalid input)
    /// - Throws `Error` if an internal error occurs during rendering
    #[napi]
    pub fn render(&self, input: String) -> Result<String> {
        // Parse JSON string to Rust struct
        let batch_input: dinja_core::models::NamedMdxBatchInput =
            serde_json::from_str(&input).map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Failed to parse input: {}", e),
                )
            })?;

        // Call render_batch on the locked service
        let outcome = {
            let service = self.service.lock().unwrap();
            match service.render_batch(&batch_input) {
                Ok(outcome) => outcome,
                Err(RenderBatchError::Forbidden(msg)) => {
                    return Err(Error::new(
                        Status::InvalidArg,
                        format!("Forbidden: {}", msg),
                    ));
                }
                Err(RenderBatchError::InvalidRequest(msg)) => {
                    return Err(Error::new(
                        Status::InvalidArg,
                        format!("Invalid request: {}", msg),
                    ));
                }
                Err(RenderBatchError::Internal(err)) => {
                    return Err(Error::new(
                        Status::GenericFailure,
                        format!("Internal error: {}", err),
                    ));
                }
            }
        };

        // Serialize outcome to JSON string
        serde_json::to_string(&outcome).map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to serialize outcome: {}", e),
            )
        })
    }
}
