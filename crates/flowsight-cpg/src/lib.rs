#![forbid(unsafe_code)]
//! FlowSight CPG — Code Property Graph
//!
//! Unifies AST + CFG + Data Flow into a single queryable graph.
//!
//! # Architecture
//!
//! ```text
//! Source Code → tree-sitter AST → CpgBuilder
//!                                    ├── AST nodes (function, variable, statement)
//!                                    ├── CFG edges (control flow from flowsight-cfg)
//!                                    └── Data flow (def-use chains, reaching definitions)
//! ```
//!
//! # Key concepts
//!
//! - **Definition**: A variable assignment (lhs of `=`, declaration with initializer)
//! - **Use**: A variable read (in expressions, conditions, arguments)
//! - **Reaching definition**: Which assignment(s) can reach a given use point
//! - **Def-use chain**: Links each definition to all its uses

pub mod builder;
pub mod dataflow;
pub mod types;

pub use builder::CpgBuilder;
pub use types::*;
