//! Node Detail API Module
//!
//! Provides node detail query interfaces for the frontend NodeDetailPanel component.
//! Supports detailed information retrieval, LLVM IR data, and caller/callee relationships.
//! Includes pagination and streaming support for large IR data.
//!
//! ## API Endpoints
//!
//! | Endpoint | Description |
//! |----------|-------------|
//! | `query_node_detail()` | Get detailed info for a specific node |
//! | `query_llvm_ir()` | Get structured LLVM IR for a function |
//! | `paginate_llvm_ir()` | Paginate through large IR data |
//! | `get_function_summary()` | Get quick summary of a function |

use flowsight_core::{
    AsyncMechanism, CallEdge, ConfidenceLevel, ExecutionContext, FlowNode, FlowNodeType,
    FunctionDef,
};
use flowsight_llvm::{
    LlvmFunction, LlvmIrParseResult, LlvmInstruction, LlvmParameter, LlvmBasicBlock,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node detail request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDetailRequest {
    /// Node ID to query
    pub node_id: String,
    /// Include LLVM IR data
    #[serde(default)]
    pub include_llvm_ir: bool,
    /// Include callers information
    #[serde(default)]
    pub include_callers: bool,
    /// Include callees (children) information
    #[serde(default)]
    pub include_callees: bool,
}

/// Simplified request for quick lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleNodeRequest {
    /// Node ID or function name
    pub node_id: String,
}

/// Node detail response - matches frontend NodeDetailData
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDetailResponse {
    /// Node name
    pub name: String,
    /// Node type
    pub node_type: FlowNodeTypeDto,
    /// Location information
    #[serde(default)]
    pub location: Option<LocationDto>,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Confidence information
    #[serde(default)]
    pub confidence: Option<ConfidenceDto>,
    /// Children nodes (callees)
    #[serde(default)]
    pub children: Vec<NodeSummaryDto>,
    /// LLVM IR data (structured format for LlvmIrPanel)
    #[serde(default)]
    pub llvm_ir: Vec<String>,
    /// Structured LLVM IR data (for advanced visualization)
    #[serde(default)]
    pub structured_llvm_ir: Option<LlvmIrData>,
    /// Function signature
    #[serde(default)]
    pub function_signature: Option<String>,
    /// Parameters
    #[serde(default)]
    pub params: Vec<ParamDto>,
    /// Return type
    #[serde(default)]
    pub return_type: Option<String>,
    /// Callers (functions that call this node)
    #[serde(default)]
    pub callers: Vec<String>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: NodeMetadataDto,
}

/// Structured LLVM IR data for advanced visualization
/// Matches LlvmIrPanel.LlvmIrParseResult structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlvmIrData {
    /// Module name
    #[serde(default)]
    pub module_name: String,
    /// Function data (only requested function)
    #[serde(default)]
    pub functions: HashMap<String, LlvmFunctionData>,
}

/// Function data for LLVM IR visualization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlvmFunctionData {
    /// Function name
    pub name: String,
    /// Return type
    #[serde(default)]
    pub return_type: String,
    /// Parameters
    #[serde(default)]
    pub parameters: Vec<LlvmParameter>,
    /// Basic blocks
    #[serde(default)]
    pub blocks: Vec<LlvmBasicBlock>,
    /// Is callback function
    #[serde(default)]
    pub is_callback: bool,
    /// Callback context
    #[serde(default)]
    pub callback_context: Option<String>,
}

/// Request for LLVM IR pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlvmIrPageRequest {
    /// Function name
    pub function_name: String,
    /// Page number (0-indexed)
    #[serde(default)]
    pub page: u32,
    /// Page size (instructions per page)
    #[serde(default)]
    pub page_size: u32,
    /// Filter by basic block name (optional)
    #[serde(default)]
    pub block_name: Option<String>,
}

/// Paginated LLVM IR response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlvmIrPageResponse {
    /// Function name
    pub function_name: String,
    /// Current page
    pub page: u32,
    /// Page size
    pub page_size: u32,
    /// Total instructions
    pub total_instructions: u32,
    /// Total pages
    pub total_pages: u32,
    /// Has next page
    pub has_next: bool,
    /// Has previous page
    pub has_previous: bool,
    /// Instructions in this page
    #[serde(default)]
    pub instructions: Vec<LlvmInstruction>,
    /// Block name (if paginating by block)
    #[serde(default)]
    pub block_name: Option<String>,
    /// Current block info
    #[serde(default)]
    pub current_block: Option<BasicBlockInfo>,
}

/// Basic block info for UI
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BasicBlockInfo {
    /// Block name
    pub name: String,
    /// Number of instructions
    pub instruction_count: u32,
    /// Predecessor blocks
    #[serde(default)]
    pub predecessors: Vec<String>,
    /// Successor blocks
    #[serde(default)]
    pub successors: Vec<String>,
}

