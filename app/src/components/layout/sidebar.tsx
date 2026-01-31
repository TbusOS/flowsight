"use client"

import * as React from "react"
import { useAtom, useSetAtom } from "jotai"
import { open } from "@tauri-apps/plugin-dialog"
import {
  LayoutDashboard,
  FolderOpen,
  FileCode,
  Zap,
  Search,
  Command,
  Settings,
  Sparkles,
} from "lucide-react"
import { motion, AnimatePresence } from "framer-motion"
import { cn } from "../../lib/utils"
import {
  sidebarOpenAtom,
  sidebarWidthAtom,
  commandMenuOpenAtom,
  viewModeAtom,
  rightPanelOpenAtom,
  rightPanelTabAtom,
} from "../../lib/atoms/layout-atoms"
import { useAnalysisStore } from "../../store/analysisStore"
import { Button } from "../ui/button"

interface NavItem {
  id: string
  icon: React.ElementType
  label: string
  shortcut?: string
  panelTab?: "outline" | "detail" | "llvm-ir" | "explorer" | "search"
}

const navItems: NavItem[] = [
  { id: "dashboard", icon: LayoutDashboard, label: "项目" },
  { id: "explorer", icon: FolderOpen, label: "文件", panelTab: "explorer" },
  { id: "outline", icon: FileCode, label: "大纲", panelTab: "outline" },
  { id: "flow", icon: Zap, label: "执行流" },
  { id: "search", icon: Search, label: "搜索", panelTab: "search" },
  { id: "command", icon: Command, label: "命令", shortcut: "K" },
]

const bottomItems: NavItem[] = [
  { id: "settings", icon: Settings, label: "设置" },
]

interface SidebarProps {
  className?: string
}

export function Sidebar({ className }: SidebarProps) {
  const [sidebarOpen, setSidebarOpen] = useAtom(sidebarOpenAtom)
  const [sidebarWidth, setSidebarWidth] = useAtom(sidebarWidthAtom)
  const setCommandMenuOpen = useSetAtom(commandMenuOpenAtom)
  const [viewMode, setViewMode] = useAtom(viewModeAtom)
  const [rightPanelOpen, setRightPanelOpen] = useAtom(rightPanelOpenAtom)
  const [rightPanelTab, setRightPanelTab] = useAtom(rightPanelTabAtom)
  const currentProject = useAnalysisStore((state) => state.currentProject)
  const openProject = useAnalysisStore((state) => state.openProject)

  const handleNavClick = async (item: NavItem) => {
    if (item.id === "command") {
      setCommandMenuOpen(true)
    } else if (item.id === "flow") {
      setViewMode("flow")
    } else if (item.id === "dashboard") {
      // 如果没有项目，打开项目选择对话框
      if (!currentProject) {
        try {
          const selected = await open({
            directory: true,
            multiple: false,
            title: "选择项目目录",
          })
          if (selected) {
            await openProject(selected as string)
          }
        } catch (error) {
          console.error("打开项目失败:", error)
        }
      } else {
        setViewMode("code")
      }
    } else if (item.panelTab) {
      if (rightPanelTab === item.panelTab && rightPanelOpen) {
        setRightPanelOpen(false)
      } else {
        setRightPanelTab(item.panelTab)
        setRightPanelOpen(true)
      }
    }
  }

  const getIsActive = (item: NavItem) => {
    if (item.id === "flow") return viewMode === "flow"
    if (item.id === "dashboard") return viewMode === "code"
    if (item.panelTab) return rightPanelOpen && rightPanelTab === item.panelTab
    return false
  }

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault()
    const startX = e.clientX
    const startWidth = sidebarWidth

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const diff = startX - moveEvent.x
      const newWidth = Math.max(56, Math.min(80, startWidth + diff))
      setSidebarWidth(newWidth)
    }

    const handleMouseUp = () => {
      document.removeEventListener("mousemove", handleMouseMove)
      document.removeEventListener("mouseup", handleMouseUp)
    }

    document.addEventListener("mousemove", handleMouseMove)
    document.addEventListener("mouseup", handleMouseUp)
  }

  return (
    <aside
      className={cn(
        "flex flex-col border-r border-[var(--border-subtle)] bg-[var(--bg-secondary)] transition-all duration-100 ease-out",
        className
      )}
      style={{ width: sidebarOpen ? sidebarWidth : 0 }}
    >
      <div className="flex flex-col items-center py-3">
        {/* Logo / App Icon - Cursor/21st.dev style */}
        <motion.div
          className="mb-2 relative"
          whileHover={{ scale: 1 }}
          whileTap={{ scale: 0.97 }}
        >
          <div className="flex h-9 w-9 items-center justify-center rounded-md bg-[var(--accent)] text-white">
            <Sparkles className="h-4.5 w-4.5" />
          </div>
        </motion.div>

        {/* Nav Items */}
        <nav className="flex flex-col gap-1.5 px-1.5 py-2">
          {navItems.map((item) => (
            <SidebarButton
              key={item.id}
              item={item}
              isActive={getIsActive(item)}
              onClick={() => handleNavClick(item)}
            />
          ))}
        </nav>

        {/* Spacer */}
        <div className="flex-1" />

        {/* Bottom Items */}
        <nav className="flex flex-col gap-1 px-1 pb-1">
          {bottomItems.map((item) => (
            <SidebarButton
              key={item.id}
              item={item}
              isActive={false}
              onClick={() => {}}
            />
          ))}
        </nav>
      </div>

      {/* Resize Handle */}
      {sidebarOpen && (
        <div
          className="absolute right-0 top-0 bottom-0 w-px cursor-col-resize hover:bg-[var(--accent)]/40 transition-colors"
          onMouseDown={handleMouseDown}
        />
      )}
    </aside>
  )
}

