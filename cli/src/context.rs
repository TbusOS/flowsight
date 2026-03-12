//! Shared analysis context
//!
//! Provides a reusable `AnalysisContext` that initializes parser, analyzer,
//! and knowledge base once, avoiding repeated setup across commands.

use anyhow::{Context, Result};
use flowsight_analysis::{AnalysisResult, Analyzer};
use flowsight_knowledge::KnowledgeBase;
use flowsight_parser::{get_parser, ParseResult, Parser};
use std::path::{Path, PathBuf};

/// Shared analysis context for all CLI commands
pub struct AnalysisContext {
    parser: Box<dyn Parser>,
    analyzer: Analyzer,
}

/// Result of parsing and analyzing a single file
pub struct FileAnalysis {
    pub file: PathBuf,
    pub source: String,
    pub parse_result: ParseResult,
    pub analysis: AnalysisResult,
}

impl AnalysisContext {
    /// Create a new analysis context with built-in knowledge base
    pub fn new() -> Self {
        Self {
            parser: get_parser(),
            analyzer: Analyzer::new(),
        }
    }

    /// Create with a custom knowledge base (used by `kb` commands)
    pub fn with_knowledge_base(kb: KnowledgeBase) -> Self {
        Self {
            parser: get_parser(),
            analyzer: Analyzer::with_knowledge_base(kb),
        }
    }

    /// Get a reference to the knowledge base
    pub fn knowledge_base(&self) -> &KnowledgeBase {
        self.analyzer.knowledge_base()
    }

    /// Parse a single file (no analysis), reads file once
    pub fn parse_file(&self, path: &Path) -> Result<ParseResult> {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let filename = path.to_string_lossy();
        self.parser
            .parse(&source, &filename)
            .map_err(|e| anyhow::anyhow!("{}", e))
            .with_context(|| format!("Failed to parse {}", path.display()))
    }

    /// Parse and analyze a single file, reads file only once
    pub fn analyze_file(&mut self, path: &Path) -> Result<FileAnalysis> {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let filename = path.to_string_lossy();
        let mut parse_result = self
            .parser
            .parse(&source, &filename)
            .map_err(|e| anyhow::anyhow!("{}", e))
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        let analysis = self
            .analyzer
            .analyze(&source, &mut parse_result)
            .map_err(|e| anyhow::anyhow!("{}", e))
            .with_context(|| format!("Failed to analyze {}", path.display()))?;

        Ok(FileAnalysis {
            file: path.to_path_buf(),
            source,
            parse_result,
            analysis,
        })
    }
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self::new()
    }
}