/// Function summary for quick display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSummary {
    /// Function name
    pub name: String,
    /// Return type
    #[serde(default)]
    pub return_type: String,
    /// Parameter count
    pub param_count: usize,
    /// Instruction count
    pub instruction_count: u32,
    /// Basic block count
    pub block_count: usize,
    /// Is callback
    pub is_callback: bool,
    /// Callback context
    #[serde(default)]
    pub callback_context: Option<String>,
    /// Location
    #[serde(default)]
    pub location: Option<LocationDto>,
}

/// Location DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationDto {
    pub file: String,
    pub line: u32,
    #[serde(default)]
    pub column: Option<u32>,
}

/// Flow node type DTO (frontend-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FlowNodeTypeDto {
    /// Normal function
    #[serde(rename = "Function")]
    Function,
    /// Entry point
    #[serde(rename = "EntryPoint")]
    EntryPoint,
    /// Async callback
    #[serde(rename = "AsyncCallback")]
    AsyncCallback {
        #[serde(default)]
        mechanism: AsyncMechanismDto,
    },
    /// Kernel API
    #[serde(rename = "KernelApi")]
    KernelApi,
    /// External function
    #[serde(rename = "External")]
    External,
    /// Separator (async boundary, context switch, etc.)
    #[serde(rename = "Separator")]
    Separator { text: String },
    /// Branch node
    #[serde(rename = "Branch")]
    Branch { condition: String, branch_type: String },
}

/// Async mechanism DTO
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
pub enum AsyncMechanismDto {
    /// WorkQueue mechanism
    #[serde(rename = "WorkQueue")]
    WorkQueue {
        #[serde(default)]
        work_struct: Option<String>,
        #[serde(default)]
        queue: Option<String>,
    },
    /// Timer mechanism
    #[serde(rename = "Timer")]
    Timer {
        #[serde(default)]
        timer_name: Option<String>,
        #[serde(default)]
        timer_type: Option<String>,
    },
    /// Tasklet mechanism
    #[serde(rename = "Tasklet")]
    Tasklet {
        #[serde(default)]
        tasklet_name: Option<String>,
    },
    /// Softirq mechanism
    #[serde(rename = "Softirq")]
    Softirq {
        #[serde(default)]
        type_name: Option<String>,
    },
    /// Threaded IRQ
    #[serde(rename = "Threaded")]
    Threaded {
        #[serde(default)]
        thread_name: Option<String>,
    },
    /// Hard IRQ
    #[serde(rename = "Irq")]
    Irq {
        #[serde(default)]
        irq_name: Option<String>,
        #[serde(default)]
        flags: Option<String>,
    },
    /// Completion mechanism
    #[serde(rename = "Completion")]
    Completion {
        #[serde(default)]
        completion_name: Option<String>,
    },
    /// RCU callback
    #[serde(rename = "Rcu")]
    Rcu {
        #[serde(default)]
        rcu_type: Option<String>,
    },
    /// Kernel thread
    #[serde(rename = "Kthread")]
    Kthread {
        #[serde(default)]
        kthread_name: Option<String>,
    },
    /// Wait queue
    #[serde(rename = "WaitQueue")]
    WaitQueue {
        #[serde(default)]
        queue_name: Option<String>,
    },
    /// Custom mechanism
    #[serde(rename = "Custom")]
    Custom {
        name: String,
    },
    /// Unknown mechanism
    #[serde(rename = "Unknown")]
    #[default]
    Unknown,
}

/// Confidence DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceDto {
    pub level: ConfidenceLevelDto,
    pub reason: String,
}

/// Confidence level DTO (frontend-compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "lowercase")]
pub enum ConfidenceLevelDto {
    #[serde(rename = "Certain")]
    Certain,
    #[serde(rename = "Possible")]
    Possible,
    #[serde(rename = "Unknown")]
    Unknown,
}

/// Parameter DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDto {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
}

/// Node summary for children/callees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSummaryDto {
    pub id: String,
    pub name: String,
    pub node_type: FlowNodeTypeDto,
    #[serde(default)]
    pub location: Option<LocationDto>,
}

/// Additional node metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeMetadataDto {
    /// Execution context
    #[serde(default)]
    pub execution_context: Option<ExecutionContext>,
    /// Can the context sleep?
    #[serde(default)]
    pub can_sleep: Option<bool>,
    /// Is kernel internal function?
    #[serde(default)]
    pub is_kernel_internal: bool,
    /// Source file (for kernel functions)
    #[serde(default)]
    pub source_file: Option<String>,
}

/// Node detail query service
pub struct NodeDetailService {
    /// Cached function definitions
    function_defs: HashMap<String, FunctionDef>,
    /// Call graph (caller -> callees)
    call_graph: HashMap<String, Vec<String>>,
    /// Reverse call graph (callee -> callers)
    reverse_call_graph: HashMap<String, Vec<String>>,
    /// LLVM IR data
    llvm_data: Option<LlvmIrParseResult>,
}

