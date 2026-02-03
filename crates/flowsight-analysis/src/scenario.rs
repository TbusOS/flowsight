//! Scenario-based symbolic execution
//!
//! Core feature: Execute code symbolically with user-defined parameter values
//! to visualize execution paths and variable states.
//!
//! ## Enhanced Features
//!
//! - Multi-path exploration with branch tracking
//! - Enhanced constraint propagation with range/pointer support
//! - Integration with symbolic execution via SymbolicBridge
//! - Path visualization support

use flowsight_core::{FlowNode, FlowNodeType, Location};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;

use crate::path_tracing::{BranchOutcome, CallCategory, ExecutionTracer, PathValue, ValueType};
use crate::propagation::{BranchResult, ConstantPropagator};

/// User-defined scenario for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Scenario name (for saving/loading)
    pub name: String,
    /// Entry function name
    pub entry_function: String,
    /// Parameter bindings
    pub bindings: Vec<ValueBinding>,
    /// Analysis options
    #[serde(default)]
    pub options: ScenarioOptions,
}

/// Parameter value binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueBinding {
    /// Variable path (e.g., "id->idVendor", "ptr", "dev.name")
    pub path: String,
    /// Bound value
    pub value: SymbolicValue,
}

/// Symbolic value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SymbolicValue {
    /// Concrete integer value (supports hex: "0x1234")
    Integer(i64),
    /// Concrete string value
    String(String),
    /// Pointer state
    Pointer { is_null: bool, size: Option<usize> },
    /// Value range (min..max)
    Range { min: i64, max: i64 },
    /// Bitfield value with mask
    Bitfield { value: u64, mask: u64 },
    /// Enumeration value with named constant
    Enum { value: i64, name: String },
    /// Unknown value with optional hint
    Unknown { hint: Option<String> },
    /// Array of values
    Array { elements: Vec<SymbolicValue> },
}

impl SymbolicValue {
    /// Parse from string input
    pub fn parse(s: &str, type_hint: &str) -> Self {
        let s = s.trim();

        // Check for pointer types
        if type_hint == "pointer" {
            return match s.to_lowercase().as_str() {
                "null" | "0" | "nullptr" => SymbolicValue::Pointer {
                    is_null: true,
                    size: None,
                },
                "valid" | "non-null" => SymbolicValue::Pointer {
                    is_null: false,
                    size: None,
                },
                _ => SymbolicValue::Pointer {
                    is_null: false,
                    size: None,
                },
            };
        }

        // Check for range (e.g., "0..100")
        if s.contains("..") {
            if let Some((min_str, max_str)) = s.split_once("..") {
                if let (Ok(min), Ok(max)) = (
                    Self::parse_int(min_str.trim()),
                    Self::parse_int(max_str.trim()),
                ) {
                    return SymbolicValue::Range { min, max };
                }
            }
        }

        // Check for enum format: "ENUM_NAME(value)" or just "name"
        if let Some(enum_start) = s.find('(') {
            let name = &s[..enum_start];
            if let Some(enum_end) = s.find(')') {
                let value_str = &s[enum_start + 1..enum_end];
                if let Ok(val) = Self::parse_int(value_str) {
                    return SymbolicValue::Enum {
                        value: val,
                        name: name.to_string(),
                    };
                }
            }
        }

        // Try to parse as integer
        if let Ok(n) = Self::parse_int(s) {
            return SymbolicValue::Integer(n);
        }

        // Otherwise treat as string
        if s.starts_with('"') && s.ends_with('"') {
            SymbolicValue::String(s[1..s.len() - 1].to_string())
        } else {
            SymbolicValue::String(s.to_string())
        }
    }

    /// Parse an integer (decimal, hex, binary)
    fn parse_int(s: &str) -> Result<i64, ()> {
        let s = s.trim();
        if s.starts_with("0x") || s.starts_with("0X") {
            i64::from_str_radix(&s[2..], 16).map_err(|_| ())
        } else if s.starts_with("0b") || s.starts_with("0B") {
            i64::from_str_radix(&s[2..], 2).map_err(|_| ())
        } else {
            s.parse::<i64>().map_err(|_| ())
        }
    }

    /// Display value
    pub fn display(&self) -> String {
        match self {
            SymbolicValue::Integer(n) => {
                if *n > 255 {
                    format!("0x{:x}", n)
                } else {
                    n.to_string()
                }
            }
            SymbolicValue::String(s) => format!("\"{}\"", s),
            SymbolicValue::Pointer { is_null, size } => {
                if *is_null {
                    "NULL".to_string()
                } else if let Some(sz) = size {
                    format!("<ptr: {} bytes>", sz)
                } else {
                    "<valid ptr>".to_string()
                }
            }
            SymbolicValue::Range { min, max } => format!("{}..{}", min, max),
            SymbolicValue::Bitfield { value, mask } => format!("0x{:x} & 0x{:x}", value, mask),
            SymbolicValue::Enum { value, name } => format!("{}({})", name, value),
            SymbolicValue::Unknown { hint } => hint
                .as_ref()
                .map(|h| format!("<?:{}>", h))
                .unwrap_or_else(|| "?".to_string()),
            SymbolicValue::Array { elements } => {
                let elements_str: Vec<String> = elements.iter().map(|e| e.display()).collect();
                format!("[{}]", elements_str.join(", "))
            }
        }
    }

