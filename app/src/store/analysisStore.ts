/**
 * FlowSight Analysis Store - 分析结果状态管理
 */

import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import {
  AnalysisResult,
  FlowTreeNode,
  FunctionDetail,
  SearchResult,
} from '../types'
import { OutlineItem } from '../components/Outline/Outline'

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

  // Actions
  setResult: (result: AnalysisResult | null) => void
  setSelectedFunction: (func: string | null) => void
  setFunctionDetail: (detail: FunctionDetail | null) => void
  setOutlineItems: (items: OutlineItem[]) => void
  setSearchQuery: (query: string) => void
  setSearchResults: (results: SearchResult[]) => void
  setLoading: (loading: boolean) => void

  // 分析操作
  analyzeFile: (path: string) => Promise<void>
  getFunctions: (path: string) => Promise<OutlineItem[]>
  searchSymbols: (query: string) => Promise<SearchResult[]>
  getFunctionDetail: (funcName: string, path: string) => Promise<FunctionDetail | null>

  // 工具函数
  findFunctionInOutline: (funcName: string) => OutlineItem | undefined
  findNodeInFlowTree: (funcName: string) => FlowTreeNode | null
  getKnownFunctions: () => string[]
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

  // 基本 Actions
  setResult: (result) => set({ result }),
  setSelectedFunction: (func) => set({ selectedFunction: func }),
  setFunctionDetail: (detail) => set({ functionDetail: detail }),
  setOutlineItems: (items) => set({ outlineItems: items }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  setSearchResults: (results) => set({ searchResults: results }),
  setLoading: (loading) => set({ loading }),

  // 分析文件
  analyzeFile: async (path) => {
    set({ loading: true })
    try {
      const analysis = await invoke<AnalysisResult>('analyze_file', { path })
      set({ result: analysis, loading: false })

      // 获取函数列表
      const functions = await get().getFunctions(path)
      set({ outlineItems: functions })
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
}))
