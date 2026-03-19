//! Output formatting module
//!
//! Provides multiple output formats for analysis results:
//! - Text: human-readable console output
//! - JSON: machine-readable structured output
//! - Ftrace: Linux ftrace-style function trace
//! - Sequence: ASCII multi-lane sequence diagram
//! - Markdown: Markdown table format
//! - Dot: Graphviz DOT graph format

pub mod dot;
pub mod json;
pub mod sequence;
pub mod text;

use clap::ValueEnum;
use std::str::FromStr;

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
    /// Graphviz DOT graph format (pipe to `dot -Tsvg` for rendering)
    Dot,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "ftrace" => Ok(Self::Ftrace),
            "sequence" => Ok(Self::Sequence),
            "markdown" => Ok(Self::Markdown),
            "dot" => Ok(Self::Dot),
            other => Err(format!("unknown output format: '{}'", other)),
        }
    }
}