    /// Check if value is definitely zero
    pub fn is_zero(&self) -> bool {
        match self {
            SymbolicValue::Integer(n) => *n == 0,
            SymbolicValue::Pointer { is_null, .. } => *is_null,
            SymbolicValue::Range { min: _, max } => *max < 0,
            _ => false,
        }
    }

    /// Check if value is definitely non-zero
    pub fn is_nonzero(&self) -> bool {
        match self {
            SymbolicValue::Integer(n) => *n != 0,
            SymbolicValue::Pointer { is_null, .. } => !*is_null,
            SymbolicValue::Range { min, .. } => *min > 0,
            _ => false,
        }
    }

    /// Get bitfield mask if applicable
    pub fn as_bitfield(&self) -> Option<(u64, u64)> {
        match self {
            SymbolicValue::Bitfield { value, mask } => Some((*value, *mask)),
            SymbolicValue::Integer(n) if *n != 0 => {
                // Extract lowest set bit as mask
                let mask = *n as u64;
                let value = mask & mask; // Value is the masked portion
                Some((value, mask))
            }
            _ => None,
        }
    }

    /// Get enum info if applicable
    pub fn as_enum(&self) -> Option<(i64, &str)> {
        match self {
            SymbolicValue::Enum { value, name } => Some((*value, name.as_str())),
            _ => None,
        }
    }
}

/// Error type for scenario operations
#[derive(Debug)]
pub enum ScenarioError {
    /// IO error
    Io(io::Error),
    /// Serialization error (JSON)
    Json(serde_json::Error),
    /// YAML serialization error
    Yaml(String),
    /// Invalid file format
    InvalidFormat(String),
}

impl std::fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScenarioError::Io(e) => write!(f, "IO error: {}", e),
            ScenarioError::Json(e) => write!(f, "JSON error: {}", e),
            ScenarioError::Yaml(e) => write!(f, "YAML error: {}", e),
            ScenarioError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
        }
    }
}

impl std::error::Error for ScenarioError {}

impl From<io::Error> for ScenarioError {
    fn from(e: io::Error) -> Self {
        ScenarioError::Io(e)
    }
}

impl From<serde_json::Error> for ScenarioError {
    fn from(e: serde_json::Error) -> Self {
        ScenarioError::Json(e)
    }
}

impl Scenario {
    /// Create a new empty scenario
    pub fn new(name: &str, entry_function: &str) -> Self {
        Self {
            name: name.to_string(),
            entry_function: entry_function.to_string(),
            bindings: Vec::new(),
            options: ScenarioOptions::default(),
        }
    }

    /// Add a value binding
    pub fn bind(&mut self, path: &str, value: SymbolicValue) -> &mut Self {
        self.bindings.push(ValueBinding {
            path: path.to_string(),
            value,
        });
        self
    }

    /// Save scenario to a JSON file
    pub fn save_json<P: AsRef<Path>>(&self, path: P) -> Result<(), ScenarioError> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load scenario from a JSON file
    pub fn load_json<P: AsRef<Path>>(path: P) -> Result<Self, ScenarioError> {
        let content = fs::read_to_string(path)?;
        let scenario = serde_json::from_str(&content)?;
        Ok(scenario)
    }

    /// Save scenario to a YAML file
    pub fn save_yaml<P: AsRef<Path>>(&self, path: P) -> Result<(), ScenarioError> {
        let yaml = serde_yaml::to_string(self).map_err(|e| ScenarioError::Yaml(e.to_string()))?;
        fs::write(path, yaml)?;
        Ok(())
    }

    /// Load scenario from a YAML file
    pub fn load_yaml<P: AsRef<Path>>(path: P) -> Result<Self, ScenarioError> {
        let content = fs::read_to_string(path)?;
        let scenario =
            serde_yaml::from_str(&content).map_err(|e| ScenarioError::Yaml(e.to_string()))?;
        Ok(scenario)
    }

    /// Load scenario from file, auto-detecting format by extension
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ScenarioError> {
        let path = path.as_ref();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        match ext.to_lowercase().as_str() {
            "json" => Self::load_json(path),
            "yaml" | "yml" => Self::load_yaml(path),
            _ => Err(ScenarioError::InvalidFormat(format!(
                "Unknown file extension: {}",
                ext
            ))),
        }
    }

    /// Save scenario to file, auto-detecting format by extension
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ScenarioError> {
        let path = path.as_ref();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        match ext.to_lowercase().as_str() {
            "json" => self.save_json(path),
            "yaml" | "yml" => self.save_yaml(path),
            _ => Err(ScenarioError::InvalidFormat(format!(
                "Unknown file extension: {}",
                ext
            ))),
        }
    }
}

