//! Execution Path Types
//!
//! Represents a single execution path discovered through symbolic execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use flowsight_core::Location;

/// Re-export SymbolicValue from constraint module
pub use crate::constraint::SymbolicValue;

/// A single execution path through the code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPath {
    /// Unique path identifier
    pub id: String,
    /// Human-readable name/description
    pub name: String,
    /// Step-by-step execution sequence
    pub steps: Vec<ExecutionStep>,
    /// Path constraints (symbolic conditions)
    pub constraints: Vec<PathConstraint>,
    /// AI-generated constraint translations
    pub condition_translations: Vec<ConditionTranslation>,
    /// Concrete test values that satisfy this path
    pub concrete_values: HashMap<String, String>,
    /// Whether this path leads to an error/return
    pub is_error_path: bool,
    /// Return value if applicable
    pub return_value: Option<String>,
    /// Path coverage (basic blocks covered)
    pub covered_blocks: Vec<String>,
    /// Branch conditions encountered
    pub branches: Vec<BranchInfo>,
}

/// A single step in the execution path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step number in the path
    pub step_id: usize,
    /// Function being called
    pub function: String,
    /// Location in source
    pub location: Option<Location>,
    /// Type of call
    pub call_type: CallType,
    /// Arguments at this call site
    pub arguments: Vec<SymbolicValue>,
    /// Return value (symbolic expression)
    pub return_value: Option<SymbolicValue>,
    /// Whether this is a user function or kernel API
    pub is_user_code: bool,
    /// Line number for display ordering
    pub line: u32,
}

/// Type of function call in execution path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallType {
    /// Direct function call
    Direct,
    /// Indirect call through function pointer
    Indirect {
        /// Pointer expression
        pointer: String,
        /// Resolved targets (may be empty if unknown)
        targets: Vec<String>,
    },
    /// System call
    SystemCall {
        /// Syscall number
        number: u64,
        /// Syscall name
        name: String,
    },
    /// Kernel API call
    KernelApi {
        /// API name
        name: String,
        /// Can block/sleep
        can_sleep: bool,
    },
    /// Callback (framework callback)
    Callback {
        /// Framework name
        framework: String,
        /// Callback name
        callback: String,
    },
}

/// Path constraint from symbolic execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConstraint {
    /// Unique constraint ID
    pub id: String,
    /// Location where constraint was generated
    pub location: Option<Location>,
    /// The constraint expression (KLEE format)
    pub expression: String,
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Variable being constrained
    pub variable: String,
    /// Operator
    pub operator: String,
    /// Expected/value
    pub value: String,
}

/// Constraint type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Branch condition (if/else)
    Branch,
    /// Loop condition
    Loop,
    /// Return condition
    Return,
    /// Assertion
    Assertion,
    /// Array bounds check
    BoundsCheck,
    /// Null pointer check
    NullCheck,
    /// Other
    Other,
}

/// AI-generated translation of a constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionTranslation {
    /// Original constraint expression
    pub original: String,
    /// Business meaning in Chinese
    pub business_meaning: String,
    /// Trigger scenario description
    pub trigger_scenario: String,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
    /// Source of translation (AI model, knowledge base, etc.)
    pub source: TranslationSource,
}

/// Source of constraint translation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TranslationSource {
    /// From AI model inference
    AiModel,
    /// From knowledge base
    KnowledgeBase,
    /// User provided
    User,
    /// Heuristic/rule-based
    Heuristic,
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Branch location
    pub location: Option<Location>,
    /// Condition expression
    pub condition: String,
    /// True branch target
    pub true_target: String,
    /// False branch target
    pub false_target: String,
    /// Which branch was taken in this path
    pub taken: BranchDirection,
}

/// Direction of branch taken
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BranchDirection {
    /// True branch taken
    True,
    /// False branch taken
    False,
    /// Default/switch case taken
    Case(usize),
    /// Fallthrough
    Fallthrough,
}

impl ExecutionPath {
    /// Create a new execution path
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            steps: Vec::new(),
            constraints: Vec::new(),
            condition_translations: Vec::new(),
            concrete_values: HashMap::new(),
            is_error_path: false,
            return_value: None,
            covered_blocks: Vec::new(),
            branches: Vec::new(),
        }
    }

    /// Add a step to the path
    pub fn add_step(&mut self, step: ExecutionStep) {
        self.steps.push(step);
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: PathConstraint) {
        self.constraints.push(constraint);
    }

    /// Get the last step
    pub fn last_step(&self) -> Option<&ExecutionStep> {
        self.steps.last()
    }

    /// Check if path is empty
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get path length (number of steps)
    pub fn len(&self) -> usize {
        self.steps.len()
    }
}

impl Default for ExecutionPath {
    fn default() -> Self {
        Self::new("default", "Default Path")
    }
}
