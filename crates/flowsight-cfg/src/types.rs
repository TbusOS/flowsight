//! CFG core type definitions

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a basic block
pub type BlockId = usize;

/// Control Flow Graph for a single function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlFlowGraph {
    /// Function name
    pub function_name: String,
    /// Source file
    pub source_file: Option<String>,
    /// All basic blocks
    pub blocks: Vec<BasicBlock>,
    /// All CFG edges
    pub edges: Vec<CfgEdge>,
    /// Entry block ID
    pub entry: BlockId,
    /// Exit block IDs (normal return + error returns)
    pub exits: Vec<BlockId>,
    /// Detected error paths
    pub error_paths: Vec<ErrorPath>,
    /// Goto labels → block ID mapping
    pub labels: Vec<(String, BlockId)>,
}

/// Basic block — the smallest unit of the CFG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    /// Unique block ID
    pub id: BlockId,
    /// Statements in this block
    pub statements: Vec<Statement>,
    /// Function calls in this block
    pub calls: Vec<CallSite>,
    /// Source line range (start, end)
    pub line_range: (usize, usize),
    /// Block type
    pub block_type: BlockType,
    /// Optional label (for goto targets)
    pub label: Option<String>,
}

/// A statement within a basic block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    /// Source line number
    pub line: usize,
    /// Statement kind
    pub kind: StatementKind,
    /// Source text (truncated to 120 chars)
    pub text: String,
}

/// Statement classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatementKind {
    /// Variable declaration or assignment
    Assignment,
    /// Function call (also stored in CallSite)
    Call,
    /// Return statement
    Return,
    /// Goto jump
    Goto { label: String },
    /// Condition expression (in if/while/for)
    Condition,
    /// Other expression
    Expression,
    /// Macro invocation with known semantics
    MacroInvocation { semantics: String },
}

/// A function call site with reachability info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSite {
    /// Called function name
    pub callee: String,
    /// Source line number
    pub line: usize,
    /// How this call is classified
    pub call_kind: CallKind,
    /// Reachability from function entry
    pub reachability: Reachability,
    /// Block ID this call belongs to
    pub block_id: BlockId,
}

/// Classification of a call site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallKind {
    /// Normal function call
    Direct,
    /// Kernel API call (not defined in current file)
    KernelApi,
    /// Macro with function-call semantics
    MacroCall,
    /// Async handler registration (not a call — registers callback)
    AsyncRegistration {
        handler_name: Option<String>,
        mechanism: String,
    },
    /// Iterator macro (expands to loop)
    IteratorMacro,
    /// Declaration macro (no runtime effect)
    DeclarationMacro,
    /// Context change (spin_lock, rcu_read_lock, etc.)
    ContextChange { new_context: String },
}

/// Reachability classification for a call or block
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reachability {
    /// Always executed on normal path
    Always,
    /// Conditionally executed
    Conditional(String),
    /// Only executed on error paths
    ErrorPath,
    /// Inside conditional compilation (#ifdef)
    ConditionalCompilation,
}

impl fmt::Display for Reachability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reachability::Always => write!(f, "always"),
            Reachability::Conditional(cond) => write!(f, "if {}", cond),
            Reachability::ErrorPath => write!(f, "error-path"),
            Reachability::ConditionalCompilation => write!(f, "#ifdef"),
        }
    }
}

/// CFG edge connecting two blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfgEdge {
    /// Source block
    pub from: BlockId,
    /// Target block
    pub to: BlockId,
    /// Edge type
    pub edge_type: EdgeType,
}

/// Type of control flow edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    /// Sequential execution (fall-through)
    FallThrough,
    /// Condition is true
    BranchTrue(String),
    /// Condition is false
    BranchFalse(String),
    /// Goto jump
    Goto(String),
    /// Function return
    Return,
    /// Switch case
    SwitchCase(String),
    /// Switch default
    SwitchDefault,
    /// Loop back edge
    LoopBack,
    /// Loop exit
    LoopExit,
}

