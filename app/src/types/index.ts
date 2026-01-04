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

