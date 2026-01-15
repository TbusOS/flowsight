/**
 * FlowSight Editor Store - 编辑器状态管理
 */

import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { FileNode, ProjectInfo, IndexStats } from '../types'

// 标签页数据类型
export interface TabData {
  id: string
  filePath: string
  fileName: string
  content: string
  isDirty: boolean
  analysisResult: any | null
}

// 编辑器状态
interface EditorState {
  // 文件内容
  filePath: string
  fileContent: string

  // 标签页
  tabs: TabData[]
  activeTabId: string | null

  // 项目状态
  project: ProjectInfo | null
  fileTree: FileNode[]
  indexStats: IndexStats | null

  // 加载状态
  loading: boolean
  error: string | null

  // Actions
  setFilePath: (path: string) => void
  setFileContent: (content: string) => void
  setLoading: (loading: boolean) => void
  setError: (error: string | null) => void

  // 项目操作
  setProject: (project: ProjectInfo | null) => void
  setFileTree: (tree: FileNode[]) => void
  setIndexStats: (stats: IndexStats | null) => void
  openProject: (path: string) => Promise<void>
  loadFileTree: (path: string) => Promise<void>

  // 标签页操作
  generateTabId: () => string
  openFileInTab: (path: string) => Promise<string | null>
  closeTab: (tabId: string, force?: boolean) => void
  switchTab: (tabId: string) => void
  reorderTabs: (fromIndex: number, toIndex: number) => void
  updateTabContent: (tabId: string, content: string) => void
  updateTabAnalysis: (tabId: string, analysis: any) => void
  markTabDirty: (tabId: string, isDirty: boolean) => void

  // 文件操作
  readFile: (path: string) => Promise<string>
  writeFile: (path: string, content: string) => Promise<void>
}

export const useEditorStore = create<EditorState>((set, get) => ({
  // 初始状态
  filePath: '',
  fileContent: '',
  tabs: [],
  activeTabId: null,
  project: null,
  fileTree: [],
  indexStats: null,
  loading: false,
  error: null,

  // 基本 Actions
  setFilePath: (path) => set({ filePath: path }),
  setFileContent: (content) => set({ fileContent: content }),
  setLoading: (loading) => set({ loading }),
  setError: (error) => set({ error }),
  setProject: (project) => set({ project }),
  setFileTree: (tree) => set({ fileTree: tree }),
  setIndexStats: (stats) => set({ indexStats: stats }),

  // 打开项目
  openProject: async (path) => {
    set({ loading: true, error: null })
    try {
      const info = await invoke<ProjectInfo>('open_project', { path })
      set({ project: info })
      const stats = await invoke<IndexStats>('get_index_stats')
      set({ indexStats: stats })
      const tree = await invoke<FileNode[]>('list_directory', { path, recursive: false })
      set({ fileTree: tree, loading: false })
    } catch (e) {
      set({ error: String(e), loading: false })
    }
  },

  // 加载文件树
  loadFileTree: async (path) => {
    try {
      const tree = await invoke<FileNode[]>('list_directory', { path, recursive: false })
      set({ fileTree: tree })
    } catch (e) {
      console.error('Failed to load file tree:', e)
    }
  },

  // 生成唯一 Tab ID
  generateTabId: () => `tab-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,

  // 在标签页中打开文件
  openFileInTab: async (path) => {
    const { tabs, generateTabId } = get()

    // 检查是否已打开
    const existingTab = tabs.find(t => t.filePath === path)
    if (existingTab) {
      set({ activeTabId: existingTab.id, filePath: existingTab.filePath, fileContent: existingTab.content })
      if (existingTab.analysisResult) {
        // 需要通过 analysisStore 更新
      }
      return existingTab.id
    }

    // 加载新文件
    try {
      const content = await invoke<string>('read_file', { path })
      const fileName = path.split('/').pop() || path

      const newTab: TabData = {
        id: generateTabId(),
        filePath: path,
        fileName,
        content,
        isDirty: false,
        analysisResult: null,
      }

      set((state) => ({
        tabs: [...state.tabs, newTab],
        activeTabId: newTab.id,
        filePath: path,
        fileContent: content,
      }))

      return newTab.id
    } catch (err) {
      console.error('打开文件失败:', err)
      return null
    }
  },

  // 关闭标签页
  closeTab: (tabId, force = false) => {
    const { tabs, activeTabId } = get()
    const tabIndex = tabs.findIndex(t => t.id === tabId)
    if (tabIndex === -1) return

    const tab = tabs[tabIndex]

    // 如果有未保存的更改且不强制关闭,返回(需要确认)
    if (tab.isDirty && !force) {
      // 返回特殊值表示需要确认
      return 'needs-confirm'
    }

    const newTabs = tabs.filter(t => t.id !== tabId)
    set({ tabs: newTabs })

    // 如果关闭的是当前标签
    if (activeTabId === tabId) {
      if (newTabs.length === 0) {
        set({ activeTabId: null, filePath: '', fileContent: '' })
      } else {
        const newIndex = Math.max(0, tabIndex - 1)
        const newActive = newTabs[newIndex]
        set({
          activeTabId: newActive.id,
          filePath: newActive.filePath,
          fileContent: newActive.content,
        })
      }
    }
  },

  // 切换标签页
  switchTab: (tabId) => {
    const tab = get().tabs.find(t => t.id === tabId)
    if (!tab) return
    set({
      activeTabId: tabId,
      filePath: tab.filePath,
      fileContent: tab.content,
    })
  },

  // 重新排序标签页
  reorderTabs: (fromIndex, toIndex) => {
    set((state) => {
      const newTabs = [...state.tabs]
      const [movedTab] = newTabs.splice(fromIndex, 1)
      newTabs.splice(toIndex, 0, movedTab)
      return { tabs: newTabs }
    })
  },

  // 更新标签页内容
  updateTabContent: (tabId, content) => {
    set((state) => ({
      tabs: state.tabs.map(tab =>
        tab.id === tabId ? { ...tab, content, isDirty: true } : tab
      ),
      fileContent: content,
    }))
  },

  // 更新标签页分析结果
  updateTabAnalysis: (tabId, analysis) => {
    set((state) => ({
      tabs: state.tabs.map(tab =>
        tab.id === tabId ? { ...tab, analysisResult: analysis } : tab
      ),
    }))
  },

  // 标记标签页为已修改
  markTabDirty: (tabId, isDirty) => {
    set((state) => ({
      tabs: state.tabs.map(tab =>
        tab.id === tabId ? { ...tab, isDirty } : tab
      ),
    }))
  },

  // 读取文件
  readFile: async (path) => {
    const content = await invoke<string>('read_file', { path })
    return content
  },

  // 写入文件
  writeFile: async (path, content) => {
    await invoke('write_file', { path, contents: content })
  },
}))