/// A collection of scenarios for a project
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScenarioCollection {
    /// Collection name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// List of scenarios
    pub scenarios: Vec<Scenario>,
}

impl ScenarioCollection {
    /// Create a new empty collection
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            scenarios: Vec::new(),
        }
    }

    /// Add a scenario to the collection
    pub fn add(&mut self, scenario: Scenario) {
        self.scenarios.push(scenario);
    }

    /// Find a scenario by name
    pub fn find(&self, name: &str) -> Option<&Scenario> {
        self.scenarios.iter().find(|s| s.name == name)
    }

    /// Save collection to a JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ScenarioError> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load collection from a JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ScenarioError> {
        let content = fs::read_to_string(path)?;
        let collection = serde_json::from_str(&content)?;
        Ok(collection)
    }
}

/// Scenario execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioOptions {
    /// Follow async callbacks
    #[serde(default = "default_true")]
    pub follow_async: bool,
    /// Show kernel API calls
    #[serde(default = "default_true")]
    pub show_kernel_api: bool,
    /// Maximum recursion depth
    #[serde(default = "default_depth")]
    pub max_depth: usize,
    /// Enable multi-path exploration
    #[serde(default = "default_true")]
    pub multi_path: bool,
    /// Maximum paths to explore
    #[serde(default = "default_max_paths")]
    pub max_paths: usize,
    /// Enable constraint propagation
    #[serde(default = "default_true")]
    pub propagate_constraints: bool,
    /// Record execution traces
    #[serde(default = "default_true")]
    pub record_traces: bool,
}

fn default_true() -> bool {
    true
}

fn default_depth() -> usize {
    10
}

fn default_max_paths() -> usize {
    100
}

impl Default for ScenarioOptions {
    fn default() -> Self {
        Self {
            follow_async: true,
            show_kernel_api: true,
            max_depth: 10,
            multi_path: true,
            max_paths: 100,
            propagate_constraints: true,
            record_traces: true,
        }
    }
}

/// Program state at a specific point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramState {
    /// Current location
    pub location: Location,
    /// Function name
    pub function: String,
    /// Variable values at this point
    pub variables: HashMap<String, SymbolicValue>,
    /// Branch condition (if at a branch point)
    pub branch_condition: Option<String>,
    /// Whether this path is reachable
    pub reachable: bool,
    /// Call depth
    pub depth: usize,
}

/// Result of multi-path scenario execution
#[derive(Debug, Clone)]
pub struct MultiPathResult {
    /// Primary/first execution path
    pub primary_path: ExecutionPath,
    /// Alternative paths discovered
    pub alternative_paths: Vec<ExecutionPath>,
    /// Annotated flow tree with variable annotations
    pub annotated_tree: Option<FlowNode>,
}

impl MultiPathResult {
    /// Get total number of paths discovered
    pub fn path_count(&self) -> usize {
        1 + self.alternative_paths.len()
    }

    /// Get all paths
    pub fn all_paths(&self) -> Vec<&ExecutionPath> {
        std::iter::once(&self.primary_path)
            .chain(self.alternative_paths.iter())
            .collect()
    }

    /// Check if there are multiple paths
    pub fn has_branches(&self) -> bool {
        !self.alternative_paths.is_empty()
    }
}

/// Execution path result (legacy, for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPath {
    /// Unique path identifier
    pub id: String,
    /// Human-readable name/description
    pub name: String,
    /// Step-by-step execution sequence
    pub steps: Vec<crate::path_tracing::PathStep>,
    /// Branch decisions made along this path
    pub branches: Vec<crate::path_tracing::BranchInfo>,
    /// Variables with their values at the end of the path
    pub final_values: HashMap<String, crate::path_tracing::PathValue>,
    /// Whether this path leads to completion
    pub completed: bool,
    /// Termination reason (if not completed)
    pub termination_reason: Option<String>,
    /// Maximum call depth reached
    pub max_depth: usize,
    /// Total steps in path
    pub step_count: usize,
}

impl ExecutionPath {
    /// Create a new execution path
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            steps: Vec::new(),
            branches: Vec::new(),
            final_values: HashMap::new(),
            completed: false,
            termination_reason: None,
            max_depth: 0,
            step_count: 0,
        }
    }

    /// Get path length
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Check if path is empty
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

impl From<crate::path_tracing::ExecutionPath> for ExecutionPath {
    fn from(path: crate::path_tracing::ExecutionPath) -> Self {
        Self {
            id: path.id,
            name: path.name,
            steps: path.steps,
            branches: path.branches,
            final_values: path.final_values,
            completed: path.completed,
            termination_reason: path.termination_reason,
            max_depth: path.max_depth,
            step_count: path.step_count,
        }
    }
}

