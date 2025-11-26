use dinja_core::service::{
    RenderBatchError, RenderService as CoreRenderService, RenderServiceConfig,
};
use once_cell::sync::OnceCell;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::fs;
use std::path::PathBuf;

// Embed static JavaScript files
const ENGINE_MIN_JS: &str = include_str!("../../core/static/engine.min.js");
const ENGINE_TO_STRING_MIN_JS: &str = include_str!("../../core/static/engine_to_string.min.js");
const CORE_JS: &str = include_str!("../../core/static/core.js");

// Global static directory path - created once on first use
static STATIC_DIR: OnceCell<PathBuf> = OnceCell::new();

/// Initialize the static directory with embedded files
fn init_static_dir() -> PyResult<PathBuf> {
    STATIC_DIR
        .get_or_try_init(|| -> PyResult<PathBuf> {
            // Create a temporary directory in the system temp location
            let temp_dir = std::env::temp_dir();
            let static_dir = temp_dir.join("dinja-static");

            // Create directory if it doesn't exist
            fs::create_dir_all(&static_dir).map_err(|e| {
                PyValueError::new_err(format!("Failed to create static directory: {}", e))
            })?;

            // Write embedded files
            fs::write(static_dir.join("engine.min.js"), ENGINE_MIN_JS).map_err(|e| {
                PyValueError::new_err(format!("Failed to write engine.min.js: {}", e))
            })?;

            fs::write(
                static_dir.join("engine_to_string.min.js"),
                ENGINE_TO_STRING_MIN_JS,
            )
            .map_err(|e| {
                PyValueError::new_err(format!("Failed to write engine_to_string.min.js: {}", e))
            })?;

            fs::write(static_dir.join("core.js"), CORE_JS)
                .map_err(|e| PyValueError::new_err(format!("Failed to write core.js: {}", e)))?;

            Ok(static_dir)
        })
        .map(|p| p.clone())
}

/// Stateless render function for MDX content
///
/// This function renders MDX content to HTML, JavaScript, or schema format.
/// It's completely stateless - no need to create service instances or manage
/// static directories.
///
/// # Arguments
/// * `input_dict` - Dictionary containing:
///   - `settings`: Dictionary with `output` ("html", "javascript", or "schema"),
///     `minify` (bool), `engine` ("base" or "custom"), `components` (list of strings)
///   - `mdx`: Dictionary mapping file names to MDX content strings
///   - `components`: Optional dictionary mapping component names to component definitions
///
/// # Returns
/// Dictionary containing:
/// - `total`: Total number of files processed
/// - `succeeded`: Number of files that rendered successfully
/// - `failed`: Number of files that failed to render
/// - `errors`: List of error dictionaries with `file` and `message` keys
/// - `files`: Dictionary mapping file names to render outcomes
///
/// # Raises
/// * `ValueError` - If the request is invalid (e.g., resource limits exceeded, invalid input)
/// * `RuntimeError` - If an internal error occurs during rendering
#[pyfunction]
fn render(py: Python, input_dict: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    // Initialize static directory (creates temp dir and writes embedded files)
    let static_dir = init_static_dir()?;

    // Create service config with embedded static directory
    let config = RenderServiceConfig {
        static_dir,
        max_cached_renderers: 4,
        resource_limits: dinja_core::models::ResourceLimits::default(),
    };

    // Create service
    let service = CoreRenderService::new(config)
        .map_err(|e| PyValueError::new_err(format!("Failed to create render service: {}", e)))?;

    // Convert Python dict to JSON string
    let json_module = py.import("json")?;
    let dumps = json_module.getattr("dumps")?;
    let input_json: String = dumps.call1((input_dict,))?.extract()?;

    // Parse JSON string to Rust struct
    let batch_input: dinja_core::models::NamedMdxBatchInput = serde_json::from_str(&input_json)
        .map_err(|e| PyValueError::new_err(format!("Failed to parse input JSON: {}", e)))?;

    // Call render_batch
    let outcome = match service.render_batch(&batch_input) {
        Ok(outcome) => outcome,
        Err(RenderBatchError::Forbidden(msg)) => {
            return Err(PyValueError::new_err(format!("Forbidden: {}", msg)));
        }
        Err(RenderBatchError::InvalidRequest(msg)) => {
            return Err(PyValueError::new_err(format!("Invalid request: {}", msg)));
        }
        Err(RenderBatchError::Internal(err)) => {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Internal error: {}",
                err
            )));
        }
    };

    // Serialize outcome to JSON, then convert back to Python dict
    let outcome_json = serde_json::to_string(&outcome)
        .map_err(|e| PyValueError::new_err(format!("Failed to serialize outcome: {}", e)))?;

    let loads = json_module.getattr("loads")?;
    let result = loads.call1((outcome_json,))?;
    Ok(result.extract::<Py<PyAny>>()?)
}

/// The dinja Python module
#[pymodule]
fn dinja<'py>(_py: Python<'py>, m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(render, m)?)?;
    Ok(())
}
