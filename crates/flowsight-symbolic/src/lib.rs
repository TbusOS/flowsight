//! FlowSight Symbolic Execution Module
//!
//! Provides symbolic execution capabilities using KLEE for:
//! - Path exploration and constraint generation
//! - Test case generation for each execution path
//! - Condition translation from symbolic constraints
//!
//! ## Architecture
//!
//! ```text
//! Source Code / LLVM IR
//!         │
//!         ├── KLEE Executor ───→ Execution Paths
//!         │        │
//!         │        ├── Path 1: constraint_1 ∧ constraint_2
//!         │        ├── Path 2: constraint_1 ∧ ¬constraint_2
//!         │        └── Path 3: ...
//!         │
//!         └── Constraint Translator (AI-assisted)
//!                  │
//!                  └── "if (ret == -ENODEV)" → "设备不匹配"
//!
//! Result: Complete execution flow with condition translations
//! ```

pub mod executor;
pub mod path;
pub mod constraint;
pub mod stubs;

pub use executor::{KleeExecutor, SymbolicConfig};
pub use path::{ExecutionPath, ExecutionStep, PathConstraint};
pub use constraint::{SymbolicConstraint, ConstraintType};
pub use stubs::KernelStubManager;
