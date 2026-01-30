import { atom } from "jotai"

// ============================================================================
// Layout Atoms - 布局状态管理
// ============================================================================

// 侧边栏状态
export const sidebarOpenAtom = atom(true)
export const sidebarWidthAtom = atom(80)  // 图标栏宽度

// 右侧面板状态
export const rightPanelOpenAtom = atom(false)
export const rightPanelTabAtom = atom<"outline" | "detail" | "llvm-ir" | "explorer" | "search">("outline")
export const rightPanelWidthAtom = atom(280)

// 底部面板状态
export const bottomPanelOpenAtom = atom(true)
export const bottomPanelTabAtom = atom<"terminal" | "output" | "problems">("terminal")

// 命令面板状态
export const commandMenuOpenAtom = atom(false)

// 当前视图模式
export const viewModeAtom = atom<"code" | "flow" | "split">("code")

// 当前打开的文件路径
export const currentFileAtom = atom<string | null>(null)

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
// Helper Atoms - 组合状态
// ============================================================================

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
    rightPanelOpen: get(rightPanelOpenAtom),
    bottomPanelOpen: get(bottomPanelOpenAtom),
    commandMenuOpen: get(commandMenuOpenAtom),
    viewMode: get(viewModeAtom),
  }
})
