//! Scenario-based symbolic execution
//!
//! Core feature: Execute code symbolically with user-defined parameter values
//! to visualize execution paths and variable states.

use flowsight_core::{ExecutionContext, FlowNode, FlowNodeType, Location};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

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
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| ScenarioError::Yaml(e.to_string()))?;
        fs::write(path, yaml)?;
        Ok(())
    }

    /// Load scenario from a YAML file
    pub fn load_yaml<P: AsRef<Path>>(path: P) -> Result<Self, ScenarioError> {
        let content = fs::read_to_string(path)?;
        let scenario = serde_yaml::from_str(&content)
            .map_err(|e| ScenarioError::Yaml(e.to_string()))?;
        Ok(scenario)
    }

    /// Load scenario from file, auto-detecting format by extension
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ScenarioError> {
        let path = path.as_ref();
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        match ext.to_lowercase().as_str() {
            "json" => Self::load_json(path),
            "yaml" | "yml" => Self::load_yaml(path),
            _ => Err(ScenarioError::InvalidFormat(
                format!("Unknown file extension: {}", ext)
            )),
        }
    }

    /// Save scenario to file, auto-detecting format by extension
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ScenarioError> {
        let path = path.as_ref();
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        match ext.to_lowercase().as_str() {
            "json" => self.save_json(path),
            "yaml" | "yml" => self.save_yaml(path),
            _ => Err(ScenarioError::InvalidFormat(
                format!("Unknown file extension: {}", ext)
            )),
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

/// Scenario executor with constant propagation
pub struct ScenarioExecutor {
    /// Constant propagator for branch analysis
    propagator: ConstantPropagator,
    /// Execution path being built
    path: Vec<ProgramState>,
    /// Options
    options: ScenarioOptions,
}

impl ScenarioExecutor {
    /// Create a new executor
    pub fn new(options: ScenarioOptions) -> Self {
        Self {
            propagator: ConstantPropagator::new(),
            path: Vec::new(),
            options,
        }
    }

    /// Execute scenario on a flow tree
    pub fn execute(&mut self, scenario: &Scenario, flow_tree: &FlowNode) -> ExecutionPath {
        // Initialize propagator from scenario bindings
        let bindings: Vec<_> = scenario.bindings.iter()
            .map(|b| (b.path.clone(), b.value.clone()))
            .collect();
        self.propagator.init_from_bindings(&bindings);

        self.path.clear();

        // Walk the flow tree and build execution path
        let annotated_tree = self.walk_tree(flow_tree, 0, true);

        ExecutionPath {
            states: self.path.clone(),
            completed: true,
            termination_reason: None,
            flow_tree: Some(annotated_tree),
        }
    }

    fn walk_tree(&mut self, node: &FlowNode, depth: usize, reachable: bool) -> FlowNode {
        if depth > self.options.max_depth {
            return node.clone();
        }

        // Get current variable values for state
        let variables = self.propagator.all_vars().clone();

        // Record state at this point
        let state = ProgramState {
            location: node.location.clone().unwrap_or_default(),
            function: node.name.clone(),
            variables,
            branch_condition: None,
            reachable,
        };
        self.path.push(state);

        // Build description with variable values
        let description = self.build_description(node, reachable);

        // Determine node type for reachability display
        let node_type = if reachable {
            node.node_type.clone()
        } else {
            // Mark unreachable nodes (could add a new type or use description)
            node.node_type.clone()
        };

        // Filter and process children
        let show_kernel_api = self.options.show_kernel_api;
        let filtered_children: Vec<_> = node.children.iter()
            .filter(|child| {
                if !show_kernel_api && matches!(child.node_type, FlowNodeType::KernelApi) {
                    return false;
                }
                true
            })
            .collect();

        // Process children with reachability
        let children: Vec<FlowNode> = filtered_children.into_iter()
            .map(|child| {
                // Check if this is a conditional branch
                let child_reachable = if reachable {
                    self.check_branch_reachability(child)
                } else {
                    false // Parent unreachable means children unreachable
                };
                self.walk_tree(child, depth + 1, child_reachable)
            })
            .collect();

        FlowNode {
            id: node.id.clone(),
            name: node.name.clone(),
            display_name: node.display_name.clone(),
            location: node.location.clone(),
            node_type,
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
        // Check if node name contains condition hints
        let name = &node.name;

        // Look for common condition patterns in function names
        if name.contains("if_") || name.contains("_check") {
            // Try to extract and evaluate condition
            if let Some(condition) = self.extract_condition(name) {
                match self.propagator.eval_condition(&condition) {
                    BranchResult::AlwaysFalse => return false,
                    BranchResult::AlwaysTrue => return true,
                    BranchResult::Unknown => return true, // Assume reachable if unknown
                }
            }
        }

        // Default: assume reachable
        true
    }

    /// Try to extract a condition from node name or description
    fn extract_condition(&self, name: &str) -> Option<String> {
        // Simple heuristic: look for patterns like "if_ptr_null" -> "ptr == NULL"
        if name.contains("_null") {
            let var = name.replace("if_", "").replace("_null", "").replace("_check", "");
            return Some(format!("{} == NULL", var));
        }
        if name.contains("_valid") || name.contains("_not_null") {
            let var = name.replace("if_", "").replace("_valid", "").replace("_not_null", "").replace("_check", "");
            return Some(format!("{} != NULL", var));
        }
        None
    }

    fn build_description(&self, node: &FlowNode, reachable: bool) -> String {
        let mut desc = Vec::new();

        // Add reachability indicator
        if !reachable {
            desc.push("[unreachable]".to_string());
        }

        // Add location info
        if let Some(loc) = &node.location {
            desc.push(format!("L{}", loc.line));
        }

        // Add relevant variable values
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
    use flowsight_core::FlowNodeType;

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
            bindings: vec![
                ValueBinding {
                    path: "x".to_string(),
                    value: SymbolicValue::Integer(42),
                },
            ],
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
            execution_context: Some(ExecutionContext::Process),
            can_sleep: Some(true),
            source_file: None,
            is_kernel_internal: false,
        };

        let mut executor = ScenarioExecutor::new(ScenarioOptions::default());
        let result = executor.execute(&scenario, &flow_tree);

        assert!(result.completed);
        assert!(result.flow_tree.is_some());
        assert!(!result.states.is_empty());
        assert!(result.states[0].reachable);
    }

    #[test]
    fn test_scenario_executor_with_null_check() {
        let scenario = Scenario {
            name: "null_test".to_string(),
            entry_function: "check_ptr".to_string(),
            bindings: vec![
                ValueBinding {
                    path: "ptr".to_string(),
                    value: SymbolicValue::Pointer { is_null: true, size: None },
                },
            ],
            options: ScenarioOptions::default(),
        };

        let flow_tree = FlowNode {
            id: "1".to_string(),
            name: "check_ptr".to_string(),
            display_name: "check_ptr()".to_string(),
            location: Some(Location::new("test.c", 1, 0)),
            node_type: FlowNodeType::Function,
            children: vec![
                FlowNode {
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
                },
            ],
            description: None,
            confidence: None,
            execution_context: Some(ExecutionContext::Process),
            can_sleep: Some(true),
            source_file: None,
            is_kernel_internal: false,
        };

        let mut executor = ScenarioExecutor::new(ScenarioOptions::default());
        let result = executor.execute(&scenario, &flow_tree);

        assert!(result.completed);
        // The if_ptr_null branch should be reachable since ptr is NULL
        let tree = result.flow_tree.unwrap();
        assert!(!tree.children.is_empty());
    }

    #[test]
    fn test_scenario_new_and_bind() {
        let mut scenario = Scenario::new("usb_probe_test", "usb_probe");
        scenario
            .bind("id->idVendor", SymbolicValue::Integer(0x1234))
            .bind("id->idProduct", SymbolicValue::Integer(0x5678))
            .bind("interface", SymbolicValue::Pointer { is_null: false, size: None });

        assert_eq!(scenario.name, "usb_probe_test");
        assert_eq!(scenario.entry_function, "usb_probe");
        assert_eq!(scenario.bindings.len(), 3);
    }

    #[test]
    fn test_scenario_save_load_json() {
        let mut scenario = Scenario::new("test_scenario", "main");
        scenario
            .bind("x", SymbolicValue::Integer(42))
            .bind("ptr", SymbolicValue::Pointer { is_null: false, size: None });

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
        scenario2.bind("interface", SymbolicValue::Pointer { is_null: true, size: None });

        collection.add(scenario1);
        collection.add(scenario2);

        // Test find
        assert!(collection.find("normal_probe").is_some());
        assert!(collection.find("null_interface").is_some());
        assert!(collection.find("nonexistent").is_none());

        // Save and load collection
        let temp_path = std::env::temp_dir().join("test_collection.json");
        collection.save(&temp_path).expect("Failed to save collection");

        let loaded = ScenarioCollection::load(&temp_path).expect("Failed to load collection");
        assert_eq!(loaded.name, "USB Driver Tests");
        assert_eq!(loaded.scenarios.len(), 2);

        let _ = fs::remove_file(&temp_path);
    }
}