impl NodeDetailService {
    /// Create a new service
    pub fn new() -> Self {
        Self {
            function_defs: HashMap::new(),
            call_graph: HashMap::new(),
            reverse_call_graph: HashMap::new(),
            llvm_data: None,
        }
    }

    /// Create with initial data
    pub fn with_data(
        function_defs: HashMap<String, FunctionDef>,
        call_edges: &[CallEdge],
        llvm_data: Option<LlvmIrParseResult>,
    ) -> Self {
        let mut call_graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut reverse_call_graph: HashMap<String, Vec<String>> = HashMap::new();

        for edge in call_edges {
            call_graph
                .entry(edge.caller.clone())
                .or_default()
                .push(edge.callee.clone());
            reverse_call_graph
                .entry(edge.callee.clone())
                .or_default()
                .push(edge.caller.clone());
        }

        Self {
            function_defs,
            call_graph,
            reverse_call_graph,
            llvm_data,
        }
    }

    /// Add function definition
    pub fn add_function(&mut self, func: FunctionDef) {
        self.function_defs.insert(func.name.clone(), func);
    }

    /// Add call edge
    pub fn add_call_edge(&mut self, edge: &CallEdge) {
        self.call_graph
            .entry(edge.caller.clone())
            .or_default()
            .push(edge.callee.clone());
        self.reverse_call_graph
            .entry(edge.callee.clone())
            .or_default()
            .push(edge.caller.clone());
    }

    /// Set LLVM IR data
    pub fn set_llvm_data(&mut self, data: LlvmIrParseResult) {
        self.llvm_data = Some(data);
    }

    /// Get LLVM IR data reference
    pub fn get_llvm_data(&self) -> Option<&LlvmIrParseResult> {
        self.llvm_data.as_ref()
    }

    /// Query node detail
    pub fn query(&self, request: &NodeDetailRequest) -> Option<NodeDetailResponse> {
        // First check if it's a function
        if let Some(func_def) = self.function_defs.get(&request.node_id) {
            return Some(self.build_function_detail(func_def, request));
        }

        // If not a function, it might be a flow node without function definition
        None
    }

    /// Query by function name
    pub fn query_by_name(&self, name: &str, include_llvm_ir: bool) -> Option<NodeDetailResponse> {
        let request = NodeDetailRequest {
            node_id: name.to_string(),
            include_llvm_ir,
            include_callers: true,
            include_callees: true,
        };
        self.query(&request)
    }

    /// Query structured LLVM IR data for a function (for LlvmIrPanel)
    /// Returns None if function not found or no LLVM IR data available
    pub fn query_llvm_ir(&self, func_name: &str) -> Option<LlvmIrData> {
        let llvm_data = self.llvm_data.as_ref()?;
        let llvm_func = llvm_data.functions.get(func_name)?;

        let mut functions = HashMap::new();
        functions.insert(
            func_name.to_string(),
            LlvmFunctionData {
                name: llvm_func.name.clone(),
                return_type: llvm_func.return_type.clone(),
                parameters: llvm_func.parameters.clone(),
                blocks: llvm_func.blocks.clone(),
                is_callback: llvm_func.is_callback,
                callback_context: llvm_func.callback_context.clone(),
            },
        );

        Some(LlvmIrData {
            module_name: llvm_data.module_name.clone(),
            functions,
        })
    }

    /// Get LLVM IR pagination for a function
    /// This is useful for large functions with many instructions
    pub fn paginate_llvm_ir(&self, request: &LlvmIrPageRequest) -> Option<LlvmIrPageResponse> {
        let llvm_data = self.llvm_data.as_ref()?;
        let func = llvm_data.functions.get(&request.function_name)?;

        // Collect all instructions across all blocks (or filter by block)
        let (all_instrs, total_instrs, block_info) = if let Some(block_name) = &request.block_name {
            let block = func.blocks.iter().find(|b| &b.name == block_name)?;
            let instrs: Vec<&LlvmInstruction> = block
                .instructions
                .iter()
                .chain(block.terminator.as_ref())
                .collect();
            let instr_count = instrs.len() as u32;
            (
                instrs,
                instr_count,
                Some(BasicBlockInfo {
                    name: block_name.clone(),
                    instruction_count: instr_count,
                    predecessors: block.predecessors.clone(),
                    successors: block.successors.clone(),
                }),
            )
        } else {
            let instrs: Vec<&LlvmInstruction> = func
                .blocks
                .iter()
                .flat_map(|block| {
                    block
                        .instructions
                        .iter()
                        .chain(block.terminator.as_ref())
                })
                .collect();
            let instr_count = instrs.len() as u32;
            (instrs, instr_count, None)
        };

        let page_size = if request.page_size == 0 { 50 } else { request.page_size };
        let total_pages = (total_instrs as f64 / page_size as f64).ceil() as u32;
        let page = if request.page >= total_pages { total_pages.saturating_sub(1) } else { request.page };

        let start = (page * page_size) as usize;
        let end = ((page + 1) * page_size) as usize;
        let page_instrs: Vec<LlvmInstruction> = all_instrs[start..end.min(all_instrs.len())]
            .iter()
            .map(|instr| (*instr).clone())
            .collect();

        Some(LlvmIrPageResponse {
            function_name: request.function_name.clone(),
            page,
            page_size,
            total_instructions: total_instrs,
            total_pages,
            has_next: page + 1 < total_pages,
            has_previous: page > 0,
            instructions: page_instrs,
            block_name: request.block_name.clone(),
            current_block: block_info,
        })
    }