/// Scenario executor with constant propagation and multi-path exploration
pub struct ScenarioExecutor {
    /// Constant propagator for branch analysis
    propagator: ConstantPropagator,
    /// Current execution path being built
    path: Vec<ProgramState>,
    /// Options
    options: ScenarioOptions,
    /// Execution tracer for path recording
    tracer: ExecutionTracer,
    /// Discovered paths during multi-path exploration
    discovered_paths: Vec<ExecutionPath>,
    /// Current path ID for multi-path exploration
    path_id: usize,
}

impl ScenarioExecutor {
    /// Create a new executor
    pub fn new(options: ScenarioOptions) -> Self {
        Self {
            propagator: ConstantPropagator::new(),
            path: Vec::new(),
            options,
            tracer: ExecutionTracer::new(),
            discovered_paths: Vec::new(),
            path_id: 0,
        }
    }

    /// Execute scenario on a flow tree
    pub fn execute(&mut self, scenario: &Scenario, flow_tree: &FlowNode) -> MultiPathResult {
        // Initialize propagator from scenario bindings
        let bindings: Vec<_> = scenario
            .bindings
            .iter()
            .map(|b| (b.path.clone(), b.value.clone()))
            .collect();
        self.propagator.init_from_bindings(&bindings);

        self.path.clear();
        self.discovered_paths.clear();
        self.path_id = 0;

        if self.options.multi_path {
            // Multi-path exploration mode
            self.execute_multi_path(scenario, flow_tree)
        } else {
            // Single path mode - use tracer to trace the tree
            self.tracer.trace_flow_tree(flow_tree);

            MultiPathResult {
                primary_path: ExecutionPath {
                    id: "primary".to_string(),
                    name: "Primary Execution Path".to_string(),
                    steps: self.convert_states_to_steps(),
                    branches: self.extract_branches(),
                    final_values: self.extract_final_values(),
                    completed: true,
                    termination_reason: None,
                    max_depth: self.path.iter().map(|s| s.depth).max().unwrap_or(0),
                    step_count: self.path.len(),
                },
                alternative_paths: Vec::new(),
                annotated_tree: None,
            }
        }
    }

    /// Multi-path exploration: explore all feasible paths
    fn execute_multi_path(&mut self, scenario: &Scenario, flow_tree: &FlowNode) -> MultiPathResult {
        // Record entry point in tracer
        self.tracer.start_path(
            format!("path_{}", self.path_id),
            format!("Path {}", self.path_id),
        );

        // Start exploring from root
        self.explore_path(flow_tree, scenario, 0, true);

        // Build annotated tree with branch information
        let annotated_tree =
            self.build_annotated_tree(flow_tree, scenario, 0, true, &HashSet::new());

        MultiPathResult {
            primary_path: self
                .discovered_paths
                .first()
                .cloned()
                .unwrap_or_else(|| ExecutionPath {
                    id: "primary".to_string(),
                    name: "Primary Path".to_string(),
                    steps: Vec::new(),
                    branches: Vec::new(),
                    final_values: HashMap::new(),
                    completed: false,
                    termination_reason: Some("No paths explored".to_string()),
                    max_depth: 0,
                    step_count: 0,
                }),
            alternative_paths: self
                .discovered_paths
                .get(1..)
                .map(|s| s.to_vec())
                .unwrap_or_default(),
            annotated_tree: Some(annotated_tree),
        }
    }

    /// Explore a single path recursively
    fn explore_path(
        &mut self,
        node: &FlowNode,
        scenario: &Scenario,
        depth: usize,
        reachable: bool,
    ) {
        if depth > self.options.max_depth {
            return;
        }

        // Check if we should stop exploring
        if self.discovered_paths.len() >= self.options.max_paths {
            return;
        }

        if !reachable {
            // Record unreachable path
            self.finalize_current_path(false, Some("Unreachable code"));
            return;
        }

        // Record this node in current path
        let args: Vec<PathValue> = self.extract_arguments_for_node(node, scenario);
        self.tracer.record_call(
            &node.name,
            node.location.clone(),
            self.classify_call(node),
            &args,
        );

        // Get current branch conditions from node
        let branch_conditions = self.extract_branch_conditions(node);

        if branch_conditions.is_empty() {
            // No branching - continue to children
            let show_kernel_api = self.options.show_kernel_api;
            for child in &node.children {
                if !show_kernel_api && matches!(child.node_type, FlowNodeType::KernelApi) {
                    continue;
                }
                self.explore_path(child, scenario, depth + 1, true);
            }
        } else {
            // Has branching - explore each branch
            for (i, condition) in branch_conditions.iter().enumerate() {
                // Clone state for this branch
                let saved_state = self.propagator.clone_state();

                // Determine branch outcome
                let branch_taken = match self.propagator.eval_condition(condition) {
                    BranchResult::AlwaysTrue => BranchOutcome::True,
                    BranchResult::AlwaysFalse => BranchOutcome::False,
                    BranchResult::Unknown => {
                        if i == 0 {
                            BranchOutcome::True
                        } else {
                            BranchOutcome::False
                        }
                    }
                };

                // Record branch decision
                self.tracer
                    .record_branch(node.location.clone(), condition, branch_taken);

                // Finalize current path and start new one for alternative
                self.finalize_current_path(true, None);
                self.path_id += 1;
                self.tracer.start_path(
                    format!("path_{}", self.path_id),
                    format!("Path {}", self.path_id),
                );
                // Re-record the call in new path
                let args: Vec<PathValue> = self.extract_arguments_for_node(node, scenario);
                self.tracer.record_call(
                    &node.name,
                    node.location.clone(),
                    self.classify_call(node),
                    &args,
                );

                // Explore this branch
                let child_reachable = matches!(
                    branch_taken,
                    BranchOutcome::True | BranchOutcome::Fallthrough(_)
                );
                let children: Vec<_> = node
                    .children
                    .iter()
                    .filter(|c| {
                        if !self.options.show_kernel_api
                            && matches!(c.node_type, FlowNodeType::KernelApi)
                        {
                            return false;
                        }
                        true
                    })
                    .collect();

                if let Some(child) = children.get(i).or(children.first()) {
                    self.explore_path(child, scenario, depth + 1, child_reachable);
                }

                // Restore state for next iteration
                self.propagator.restore_state(saved_state);
            }
        }

        // Record return
        self.tracer.record_return(None);

        // Finalize path for leaf nodes (no children and no branches to explore)
        if node.children.is_empty() || depth >= self.options.max_depth {
            self.finalize_current_path(true, None);
        }
    }

