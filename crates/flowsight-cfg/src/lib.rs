#![forbid(unsafe_code)]
//! FlowSight CFG — Control Flow Graph construction from tree-sitter AST
//!
//! Builds accurate control flow graphs for C functions, with special
//! support for Linux kernel patterns (goto error handling, macro semantics,
//! execution context tracking).
//!
//! # Architecture
//!
//! ```text
//! tree-sitter AST → CfgBuilder → ControlFlowGraph
//!                                    ├── BasicBlock[]
//!                                    ├── CfgEdge[]
//!                                    └── ErrorPath[]
//! ```
//!
//! Algorithm adapted from tree-climber (Joern CfgCreator).

pub mod builder;
pub mod error_path;
pub mod macro_semantics;
pub mod types;

pub use builder::CfgBuilder;
pub use error_path::ErrorPathDetector;
pub use macro_semantics::{MacroSemantics, MacroTable};
pub use types::*;
