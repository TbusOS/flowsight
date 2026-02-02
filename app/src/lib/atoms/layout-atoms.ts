import { atom } from "jotai"

// ============================================================================
// Layout Atoms - 布局状态管理
// ============================================================================

// 侧边栏状态
export const sidebarOpenAtom = atom(true)
export const sidebarWidthAtom = atom(80)  // 图标栏宽度

// 左侧面板状态（文件浏览器）
export const leftPanelOpenAtom = atom(true)
export const leftPanelWidthAtom = atom(240)

// 设置左侧面板宽度（带约束）
export const setLeftPanelWidthAtom = atom(
  null,
  (get, set, width: number) => {
    const constrainedWidth = Math.max(150, Math.min(500, width))
    set(leftPanelWidthAtom, constrainedWidth)
  }
)

// 右侧面板状态
export const rightPanelOpenAtom = atom(false)
export const rightPanelTabAtom = atom<"outline" | "detail" | "llvm-ir" | "explorer" | "search">("outline")
export const rightPanelWidthAtom = atom(280)

// 设置右侧面板宽度（带约束）
export const setRightPanelWidthAtom = atom(
  null,
  (get, set, width: number) => {
    const constrainedWidth = Math.max(200, Math.min(500, width))
    set(rightPanelWidthAtom, constrainedWidth)
  }
)

// 底部面板高度
export const bottomPanelHeightAtom = atom(200)

// 设置底部面板高度（带约束）
export const setBottomPanelHeightAtom = atom(
  null,
  (get, set, height: number) => {
    const constrainedHeight = Math.max(100, Math.min(400, height))
    set(bottomPanelHeightAtom, constrainedHeight)
  }
)

// 底部面板状态
export const bottomPanelOpenAtom = atom(true)
export const bottomPanelTabAtom = atom<"terminal" | "output" | "problems">("terminal")

// 命令面板状态
export const commandMenuOpenAtom = atom(false)

// 当前视图模式
export const viewModeAtom = atom<"code" | "flow" | "split">("code")

// 当前打开的文件路径
export const currentFileAtom = atom<string | null>(null)

// 编辑器光标位置
export interface CursorPosition {
  line: number
  column: number
  selection?: {
    startLine: number
    startColumn: number
    endLine: number
    endColumn: number
  }
}

export const cursorPositionAtom = atom<CursorPosition>({
  line: 1,
  column: 1,
})

// 代码跳转目标（文件路径 + 行号）
export interface JumpTarget {
  filePath: string
  line: number
  column?: number
}

export const jumpTargetAtom = atom<JumpTarget | null>(null)

// 触发代码跳转的 action atom
export const triggerJumpAtom = atom(
  null,
  (_get, set, target: JumpTarget) => {
    set(jumpTargetAtom, target)
    // 如果目标文件不同，先切换文件
    set(currentFileAtom, target.filePath)
    // 切换到代码视图
    set(viewModeAtom, "code")
  }
)

// ============================================================================
// Theme Atoms - 主题状态管理
// ============================================================================

export type Theme = "dark" | "light" | "slate" | "forest" | "sunset" | "lavender"

const THEME_STORAGE_KEY = "flowsight-theme"

// 从 localStorage 加载主题
const getInitialTheme = (): Theme => {
  if (typeof window === "undefined") return "dark"
  const stored = localStorage.getItem(THEME_STORAGE_KEY)
  if (stored && ["dark", "light", "slate", "forest", "sunset", "lavender"].includes(stored)) {
    return stored as Theme
  }
  return "dark"
}

// 应用主题到 document
const applyTheme = (theme: Theme) => {
  if (typeof document !== "undefined") {
    document.documentElement.setAttribute("data-theme", theme)
  }
}

// 主题状态 - 只读 atom
export const themeAtom = atom<Theme>(getInitialTheme())