    /// Get function summary for quick display
    pub fn get_function_summary(&self, func_name: &str) -> Option<FunctionSummary> {
        let func_def = self.function_defs.get(func_name)?;
        let llvm_data = self.llvm_data.as_ref();

        // Get LLVM IR info if available
        let (instruction_count, block_count, is_callback, callback_context) =
            if let Some(llvm_func) = llvm_data.and_then(|d| d.functions.get(func_name)) {
                let instr_count: u32 = llvm_func
                    .blocks
                    .iter()
                    .map(|b| b.instructions.len() as u32 + b.terminator.as_ref().map_or(0, |_| 1))
                    .sum();
                (
                    instr_count,
                    llvm_func.blocks.len(),
                    llvm_func.is_callback,
                    llvm_func.callback_context.clone(),
                )
            } else {
                (0, 0, func_def.is_callback, func_def.callback_context.clone())
            };

        Some(FunctionSummary {
            name: func_name.to_string(),
            return_type: func_def.return_type.clone(),
            param_count: func_def.params.len(),
            instruction_count,
            block_count,
            is_callback,
            callback_context,
            location: func_def.location.as_ref().map(|loc| LocationDto {
                file: loc.file.clone(),
                line: loc.line,
                column: Some(loc.column),
            }),
        })
    }

    /// Get all function summaries
    pub fn get_all_function_summaries(&self) -> Vec<FunctionSummary> {
        self.function_defs
            .keys()
            .filter_map(|name| self.get_function_summary(name))
            .collect()
    }

    /// Get list of all function names
    pub fn get_function_names(&self) -> Vec<String> {
        self.function_defs.keys().cloned().collect()
    }

    /// Check if function exists
    pub fn has_function(&self, name: &str) -> bool {
        self.function_defs.contains_key(name)
    }