impl fmt::Display for EdgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdgeType::FallThrough => write!(f, "fall-through"),
            EdgeType::BranchTrue(c) => write!(f, "true: {}", c),
            EdgeType::BranchFalse(c) => write!(f, "false: {}", c),
            EdgeType::Goto(l) => write!(f, "goto {}", l),
            EdgeType::Return => write!(f, "return"),
            EdgeType::SwitchCase(v) => write!(f, "case {}", v),
            EdgeType::SwitchDefault => write!(f, "default"),
            EdgeType::LoopBack => write!(f, "loop-back"),
            EdgeType::LoopExit => write!(f, "loop-exit"),
        }
    }
}

/// Block type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    /// Entry block
    Entry,
    /// Exit block (normal return)
    Exit,
    /// Normal code block
    Normal,
    /// Error handler (goto target with cleanup)
    ErrorHandler,
    /// Loop header (condition check)
    LoopHeader,
    /// Loop body
    LoopBody,
    /// Switch dispatch
    SwitchDispatch,
    /// Conditional compilation block (#ifdef)
    ConditionalCompilation,
}

/// Detected error handling path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPath {
    /// Error check expression (e.g., "ret < 0", "!ptr", "IS_ERR(clk)")
    pub check_expression: String,
    /// Line of the error check
    pub check_line: usize,
    /// Block ID of the error check
    pub check_block: BlockId,
    /// Error handling strategy
    pub strategy: ErrorStrategy,
    /// Cleanup calls on this error path
    pub cleanup_calls: Vec<String>,
    /// Error label (if goto-based)
    pub label: Option<String>,
}

/// Error handling strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorStrategy {
    /// return -EXXX
    EarlyReturn { error_code: String },
    /// goto err_label
    GotoCleanup { label: String },
    /// Combination of cleanup + return
    CleanupAndReturn { label: String, error_code: String },
}

impl ControlFlowGraph {
    /// Create an empty CFG
    pub fn new(function_name: String) -> Self {
        let entry_block = BasicBlock {
            id: 0,
            statements: vec![],
            calls: vec![],
            line_range: (0, 0),
            block_type: BlockType::Entry,
            label: None,
        };

        Self {
            function_name,
            source_file: None,
            blocks: vec![entry_block],
            edges: vec![],
            entry: 0,
            exits: vec![],
            error_paths: vec![],
            labels: vec![],
        }
    }

