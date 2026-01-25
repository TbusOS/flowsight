//! Execution Path Tracing
//!
//! Records the complete function call path and branch conditions
//! for execution flow analysis and visualization.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use flowsight_core::{Location, FlowNode, FlowNodeType};

/// A recorded execution path step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathStep {
    /// Step ID (0-indexed)
    pub step_id: usize,
    /// Function name
    pub function: String,
    /// Location in source code
    pub location: Option<Location>,
    /// Arguments at call site
    pub arguments: Vec<PathValue>,
    /// Return value (if applicable)
    pub return_value: Option<PathValue>,
    /// Call type
    pub call_type: CallCategory,
    /// Depth in call stack
    pub depth: usize,
    /// Timestamp or sequence number
    pub sequence: u64,
}

/// Value at a path point (concrete or symbolic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathValue {
    /// Variable name or expression
    pub name: String,
    /// Concrete value (if known)
    pub concrete: Option<String>,
    /// Symbolic expression (if symbolic)
    pub symbolic: Option<String>,
    /// Value type
    pub value_type: ValueType,
}

/// Type of value
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ValueType {
    /// Integer value
    Integer,
    /// Pointer value
    Pointer,
    /// String value
    String,
    /// Boolean value
    Boolean,
    /// Struct/union value
    Struct,
    /// Array value
    Array,
    /// Unknown/undefined
    Unknown,
}

/// Category of call
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CallCategory {
    /// Direct function call
    Direct,
    /// Indirect call via function pointer
    IndirectPointer,
    /// System call
    SystemCall,
    /// Kernel API call
    KernelApi,
    /// Async callback (work queue, timer, etc.)
    AsyncCallback,
    /// Macro expansion or inline call
    Inline,
}

/// Branch information in execution path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Branch location
    pub location: Option<Location>,
    /// Condition expression
    pub condition: String,
    /// Which branch was taken
    pub taken: BranchOutcome,
    /// Alternative path (for path exploration)
    pub alternative: Option<Box<ExecutionPath>>,
}

/// Outcome of a branch
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BranchOutcome {
    /// True branch taken
    True,
    /// False branch taken
    False,
    /// Fallthrough (switch case)
    Fallthrough(usize),
    /// Default case taken
    Default,
    /// Unknown (could not determine)
    Unknown,
}

/// Complete execution path
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExecutionPath {
    /// Unique path identifier
    pub id: String,
    /// Path name/description
    pub name: String,
    /// Steps in this path
    pub steps: Vec<PathStep>,
    /// Branch decisions made along this path
    pub branches: Vec<BranchInfo>,
    /// Variables with their values at the end of the path
    pub final_values: HashMap<String, PathValue>,
    /// Whether this path completed (reached end or error)
    pub completed: bool,
    /// Termination reason (if not completed)
    pub termination_reason: Option<String>,
    /// Call depth statistics
    pub max_depth: usize,
    /// Total steps in path
    pub step_count: usize,
}

/// Builder for ExecutionPath
#[derive(Debug, Default, Clone)]
pub struct ExecutionPathBuilder {
    path: ExecutionPath,
    step_counter: u64,
    depth: usize,
}

impl ExecutionPathBuilder {
    /// Create a new builder
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            path: ExecutionPath {
                id: id.into(),
                name: name.into(),
                steps: Vec::new(),
                branches: Vec::new(),
                final_values: HashMap::new(),
                completed: false,
                termination_reason: None,
                max_depth: 0,
                step_count: 0,
            },
            step_counter: 0,
            depth: 0,
        }
    }

    /// Add a step to the path
    pub fn add_step(
        &mut self,
        function: impl Into<String>,
        location: Option<Location>,
        call_type: CallCategory,
        arguments: Vec<PathValue>,
    ) -> &mut Self {
        self.step_counter += 1;
        let step = PathStep {
            step_id: self.path.steps.len(),
            function: function.into(),
            location,
            arguments,
            return_value: None,
            call_type,
            depth: self.depth,
            sequence: self.step_counter,
        };
        self.path.steps.push(step);
        self.path.step_count += 1;
        self.path.max_depth = self.path.max_depth.max(self.depth);
        self
    }

    /// Record a function entry (increments depth)
    pub fn enter_function(&mut self, _name: impl Into<String>) {
        self.depth += 1;
    }

    /// Record a function exit (decrements depth)
    pub fn exit_function(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    /// Add a branch decision
    pub fn add_branch(
        &mut self,
        location: Option<Location>,
        condition: impl Into<String>,
        taken: BranchOutcome,
    ) -> &mut Self {
        self.path.branches.push(BranchInfo {
            location,
            condition: condition.into(),
            taken,
            alternative: None,
        });
        self
    }

    /// Set final value of a variable
    pub fn set_final_value(&mut self, name: impl Into<String>, value: PathValue) -> &mut Self {
        self.path.final_values.insert(name.into(), value);
        self
    }

    /// Mark path as completed
    pub fn completed(&mut self, reason: Option<String>) -> &mut Self {
        self.path.completed = true;
        self.path.termination_reason = reason;
        self
    }

    /// Mark path as terminated with error
    pub fn terminated(&mut self, reason: impl Into<String>) -> &mut Self {
        self.path.completed = false;
        self.path.termination_reason = Some(reason.into());
        self
    }

    /// Build the execution path
    pub fn build(&mut self) -> ExecutionPath {
        std::mem::take(&mut self.path)
    }
}