    /// Finalize current path and store it
    fn finalize_current_path(&mut self, completed: bool, reason: Option<&str>) {
        self.tracer.finalize_path(completed, reason);
        if let Some(path) = self.tracer.paths().last().cloned() {
            self.discovered_paths.push(path.into());
        }
    }

    /// Convert program states to path steps
    fn convert_states_to_steps(&self) -> Vec<crate::path_tracing::PathStep> {
        self.path
            .iter()
            .enumerate()
            .map(|(i, state)| crate::path_tracing::PathStep {
                step_id: i,
                function: state.function.clone(),
                location: Some(state.location.clone()),
                arguments: Vec::new(),
                return_value: None,
                call_type: crate::path_tracing::CallCategory::Direct,
                depth: state.depth,
                sequence: i as u64,
            })
            .collect()
    }

    /// Extract branch information from visited nodes
    fn extract_branches(&self) -> Vec<crate::path_tracing::BranchInfo> {
        self.path
            .iter()
            .filter_map(|state| {
                state.branch_condition.as_ref().map(|cond| {
                    crate::path_tracing::BranchInfo {
                        location: Some(state.location.clone()),
                        condition: cond.clone(),
                        taken: BranchOutcome::True, // Would need more info
                        alternative: None,
                    }
                })
            })
            .collect()
    }

    /// Extract final variable values
    fn extract_final_values(&self) -> HashMap<String, PathValue> {
        let mut values = HashMap::new();
        for (path, sym_val) in self.propagator.all_vars() {
            let value_type = match sym_val {
                SymbolicValue::Integer(_) => ValueType::Integer,
                SymbolicValue::Pointer { .. } => ValueType::Pointer,
                SymbolicValue::String(_) => ValueType::String,
                SymbolicValue::Range { .. } => ValueType::Integer,
                SymbolicValue::Bitfield { .. } => ValueType::Integer,
                SymbolicValue::Enum { .. } => ValueType::Integer,
                SymbolicValue::Unknown { .. } => ValueType::Unknown,
                SymbolicValue::Array { .. } => ValueType::Unknown,
            };
            values.insert(
                path.clone(),
                PathValue {
                    name: path.clone(),
                    concrete: Some(sym_val.display()),
                    symbolic: None,
                    value_type,
                },
            );
        }
        values
    }

    /// Extract branch conditions from node name/description
    fn extract_branch_conditions(&self, node: &FlowNode) -> Vec<String> {
        let mut conditions = Vec::new();

        // Check node name for condition patterns
        if node.name.contains("_if_") || node.name.contains("_check_") {
            if let Some(cond) = self.extract_condition_from_name(&node.name) {
                conditions.push(cond);
            }
        }

        // Check description for conditions
        if let Some(desc) = &node.description {
            if desc.contains("if ") || desc.contains("when ") {
                if let Some(cond) = self.extract_condition_from_desc(desc) {
                    conditions.push(cond);
                }
            }
        }

        conditions
    }

    /// Extract condition from node name
    fn extract_condition_from_name(&self, name: &str) -> Option<String> {
        // Pattern: if_ptr_null -> ptr == NULL
        if name.contains("_null") {
            let var = name
                .replace("if_", "")
                .replace("_null", "")
                .replace("_check", "");
            return Some(format!("{} == NULL", var));
        }
        if name.contains("_valid") || name.contains("_not_null") {
            let var = name
                .replace("if_", "")
                .replace("_valid", "")
                .replace("_not_null", "")
                .replace("_check", "");
            return Some(format!("{} != NULL", var));
        }
        // Pattern: if_x_gt_0 -> x > 0
        if name.contains("_gt_") {
            if let Some(rest) = name.split("_gt_").nth(1) {
                if let Some(val) = rest.split('_').next() {
                    let var = name.replace("if_", "").replace("_gt_", "");
                    return Some(format!("{} > {}", var, val));
                }
            }
        }
        None
    }

