//! JavaScript rendering engine using Deno Core
//!
//! This module provides functionality to render JavaScript components using
//! the Deno Core engine with engine and engine-render-to-string loaded.
//!
//! ## Architecture
//!
//! The renderer module is organized into several submodules:
//!
//! - **`pool`**: Thread-local renderer pooling for performance optimization
//! - **`runtime`**: JavaScript runtime lifecycle management and cleanup
//! - **`scripts`**: JavaScript code generation for component rendering
//! - **`engine`**: Static file loading and engine initialization
//! - **`constants`**: Script tags and constants for runtime operations
//!
//! ## Thread Safety
//!
//! **Important**: `JsRenderer` uses `Rc<RefCell<JsRuntime>>` instead of `Arc<Mutex<JsRuntime>>`
//! because `JsRuntime` is not `Send` or `Sync`. This means:
//!
//! - Renderers cannot be shared across threads
//! - Each thread must create its own renderer instances
//! - The renderer pool uses thread-local storage to maintain per-thread caches
//!
//! ## Usage
//!
//! ```no_run
//! use dinja_core::renderer::JsRenderer;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let renderer = JsRenderer::new("static")?;
//! let html = renderer.render_component(
//!     "function View(context = {{}}) {{ return engine.h('div', null, 'Hello'); }}",
//!     Some(r#"{"name": "World"}"#)
//! )?;
//! # Ok(())
//! # }
//! ```

mod constants;
mod engine;
pub mod pool;
mod runtime;
mod scripts;

pub use pool::{RendererPool, RendererProfile};

use crate::error::MdxError;
use crate::models::ComponentDefinition;
use anyhow::Result as AnyhowResult;
use deno_core::{JsRuntime, RuntimeOptions};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use constants::script_tags;
use engine::load_static_files_internal;
use runtime::{extract_string_from_v8, setup_context, with_runtime};
use scripts::{
    component_bootstrap_script, component_render_script, schema_render_script,
    wrap_transformed_component,
};

/// A renderer that manages a Deno Core runtime with engine libraries loaded
///
/// Note: Uses `Rc<RefCell<JsRuntime>>` instead of `Arc<Mutex<JsRuntime>>` because
/// `JsRuntime` is not `Send` or `Sync`, so it cannot be safely shared across threads.
/// Each request handler creates its own renderer instance.
pub struct JsRenderer {
    runtime: Rc<RefCell<JsRuntime>>,
}

impl JsRenderer {
    fn create_with_engine(static_dir: impl AsRef<Path>) -> AnyhowResult<Self> {
        let mut runtime = JsRuntime::new(RuntimeOptions::default());

        // Load static JavaScript files into the context
        load_static_files_internal(&mut runtime, static_dir)?;

        let renderer = Self {
            runtime: Rc::new(RefCell::new(runtime)),
        };

        Ok(renderer)
    }

    /// Creates a new renderer instance and loads the static JavaScript files
    ///
    /// # Arguments
    /// * `static_dir` - Path to the directory containing static JavaScript files
    ///
    /// # Returns
    /// A new `JsRenderer` instance with libraries loaded
    pub fn new(static_dir: impl AsRef<Path>) -> AnyhowResult<Self> {
        Self::create_with_engine(static_dir)
    }

    /// Renders a JavaScript component to HTML string
    ///
    /// # Arguments
    /// * `component_code` - JavaScript code that defines and exports a component
    /// * `props` - Optional JSON string of props to pass to the component
    ///
    /// # Returns
    /// Rendered HTML string
    pub fn render_component(
        &self,
        component_code: &str,
        props: Option<&str>,
    ) -> AnyhowResult<String> {
        let props_json = props.unwrap_or("{}");
        with_runtime(Rc::clone(&self.runtime), |runtime| {
            // Set up the context variable globally before executing component code
            setup_context(runtime, props_json).map_err(anyhow::Error::from)?;

            let render_script =
                component_render_script(component_code, props_json).map_err(anyhow::Error::from)?;

            // Evaluate and get the result
            let result = runtime
                .execute_script(script_tags::RENDER, render_script)
                .map_err(|e| {
                    anyhow::Error::from(MdxError::TsxTransform(format!(
                        "Failed to render component: {e:?}"
                    )))
                })?;

            extract_string_from_v8(result, runtime, "Failed to convert result to string")
                .map_err(anyhow::Error::from)
        })
    }

