//! Scenario-based symbolic execution
//!
//! Core feature: Execute code symbolically with user-defined parameter values
//! to visualize execution paths and variable states.

use flowsight_core::{FlowNode, FlowNodeType, Location};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::propagation::{ConstantPropagator, BranchResult};

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
    /// Unknown value with optional hint
    Unknown { hint: Option<String> },
}

impl SymbolicValue {
    /// Parse from string input
    pub fn parse(s: &str, type_hint: &str) -> Self {
        let s = s.trim();
        
        // Check for pointer types
        if type_hint == "pointer" {
            return match s.to_lowercase().as_str() {
                "null" | "0" | "nullptr" => SymbolicValue::Pointer { is_null: true, size: None },
                "valid" | "non-null" => SymbolicValue::Pointer { is_null: false, size: None },
                _ => SymbolicValue::Pointer { is_null: false, size: None },
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
            SymbolicValue::Unknown { hint } => {
                hint.as_ref().map(|h| format!("<?:{}>", h)).unwrap_or_else(|| "?".to_string())
            }
        }
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
}

fn default_true() -> bool {
    true
}

fn default_depth() -> usize {
    10
}

impl Default for ScenarioOptions {
    fn default() -> Self {
        Self {
            follow_async: true,
            show_kernel_api: true,
            max_depth: 10,
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
}

/// Execution path result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPath {
    /// States along the path
    pub states: Vec<ProgramState>,
    /// Whether the path completed normally
    pub completed: bool,
    /// Termination reason (if not completed)
    pub termination_reason: Option<String>,
    /// Flow tree with variable annotations
    pub flow_tree: Option<FlowNode>,
}

/// Scenario executor
pub struct ScenarioExecutor {
    /// Current variable bindings
    bindings: HashMap<String, SymbolicValue>,
    /// Execution path being built
    path: Vec<ProgramState>,
    /// Options
    options: ScenarioOptions,
}

impl ScenarioExecutor {
    /// Create a new executor
    pub fn new(options: ScenarioOptions) -> Self {
        Self {
            bindings: HashMap::new(),
            path: Vec::new(),
            options,
        }
    }
    
    /// Execute scenario on a flow tree
    pub fn execute(&mut self, scenario: &Scenario, flow_tree: &FlowNode) -> ExecutionPath {
        // Initialize bindings from scenario
        self.bindings.clear();
        for binding in &scenario.bindings {
            self.bindings.insert(binding.path.clone(), binding.value.clone());
        }
        
        self.path.clear();
        
        // Walk the flow tree and build execution path
        let annotated_tree = self.walk_tree(flow_tree, 0);
        
        ExecutionPath {
            states: self.path.clone(),
            completed: true,
            termination_reason: None,
            flow_tree: Some(annotated_tree),
        }
    }
    
    fn walk_tree(&mut self, node: &FlowNode, depth: usize) -> FlowNode {
        if depth > self.options.max_depth {
            return node.clone();
        }
        
        // Record state at this point
        let state = ProgramState {
            location: node.location.clone().unwrap_or_default(),
            function: node.name.clone(),
            variables: self.bindings.clone(),
            branch_condition: None,
            reachable: true,
        };
        self.path.push(state);
        
        // Build description with variable values
        let description = self.build_description(node);
        
        // Filter children first to avoid borrow issues
        let show_kernel_api = self.options.show_kernel_api;
        let filtered_children: Vec<_> = node.children.iter()
            .filter(|child| {
                // Filter based on options
                if !show_kernel_api {
                    if matches!(child.node_type, FlowNodeType::KernelApi) {
                        return false;
                    }
                }
                true
            })
            .collect();
        
        // Process children
        let children: Vec<FlowNode> = filtered_children.into_iter()
            .map(|child| self.walk_tree(child, depth + 1))
            .collect();
        
        FlowNode {
            id: node.id.clone(),
            name: node.name.clone(),
            display_name: node.display_name.clone(),
            location: node.location.clone(),
            node_type: node.node_type.clone(),
            children,
            description: Some(description),
        }
    }
    
    fn build_description(&self, node: &FlowNode) -> String {
        let mut desc = Vec::new();
        
        // Add location info
        if let Some(loc) = &node.location {
            desc.push(format!("L{}", loc.line));
        }
        
        // Add relevant variable values
        // Try to match variable names in function context
        for (path, value) in &self.bindings {
            // Simple heuristic: show variables that might be relevant
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
        // For now, show all bound variables
        // In the future, could do more sophisticated matching
        !path.is_empty()
    }
}

/// Annotate a flow tree with scenario values
pub fn annotate_flow_tree(
    flow_tree: &FlowNode,
    scenario: &Scenario,
) -> FlowNode {
    let mut executor = ScenarioExecutor::new(scenario.options.clone());
    let result = executor.execute(scenario, flow_tree);
    result.flow_tree.unwrap_or_else(|| flow_tree.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
}