    /// Extract condition from description
    fn extract_condition_from_desc(&self, desc: &str) -> Option<String> {
        // Simple patterns
        if desc.contains("== NULL") {
            return Some(desc.to_string());
        }
        if desc.contains("!= NULL") {
            return Some(desc.to_string());
        }
        None
    }

    /// Classify the type of call
    fn classify_call(&self, node: &FlowNode) -> CallCategory {
        match node.node_type {
            FlowNodeType::Function => CallCategory::Direct,
            FlowNodeType::EntryPoint => CallCategory::Direct,
            FlowNodeType::AsyncCallback { .. } => CallCategory::AsyncCallback,
            FlowNodeType::KernelApi => CallCategory::KernelApi,
            FlowNodeType::External => CallCategory::IndirectPointer,
            FlowNodeType::Separator { .. } => CallCategory::Direct, // Not a call, but use Direct as default
            FlowNodeType::Branch { .. } => CallCategory::Direct,    // Control flow, not a call
        }
    }

    /// Extract arguments for a node from scenario bindings
    fn extract_arguments_for_node(&self, node: &FlowNode, scenario: &Scenario) -> Vec<PathValue> {
        scenario
            .bindings
            .iter()
            .filter(|b| b.path.contains(&node.name) || b.path.contains("arg"))
            .map(|b| {
                let value_type = match b.value {
                    SymbolicValue::Integer(_) => ValueType::Integer,
                    SymbolicValue::Pointer { .. } => ValueType::Pointer,
                    SymbolicValue::String(_) => ValueType::String,
                    _ => ValueType::Unknown,
                };
                PathValue::from_concrete(&b.path, b.value.display(), value_type)
            })
            .collect()
    }

    /// Build annotated tree with branch information
    fn build_annotated_tree(
        &mut self,
        node: &FlowNode,
        _scenario: &Scenario,
        depth: usize,
        reachable: bool,
        _visited: &HashSet<String>,
    ) -> FlowNode {
        if depth > self.options.max_depth {
            return node.clone();
        }

        // Determine reachability
        let child_reachable = if reachable {
            self.check_branch_reachability(node)
        } else {
            false
        };

        // Build description with variable values
        let description = self.build_description(node, reachable);

        // Filter and process children
        let show_kernel_api = self.options.show_kernel_api;
        let filtered_children: Vec<_> = node
            .children
            .iter()
            .filter(|child| {
                if !show_kernel_api && matches!(child.node_type, FlowNodeType::KernelApi) {
                    return false;
                }
                true
            })
            .collect();

        let children: Vec<FlowNode> = filtered_children
            .into_iter()
            .map(|child| {
                self.build_annotated_tree(child, _scenario, depth + 1, child_reachable, _visited)
            })
            .collect();

        FlowNode {
            id: node.id.clone(),
            name: node.name.clone(),
            display_name: node.display_name.clone(),
            location: node.location.clone(),
            node_type: node.node_type.clone(),
            children,
            description: Some(description),
            confidence: node.confidence.clone(),
            execution_context: node.execution_context.clone(),
            can_sleep: node.can_sleep,
            source_file: node.source_file.clone(),
            is_kernel_internal: node.is_kernel_internal,
        }
    }

    /// Check if a branch is reachable based on conditions
    fn check_branch_reachability(&mut self, node: &FlowNode) -> bool {
        let name = &node.name;

        if name.contains("if_") || name.contains("_check") {
            if let Some(condition) = self.extract_condition_from_name(name) {
                match self.propagator.eval_condition(&condition) {
                    BranchResult::AlwaysFalse => return false,
                    BranchResult::AlwaysTrue | BranchResult::Unknown => return true,
                }
            }
        }

        true
    }

    fn build_description(&self, node: &FlowNode, reachable: bool) -> String {
        let mut desc = Vec::new();

        if !reachable {
            desc.push("[unreachable]".to_string());
        }

        if let Some(loc) = &node.location {
            desc.push(format!("L{}", loc.line));
        }

        let vars = self.propagator.all_vars();
        for (path, value) in vars {
            if self.is_relevant_variable(path, &node.name) {
                desc.push(format!("{}={}", path, value.display()));
            }
        }

        if desc.is_empty() {
            node.description.clone().unwrap_or_default()
        } else {
            desc.join(" | ")
        }
    }

    fn is_relevant_variable(&self, path: &str, _func_name: &str) -> bool {
        !path.is_empty()
    }
}