interface SidebarButtonProps {
  item: NavItem
  isActive?: boolean
  onClick: () => void
}

function SidebarButton({ item, isActive, onClick }: SidebarButtonProps) {
  const Icon = item.icon
  const [showTooltip, setShowTooltip] = React.useState(false)
  const tooltipRef = React.useRef<HTMLSpanElement>(null)
  const buttonRef = React.useRef<HTMLButtonElement>(null)

  React.useEffect(() => {
    if (showTooltip && buttonRef.current && tooltipRef.current) {
      const buttonRect = buttonRef.current.getBoundingClientRect()
      const tooltipRect = tooltipRef.current.getBoundingClientRect()
    }
  }, [showTooltip])

  return (
    <div className="relative">
      <motion.div
        whileHover={{ scale: 1 }}
        whileTap={{ scale: 0.97 }}
      >
        <Button
          ref={buttonRef}
          variant="ghost"
          aria-label={item.label}
          data-testid={`sidebar-${item.id}`}
          className={cn(
            "group relative flex h-9 w-9 items-center justify-center rounded-md transition-all duration-100",
            isActive
              ? "bg-[var(--bg-tertiary)] text-[var(--accent)]"
              : "text-[var(--text-muted)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]"
          )}
          onClick={onClick}
          onMouseEnter={() => setShowTooltip(true)}
          onMouseLeave={() => setShowTooltip(false)}
        >
          <Icon className={cn(
            "h-4.5 w-4.5 transition-all duration-100",
            isActive && "scale-105"
          )} />

          {/* Active Indicator */}
          {isActive && (
            <motion.div
              layoutId="active-indicator"
              className="absolute left-0 top-1/2 h-3.5 w-px -translate-y-1/2 rounded-r-full bg-[var(--accent)]"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
            />
          )}
        </Button>
      </motion.div>

      {/* Tooltip - Cursor/21st.dev style */}
      <AnimatePresence>
        {showTooltip && (
          <motion.span
            ref={tooltipRef}
            initial={{ opacity: 0, x: 4 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 2 }}
            transition={{ duration: 0.1 }}
            className="absolute left-full ml-1.5 top-1/2 -translate-y-1/2 z-50 pointer-events-none"
          >
            <div className="flex items-center gap-2 px-2 py-1 rounded-sm bg-[var(--bg-tertiary)] border border-[var(--border-light)] shadow-sm">
              <span className="text-xs font-medium text-[var(--text-primary)] whitespace-nowrap">
                {item.label}
              </span>
              {item.shortcut && (
                <span className="text-[10px] px-1 py-0.5 rounded bg-[var(--bg-secondary)] text-[var(--text-muted)] font-mono">
                  {item.shortcut}
                </span>
              )}
            </div>
          </motion.span>
        )}
      </AnimatePresence>
    </div>
  )
}
