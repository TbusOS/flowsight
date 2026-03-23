#![forbid(unsafe_code)]
//! FlowSight Evolve — Autonomous analysis evolution engine
//!
//! Inspired by Karpathy's autoresearch: constraint-driven automation
//! with fixed budgets, single scalar metrics, and keep/discard loops.
//!
//! # Modules
//!
//! - `aqs`: Analysis Quality Score — single scalar metric (0.0–1.0)

pub mod aqs;
