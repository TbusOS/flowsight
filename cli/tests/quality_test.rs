//! FlowSight Quality Test Suite
//!
//! Standalone test binary that validates the core promise:
//! **accurate function execution flow.**
//!
//! Run: cargo test --test quality_test
//!
//! This is separate from integration_test.rs (which tests command mechanics).
//! Quality tests check the CORRECTNESS of analysis output.

mod quality;
