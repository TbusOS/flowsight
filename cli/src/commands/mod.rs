//! CLI command implementations
//!
//! Each command is in its own module for maintainability.

pub mod analyze;
pub mod async_cmd;
pub mod flow;
pub mod graph;
pub mod index;
pub mod kb;
pub mod patterns;
#[path = "train/mod.rs"]
pub mod train;