// 主题元信息
export const themeInfoAtom = atom((get) => {
  const theme = get(themeAtom)
  const themeMap: Record<Theme, { label: string; color: string; description: string }> = {
    dark: { label: "深色", color: "#3b82f6", description: "默认深色主题" },
    light: { label: "浅色", color: "#f5f5f7", description: "清爽浅色主题" },
    slate: { label: "Slate", color: "#64748b", description: "专业灰蓝主题" },
    forest: { label: "Forest", color: "#10b981", description: "清新森林主题" },
    sunset: { label: "Sunset", color: "#d97706", description: "温暖落日主题" },
    lavender: { label: "Lavender", color: "#8b5cf6", description: "淡雅薰衣草主题" },
  }
  return themeMap[theme]
})

// 所有可用主题列表
export const themesAtom = atom<Theme[]>(["dark", "light", "slate", "forest", "sunset", "lavender"])

// 切换到下一个主题
export const nextThemeAtom = atom(null, (get, set) => {
  const themes = get(themesAtom)
  const current = get(themeAtom)
  const currentIndex = themes.indexOf(current)
  const nextIndex = (currentIndex + 1) % themes.length
  const nextTheme = themes[nextIndex]
  set(themeAtom, nextTheme)
})

// 设置特定主题（带持久化）
export const setThemeWithPersistenceAtom = atom(
  null,
  (get, set, theme: Theme) => {
    set(themeAtom, theme)
    if (typeof window !== "undefined") {
      localStorage.setItem(THEME_STORAGE_KEY, theme)
      applyTheme(theme)
    }
  }
)

// 初始化主题（客户端 hydration 后调用）
export const initThemeAtom = atom(null, (get, set) => {
  const theme = get(themeAtom)
  if (typeof window !== "undefined") {
    applyTheme(theme)
  }
})

// 加载状态
export const isLoadingAtom = atom(false)

// ============================================================================
// Node Selection Atoms - 节点选择状态
// ============================================================================

// 选中节点详情数据结构
export interface SelectedNodeDetail {
  id: string
  name: string
  return_type: string
  parameters: Array<{ name: string; type: string }>
  file_path: string | null
  line: number
  is_callback: boolean
  callback_context?: string
  calls: string[]
  called_by: string[]
  node_type?: string
  description?: string
}

// 当前选中的节点
export const selectedNodeAtom = atom<SelectedNodeDetail | null>(null)

// 设置选中节点的 action atom
export const setSelectedNodeAtom = atom(
  null,
  (_get, set, node: SelectedNodeDetail | null) => {
    set(selectedNodeAtom, node)
  }
)

// ============================================================================
// Entry Function Atoms - 入口函数选择状态
// ============================================================================

// 当前选中的入口函数（用于触发执行流分析）
export const selectedEntryFunctionAtom = atom<string | null>(null)

// 设置入口函数的 action atom（同时切换到执行流视图）
export const setEntryFunctionAtom = atom(
  null,
  (_get, set, funcName: string | null) => {
    set(selectedEntryFunctionAtom, funcName)
    if (funcName) {
      // 自动切换到执行流视图
      set(viewModeAtom, "flow")
    }
  }
)

// ============================================================================
// Helper Atoms - 组合状态
// ============================================================================

export const isLeftPanelVisibleAtom = atom((get) => {
  return get(leftPanelOpenAtom)
})

export const isRightPanelVisibleAtom = atom((get) => {
  return get(rightPanelOpenAtom) && get(rightPanelTabAtom) !== null
})

export const isBottomPanelVisibleAtom = atom((get) => {
  return get(bottomPanelOpenAtom) && get(bottomPanelTabAtom) !== null
})

// 布局状态组合
export const layoutStateAtom = atom((get) => {
  return {
    sidebarOpen: get(sidebarOpenAtom),
    leftPanelOpen: get(leftPanelOpenAtom),
    rightPanelOpen: get(rightPanelOpenAtom),
    bottomPanelOpen: get(bottomPanelOpenAtom),
    commandMenuOpen: get(commandMenuOpenAtom),
    viewMode: get(viewModeAtom),
  }
})