    /// Add a new basic block, returns its ID
    pub fn add_block(&mut self, block_type: BlockType) -> BlockId {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            id,
            statements: vec![],
            calls: vec![],
            line_range: (0, 0),
            block_type,
            label: None,
        });
        id
    }

    /// Add an edge between two blocks
    pub fn add_edge(&mut self, from: BlockId, to: BlockId, edge_type: EdgeType) {
        self.edges.push(CfgEdge {
            from,
            to,
            edge_type,
        });
    }

    /// Get block by ID
    pub fn block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(id)
    }

    /// Get mutable block by ID
    pub fn block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(id)
    }

    /// Get all successor blocks of a given block
    pub fn successors(&self, block_id: BlockId) -> Vec<BlockId> {
        self.edges
            .iter()
            .filter(|e| e.from == block_id)
            .map(|e| e.to)
            .collect()
    }

    /// Get all predecessor blocks of a given block
    pub fn predecessors(&self, block_id: BlockId) -> Vec<BlockId> {
        self.edges
            .iter()
            .filter(|e| e.to == block_id)
            .map(|e| e.from)
            .collect()
    }

    /// Get all calls in the CFG with their reachability
    pub fn all_calls(&self) -> Vec<&CallSite> {
        self.blocks.iter().flat_map(|b| b.calls.iter()).collect()
    }

    /// Get calls that are always executed (normal path)
    pub fn always_calls(&self) -> Vec<&CallSite> {
        self.all_calls()
            .into_iter()
            .filter(|c| c.reachability == Reachability::Always)
            .collect()
    }

    /// Get calls only on error paths
    pub fn error_calls(&self) -> Vec<&CallSite> {
        self.all_calls()
            .into_iter()
            .filter(|c| c.reachability == Reachability::ErrorPath)
            .collect()
    }

    /// Count statistics
    pub fn stats(&self) -> CfgStats {
        let total_calls = self.all_calls().len();
        let always_calls = self.always_calls().len();
        let error_calls = self.error_calls().len();
        let conditional_calls = total_calls - always_calls - error_calls;

        CfgStats {
            block_count: self.blocks.len(),
            edge_count: self.edges.len(),
            total_calls,
            always_calls,
            conditional_calls,
            error_calls,
            error_path_count: self.error_paths.len(),
            exit_count: self.exits.len(),
        }
    }

    /// Generate DOT graph representation
    pub fn to_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str(&format!(
            "digraph \"{}\" {{\n",
            self.function_name
        ));
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box, style=filled, fontname=\"monospace\", fontsize=10];\n");
        dot.push_str("  edge [fontname=\"monospace\", fontsize=9];\n\n");

        for block in &self.blocks {
            let color = match block.block_type {
                BlockType::Entry => "#b8d4b8",
                BlockType::Exit => "#d4b8b8",
                BlockType::ErrorHandler => "#e8c8c8",
                BlockType::LoopHeader => "#c8d4e8",
                BlockType::Normal => "#e8e8e0",
                _ => "#e0e0e0",
            };

            let label_prefix = block
                .label
                .as_ref()
                .map(|l| format!("{}:\\n", l))
                .unwrap_or_default();

            let calls_text: Vec<String> = block
                .calls
                .iter()
                .map(|c| format!("{}() [{}]", c.callee, c.reachability))
                .collect();

            let block_label = if calls_text.is_empty() {
                format!(
                    "{}B{} ({:?})\\nL{}-{}",
                    label_prefix, block.id, block.block_type, block.line_range.0, block.line_range.1
                )
            } else {
                format!(
                    "{}B{} L{}-{}\\n{}",
                    label_prefix,
                    block.id,
                    block.line_range.0,
                    block.line_range.1,
                    calls_text.join("\\n")
                )
            };

            dot.push_str(&format!(
                "  B{} [label=\"{}\", fillcolor=\"{}\"];\n",
                block.id, block_label, color
            ));
        }

        dot.push_str("\n");

        for edge in &self.edges {
            let style = match &edge.edge_type {
                EdgeType::BranchTrue(_) => "color=\"#4a7c4a\"",
                EdgeType::BranchFalse(_) => "color=\"#7c4a4a\"",
                EdgeType::Goto(_) => "style=dashed, color=\"#7c6a4a\"",
                EdgeType::Return => "color=\"#4a4a7c\"",
                EdgeType::LoopBack => "style=bold, color=\"#6a4a7c\"",
                _ => "",
            };

            dot.push_str(&format!(
                "  B{} -> B{} [label=\"{}\", {}];\n",
                edge.from, edge.to, edge.edge_type, style
            ));
        }

        dot.push_str("}\n");
        dot
    }
}

/// CFG statistics
#[derive(Debug, Clone)]
pub struct CfgStats {
    pub block_count: usize,
    pub edge_count: usize,
    pub total_calls: usize,
    pub always_calls: usize,
    pub conditional_calls: usize,
    pub error_calls: usize,
    pub error_path_count: usize,
    pub exit_count: usize,
}

impl fmt::Display for CfgStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "blocks={} edges={} calls={} (always={} cond={} error={}) error_paths={} exits={}",
            self.block_count,
            self.edge_count,
            self.total_calls,
            self.always_calls,
            self.conditional_calls,
            self.error_calls,
            self.error_path_count,
            self.exit_count
        )
    }
}
