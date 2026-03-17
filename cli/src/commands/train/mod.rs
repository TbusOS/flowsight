//! `flowsight train` command group - training data generation pipeline
//!
//! Generates JSONL training data (SFT/DPO/ChatML) from kernel source analysis
//! for fine-tuning LLM models as Linux kernel experts.

pub mod formats;
mod generate;
mod stats;

use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

/// Training data output format
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum TrainFormat {
    /// Alpaca-style supervised fine-tuning
    #[default]
    Sft,
    /// Direct preference optimization (chosen/rejected pairs)
    Dpo,
    /// OpenAI ChatML format
    Chatml,
}

/// Training data generation categories
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Hash)]
pub enum TrainCategory {
    /// Function execution flow Q&A pairs
    Flow,
    /// Async handler analysis Q&A
    Async,
    /// Callback mechanism Q&A
    Callbacks,
    /// Call chain tracing Q&A
    Chains,
    /// Design pattern recognition Q&A
    Patterns,
}

impl std::fmt::Display for TrainCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flow => write!(f, "flow"),
            Self::Async => write!(f, "async"),
            Self::Callbacks => write!(f, "callbacks"),
            Self::Chains => write!(f, "chains"),
            Self::Patterns => write!(f, "patterns"),
        }
    }
}

/// Train subcommands
#[derive(Subcommand)]
pub enum TrainCommands {
    /// Generate training data from a directory of C files
    Generate {
        /// Directory of C source files to analyze
        #[arg(value_name = "DIR")]
        dir: PathBuf,

        /// Training data format (sft, dpo, chatml)
        #[arg(short = 't', long = "train-format", default_value = "sft")]
        train_format: TrainFormat,

        /// Output JSONL file path (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Categories to generate (comma-separated)
        #[arg(short, long, value_delimiter = ',', default_values_t = all_categories())]
        categories: Vec<TrainCategory>,

        /// Max call depth for flow examples
        #[arg(long, default_value = "8")]
        max_depth: usize,

        /// Minimum function complexity (call count) to include
        #[arg(long, default_value = "2")]
        min_complexity: usize,
    },

    /// Show statistics about generated training data
    Stats {
        /// JSONL file to analyze
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

fn all_categories() -> Vec<TrainCategory> {
    vec![
        TrainCategory::Flow,
        TrainCategory::Async,
        TrainCategory::Callbacks,
        TrainCategory::Chains,
        TrainCategory::Patterns,
    ]
}

/// Run train subcommand
pub fn run(cmd: &TrainCommands) -> anyhow::Result<()> {
    match cmd {
        TrainCommands::Generate {
            dir,
            train_format,
            output,
            categories,
            max_depth,
            min_complexity,
        } => {
            let config = generate::GenerateConfig {
                dir: dir.clone(),
                format: *train_format,
                output: output.clone(),
                categories: categories.clone(),
                max_depth: *max_depth,
                min_complexity: *min_complexity,
            };
            generate::run(&config)
        }
        TrainCommands::Stats { file } => stats::run(file),
    }
}
