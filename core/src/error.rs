//! Error types for MDX processing
//!
//! This module defines domain-specific error types for MDX processing operations.
//! All errors use `thiserror` for automatic `std::error::Error` implementation.
//!
//! ## Error Hierarchy
//!
//! - **Domain Errors**: Use `MdxError` for parsing, transformation, and rendering errors
//! - **Service Boundary**: Convert `MdxError` to `anyhow::Error` at HTTP handler boundaries
//!
//! ## Error Types
//!
//! - `FrontmatterParse`: YAML frontmatter parsing failures
//! - `MarkdownRender`: Markdown to HTML conversion failures
//! - `TsxParse`: TSX/JSX syntax parsing errors
//! - `TsxTransform`: TSX to JavaScript transformation errors
//! - `SourceType`: Source type detection failures
//! - Resource limit errors: `ContentTooLarge`, `BatchTooLarge`, `ComponentCodeTooLarge`, `EngineCodeTooLarge`

use thiserror::Error;

/// Custom error type for MDX processing
#[derive(Error, Debug)]
pub enum MdxError {
    /// Failed to parse YAML frontmatter from MDX content
    #[error("Failed to parse frontmatter: {0}")]
    FrontmatterParse(String),

    /// Failed to render markdown to HTML
    #[error("Failed to render markdown: {0}")]
    MarkdownRender(String),

    /// Failed to parse TSX/JSX syntax
    #[error("Failed to parse TSX: {0}")]
    TsxParse(String),

    /// Failed to transform TSX to JavaScript
    #[error("Failed to transform TSX: {0}")]
    TsxTransform(String),

    /// Failed to determine source type from file path
    #[error("Failed to determine source type: {0}")]
    SourceType(String),

    /// Content size exceeds maximum allowed limit
    #[error("Content size exceeds maximum allowed: {0} bytes")]
    ContentTooLarge(usize),

    /// Batch size exceeds maximum allowed limit
    #[error("Batch size exceeds maximum allowed: {0} files")]
    BatchTooLarge(usize),

    /// Component code size exceeds maximum allowed limit
    #[error("Component code size exceeds maximum allowed: {0} bytes")]
    ComponentCodeTooLarge(usize),

    /// Engine code size exceeds maximum allowed limit
    #[error("Engine code size exceeds maximum allowed: {0} bytes")]
    EngineCodeTooLarge(usize),

    /// Invalid export default statement - must be `export default function Component`
    #[error("Invalid export: '{0}' violates the naming convention. Use 'export default function Component() {{ ... }}' instead")]
    InvalidExportDefault(String),
}