/// Main tracer for execution paths
pub struct ExecutionTracer {
    /// All recorded paths
    paths: Vec<ExecutionPath>,
    /// Current path being recorded
    current_path: Option<ExecutionPath>,
    /// Call stack for current path
    call_stack: Vec<String>,
    /// Known function entry points
    entry_points: HashSet<String>,
    /// Maximum paths to record
    max_paths: usize,
    /// Sequence counter for steps
    sequence: u64,
}

impl ExecutionTracer {
    /// Create a new tracer
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            current_path: None,
            call_stack: Vec::new(),
            entry_points: HashSet::new(),
            max_paths: 100,
            sequence: 0,
        }
    }

    /// Set maximum paths to record
    pub fn with_max_paths(mut self, max: usize) -> Self {
        self.max_paths = max;
        self
    }

    /// Add a known entry point
    pub fn add_entry_point(&mut self, name: &str) {
        self.entry_points.insert(name.to_string());
    }

    /// Add known entry points
    pub fn add_entry_points(&mut self, names: impl IntoIterator<Item = String>) {
        self.entry_points.extend(names);
    }

    /// Start recording a new path
    pub fn start_path(&mut self, id: impl Into<String>, name: impl Into<String>) {
        self.current_path = Some(ExecutionPath {
            id: id.into(),
            name: name.into(),
            steps: Vec::new(),
            branches: Vec::new(),
            final_values: HashMap::new(),
            completed: false,
            termination_reason: None,
            max_depth: 0,
            step_count: 0,
        });
        self.call_stack.clear();
        self.sequence = 0;
    }

    /// Record a function call
    pub fn record_call(
        &mut self,
        function: &str,
        location: Option<Location>,
        call_type: CallCategory,
        arguments: &[PathValue],
    ) {
        if let Some(path) = &mut self.current_path {
            self.sequence += 1;
            path.steps.push(PathStep {
                step_id: path.steps.len(),
                function: function.to_string(),
                location,
                arguments: arguments.to_vec(),
                return_value: None,
                call_type,
                depth: self.call_stack.len(),
                sequence: self.sequence,
            });
            path.step_count += 1;
            path.max_depth = path.max_depth.max(self.call_stack.len());
        }
        self.call_stack.push(function.to_string());
    }

    /// Record function return
    pub fn record_return(&mut self, value: Option<PathValue>) {
        if let Some(path) = &mut self.current_path {
            if let Some(last_step) = path.steps.last_mut() {
                last_step.return_value = value;
            }
        }
        self.call_stack.pop();
    }

    /// Record a branch decision
    pub fn record_branch(
        &mut self,
        location: Option<Location>,
        condition: &str,
        taken: BranchOutcome,
    ) {
        if let Some(path) = &mut self.current_path {
            path.branches.push(BranchInfo {
                location,
                condition: condition.to_string(),
                taken,
                alternative: None,
            });
        }
    }

    /// Finalize current path and store it
    pub fn finalize_path(&mut self, completed: bool, reason: Option<&str>) {
        if let Some(mut path) = self.current_path.take() {
            path.completed = completed;
            path.termination_reason = reason.map(|s| s.to_string());
            self.paths.push(path);
        }
        self.call_stack.clear();
    }

    /// Get all recorded paths
    pub fn paths(&self) -> &[ExecutionPath] {
        &self.paths
    }

    /// Get count of recorded paths
    pub fn path_count(&self) -> usize {
        self.paths.len()
    }

    /// Clear all recorded paths
    pub fn clear(&mut self) {
        self.paths.clear();
        self.current_path = None;
        self.call_stack.clear();
    }

    /// Convert a FlowNode tree to execution paths
    pub fn trace_flow_tree(&self, tree: &FlowNode) -> Vec<ExecutionPath> {
        let mut paths = Vec::new();
        let mut builder = ExecutionPathBuilder::new("default", "Default Path");
        self.trace_node(tree, &mut builder, &mut paths);
        paths
    }

    /// Recursively trace nodes
    fn trace_node(
        &self,
        node: &FlowNode,
        current_path: &mut ExecutionPathBuilder,
        all_paths: &mut Vec<ExecutionPath>,
    ) {
        // Determine call category from node type
        let call_type = match node.node_type {
            FlowNodeType::Function => CallCategory::Direct,
            FlowNodeType::EntryPoint => CallCategory::Direct,
            FlowNodeType::AsyncCallback { .. } => CallCategory::AsyncCallback,
            FlowNodeType::KernelApi => CallCategory::KernelApi,
            FlowNodeType::External => CallCategory::IndirectPointer,
        };

        // Add step for this node
        current_path.add_step(
            &node.name,
            node.location.clone(),
            call_type,
            Vec::new(),
        );

        // Process children - create new builder for each child
        if node.children.is_empty() {
            // Leaf node - complete the path
            let mut leaf_builder = current_path.clone();
            all_paths.push(leaf_builder.build());
        } else {
            for child in &node.children {
                let mut child_builder = current_path.clone();
                self.trace_node(child, &mut child_builder, all_paths);
            }
        }
    }
}

