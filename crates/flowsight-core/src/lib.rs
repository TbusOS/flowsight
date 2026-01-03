//! FlowSight Core
//!
//! Core types and interfaces for the FlowSight code analysis engine.

pub mod error;
pub mod types;
pub mod location;
pub mod config;

pub use error::{Error, Result};
pub use types::*;
pub use location::Location;

