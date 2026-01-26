import { atom } from "jotai"

// ============================================================================
// Layout Atoms - 布局状态管理
// ============================================================================

// 侧边栏状态
export const sidebarOpenAtom = atom(true)
export const sidebarWidthAtom = atom(72)  // 图标栏宽度

// 右侧面板状态
export const rightPanelOpenAtom = atom(false)
export const rightPanelTabAtom = atom<"outline" | "detail" | "llvm-ir" | "explorer">("outline")
export const rightPanelWidthAtom = atom(280)

// 底部面板状态
export const bottomPanelOpenAtom = atom(true)
export const bottomPanelTabAtom = atom<"terminal" | "output" | "problems">("terminal")

// 命令面板状态
export const commandMenuOpenAtom = atom(false)

// 当前视图模式
export const viewModeAtom = atom<"code" | "flow" | "split">("code")

// 主题状态
export const themeAtom = atom<"dark" | "light">("dark")

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
