//! # Rust CMS - MDX Rendering Service Library
//!
//! A high-performance library for converting MDX (Markdown with JSX) to HTML and JavaScript.
//! Extracts YAML frontmatter and transforms TSX to JavaScript using the Oxc compiler.
//!
//! ## Architecture Overview
//!
//! The library is organized into several key modules:
//!
//! - **`service`**: High-level batch rendering service with resource limits and error handling
//! - **`mdx`**: MDX parsing, frontmatter extraction, and rendering pipeline orchestration
//! - **`renderer`**: JavaScript runtime management using Deno Core for component rendering
//! - **`transform`**: TSX/JSX to JavaScript transformation using Oxc compiler
//! - **`models`**: Data structures for MDX content, components, and configuration
//! - **`error`**: Domain-specific error types for MDX processing
//!
//! ### Rendering Pipeline
//!
//! ```text
//! MDX Content
//!     |
//!     +-> Extract YAML Frontmatter
//!     |
//!     +-> Convert Markdown to HTML (with JSX support)
//!     |
//!     +-> Transform TSX to JavaScript (Oxc compiler)
//!     |
//!     +-> Render Component (Deno Core + engine)
//!             |
//!             +-> HTML/JavaScript Output
//! ```
//!
//! ## Thread Safety
//!
//! This library uses thread-local storage for JavaScript runtimes because `JsRuntime` is not
//! `Send` or `Sync`. Each thread maintains its own renderer pool. The library is designed to be
//! used in a multi-threaded web server context where each request handler runs on its own thread.
//!
//! ```text
//! Thread 1          Thread 2          Thread 3
//!    |                 |                 |
//!    +- Renderer Pool  +- Renderer Pool  +- Renderer Pool
//!    |  (thread-local)  |  (thread-local)  |  (thread-local)
//!    |                 |                 |
//!    +- Request 1      +- Request 2      +- Request 3
//! ```
//!
//! ## Performance Tuning
//!
//! ### Renderer Pool Configuration
//!
//! - **`max_cached_renderers`**: Controls how many renderers are cached per profile per thread.
//!   Higher values reduce renderer creation overhead but increase memory usage.
//!   Recommended: 2-4 for most workloads.
//!
//! - **Pool Warming**: Use `pool.warm()` to pre-create renderers and reduce first-request latency.
//!
//! ### Resource Limits
//!
//! Configure resource limits to prevent memory exhaustion:
//!
//! - **`max_batch_size`**: Maximum number of files in a batch (default: 1000)
//! - **`max_mdx_content_size`**: Maximum MDX file size (default: 10 MB)
//! - **`max_component_code_size`**: Maximum component code size (default: 1 MB)
//!
//! ### String Pre-allocation
//!
//! The library pre-allocates strings with estimated capacity to reduce reallocations.
//! This is handled automatically, but understanding the approach helps with profiling.
//!
//! ## Security Considerations
//!
//!
//! ### Resource Limits
//!
//! Resource limits prevent memory exhaustion but are **not security controls**. They are
//! reliability measures. Security features like rate limiting, authentication, and input
//! validation should be implemented at the HTTP/web layer.
//!
//! ### Component Code
//!
//! Component code is transformed and executed in the JavaScript runtime. Validate and sanitize
//! component code before passing it to the library if it originates from untrusted sources.
//!
//! ## Usage Example
//!
//! ```no_run
//! use dinja_core::service::{RenderService, RenderServiceConfig};
//! use dinja_core::models::NamedMdxBatchInput;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RenderServiceConfig::from_env();
//! let service = RenderService::new(config)?;
//!
//! let input = NamedMdxBatchInput {
//!     settings: Default::default(),
//!     mdx: std::collections::HashMap::new(),
//!     components: None,
//! };
//!
//! let outcome = service.render_batch(&input)?;
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]

pub mod error;
#[cfg(feature = "http")]
pub mod handlers;
pub mod mdx;
pub mod models;
pub mod renderer;
pub mod service;
pub mod transform;