    /// Renders a JavaScript component using the transformed code from TSX
    ///
    /// # Arguments
    /// * `transformed_js` - JavaScript code from TSX transformation
    /// * `props` - Optional JSON string of props
    /// * `components` - Optional map of component definitions to inject
    ///
    /// # Returns
    /// Rendered HTML string
    pub fn render_transformed_component(
        &self,
        transformed_js: &str,
        props: Option<&str>,
        components: Option<&HashMap<String, ComponentDefinition>>,
    ) -> AnyhowResult<String> {
        let component_bootstrap = component_bootstrap_script(components)?;

        // Extract component names for variable declarations
        let component_names: Vec<String> = components
            .map(|comp_map| {
                comp_map
                    .iter()
                    .map(|(key, comp_def)| {
                        comp_def
                            .name
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| key.clone())
                    })
                    .collect()
            })
            .unwrap_or_default();

        let wrapped_code = wrap_transformed_component(&component_bootstrap, transformed_js, &component_names);

        self.render_component(&wrapped_code, props)
    }

    /// Renders a JavaScript component to schema (JSON string) using core.js engine
    ///
    /// # Arguments
    /// * `component_code` - JavaScript code that defines and exports a component
    /// * `props` - Optional JSON string of props to pass to the component
    ///
    /// # Returns
    /// Rendered schema as JSON string
    pub fn render_component_to_schema(
        &self,
        component_code: &str,
        props: Option<&str>,
    ) -> AnyhowResult<String> {
        let props_json = props.unwrap_or("{}");
        with_runtime(Rc::clone(&self.runtime), |runtime| {
            // Set up the context variable globally before executing component code
            setup_context(runtime, props_json).map_err(anyhow::Error::from)?;

            let render_script =
                schema_render_script(component_code, props_json).map_err(anyhow::Error::from)?;

            // Evaluate and get the result
            let result = runtime
                .execute_script(script_tags::RENDER, render_script)
                .map_err(|e| {
                    anyhow::Error::from(MdxError::TsxTransform(format!(
                        "Failed to render component to schema: {e:?}"
                    )))
                })?;

            extract_string_from_v8(result, runtime, "Failed to convert result to string")
                .map_err(anyhow::Error::from)
        })
    }

    /// Renders a JavaScript component to schema using the transformed code from TSX
    ///
    /// # Arguments
    /// * `transformed_js` - JavaScript code from TSX transformation
    /// * `props` - Optional JSON string of props
    /// * `components` - Optional map of component definitions to inject
    ///
    /// # Returns
    /// Rendered schema as JSON string
    pub fn render_transformed_component_to_schema(
        &self,
        transformed_js: &str,
        props: Option<&str>,
        components: Option<&HashMap<String, ComponentDefinition>>,
    ) -> AnyhowResult<String> {
        let component_bootstrap = component_bootstrap_script(components)?;

        // Extract component names for variable declarations
        let component_names: Vec<String> = components
            .map(|comp_map| {
                comp_map
                    .iter()
                    .map(|(key, comp_def)| {
                        comp_def
                            .name
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| key.clone())
                    })
                    .collect()
            })
            .unwrap_or_default();

        let wrapped_code = wrap_transformed_component(&component_bootstrap, transformed_js, &component_names);

        self.render_component_to_schema(&wrapped_code, props)
    }
}

impl Clone for JsRenderer {
    fn clone(&self) -> Self {
        Self {
            runtime: Rc::clone(&self.runtime),
        }
    }
}
