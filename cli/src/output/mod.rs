//! Output formatting module
//!
//! Provides multiple output formats for analysis results:
//! - Text: human-readable console output
//! - JSON: machine-readable structured output
//! - Ftrace: Linux ftrace-style function trace
//! - Mermaid: Mermaid diagram syntax
//! - Markdown: Markdown table format

pub mod json;
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
    /// Mermaid diagram syntax
    Mermaid,
    /// Markdown table format
    Markdown,
}

