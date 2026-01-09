/**
 * FlowSight 类型定义
 */

// 位置信息
export interface Location {
  file: string
  line: number
  column: number
}

// 流程节点类型
export type FlowNodeType =
  | 'Function'
  | 'EntryPoint'
  | { AsyncCallback: { mechanism: AsyncMechanism } }
  | 'KernelApi'
  | 'External'

// 异步机制类型
export type AsyncMechanism =
  | { WorkQueue: { work_struct: string; queue: string | null } }
  | { Timer: { timer_name: string; timer_type: string } }
  | { Tasklet: { tasklet_name: string } }
  | { Softirq: { type_name: string } }
  | { Threaded: { thread_name: string } }
  | { Irq: { irq_name: string; flags: string | null } }
  | { Completion: { completion_name: string } }
  | { Rcu: { rcu_type: string } }
  | { Kthread: { kthread_name: string } }
  | { WaitQueue: { queue_name: string } }
  | { Custom: string }

// 置信度等级
export type ConfidenceLevel = 'Certain' | 'Possible' | 'Unknown'

// 调用置信度信息
export interface CallConfidence {
  level: ConfidenceLevel
  reason: string
}

// 流程节点
export interface FlowTreeNode {
  id: string
  name: string
  display_name: string
  node_type: FlowNodeType
  location?: Location
  children?: FlowTreeNode[]
  description?: string
  confidence?: CallConfidence
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

// 项目信息
export interface ProjectInfo {
  path: string
  files_count: number
  functions_count: number
  structs_count: number
  indexed: boolean
}

// 搜索结果
export interface SearchResult {
  name: string
  kind: string
  file: string | null
  line: number | null
  is_callback: boolean
}

// 索引统计
export interface IndexStats {
  functions: number
  structs: number
  files: number
}

// 函数详情
export interface FunctionDetail {
  name: string
  return_type: string
  file: string | null
  line: number
  end_line: number
  is_callback: boolean
  callback_context: string | null
  calls: string[]
  called_by: string[]
  params: { name: string; type_name: string }[]
}

// 文件节点
export interface FileNode {
  name: string
  path: string
  is_dir: boolean
  children?: FileNode[]
  extension?: string
}

