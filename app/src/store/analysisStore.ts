/**
 * FlowSight Analysis Store - 分析结果状态管理
 * 
 * Phase 2 增强:
 * - ExecutionFlow 完整执行流支持
 * - 入口点管理
 * - 异步绑定信息
 */

import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import {
  AnalysisResult,
  FlowTreeNode,
  FunctionDetail,
  SearchResult,
  ExecutionFlow,
  AsyncBoundary,
} from '../types'
import { OutlineItem } from '../components/panels/outline-panel'

// 入口点信息
interface EntryPointInfo {
  name: string
  kind: string
  line: number
}

// 异步绑定信息
interface AsyncBindingInfo {
  variable: string
  handler: string
  mechanism: string
  context: string
  bind_line: number | null
  trigger_lines: number[]
}

// ExecutionFlow 构建选项
interface ExecutionFlowOptions {
  max_depth?: number
  include_kernel_chains?: boolean
  expand_async?: boolean
}

// 分析状态
interface AnalysisState {
  // 分析结果
  result: AnalysisResult | null
  selectedFunction: string | null
  functionDetail: FunctionDetail | null
  outlineItems: OutlineItem[]
  searchQuery: string
  searchResults: SearchResult[]
  loading: boolean
  
  // Phase 2: ExecutionFlow 状态
  executionFlow: ExecutionFlow | null
  entryPoints: EntryPointInfo[]
  asyncBindings: AsyncBindingInfo[]
  selectedEntryPoint: string | null
  executionFlowLoading: boolean

  // Actions
  setResult: (result: AnalysisResult | null) => void
  setSelectedFunction: (func: string | null) => void
  setFunctionDetail: (detail: FunctionDetail | null) => void
  setOutlineItems: (items: OutlineItem[]) => void
  setSearchQuery: (query: string) => void
  setSearchResults: (results: SearchResult[]) => void
  setLoading: (loading: boolean) => void
  
  // Phase 2 Actions
  setExecutionFlow: (flow: ExecutionFlow | null) => void
  setEntryPoints: (points: EntryPointInfo[]) => void
  setAsyncBindings: (bindings: AsyncBindingInfo[]) => void
  setSelectedEntryPoint: (name: string | null) => void
  setExecutionFlowLoading: (loading: boolean) => void

  // 分析操作
  analyzeFile: (path: string) => Promise<void>
  getFunctions: (path: string) => Promise<OutlineItem[]>
  searchSymbols: (query: string) => Promise<SearchResult[]>
  getFunctionDetail: (funcName: string, path: string) => Promise<FunctionDetail | null>
  
  // Phase 2 操作
  buildExecutionFlow: (path: string, entryFunction: string, options?: ExecutionFlowOptions) => Promise<ExecutionFlow | null>
  loadEntryPoints: (path: string) => Promise<EntryPointInfo[]>
  loadAsyncBindings: (path: string) => Promise<AsyncBindingInfo[]>

  // 工具函数
  findFunctionInOutline: (funcName: string) => OutlineItem | undefined
  findNodeInFlowTree: (funcName: string) => FlowTreeNode | null
  getKnownFunctions: () => string[]
  getAsyncBoundaries: () => AsyncBoundary[]
}

