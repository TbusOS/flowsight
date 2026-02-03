/**
 * FlowSight 前端类型定义
 */

// 执行流节点类型
export type FlowNodeType =
  | 'Function'
  | 'EntryPoint'
  | 'KernelApi'
  | 'External'
  | { AsyncCallback: { mechanism: AsyncMechanism } }

// 异步机制类型
export type AsyncMechanism =
  | { WorkQueue: { delayed: boolean } }
  | { Timer: { high_resolution: boolean } }
  | { Interrupt: { threaded: boolean } }
  | 'Tasklet'
  | 'Softirq'
  | 'KThread'
  | 'RcuCallback'
  | 'Notifier'
  | { Custom: string }

// 执行流树节点
export interface FlowTreeNode {
  id: string
  name: string
  display_name: string
  location?: Location
  node_type: FlowNodeType
  children: FlowTreeNode[]
  description?: string
  confidence?: CallConfidence
}

// 源码位置
export interface Location {
  file: string
  line: number
  column: number
}

// 分析结果
export interface AnalysisResult {
  file: string
  functions_count: number
  structs_count: number
  async_handlers_count: number
  entry_points: string[]
  flow_trees: FlowTreeNode[]
}

// 函数信息
export interface FunctionInfo {
  name: string
  return_type: string
  line: number
  is_callback: boolean
  callback_context?: string
  calls: string[]
}

// 异步绑定信息
export interface AsyncBinding {
  mechanism: AsyncMechanism
  variable: string
  handler: string
  context: 'Process' | 'SoftIrq' | 'HardIrq' | 'Unknown'
}

// ============================================
// 知识库类型定义
// ============================================

// 调用链节点
export interface CallChainNode {
  function: string
  file?: string
  context: string
  description?: string
  is_user_entry: boolean
}

// 知识库信息
export interface KnowledgeInfo {
  name: string
  description?: string
  context?: 'process' | 'softirq' | 'hardirq' | 'user' | 'unknown' | 'any'
  can_sleep?: boolean
  trigger?: string
  signature?: string
  call_chain?: CallChainNode[]
  examples?: string[]
  framework?: string
  callback_type?: string
  notes?: string[]
}

// 异步模式信息
export interface AsyncPatternInfo {
  name: string
  description: string
  context: string
  can_sleep: boolean
  handler_signature?: string
  bind_patterns: string[]
  trigger_patterns: string[]
  handler_call_chain?: CallChainNode[]
}

// 框架摘要
export interface FrameworkSummary {
  name: string
  description: string
  header?: string
  callback_count: number
  callbacks: string[]
}

// 异步模式摘要
export interface AsyncPatternSummary {
  name: string
  description: string
  context: string
  can_sleep: boolean
}

// 执行上下文类型
export type ExecutionContext = 'process' | 'softirq' | 'hardirq' | 'user' | 'unknown'

// 置信度级别
export type ConfidenceLevel = 'Certain' | 'Possible' | 'Unknown'

// 调用置信度
export interface CallConfidence {
  level: ConfidenceLevel
  reason?: string
}

// ExecutionFlow 类型 (Phase 2)
export interface ExecutionFlow {
  entry_function: string
  root: FlowTreeNode
  async_boundaries: AsyncBoundary[]
  analysis_info: AnalysisInfo
}

// 异步边界
export interface AsyncBoundary {
  id: string
  mechanism: string
  handler_function: string
  context_description: string
}

// 分析信息
export interface AnalysisInfo {
  source_file?: string
  total_nodes: number
  direct_calls: number
  async_calls: number
  warnings: string[]
}
