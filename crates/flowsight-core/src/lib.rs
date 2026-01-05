//! FlowSight Core
//!
//! Core types and interfaces for the FlowSight code analysis engine.

pub mod config;
pub mod error;
pub mod location;
pub mod types;

pub use error::{Error, Result};
pub use location::Location;
pub use types::*;
