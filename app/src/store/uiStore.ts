/**
 * FlowSight UI Store - UI 状态管理
 */

import { create } from 'zustand'
import { AppSettings, defaultSettings } from '../components/Settings'

// 视图模式
export type ViewMode = 'flow' | 'code' | 'split'
export type FlowDisplayMode = 'graph' | 'text'

// 导航历史记录项
export interface NavigationEntry {
  filePath: string
  selectedFunction: string | null
  line?: number
  timestamp: number
}

// 索引进度
export interface IndexProgress {
  phase: string
  current: number
  total: number
  message: string
}

// UI 状态
interface UIState {
  // 视图模式
  viewMode: ViewMode
  flowDisplayMode: FlowDisplayMode

  // 面板状态
  leftPanelOpen: boolean
  rightPanelOpen: boolean
  leftPanelWidth: number
  rightPanelWidth: number

  // 面板展开状态
  leftPanelExpanded: boolean
  rightPanelExpanded: boolean

  // 导航历史
  navHistory: NavigationEntry[]
  navIndex: number

  // 模态框状态
  commandPaletteOpen: boolean
  settingsOpen: boolean
  shortcutsOpen: boolean
  goToLineOpen: boolean
  aboutOpen: boolean
  quickOpenOpen: boolean
  findReplaceOpen: boolean

  // 特殊面板状态
  scenarioPanelOpen: boolean
  scenarioResultsOpen: boolean
  callersViewOpen: boolean
  callersTargetFunc: string

  // 场景分析结果
  currentScenarioName: string
  scenarioResults: {
    path: string[]
    states: { location: string; variables: Record<string, string> }[]
  } | null

  // 索引进度
  indexProgress: IndexProgress | null

  // 设置
  appSettings: AppSettings

  // Actions - 视图模式
  setViewMode: (mode: ViewMode) => void
  setFlowDisplayMode: (mode: FlowDisplayMode) => void
  toggleViewMode: () => void

  // Actions - 面板
  setLeftPanelOpen: (open: boolean) => void
  setRightPanelOpen: (open: boolean) => void
  setLeftPanelWidth: (width: number) => void
  setRightPanelWidth: (width: number) => void
  toggleLeftPanel: () => void
  toggleRightPanel: () => void
  setLeftPanelExpanded: (expanded: boolean) => void
  setRightPanelExpanded: (expanded: boolean) => void

  // Actions - 导航历史
  pushNavHistory: (entry: Omit<NavigationEntry, 'timestamp'>) => void
  goBack: () => Promise<NavigationEntry | undefined>
  goForward: () => Promise<NavigationEntry | undefined>
  setNavHistory: (history: NavigationEntry[]) => void
  setNavIndex: (index: number) => void

  // Actions - 模态框
  setCommandPaletteOpen: (open: boolean) => void
  setSettingsOpen: (open: boolean) => void
  setShortcutsOpen: (open: boolean) => void
  setGoToLineOpen: (open: boolean) => void
  setAboutOpen: (open: boolean) => void
  setQuickOpenOpen: (open: boolean) => void
  setFindReplaceOpen: (open: boolean) => void

  // Actions - 特殊面板
  setScenarioPanelOpen: (open: boolean) => void
  setScenarioResultsOpen: (open: boolean) => void
  setCallersViewOpen: (open: boolean) => void
  setCallersTargetFunc: (func: string) => void

  // Actions - 场景分析
  setCurrentScenarioName: (name: string) => void
  setScenarioResults: (results: UIState['scenarioResults']) => void
  clearScenarioResults: () => void

  // Actions - 索引进度
  setIndexProgress: (progress: IndexProgress | null) => void

  // Actions - 设置
  setAppSettings: (settings: AppSettings) => void
  updateAppSettings: (updates: Partial<AppSettings>) => void

  // 计算属性
  canGoBack: () => boolean
  canGoForward: () => boolean
}