    /// Get callers of a function
    pub fn get_callers(&self, func_name: &str) -> Vec<String> {
        self.reverse_call_graph
            .get(func_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Get callees of a function
    pub fn get_callees(&self, func_name: &str) -> Vec<String> {
        self.call_graph
            .get(func_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Build function detail response
    fn build_function_detail(
        &self,
        func: &FunctionDef,
        request: &NodeDetailRequest,
    ) -> NodeDetailResponse {
        let location = func.location.as_ref().map(|loc| LocationDto {
            file: loc.file.clone(),
            line: loc.line,
            column: Some(loc.column),
        });

        let params: Vec<ParamDto> = func
            .params
            .iter()
            .map(|p| ParamDto {
                name: p.name.clone(),
                type_name: p.type_name.clone(),
            })
            .collect();

        let children: Vec<NodeSummaryDto> = if request.include_callees {
            self.build_children_summary(&func.calls)
        } else {
            Vec::new()
        };

        let callers: Vec<String> = if request.include_callers {
            func.called_by.clone()
        } else {
            Vec::new()
        };

        let (llvm_ir, function_signature, structured_llvm_ir) =
            if request.include_llvm_ir {
                let (ir_lines, signature) = self.get_llvm_ir_data(func);
                let structured = self.query_llvm_ir(&func.name);
                (ir_lines, signature, structured)
            } else {
                (Vec::new(), None, None)
            };

        NodeDetailResponse {
            name: func.name.clone(),
            node_type: if func.is_callback {
                FlowNodeTypeDto::AsyncCallback {
                    mechanism: func
                        .callback_context
                        .as_ref()
                        .map(|ctx| AsyncMechanismDto::Custom {
                            name: ctx.clone(),
                        })
                        .unwrap_or(AsyncMechanismDto::Unknown),
                }
            } else {
                FlowNodeTypeDto::Function
            },
            location,
            description: None,
            confidence: None,
            children,
            llvm_ir,
            structured_llvm_ir,
            function_signature,
            params,
            return_type: Some(func.return_type.clone()),
            callers,
            metadata: NodeMetadataDto::default(),
        }
    }

    /// Build children summary
    fn build_children_summary(&self, children: &[String]) -> Vec<NodeSummaryDto> {
        children
            .iter()
            .filter_map(|name| {
                self.function_defs.get(name).map(|func| NodeSummaryDto {
                    id: func.name.clone(),
                    name: func.name.clone(),
                    node_type: if func.is_callback {
                        FlowNodeTypeDto::AsyncCallback {
                            mechanism: AsyncMechanismDto::Unknown,
                        }
                    } else {
                        FlowNodeTypeDto::Function
                    },
                    location: func.location.as_ref().map(|loc| LocationDto {
                        file: loc.file.clone(),
                        line: loc.line,
                        column: Some(loc.column),
                    }),
                })
            })
            .collect()
    }

    /// Get LLVM IR data for a function
    fn get_llvm_ir_data(&self, func: &FunctionDef) -> (Vec<String>, Option<String>) {
        if let Some(ref llvm_data) = self.llvm_data {
            if let Some(llvm_func) = llvm_data.functions.get(&func.name) {
                let ir_lines = Self::format_llvm_function(llvm_func);
                let signature = Self::format_function_signature(llvm_func);
                return (ir_lines, Some(signature));
            }
        }
        (Vec::new(), None)
    }

    /// Format LLVM function as text lines
    fn format_llvm_function(func: &LlvmFunction) -> Vec<String> {
        let mut lines = Vec::new();

        // Function signature
        let params: Vec<String> = func
            .parameters
            .iter()
            .map(|p| format!("{} %{}", p.type_str, p.name))
            .collect();

        let params_str = if params.is_empty() {
            "".to_string()
        } else {
            format!("({})", params.join(", "))
        };

        lines.push(format!(
            "define {} @{}{}",
            func.return_type, func.name, params_str
        ));

        // Basic blocks
        for block in &func.blocks {
            lines.push(format!("{}:", block.name));

            // Instructions
            for instr in &block.instructions {
                if let Some(dest) = &instr.dest {
                    lines.push(format!(
                        "  %{} = {} {} {}",
                        dest, instr.opcode, instr.type_str, instr.operands.join(", ")
                    ));
                } else {
                    lines.push(format!(
                        "  {} {}",
                        instr.opcode, instr.operands.join(", ")
                    ));
                }
            }

            // Terminator
            if let Some(term) = &block.terminator {
                if let Some(dest) = &term.dest {
                    lines.push(format!(
                        "  %{} = {} {} {}",
                        dest, term.opcode, term.type_str, term.operands.join(", ")
                    ));
                } else {
                    lines.push(format!("  {}", term.opcode));
                }
            }
        }

        lines.push("}".to_string());
        lines
    }

    /// Format function signature
    fn format_function_signature(func: &LlvmFunction) -> String {
        let params: Vec<String> = func
            .parameters
            .iter()
            .map(|p| format!("{} {}", p.type_str, p.name))
            .collect();

        format!(
            "{}({})",
            func.return_type,
            if params.is_empty() {
                "void".to_string()
            } else {
                params.join(", ")
            }
        )
    }
}

impl Default for NodeDetailService {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert internal FlowNode to NodeDetailResponse
impl From<&FlowNode> for NodeDetailResponse {
    fn from(node: &FlowNode) -> Self {
        let location = node.location.as_ref().map(|loc| LocationDto {
            file: loc.file.clone(),
            line: loc.line,
            column: Some(loc.column),
        });

        let children: Vec<NodeSummaryDto> = node
            .children
            .iter()
            .map(|child| NodeSummaryDto {
                id: child.id.clone(),
                name: child.name.clone(),
                node_type: convert_node_type(&child.node_type),
                location: child.location.as_ref().map(|loc| LocationDto {
                    file: loc.file.clone(),
                    line: loc.line,
                    column: Some(loc.column),
                }),
            })
            .collect();

        NodeDetailResponse {
            name: node.name.clone(),
            node_type: convert_node_type(&node.node_type),
            location,
            description: node.description.clone(),
            confidence: node.confidence.as_ref().map(|c| ConfidenceDto {
                level: convert_confidence_level(c.level),
                reason: c.reason.clone(),
            }),
            children,
            llvm_ir: Vec::new(), // Will be populated if LLVM IR is available
            structured_llvm_ir: None,
            function_signature: None,
            params: Vec::new(),
            return_type: None,
            callers: Vec::new(),
            metadata: NodeMetadataDto {
                execution_context: node.execution_context.clone(),
                can_sleep: node.can_sleep,
                is_kernel_internal: node.is_kernel_internal,
                source_file: node.source_file.clone(),
            },
        }
    }
}

/// Convert internal FlowNodeType to DTO
fn convert_node_type(node_type: &FlowNodeType) -> FlowNodeTypeDto {
    match node_type {
        FlowNodeType::Function => FlowNodeTypeDto::Function,
        FlowNodeType::EntryPoint => FlowNodeTypeDto::EntryPoint,
        FlowNodeType::AsyncCallback { mechanism } => FlowNodeTypeDto::AsyncCallback {
            mechanism: convert_async_mechanism(mechanism),
        },
        FlowNodeType::KernelApi => FlowNodeTypeDto::KernelApi,
        FlowNodeType::External => FlowNodeTypeDto::External,
        FlowNodeType::Separator { text } => FlowNodeTypeDto::Separator { text: text.clone() },
        FlowNodeType::Branch { condition, branch_type } => FlowNodeTypeDto::Branch {
            condition: condition.clone(),
            branch_type: format!("{:?}", branch_type),
        },
    }
}

/// Convert internal AsyncMechanism to DTO
fn convert_async_mechanism(mechanism: &AsyncMechanism) -> AsyncMechanismDto {
    match mechanism {
        AsyncMechanism::WorkQueue { delayed: _ } => AsyncMechanismDto::WorkQueue {
            work_struct: None,
            queue: None,
        },
        AsyncMechanism::Timer { high_resolution: _ } => AsyncMechanismDto::Timer {
            timer_name: None,
            timer_type: None,
        },
        AsyncMechanism::Interrupt { threaded: _ } => AsyncMechanismDto::Irq {
            irq_name: None,
            flags: None,
        },
        AsyncMechanism::Tasklet => AsyncMechanismDto::Tasklet {
            tasklet_name: None,
        },
        AsyncMechanism::Softirq => AsyncMechanismDto::Softirq {
            type_name: None,
        },
        AsyncMechanism::KThread => AsyncMechanismDto::Kthread {
            kthread_name: None,
        },
        AsyncMechanism::RcuCallback => AsyncMechanismDto::Rcu {
            rcu_type: None,
        },
        AsyncMechanism::Notifier => AsyncMechanismDto::Custom {
            name: "Notifier".to_string(),
        },
        AsyncMechanism::Custom(name) => AsyncMechanismDto::Custom {
            name: name.clone(),
        },
    }
}

/// Convert ConfidenceLevel to DTO
fn convert_confidence_level(level: ConfidenceLevel) -> ConfidenceLevelDto {
    match level {
        ConfidenceLevel::Certain => ConfidenceLevelDto::Certain,
        ConfidenceLevel::Possible => ConfidenceLevelDto::Possible,
        ConfidenceLevel::Unknown => ConfidenceLevelDto::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowsight_core::Location;
    use flowsight_llvm::{LlvmBasicBlock, LlvmFunction, LlvmInstruction, LlvmParameter};

    #[test]
    fn test_node_detail_request() {
        let request = NodeDetailRequest {
            node_id: "test_func".to_string(),
            include_llvm_ir: true,
            include_callers: true,
            include_callees: true,
        };

        assert_eq!(request.node_id, "test_func");
        assert!(request.include_llvm_ir);
    }

    #[test]
    fn test_flow_node_type_dto() {
        let async_type = FlowNodeTypeDto::AsyncCallback {
            mechanism: AsyncMechanismDto::WorkQueue {
                work_struct: Some("work".to_string()),
                queue: Some("wq".to_string()),
            },
        };

        let json = serde_json::to_string(&async_type).unwrap();
        assert!(json.contains("WorkQueue"));
    }

    #[test]
    fn test_location_dto() {
        let location = LocationDto {
            file: "test.c".to_string(),
            line: 42,
            column: Some(10),
        };

        let json = serde_json::to_string(&location).unwrap();
        assert!(json.contains("test.c"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_llvm_ir_page_request() {
        let request = LlvmIrPageRequest {
            function_name: "test_func".to_string(),
            page: 0,
            page_size: 50,
            block_name: Some("entry".to_string()),
        };

        assert_eq!(request.function_name, "test_func");
        assert_eq!(request.page, 0);
        assert_eq!(request.page_size, 50);
        assert_eq!(request.block_name, Some("entry".to_string()));
    }

    #[test]
    fn test_llvm_ir_page_response() {
        let response = LlvmIrPageResponse {
            function_name: "test_func".to_string(),
            page: 0,
            page_size: 50,
            total_instructions: 100,
            total_pages: 2,
            has_next: true,
            has_previous: false,
            instructions: vec![LlvmInstruction {
                opcode: "add".to_string(),
                dest: Some("1".to_string()),
                type_str: "i32".to_string(),
                operands: vec!["%a".to_string(), "%b".to_string()],
                location: None,
            }],
            block_name: Some("entry".to_string()),
            current_block: Some(BasicBlockInfo {
                name: "entry".to_string(),
                instruction_count: 10,
                predecessors: vec![],
                successors: vec!["exit".to_string()],
            }),
        };

        assert_eq!(response.function_name, "test_func");
        assert!(response.has_next);
        assert!(!response.has_previous);
        assert_eq!(response.instructions.len(), 1);
    }

    #[test]
    fn test_function_summary() {
        let summary = FunctionSummary {
            name: "test_func".to_string(),
            return_type: "i32".to_string(),
            param_count: 2,
            instruction_count: 50,
            block_count: 5,
            is_callback: true,
            callback_context: Some("usb_probe".to_string()),
            location: Some(LocationDto {
                file: "driver.c".to_string(),
                line: 100,
                column: Some(5),
            }),
        };

        assert_eq!(summary.name, "test_func");
        assert_eq!(summary.return_type, "i32");
        assert!(summary.is_callback);
        assert_eq!(summary.block_count, 5);
    }

    #[test]
    fn test_llvm_ir_data_structure() {
        let mut functions = HashMap::new();
        functions.insert(
            "test_func".to_string(),
            LlvmFunctionData {
                name: "test_func".to_string(),
                return_type: "i32".to_string(),
                parameters: vec![LlvmParameter {
                    name: "x".to_string(),
                    type_str: "i32".to_string(),
                }],
                blocks: vec![LlvmBasicBlock {
                    name: "entry".to_string(),
                    instructions: vec![LlvmInstruction {
                        opcode: "add".to_string(),
                        dest: Some("1".to_string()),
                        type_str: "i32".to_string(),
                        operands: vec!["%x".to_string(), "1".to_string()],
                        location: None,
                    }],
                    predecessors: vec![],
                    successors: vec!["exit".to_string()],
                    terminator: Some(LlvmInstruction {
                        opcode: "ret".to_string(),
                        dest: None,
                        type_str: "i32".to_string(),
                        operands: vec!["%1".to_string()],
                        location: None,
                    }),
                }],
                is_callback: false,
                callback_context: None,
            },
        );

        let ir_data = LlvmIrData {
            module_name: "test_module".to_string(),
            functions,
        };

        assert_eq!(ir_data.module_name, "test_module");
        assert!(ir_data.functions.contains_key("test_func"));
        assert_eq!(ir_data.functions["test_func"].blocks.len(), 1);
    }

    #[test]
    fn test_node_detail_service_basic() {
        let mut service = NodeDetailService::new();

        // Add a helper function first (it's called by my_function)
        let helper_func = FunctionDef {
            name: "helper".to_string(),
            return_type: "void".to_string(),
            params: vec![],
            location: Some(Location::new("test.c", 20, 1)),
            calls: vec![],
            called_by: vec!["my_function".to_string()],
            is_callback: false,
            callback_context: None,
            attributes: vec![],
        };
        service.add_function(helper_func);

        // Add the main function
        let func = FunctionDef {
            name: "my_function".to_string(),
            return_type: "void".to_string(),
            params: vec![],
            location: Some(Location::new("test.c", 10, 1)),
            calls: vec!["helper".to_string()],
            called_by: vec!["caller".to_string()],
            is_callback: false,
            callback_context: None,
            attributes: vec![],
        };

        service.add_function(func);

        // Query the function
        let request = NodeDetailRequest {
            node_id: "my_function".to_string(),
            include_llvm_ir: false,
            include_callers: true,
            include_callees: true,
        };

        let response = service.query(&request);
        assert!(response.is_some());

        let detail = response.unwrap();
        assert_eq!(detail.name, "my_function");
        assert_eq!(detail.children.len(), 1);
        assert_eq!(detail.callers.len(), 1);
    }

    #[test]
    fn test_paginate_llvm_ir() {
        let mut service = NodeDetailService::new();

        // Create mock LLVM IR data
        let mut functions = HashMap::new();
        let mut blocks = Vec::new();

        // Create a block with 10 instructions
        for i in 0..10 {
            blocks.push(LlvmBasicBlock {
                name: format!("block_{}", i),
                instructions: vec![LlvmInstruction {
                    opcode: "add".to_string(),
                    dest: Some(format!("{}", i)),
                    type_str: "i32".to_string(),
                    operands: vec![format!("%{}", i - 1), "1".to_string()],
                    location: None,
                }],
                predecessors: if i == 0 { vec![] } else { vec![format!("block_{}", i - 1)] },
                successors: if i < 9 { vec![format!("block_{}", i + 1)] } else { vec![] },
                terminator: Some(LlvmInstruction {
                    opcode: "br".to_string(),
                    dest: None,
                    type_str: "label".to_string(),
                    operands: if i < 9 { vec![format!("label %block_{}", i + 1)] } else { vec![] },
                    location: None,
                }),
            });
        }

        functions.insert(
            "paginated_func".to_string(),
            LlvmFunction {
                name: "paginated_func".to_string(),
                return_type: "void".to_string(),
                parameters: vec![],
                blocks,
                is_callback: false,
                callback_context: None,
            },
        );

        let llvm_data = LlvmIrParseResult {
            module_name: "test".to_string(),
            functions,
        };

        service.set_llvm_data(llvm_data);

        // Test pagination - page 0 with 3 instructions per page
        let page_request = LlvmIrPageRequest {
            function_name: "paginated_func".to_string(),
            page: 0,
            page_size: 3,
            block_name: None,
        };

        let page_response = service.paginate_llvm_ir(&page_request);
        assert!(page_response.is_some());

        let response = page_response.unwrap();
        assert_eq!(response.page, 0);
        assert_eq!(response.page_size, 3);
        assert_eq!(response.total_instructions, 20); // 10 blocks * 2 instructions each
        assert_eq!(response.instructions.len(), 3);
        assert!(response.has_next);
        assert!(!response.has_previous);

        // Test page 1
        let page_request = LlvmIrPageRequest {
            function_name: "paginated_func".to_string(),
            page: 1,
            page_size: 3,
            block_name: None,
        };

        let page_response = service.paginate_llvm_ir(&page_request).unwrap();
        assert_eq!(page_response.page, 1);
        assert!(page_response.has_next);
        assert!(page_response.has_previous);
    }

    #[test]
    fn test_get_function_summary() {
        let mut service = NodeDetailService::new();

        // Add a function definition
        let func = FunctionDef {
            name: "complex_func".to_string(),
            return_type: "int".to_string(),
            params: vec![
                flowsight_core::Parameter {
                    name: "a".to_string(),
                    type_name: "int".to_string(),
                },
                flowsight_core::Parameter {
                    name: "b".to_string(),
                    type_name: "int".to_string(),
                },
            ],
            location: Some(Location::new("math.c", 50, 1)),
            calls: vec![],
            called_by: vec![],
            is_callback: true,
            callback_context: Some("timer_callback".to_string()),
            attributes: vec!["static".to_string()],
        };

        service.add_function(func);

        let summary = service.get_function_summary("complex_func");
        assert!(summary.is_some());

        let s = summary.unwrap();
        assert_eq!(s.name, "complex_func");
        assert_eq!(s.return_type, "int");
        assert_eq!(s.param_count, 2);
        assert!(s.is_callback);
        assert_eq!(s.callback_context, Some("timer_callback".to_string()));
    }

    #[test]
    fn test_get_callers_and_callees() {
        let mut service = NodeDetailService::new();

        // Add call edges
        service.add_call_edge(&CallEdge {
            caller: "caller_func".to_string(),
            callee: "callee_func".to_string(),
            location: None,
            call_type: flowsight_core::CallType::Direct,
        });

        service.add_call_edge(&CallEdge {
            caller: "caller_func".to_string(),
            callee: "another_func".to_string(),
            location: None,
            call_type: flowsight_core::CallType::Direct,
        });

        // Get callees of caller_func
        let callees = service.get_callees("caller_func");
        assert_eq!(callees.len(), 2);
        assert!(callees.contains(&"callee_func".to_string()));
        assert!(callees.contains(&"another_func".to_string()));

        // Get callers of callee_func
        let callers = service.get_callers("callee_func");
        assert_eq!(callers.len(), 1);
        assert!(callers.contains(&"caller_func".to_string()));
    }

    #[test]
    fn test_query_llvm_ir() {
        let mut service = NodeDetailService::new();

        // Create LLVM IR data
        let mut functions = HashMap::new();
        functions.insert(
            "llvm_func".to_string(),
            LlvmFunction {
                name: "llvm_func".to_string(),
                return_type: "i32".to_string(),
                parameters: vec![LlvmParameter {
                    name: "x".to_string(),
                    type_str: "i32".to_string(),
                }],
                blocks: vec![],
                is_callback: true,
                callback_context: Some("interrupt".to_string()),
            },
        );

        let llvm_data = LlvmIrParseResult {
            module_name: "test_module".to_string(),
            functions,
        };

        service.set_llvm_data(llvm_data);

        // Query LLVM IR for the function
        let ir_data = service.query_llvm_ir("llvm_func");
        assert!(ir_data.is_some());

        let data = ir_data.unwrap();
        assert_eq!(data.module_name, "test_module");
        assert!(data.functions.contains_key("llvm_func"));

        let func_data = &data.functions["llvm_func"];
        assert_eq!(func_data.name, "llvm_func");
        assert!(func_data.is_callback);
        assert_eq!(func_data.callback_context, Some("interrupt".to_string()));
    }

    #[test]
    fn test_async_mechanism_dto_serialization() {
        // Test all async mechanism variants
        let mechanisms = vec![
            AsyncMechanismDto::WorkQueue {
                work_struct: Some("work".to_string()),
                queue: Some("system_wq".to_string()),
            },
            AsyncMechanismDto::Timer {
                timer_name: Some("timer1".to_string()),
                timer_type: Some("hrtimer".to_string()),
            },
            AsyncMechanismDto::Tasklet {
                tasklet_name: Some("my_tasklet".to_string()),
            },
            AsyncMechanismDto::Irq {
                irq_name: Some("irq_12".to_string()),
                flags: Some("IRQF_SHARED".to_string()),
            },
            AsyncMechanismDto::Kthread {
                kthread_name: Some("kworker".to_string()),
            },
        ];

        for mech in mechanisms {
            let json = serde_json::to_string(&mech).unwrap();
            let deserialized: AsyncMechanismDto = serde_json::from_str(&json).unwrap();
            assert_eq!(serde_json::to_string(&deserialized).unwrap(), json);
        }
    }
}
