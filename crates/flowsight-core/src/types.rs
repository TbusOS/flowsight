//! Core type definitions

use serde::{Deserialize, Serialize};
use crate::location::Location;

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    /// Function name
    pub name: String,
    /// Return type
    pub return_type: String,
    /// Parameters
    pub params: Vec<Parameter>,
    /// Location in source
    pub location: Option<Location>,
    /// Functions called by this function
    pub calls: Vec<String>,
    /// Functions that call this function
    pub called_by: Vec<String>,
    /// Whether this is a callback function
    pub is_callback: bool,
    /// Callback context (e.g., "usb_driver.probe")
    pub callback_context: Option<String>,
    /// Attributes (static, inline, __init, etc.)
    pub attributes: Vec<String>,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub type_name: String,
}

/// Struct definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDef {
    /// Struct name
    pub name: String,
    /// Fields
    pub fields: Vec<StructField>,
    /// Location in source
    pub location: Option<Location>,
    /// Referenced structs
    pub referenced_structs: Vec<String>,
}

/// Struct field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    /// Field name
    pub name: String,
    /// Field type
    pub type_name: String,
    /// Is pointer type
    pub is_pointer: bool,
    /// Is function pointer
    pub is_function_ptr: bool,
    /// Function pointer signature (if applicable)
    pub func_ptr_signature: Option<String>,
    /// Array size (if applicable)
    pub array_size: Option<String>,
}

/// Call edge in call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    /// Caller function
    pub caller: String,
    /// Callee function
    pub callee: String,
    /// Location of the call
    pub location: Option<Location>,
    /// Call type
    pub call_type: CallType,
}

/// Type of function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallType {
    /// Direct function call
    Direct,
    /// Indirect call through function pointer
    Indirect { confidence: Confidence },
    /// Async call (work queue, timer, etc.)
    Async { mechanism: AsyncMechanism },
}

/// Confidence level for indirect call resolution
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// Async mechanism type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AsyncMechanism {
    WorkQueue { delayed: bool },
    Timer { high_resolution: bool },
    Interrupt { threaded: bool },
    Tasklet,
    Softirq,
    KThread,
    RcuCallback,
    Notifier,
    Custom(String),
}

/// Execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionContext {
    /// Process context, can sleep
    Process,
    /// Soft IRQ context, cannot sleep
    SoftIrq,
    /// Hard IRQ context, cannot sleep
    HardIrq,
    /// Unknown context
    Unknown,
}

impl ExecutionContext {
    /// Whether this context can sleep
    pub fn can_sleep(&self) -> bool {
        matches!(self, ExecutionContext::Process)
    }
}

/// Async binding information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncBinding {
    /// Type of async mechanism
    pub mechanism: AsyncMechanism,
    /// Variable that holds the handler
    pub variable: String,
    /// Handler function name
    pub handler: String,
    /// Location of binding (e.g., INIT_WORK)
    pub bind_location: Option<Location>,
    /// Locations of triggers (e.g., schedule_work)
    pub trigger_locations: Vec<Location>,
    /// Execution context of the handler
    pub context: ExecutionContext,
}

/// Flow node for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    /// Unique ID
    pub id: String,
    /// Function name
    pub name: String,
    /// Display name (with decorations)
    pub display_name: String,
    /// Location
    pub location: Option<Location>,
    /// Node type
    pub node_type: FlowNodeType,
    /// Children nodes
    pub children: Vec<FlowNode>,
    /// Description
    pub description: Option<String>,
}

/// Type of flow node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowNodeType {
    /// Normal function call
    Function,
    /// Entry point (callback)
    EntryPoint,
    /// Async callback
    AsyncCallback { mechanism: AsyncMechanism },
    /// Kernel API
    KernelApi,
    /// External function
    External,
}

