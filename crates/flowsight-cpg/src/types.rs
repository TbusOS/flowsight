//! CPG core type definitions

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique node identifier
pub type NodeId = usize;

/// Code Property Graph for a single function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePropertyGraph {
    /// Function name
    pub function_name: String,
    /// Source file
    pub source_file: Option<String>,
    /// All variable definitions (assignments and declarations)
    pub definitions: Vec<VarDef>,
    /// All variable uses (reads)
    pub uses: Vec<VarUse>,
    /// Def-use chains: each definition → its uses
    pub def_use_chains: Vec<DefUseChain>,
    /// Reaching definitions: at each use point, which defs can reach it
    pub reaching_defs: Vec<ReachingDef>,
    /// Data flow edges (def → use pairs with variable info)
    pub data_edges: Vec<DataFlowEdge>,
    /// Function parameters (treated as initial definitions)
    pub parameters: Vec<ParameterDef>,
    /// Return statements with their values
    pub returns: Vec<ReturnInfo>,
    /// Statistics
    pub stats: CpgStats,
}

/// A variable definition (assignment or declaration with initializer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarDef {
    pub id: NodeId,
    /// Variable name being defined
    pub var_name: String,
    /// Source line
    pub line: usize,
    /// Block ID in CFG
    pub block_id: usize,
    /// Definition kind
    pub kind: DefKind,
    /// Right-hand side expression (if available)
    pub rhs_text: Option<String>,
    /// Variables used in the RHS (data dependencies)
    pub rhs_uses: Vec<String>,
    /// Type (if known from declaration)
    pub var_type: Option<String>,
}

/// How a variable is defined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefKind {
    /// Declaration with initializer: `int x = expr;`
    Declaration,
    /// Assignment: `x = expr;`
    Assignment,
    /// Compound assignment: `x += expr;`
    CompoundAssignment(String),
    /// Parameter: defined at function entry
    Parameter { index: usize },
    /// Function call return: `x = func();`
    CallReturn { callee: String },
    /// Struct field assignment: `s->field = expr;`
    FieldAssignment { base: String, field: String },
}

/// A variable use (read)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarUse {
    pub id: NodeId,
    /// Variable name being used
    pub var_name: String,
    /// Source line
    pub line: usize,
    /// Block ID in CFG
    pub block_id: usize,
    /// Context of use
    pub kind: UseKind,
}

/// How a variable is used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UseKind {
    /// In a function call argument
    Argument { callee: String, arg_index: usize },
    /// In a condition expression
    Condition,
    /// In an assignment RHS
    RhsExpression,
    /// As a return value
    ReturnValue,
    /// In a goto condition
    GotoCondition,
    /// General expression
    Expression,
    /// Pointer dereference
    Dereference,
    /// Struct field access
    FieldAccess { field: String },
}

/// A def-use chain: one definition → all its uses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefUseChain {
    /// Definition node ID
    pub def_id: NodeId,
    /// Variable name
    pub var_name: String,
    /// Definition line
    pub def_line: usize,
    /// Use node IDs that this definition reaches
    pub use_ids: Vec<NodeId>,
    /// Use lines
    pub use_lines: Vec<usize>,
}

/// A reaching definition: at a use point, which defs reach it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachingDef {
    /// Use node ID
    pub use_id: NodeId,
    /// Variable name
    pub var_name: String,
    /// Use line
    pub use_line: usize,
    /// Definition node IDs that reach this use
    pub def_ids: Vec<NodeId>,
    /// Definition lines
    pub def_lines: Vec<usize>,
}

/// A single data flow edge (definition → use)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowEdge {
    /// Definition ID
    pub from_def: NodeId,
    /// Use ID
    pub to_use: NodeId,
    /// Variable name
    pub var_name: String,
    /// Definition line
    pub def_line: usize,
    /// Use line
    pub use_line: usize,
}

/// Function parameter treated as initial definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub name: String,
    pub type_name: String,
    pub index: usize,
    pub def_id: NodeId,
}

/// Return statement info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnInfo {
    pub line: usize,
    /// Variables used in return expression
    pub used_vars: Vec<String>,
    /// Return expression text
    pub expression: Option<String>,
    /// Is this an error return?
    pub is_error: bool,
}

/// CPG statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpgStats {
    pub total_definitions: usize,
    pub total_uses: usize,
    pub total_data_edges: usize,
    pub total_def_use_chains: usize,
    pub variables_tracked: usize,
    pub parameters: usize,
    pub returns: usize,
}

impl fmt::Display for CpgStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "defs={} uses={} data-edges={} chains={} vars={} params={} returns={}",
            self.total_definitions,
            self.total_uses,
            self.total_data_edges,
            self.total_def_use_chains,
            self.variables_tracked,
            self.parameters,
            self.returns,
        )
    }
}

impl CodePropertyGraph {
    /// Get all definitions of a variable
    pub fn defs_of(&self, var_name: &str) -> Vec<&VarDef> {
        self.definitions
            .iter()
            .filter(|d| d.var_name == var_name)
            .collect()
    }

    /// Get all uses of a variable
    pub fn uses_of(&self, var_name: &str) -> Vec<&VarUse> {
        self.uses.iter().filter(|u| u.var_name == var_name).collect()
    }

    /// Get the def-use chain for a specific definition
    pub fn chain_for_def(&self, def_id: NodeId) -> Option<&DefUseChain> {
        self.def_use_chains.iter().find(|c| c.def_id == def_id)
    }

    /// Get reaching definitions at a use point
    pub fn reaching_at(&self, use_id: NodeId) -> Option<&ReachingDef> {
        self.reaching_defs.iter().find(|r| r.use_id == use_id)
    }

    /// Get all unique variable names in the function
    pub fn all_variables(&self) -> Vec<String> {
        let mut vars: Vec<String> = self
            .definitions
            .iter()
            .map(|d| d.var_name.clone())
            .chain(self.uses.iter().map(|u| u.var_name.clone()))
            .collect();
        vars.sort();
        vars.dedup();
        vars
    }

    /// Backward slice: find all definitions that influence a given use
    pub fn backward_slice(&self, use_id: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut worklist = vec![use_id];
        let mut visited = std::collections::HashSet::new();

        while let Some(current) = worklist.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            // Find reaching defs for this use
            if let Some(rd) = self.reaching_at(current) {
                for def_id in &rd.def_ids {
                    result.push(*def_id);

                    // Follow data dependencies in the RHS of this def
                    if let Some(def) = self.definitions.iter().find(|d| d.id == *def_id) {
                        for rhs_var in &def.rhs_uses {
                            // Find uses of rhs_var at this def line
                            for u in &self.uses {
                                if u.var_name == *rhs_var && u.line == def.line {
                                    worklist.push(u.id);
                                }
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Forward slice: find all uses influenced by a given definition
    pub fn forward_slice(&self, def_id: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut worklist = vec![def_id];
        let mut visited = std::collections::HashSet::new();

        while let Some(current) = worklist.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            // Find uses of this definition
            if let Some(chain) = self.chain_for_def(current) {
                for use_id in &chain.use_ids {
                    result.push(*use_id);

                    // If a use is part of a definition's RHS, follow that def too
                    let use_node = self.uses.iter().find(|u| u.id == *use_id);
                    if let Some(u) = use_node {
                        for def in &self.definitions {
                            if def.line == u.line && def.rhs_uses.contains(&u.var_name) {
                                worklist.push(def.id);
                            }
                        }
                    }
                }
            }
        }

        result
    }
}
