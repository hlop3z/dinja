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
//! - `TsxParse`: TSX/JSX syntax parsing errors (with location info)
//! - `TsxTransform`: TSX to JavaScript transformation errors (with location info)
//! - `SourceType`: Source type detection failures
//! - Resource limit errors: `ContentTooLarge`, `BatchTooLarge`, `ComponentCodeTooLarge`, `EngineCodeTooLarge`
//!
//! ## Source Location
//!
//! Parse and transform errors include source location information when available,
//! allowing IDEs and tools to pinpoint exact error positions.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Source code location information (0-indexed line and column)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceLocation {
    /// 0-indexed line number
    pub line: u32,
    /// 0-indexed column number
    pub column: u32,
    /// Byte offset in source code
    pub offset: u32,
    /// Length of the error span in bytes
    pub length: u32,
}

impl SourceLocation {
    /// Creates a new source location
    pub fn new(line: u32, column: u32, offset: u32, length: u32) -> Self {
        Self {
            line,
            column,
            offset,
            length,
        }
    }

    /// Returns 1-indexed line number for display
    pub fn display_line(&self) -> u32 {
        self.line + 1
    }

    /// Returns 1-indexed column number for display
    pub fn display_column(&self) -> u32 {
        self.column + 1
    }
}

/// A single parse or transform error with optional location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    /// Error message
    pub message: String,
    /// Source location (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
    /// Help text or suggestion (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

impl ParseError {
    /// Creates a new parse error with just a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
            help: None,
        }
    }

    /// Creates a new parse error with location
    pub fn with_location(message: impl Into<String>, location: SourceLocation) -> Self {
        Self {
            message: message.into(),
            location: Some(location),
            help: None,
        }
    }

    /// Adds help text to this error
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(loc) = &self.location {
            write!(
                f,
                "{}:{}: {}",
                loc.display_line(),
                loc.display_column(),
                self.message
            )
        } else {
            write!(f, "{}", self.message)
        }
    }
}

/// Converts a byte offset to line and column numbers (0-indexed)
///
/// This function iterates through the source string character by character,
/// tracking newlines to compute the line and column for a given byte offset.
///
/// # Arguments
/// * `source` - The source code string
/// * `offset` - Byte offset into the source
///
/// # Returns
/// A tuple of (line, column), both 0-indexed
pub fn byte_offset_to_line_col(source: &str, offset: u32) -> (u32, u32) {
    if offset == 0 {
        return (0, 0);
    }

    let target_byte = offset as usize;
    let mut line = 0u32;
    let mut col = 0u32;
    let mut byte_pos = 0;

    for ch in source.chars() {
        if byte_pos >= target_byte {
            break;
        }

        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }

        byte_pos += ch.len_utf8();
    }

    (line, col)
}

/// Custom error type for MDX processing
#[derive(Error, Debug)]
pub enum MdxError {
    /// Failed to parse YAML frontmatter from MDX content
    #[error("Failed to parse frontmatter: {0}")]
    FrontmatterParse(String),

    /// Failed to render markdown to HTML
    #[error("Failed to render markdown: {0}")]
    MarkdownRender(String),

    /// Failed to parse TSX/JSX syntax (with location info)
    #[error("{}", format_parse_errors(.0))]
    TsxParse(Vec<ParseError>),

    /// Failed to transform TSX to JavaScript (with location info)
    #[error("{}", format_parse_errors(.0))]
    TsxTransform(Vec<ParseError>),

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

impl MdxError {
    /// Creates a TsxParse error from a single message (without location)
    pub fn tsx_parse(message: impl Into<String>) -> Self {
        Self::TsxParse(vec![ParseError::new(message)])
    }

    /// Creates a TsxTransform error from a single message (without location)
    pub fn tsx_transform(message: impl Into<String>) -> Self {
        Self::TsxTransform(vec![ParseError::new(message)])
    }

    /// Returns the first parse error if this is a TsxParse or TsxTransform error
    pub fn first_error(&self) -> Option<&ParseError> {
        match self {
            Self::TsxParse(errors) | Self::TsxTransform(errors) => errors.first(),
            _ => None,
        }
    }

    /// Returns all parse errors if this is a TsxParse or TsxTransform error
    pub fn errors(&self) -> Option<&[ParseError]> {
        match self {
            Self::TsxParse(errors) | Self::TsxTransform(errors) => Some(errors),
            _ => None,
        }
    }
}

/// Formats a list of parse errors for display
fn format_parse_errors(errors: &[ParseError]) -> String {
    if errors.is_empty() {
        return "Unknown error".to_string();
    }

    if errors.len() == 1 {
        return errors[0].to_string();
    }

    errors
        .iter()
        .enumerate()
        .map(|(i, e)| format!("[{}] {}", i + 1, e))
        .collect::<Vec<_>>()
        .join("; ")
}
