//! CLI command implementations
//!
//! Each command is in its own module for maintainability.

pub mod analyze;
pub mod ask;
pub mod async_cmd;
pub mod cfg;
pub mod config;
pub mod dataflow;
pub mod diff;
pub mod experiment;
pub mod flow;
pub mod graph;
pub mod index;
pub mod kb;
pub mod path;
pub mod patterns;
pub mod quality;
pub mod report;
pub mod scenario;
pub mod search;
pub mod selftest;
pub mod subsystem;
#[path = "train/mod.rs"]
pub mod train;
