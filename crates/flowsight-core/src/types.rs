//! Core type definitions
//!
//! 执行流数据结构 - FlowSight 的核心数据模型
//!
//! ## 核心概念
//!
//! - `ExecutionFlow`: 顶层执行流，包含完整的分析结果
//! - `FlowNode`: 执行流节点，表示一个函数调用
//! - `AsyncMechanism`: 异步机制类型（WorkQueue, Timer, IRQ 等）
//! - `ExecutionContext`: 执行上下文（进程/软中断/硬中断）

use crate::location::Location;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Confidence level for this call edge (None for direct calls)
    pub confidence: Option<CallConfidence>,
    /// ⭐ 执行上下文 (process/softirq/hardirq)
    pub execution_context: Option<ExecutionContext>,
    /// ⭐ 是否可以睡眠（基于执行上下文）
    pub can_sleep: Option<bool>,
    /// ⭐ 源文件路径（内核函数）
    pub source_file: Option<String>,
    /// ⭐ 是否是内核调用链的一部分（而非用户代码）
    pub is_kernel_internal: bool,
}

/// Call confidence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallConfidence {
    /// Confidence level
    pub level: ConfidenceLevel,
    /// Reason for this confidence level
    pub reason: String,
}

/// Confidence level enum for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// 100% certain - direct call or known binding
    Certain,
    /// Likely correct - type-matched or single candidate
    Possible,
    /// Uncertain - multiple candidates or unresolved
    Unknown,
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
    /// Separator (time passes, context switch, etc.)
    Separator { text: String },
    /// Branch node
    Branch { condition: String, branch_type: BranchType },
}

/// Branch type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchType {
    /// If condition
    If,
    /// Else branch
    Else,
    /// Switch case
    Case { value: String },
    /// Default case
    Default,
    /// Error handling path
    ErrorPath,
}

// ============================================================================
// ExecutionFlow - 顶层执行流结构
// ============================================================================

/// 执行流 - 完整的函数执行过程分析结果
///
/// 这是 FlowSight 的核心输出，包含：
/// - 入口函数信息
/// - 完整的调用树（包括同步和异步调用）
/// - 分析元数据
///
/// # Example
///
/// ```text
/// ExecutionFlow {
///     entry_function: "my_probe",
///     root: FlowNode { ... },
///     async_boundaries: [...],
///     analysis_info: AnalysisInfo { ... },
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFlow {
    /// 入口函数名
    pub entry_function: String,
    
    /// 入口函数位置
    pub entry_location: Option<Location>,
    
    /// 执行流根节点
    pub root: FlowNode,
    
    /// 异步边界列表（schedule_work, add_timer 等调用点）
    pub async_boundaries: Vec<AsyncBoundary>,
    
    /// 分析信息
    pub analysis_info: AnalysisInfo,
}

/// 异步边界 - 同步代码和异步回调之间的分界点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncBoundary {
    /// 边界 ID
    pub id: String,
    
    /// 异步机制类型
    pub mechanism: AsyncMechanism,
    
    /// 触发调用（如 schedule_work）
    pub trigger_call: String,
    
    /// 触发位置
    pub trigger_location: Option<Location>,
    
    /// 处理函数名
    pub handler_function: String,
    
    /// 处理函数在执行流中的节点 ID
    pub handler_node_id: Option<String>,
    
    /// 执行上下文说明
    pub context_description: String,
}

/// 分析信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisInfo {
    /// 分析时间
    pub analyzed_at: DateTime<Utc>,
    
    /// 分析的文件路径
    pub source_file: Option<String>,
    
    /// 使用的知识库版本
    pub knowledge_version: Option<String>,
    
    /// 总节点数
    pub total_nodes: usize,
    
    /// 直接调用数（100% 确定）
    pub direct_calls: usize,
    
    /// 间接调用数（函数指针）
    pub indirect_calls: usize,
    
    /// 异步调用数
    pub async_calls: usize,
    
    /// 分析警告
    pub warnings: Vec<AnalysisWarning>,
}

/// 分析警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisWarning {
    /// 警告类型
    pub kind: WarningKind,
    
    /// 警告消息
    pub message: String,
    
    /// 相关位置
    pub location: Option<Location>,
}

/// 警告类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningKind {
    /// 未解析的函数指针
    UnresolvedFunctionPointer,
    
    /// 未知的异步模式
    UnknownAsyncPattern,
    
    /// 缺少知识库条目
    MissingKnowledge,
    
    /// 递归调用检测
    RecursionDetected,
    
    /// 分析深度限制
    DepthLimitReached,
}

impl ExecutionFlow {
    /// 创建新的执行流
    pub fn new(entry_function: String, root: FlowNode) -> Self {
        Self {
            entry_function,
            entry_location: None,
            root,
            async_boundaries: Vec::new(),
            analysis_info: AnalysisInfo::default(),
        }
    }
    
    /// 添加异步边界
    pub fn add_async_boundary(&mut self, boundary: AsyncBoundary) {
        self.async_boundaries.push(boundary);
    }
    
    /// 获取所有节点的迭代器（深度优先遍历）
    pub fn iter_nodes(&self) -> FlowNodeIterator<'_> {
        FlowNodeIterator::new(&self.root)
    }
    
    /// 统计节点数量
    pub fn count_nodes(&self) -> usize {
        self.iter_nodes().count()
    }
    
    /// 查找节点
    pub fn find_node(&self, id: &str) -> Option<&FlowNode> {
        self.iter_nodes().find(|n| n.id == id)
    }
}

impl Default for AnalysisInfo {
    fn default() -> Self {
        Self {
            analyzed_at: Utc::now(),
            source_file: None,
            knowledge_version: None,
            total_nodes: 0,
            direct_calls: 0,
            indirect_calls: 0,
            async_calls: 0,
            warnings: Vec::new(),
        }
    }
}

/// 执行流节点迭代器（深度优先）
pub struct FlowNodeIterator<'a> {
    stack: Vec<&'a FlowNode>,
}

impl<'a> FlowNodeIterator<'a> {
    fn new(root: &'a FlowNode) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a> Iterator for FlowNodeIterator<'a> {
    type Item = &'a FlowNode;
    
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        // 逆序压栈，保证先访问第一个子节点
        for child in node.children.iter().rev() {
            self.stack.push(child);
        }
        Some(node)
    }
}