export const useUIStore = create<UIState>((set, get) => ({
  // 初始状态
  viewMode: 'split',
  flowDisplayMode: 'graph',

  leftPanelOpen: true,
  rightPanelOpen: true,
  leftPanelWidth: 220,
  rightPanelWidth: 280,

  leftPanelExpanded: true,
  rightPanelExpanded: true,

  navHistory: [],
  navIndex: -1,

  commandPaletteOpen: false,
  settingsOpen: false,
  shortcutsOpen: false,
  goToLineOpen: false,
  aboutOpen: false,
  quickOpenOpen: false,
  findReplaceOpen: false,

  scenarioPanelOpen: false,
  scenarioResultsOpen: false,
  callersViewOpen: false,
  callersTargetFunc: '',

  currentScenarioName: '',
  scenarioResults: null,

  indexProgress: null,

  appSettings: defaultSettings,

  // 视图模式 Actions
  setViewMode: (mode) => set({ viewMode: mode }),
  setFlowDisplayMode: (mode) => set({ flowDisplayMode: mode }),
  toggleViewMode: () => {
    const { viewMode } = get()
    const modes: ViewMode[] = ['code', 'split', 'flow']
    const currentIndex = modes.indexOf(viewMode)
    const nextIndex = (currentIndex + 1) % modes.length
    set({ viewMode: modes[nextIndex] })
  },

  // 面板 Actions
  setLeftPanelOpen: (open) => set({ leftPanelOpen: open }),
  setRightPanelOpen: (open) => set({ rightPanelOpen: open }),
  setLeftPanelWidth: (width) => set({ leftPanelWidth: Math.max(180, Math.min(400, width)) }),
  setRightPanelWidth: (width) => set({ rightPanelWidth: Math.max(200, Math.min(450, width)) }),
  toggleLeftPanel: () => set((state) => ({ leftPanelOpen: !state.leftPanelOpen })),
  toggleRightPanel: () => set((state) => ({ rightPanelOpen: !state.rightPanelOpen })),
  setLeftPanelExpanded: (expanded) => set({ leftPanelExpanded: expanded }),
  setRightPanelExpanded: (expanded) => set({ rightPanelExpanded: expanded }),

  // 导航历史 Actions
  pushNavHistory: (entry) => {
    const { navHistory, navIndex } = get()

    // 检查是否和当前位置相同
    const current = navHistory[navIndex]
    if (current &&
        current.filePath === entry.filePath &&
        current.selectedFunction === entry.selectedFunction &&
        current.line === entry.line) {
      return
    }

    const newHistory = navHistory.slice(0, navIndex + 1)
    newHistory.push({ ...entry, timestamp: Date.now() })

    // 限制历史长度
    if (newHistory.length > 50) {
      newHistory.shift()
    }

    set({
      navHistory: newHistory,
      navIndex: Math.min(navIndex + 1, 49),
    })
  },

  goBack: async () => {
    const { navHistory, navIndex } = get()
    if (navIndex <= 0) return undefined

    const newIndex = navIndex - 1
    const entry = navHistory[newIndex]
    set({ navIndex: newIndex })

    return entry
  },

  goForward: async () => {
    const { navHistory, navIndex } = get()
    if (navIndex >= navHistory.length - 1) return undefined

    const newIndex = navIndex + 1
    const entry = navHistory[newIndex]
    set({ navIndex: newIndex })

    return entry
  },

  setNavHistory: (history) => set({ navHistory: history }),
  setNavIndex: (index) => set({ navIndex: index }),

  // 模态框 Actions
  setCommandPaletteOpen: (open) => set({ commandPaletteOpen: open }),
  setSettingsOpen: (open) => set({ settingsOpen: open }),
  setShortcutsOpen: (open) => set({ shortcutsOpen: open }),
  setGoToLineOpen: (open) => set({ goToLineOpen: open }),
  setAboutOpen: (open) => set({ aboutOpen: open }),
  setQuickOpenOpen: (open) => set({ quickOpenOpen: open }),
  setFindReplaceOpen: (open) => set({ findReplaceOpen: open }),

  // 特殊面板 Actions
  setScenarioPanelOpen: (open) => set({ scenarioPanelOpen: open }),
  setScenarioResultsOpen: (open) => set({ scenarioResultsOpen: open }),
  setCallersViewOpen: (open) => set({ callersViewOpen: open }),
  setCallersTargetFunc: (func) => set({ callersTargetFunc: func }),

  // 场景分析 Actions
  setCurrentScenarioName: (name) => set({ currentScenarioName: name }),
  setScenarioResults: (results) => set({ scenarioResults: results }),
  clearScenarioResults: () => set({ scenarioResults: null, currentScenarioName: '' }),

  // 索引��度 Actions
  setIndexProgress: (progress) => set({ indexProgress: progress }),

  // 设置 Actions
  setAppSettings: (settings) => set({ appSettings: settings }),
  updateAppSettings: (updates) => set((state) => ({
    appSettings: { ...state.appSettings, ...updates }
  })),

  // 计算属性
  canGoBack: () => get().navIndex > 0,
  canGoForward: () => get().navIndex < get().navHistory.length - 1,
}))
