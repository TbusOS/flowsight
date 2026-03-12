//! Output formatting module
//!
//! Provides multiple output formats for analysis results:
//! - Text: human-readable console output
//! - JSON: machine-readable structured output
//! - Ftrace: Linux ftrace-style function trace
//! - Sequence: ASCII multi-lane sequence diagram
//! - Markdown: Markdown table format

pub mod json;
pub mod sequence;
pub mod text;

use clap::ValueEnum;

/// Output format for CLI commands
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum OutputFormat {
    /// Human-readable text output
    #[default]
    Text,
    /// JSON structured output
    Json,
    /// Linux ftrace-style function trace
    Ftrace,
    /// ASCII multi-lane sequence diagram (for async/chain)
    Sequence,
    /// Markdown table format
    Markdown,
}