/// Annotate a flow tree with scenario values
pub fn annotate_flow_tree(flow_tree: &FlowNode, scenario: &Scenario) -> FlowNode {
    let mut executor = ScenarioExecutor::new(scenario.options.clone());
    let result = executor.execute(scenario, flow_tree);
    result.annotated_tree.unwrap_or_else(|| flow_tree.clone())
}

/// Execute scenario and return all paths
pub fn execute_scenario(scenario: &Scenario, flow_tree: &FlowNode) -> MultiPathResult {
    let mut executor = ScenarioExecutor::new(scenario.options.clone());
    executor.execute(scenario, flow_tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowsight_core::{ExecutionContext, FlowNodeType};

    #[test]
    fn test_parse_integer() {
        assert!(matches!(
            SymbolicValue::parse("42", "integer"),
            SymbolicValue::Integer(42)
        ));

        assert!(matches!(
            SymbolicValue::parse("0x1234", "integer"),
            SymbolicValue::Integer(0x1234)
        ));
    }

    #[test]
    fn test_parse_pointer() {
        assert!(matches!(
            SymbolicValue::parse("null", "pointer"),
            SymbolicValue::Pointer { is_null: true, .. }
        ));

        assert!(matches!(
            SymbolicValue::parse("valid", "pointer"),
            SymbolicValue::Pointer { is_null: false, .. }
        ));
    }

    #[test]
    fn test_parse_range() {
        if let SymbolicValue::Range { min, max } = SymbolicValue::parse("0..100", "range") {
            assert_eq!(min, 0);
            assert_eq!(max, 100);
        } else {
            panic!("Expected Range");
        }
    }

    #[test]
    fn test_scenario_executor_basic() {
        let scenario = Scenario {
            name: "test".to_string(),
            entry_function: "main".to_string(),
            bindings: vec![ValueBinding {
                path: "x".to_string(),
                value: SymbolicValue::Integer(42),
            }],
            options: ScenarioOptions::default(),
        };

        let flow_tree = FlowNode {
            id: "1".to_string(),
            name: "main".to_string(),
            display_name: "main()".to_string(),
            location: Some(Location::new("test.c", 1, 0)),
            node_type: FlowNodeType::Function,
            children: vec![],
            description: None,
            confidence: None,
            execution_context: Some(flowsight_core::ExecutionContext::Process),
            can_sleep: Some(true),
            source_file: None,
            is_kernel_internal: false,
        };

        let mut executor = ScenarioExecutor::new(ScenarioOptions::default());
        let result = executor.execute(&scenario, &flow_tree);

        // Check primary path has steps
        assert!(result.primary_path.step_count > 0 || result.primary_path.completed);
    }

    #[test]
    fn test_scenario_with_multi_path() {
        let scenario = Scenario {
            name: "multi_path_test".to_string(),
            entry_function: "branch_func".to_string(),
            bindings: vec![],
            options: ScenarioOptions {
                multi_path: true,
                max_paths: 10,
                ..Default::default()
            },
        };

        let flow_tree = FlowNode {
            id: "root".to_string(),
            name: "branch_func".to_string(),
            display_name: "branch_func()".to_string(),
            location: Some(Location::new("test.c", 1, 0)),
            node_type: FlowNodeType::Function,
            children: vec![
                FlowNode {
                    id: "child1".to_string(),
                    name: "if_true".to_string(),
                    display_name: "if_true()".to_string(),
                    location: None,
                    node_type: FlowNodeType::Function,
                    children: vec![],
                    description: None,
                    confidence: None,
                    execution_context: None,
                    can_sleep: None,
                    source_file: None,
                    is_kernel_internal: false,
                },
                FlowNode {
                    id: "child2".to_string(),
                    name: "if_false".to_string(),
                    display_name: "if_false()".to_string(),
                    location: None,
                    node_type: FlowNodeType::Function,
                    children: vec![],
                    description: None,
                    confidence: None,
                    execution_context: None,
                    can_sleep: None,
                    source_file: None,
                    is_kernel_internal: false,
                },
            ],
            description: None,
            confidence: None,
            execution_context: None,
            can_sleep: None,
            source_file: None,
            is_kernel_internal: false,
        };

        let mut executor = ScenarioExecutor::new(scenario.options.clone());
        let result = executor.execute(&scenario, &flow_tree);

        // Should have explored at least one path
        assert!(result.primary_path.step_count > 0);
    }
    #[test]
    fn test_scenario_executor_with_null_check() {
        let scenario = Scenario {
            name: "null_test".to_string(),
            entry_function: "check_ptr".to_string(),
            bindings: vec![ValueBinding {
                path: "ptr".to_string(),
                value: SymbolicValue::Pointer {
                    is_null: true,
                    size: None,
                },
            }],
            options: ScenarioOptions::default(),
        };

        let flow_tree = FlowNode {
            id: "1".to_string(),
            name: "check_ptr".to_string(),
            display_name: "check_ptr()".to_string(),
            location: Some(Location::new("test.c", 1, 0)),
            node_type: FlowNodeType::Function,
            children: vec![FlowNode {
                id: "2".to_string(),
                name: "if_ptr_null".to_string(),
                display_name: "if (ptr == NULL)".to_string(),
                location: Some(Location::new("test.c", 2, 0)),
                node_type: FlowNodeType::Function,
                children: vec![],
                description: None,
                confidence: None,
                execution_context: Some(ExecutionContext::Process),
                can_sleep: Some(true),
                source_file: None,
                is_kernel_internal: false,
            }],
            description: None,
            confidence: None,
            execution_context: Some(ExecutionContext::Process),
            can_sleep: Some(true),
            source_file: None,
            is_kernel_internal: false,
        };

        let mut executor = ScenarioExecutor::new(ScenarioOptions::default());
        let result = executor.execute(&scenario, &flow_tree);

        assert!(result.primary_path.completed);
        // The if_ptr_null branch should be reachable since ptr is NULL
        let tree = result.annotated_tree.unwrap();
        assert!(!tree.children.is_empty());
    }

    #[test]
    fn test_scenario_new_and_bind() {
        let mut scenario = Scenario::new("usb_probe_test", "usb_probe");
        scenario
            .bind("id->idVendor", SymbolicValue::Integer(0x1234))
            .bind("id->idProduct", SymbolicValue::Integer(0x5678))
            .bind(
                "interface",
                SymbolicValue::Pointer {
                    is_null: false,
                    size: None,
                },
            );

        assert_eq!(scenario.name, "usb_probe_test");
        assert_eq!(scenario.entry_function, "usb_probe");
        assert_eq!(scenario.bindings.len(), 3);
    }

    #[test]
    fn test_scenario_save_load_json() {
        let mut scenario = Scenario::new("test_scenario", "main");
        scenario.bind("x", SymbolicValue::Integer(42)).bind(
            "ptr",
            SymbolicValue::Pointer {
                is_null: false,
                size: None,
            },
        );

        // Save to a temp file
        let temp_path = std::env::temp_dir().join("test_scenario.json");
        scenario.save_json(&temp_path).expect("Failed to save JSON");

        // Load it back
        let loaded = Scenario::load_json(&temp_path).expect("Failed to load JSON");

        assert_eq!(loaded.name, "test_scenario");
        assert_eq!(loaded.entry_function, "main");
        assert_eq!(loaded.bindings.len(), 2);

        // Clean up
        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_scenario_save_load_yaml() {
        let mut scenario = Scenario::new("yaml_test", "usb_probe");
        scenario
            .bind("id->idVendor", SymbolicValue::Integer(0x1234))
            .bind("range_val", SymbolicValue::Range { min: 0, max: 100 });

        // Save to a temp file
        let temp_path = std::env::temp_dir().join("test_scenario.yaml");
        scenario.save_yaml(&temp_path).expect("Failed to save YAML");

        // Load it back
        let loaded = Scenario::load_yaml(&temp_path).expect("Failed to load YAML");

        assert_eq!(loaded.name, "yaml_test");
        assert_eq!(loaded.bindings.len(), 2);

        // Clean up
        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_scenario_auto_format_detection() {
        let scenario = Scenario::new("auto_test", "main");

        // Test JSON extension
        let json_path = std::env::temp_dir().join("auto_test.json");
        scenario.save(&json_path).expect("Failed to save");
        let loaded = Scenario::load(&json_path).expect("Failed to load");
        assert_eq!(loaded.name, "auto_test");
        let _ = fs::remove_file(&json_path);

        // Test YAML extension
        let yaml_path = std::env::temp_dir().join("auto_test.yml");
        scenario.save(&yaml_path).expect("Failed to save");
        let loaded = Scenario::load(&yaml_path).expect("Failed to load");
        assert_eq!(loaded.name, "auto_test");
        let _ = fs::remove_file(&yaml_path);
    }

    #[test]
    fn test_scenario_collection() {
        let mut collection = ScenarioCollection::new("USB Driver Tests");
        collection.description = Some("Test scenarios for USB driver".to_string());

        let mut scenario1 = Scenario::new("normal_probe", "usb_probe");
        scenario1.bind("id->idVendor", SymbolicValue::Integer(0x1234));

        let mut scenario2 = Scenario::new("null_interface", "usb_probe");
        scenario2.bind(
            "interface",
            SymbolicValue::Pointer {
                is_null: true,
                size: None,
            },
        );

        collection.add(scenario1);
        collection.add(scenario2);

        // Test find
        assert!(collection.find("normal_probe").is_some());
        assert!(collection.find("null_interface").is_some());
        assert!(collection.find("nonexistent").is_none());

        // Save and load collection
        let temp_path = std::env::temp_dir().join("test_collection.json");
        collection
            .save(&temp_path)
            .expect("Failed to save collection");

        let loaded = ScenarioCollection::load(&temp_path).expect("Failed to load collection");
        assert_eq!(loaded.name, "USB Driver Tests");
        assert_eq!(loaded.scenarios.len(), 2);

        let _ = fs::remove_file(&temp_path);
    }
}