impl Default for ExecutionTracer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a value to PathValue
impl PathValue {
    /// Create from concrete string value
    pub fn from_concrete(name: impl Into<String>, value: impl Into<String>, value_type: ValueType) -> Self {
        Self {
            name: name.into(),
            concrete: Some(value.into()),
            symbolic: None,
            value_type,
        }
    }

    /// Create from symbolic expression
    pub fn from_symbolic(name: impl Into<String>, expression: impl Into<String>, value_type: ValueType) -> Self {
        Self {
            name: name.into(),
            concrete: None,
            symbolic: Some(expression.into()),
            value_type,
        }
    }

    /// Get display string
    pub fn display(&self) -> String {
        if let Some(ref concrete) = self.concrete {
            concrete.clone()
        } else if let Some(ref symbolic) = self.symbolic {
            format!("[symbolic: {}]", symbolic)
        } else {
            "<unknown>".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_builder() {
        let path = ExecutionPathBuilder::new("path1", "Test Path")
            .add_step("main", None, CallCategory::Direct, Vec::new())
            .add_step("foo", None, CallCategory::Direct, Vec::new())
            .add_branch(None, "x > 0", BranchOutcome::True)
            .completed(None)
            .build();

        assert_eq!(path.id, "path1");
        assert_eq!(path.steps.len(), 2);
        assert_eq!(path.branches.len(), 1);
        assert!(path.completed);
    }

    #[test]
    fn test_path_value_display() {
        let concrete = PathValue::from_concrete("x", "42", ValueType::Integer);
        assert_eq!(concrete.display(), "42");

        let symbolic = PathValue::from_symbolic("y", "arg_0 * 2", ValueType::Integer);
        assert_eq!(symbolic.display(), "[symbolic: arg_0 * 2]");
    }

    #[test]
    fn test_tracer_basic() {
        let mut tracer = ExecutionTracer::new();
        tracer.start_path("p1", "Path 1");

        tracer.record_call("main", None, CallCategory::Direct, &[]);
        tracer.record_call("helper", None, CallCategory::Direct, &[]);
        tracer.record_return(Some(PathValue::from_concrete("ret", "0", ValueType::Integer)));
        tracer.record_return(Some(PathValue::from_concrete("result", "0", ValueType::Integer)));

        tracer.finalize_path(true, None);

        assert_eq!(tracer.path_count(), 1);
        let paths = tracer.paths();
        assert_eq!(paths[0].steps.len(), 2);
    }
}