export const useAnalysisStore = create<AnalysisState>((set, get) => ({
  // 初始状态
  result: null,
  selectedFunction: null,
  functionDetail: null,
  outlineItems: [],
  searchQuery: '',
  searchResults: [],
  loading: false,
  
  // Phase 2 初始状态
  executionFlow: null,
  entryPoints: [],
  asyncBindings: [],
  selectedEntryPoint: null,
  executionFlowLoading: false,

  // 基本 Actions
  setResult: (result) => set({ result }),
  setSelectedFunction: (func) => set({ selectedFunction: func }),
  setFunctionDetail: (detail) => set({ functionDetail: detail }),
  setOutlineItems: (items) => set({ outlineItems: items }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  setSearchResults: (results) => set({ searchResults: results }),
  setLoading: (loading) => set({ loading }),
  
  // Phase 2 Actions
  setExecutionFlow: (flow) => set({ executionFlow: flow }),
  setEntryPoints: (points) => set({ entryPoints: points }),
  setAsyncBindings: (bindings) => set({ asyncBindings: bindings }),
  setSelectedEntryPoint: (name) => set({ selectedEntryPoint: name }),
  setExecutionFlowLoading: (loading) => set({ executionFlowLoading: loading }),

  // 分析文件
  analyzeFile: async (path) => {
    set({ loading: true })
    try {
      const analysis = await invoke<AnalysisResult>('analyze_file', { path })
      set({ result: analysis, loading: false })

      // 获取函数列表
      const functions = await get().getFunctions(path)
      set({ outlineItems: functions })
      
      // Phase 2: 同时加载入口点和异步绑定
      await Promise.all([
        get().loadEntryPoints(path),
        get().loadAsyncBindings(path),
      ])
    } catch (e) {
      console.error('分析失败:', e)
      set({ loading: false })
    }
  },

  // 获取函数列表
  getFunctions: async (path) => {
    try {
      const funcs = await invoke<Array<{
        name: string
        return_type: string
        line: number
        is_callback: boolean
      }>>('get_functions', { path })

      const items: OutlineItem[] = funcs.map(f => ({
        name: f.name,
        kind: 'function' as const,
        line: f.line,
        isCallback: f.is_callback,
        returnType: f.return_type,
      }))

      return items
    } catch (e) {
      console.error('获取函数列表失败:', e)
      return []
    }
  },

  // 搜索符号
  searchSymbols: async (query) => {
    try {
      const results = await invoke<SearchResult[]>('search_symbols', { query })
      return results
    } catch (e) {
      console.error('搜索失败:', e)
      return []
    }
  },

  // 获取函数详情
  getFunctionDetail: async (funcName, path) => {
    const { outlineItems, result } = get()

    // 首先从大纲查找
    const funcFromOutline = outlineItems.find(item => item.name === funcName)
    if (funcFromOutline) {
      return {
        name: funcFromOutline.name,
        return_type: funcFromOutline.returnType || 'void',
        file: path || null,
        line: funcFromOutline.line,
        end_line: funcFromOutline.line + 10,
        is_callback: funcFromOutline.isCallback || false,
        callback_context: null,
        calls: [],
        called_by: [],
        params: [],
      }
    }

    // 从流程树查找
    if (result) {
      const findInTree = (nodes: FlowTreeNode[]): FlowTreeNode | null => {
        for (const node of nodes) {
          if (node.name === funcName) return node
          if (node.children) {
            const found = findInTree(node.children)
            if (found) return found
          }
        }
        return null
      }

      const node = findInTree(result.flow_trees)
      if (node) {
        return {
          name: node.name,
          return_type: 'unknown',
          file: node.location?.file || null,
          line: node.location?.line || 0,
          end_line: (node.location?.line || 0) + 10,
          is_callback: typeof node.node_type === 'object' && 'AsyncCallback' in node.node_type,
          callback_context: node.description || null,
          calls: node.children?.map(c => c.name) || [],
          called_by: [],
          params: [],
        }
      }
    }

    // 外部函数
    return {
      name: funcName,
      return_type: 'unknown',
      file: null,
      line: 0,
      end_line: 0,
      is_callback: false,
      callback_context: null,
      calls: [],
      called_by: [],
      params: [],
    }
  },

  // 在大纲中查找函数
  findFunctionInOutline: (funcName) => {
    return get().outlineItems.find(item => item.name === funcName)
  },

  // 在流程树中查找节点
  findNodeInFlowTree: (funcName) => {
    const { result } = get()
    if (!result) return null

    const findInTree = (nodes: FlowTreeNode[]): FlowTreeNode | null => {
      for (const node of nodes) {
        if (node.name === funcName) return node
        if (node.children) {
          const found = findInTree(node.children)
          if (found) return found
        }
      }
      return null
    }

    return findInTree(result.flow_trees)
  },

  // 获取已知函数列表
  getKnownFunctions: () => {
    const { outlineItems, result } = get()
    const names = new Set<string>()

    outlineItems.forEach(item => names.add(item.name))

    if (result) {
      const addFromTree = (nodes: FlowTreeNode[]) => {
        nodes.forEach(node => {
          names.add(node.name)
          if (node.children) {
            addFromTree(node.children)
          }
        })
      }
      addFromTree(result.flow_trees)
    }

    return Array.from(names)
  },
  
  // ========================================================================
  // Phase 2: ExecutionFlow API
  // ========================================================================
  
  // 构建 ExecutionFlow
  buildExecutionFlow: async (path, entryFunction, options) => {
    set({ executionFlowLoading: true, selectedEntryPoint: entryFunction })
    try {
      const flow = await invoke<ExecutionFlow>('build_execution_flow', {
        filePath: path,
        entryFunction,
        options: options || {
          max_depth: 50,
          include_kernel_chains: true,
          expand_async: true,
        },
      })
      set({ executionFlow: flow, executionFlowLoading: false })
      return flow
    } catch (e) {
      console.error('构建执行流失败:', e)
      set({ executionFlowLoading: false })
      return null
    }
  },
  
  // 加载入口点列表
  loadEntryPoints: async (path) => {
    try {
      const points = await invoke<EntryPointInfo[]>('get_entry_points', { filePath: path })
      set({ entryPoints: points })
      
      // 如果有入口点且未选择，自动选择第一个
      const { selectedEntryPoint } = get()
      if (points.length > 0 && !selectedEntryPoint) {
        set({ selectedEntryPoint: points[0].name })
      }
      
      return points
    } catch (e) {
      console.error('加载入口点失败:', e)
      return []
    }
  },
  
  // 加载异步绑定
  loadAsyncBindings: async (path) => {
    try {
      const bindings = await invoke<AsyncBindingInfo[]>('get_async_bindings', { filePath: path })
      set({ asyncBindings: bindings })
      return bindings
    } catch (e) {
      console.error('加载异步绑定失败:', e)
      return []
    }
  },
  
  // 获取异步边界（从 ExecutionFlow）
  getAsyncBoundaries: () => {
    const { executionFlow } = get()
    return executionFlow?.async_boundaries || []
  },
}))
