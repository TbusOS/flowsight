//! Quality test suite for FlowSight CLI
//!
//! Standalone test binary that validates the core promise:
//! **accurate function execution flow, not just call lists.**
//!
//! Run: cargo test --test quality_test
//!
//! Scoring:
//! - Each test is weighted
//! - Total score 0-100 printed at end
//! - 90+ = Release candidate
//! - 100 = Production ready

pub mod execution_flow;
