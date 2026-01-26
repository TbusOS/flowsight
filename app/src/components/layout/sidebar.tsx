"use client"

import * as React from "react"
import { useAtom, useSetAtom } from "jotai"
import {
  LayoutDashboard,
  FolderOpen,
  FileCode,
  Zap,
  Search,
  Command,
  Settings,
  GitBranch,
  Box,
  Cpu,
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
import { Button } from "../ui/button"

interface NavItem {
  id: string
  icon: React.ElementType
  label: string
  shortcut?: string
  panelTab?: "outline" | "detail" | "llvm-ir" | "explorer"
}

const navItems: NavItem[] = [
  { id: "dashboard", icon: LayoutDashboard, label: "项目" },
  { id: "explorer", icon: FolderOpen, label: "文件", panelTab: "explorer" },
  { id: "outline", icon: FileCode, label: "大纲", panelTab: "outline" },
  { id: "flow", icon: Zap, label: "执行流" },
  { id: "search", icon: Search, label: "搜索" },
  { id: "command", icon: Command, label: "命令", shortcut: "⌘K" },
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

  const handleNavClick = (item: NavItem) => {
    if (item.id === "command") {
      setCommandMenuOpen(true)
    } else if (item.id === "flow") {
      setViewMode("flow")
    } else if (item.id === "dashboard") {
      setViewMode("code")
    } else if (item.panelTab) {
      // Toggle right panel with specific tab
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
        "flex flex-col border-r border-[var(--border-subtle)] bg-[var(--bg-secondary)] transition-all duration-300 ease-out",
        className
      )}
      style={{ width: sidebarOpen ? sidebarWidth : 0 }}
    >
      <div className="flex flex-col items-center py-4">
        {/* Logo / App Icon - macOS style */}
        <motion.div
          className="mb-4 relative"
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
        >
          <div className="flex h-11 w-11 items-center justify-center rounded-xl bg-gradient-to-br from-[var(--accent)] to-[var(--accent-hover)] text-white shadow-lg shadow-[var(--accent)]/25">
            <Sparkles className="h-5 w-5" />
          </div>
          {/* macOS traffic light indicator */}
          <div className="absolute -top-1 -right-1 flex gap-1">
            <div className="h-2 w-2 rounded-full bg-[var(--accent)]/80" />
          </div>
        </motion.div>

        {/* Nav Items */}
        <nav className="flex flex-col gap-1.5 px-2">
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
        <nav className="flex flex-col gap-1.5 px-2 pb-3">
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
          className="absolute right-0 top-0 bottom-0 w-0.5 cursor-col-resize hover:bg-[var(--accent)]/30 transition-colors"
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

  // Calculate tooltip position
  React.useEffect(() => {
    if (showTooltip && buttonRef.current && tooltipRef.current) {
      const buttonRect = buttonRef.current.getBoundingClientRect()
      const tooltipRect = tooltipRef.current.getBoundingClientRect()
      // Tooltip is positioned to the right with fixed offset
    }
  }, [showTooltip])

  return (
    <div className="relative">
      <motion.div
        whileHover={{ scale: 1.02 }}
        whileTap={{ scale: 0.98 }}
      >
        <Button
          ref={buttonRef}
          variant="ghost"
          className={cn(
            "group relative flex h-10 w-10 items-center justify-center rounded-xl transition-all duration-200",
            isActive
              ? "bg-[var(--bg-tertiary)] text-[var(--accent)] shadow-sm"
              : "text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
          )}
          onClick={onClick}
          onMouseEnter={() => setShowTooltip(true)}
          onMouseLeave={() => setShowTooltip(false)}
        >
          <Icon className={cn(
            "h-5 w-5 transition-all duration-200",
            isActive && "scale-110"
          )} />

          {/* Active Indicator - macOS style pill */}
          {isActive && (
            <motion.div
              layoutId="active-indicator"
              className="absolute left-0 top-1/2 h-5 w-0.5 -translate-y-1/2 rounded-r-full bg-[var(--accent)]"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
            />
          )}
        </Button>
      </motion.div>

      {/* Tooltip - macOS style */}
      <AnimatePresence>
        {showTooltip && (
          <motion.span
            ref={tooltipRef}
            initial={{ opacity: 0, x: 8 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 4 }}
            transition={{ duration: 0.15 }}
            className="absolute left-full ml-2 top-1/2 -translate-y-1/2 z-50 pointer-events-none"
          >
            <div className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-light)] shadow-lg">
              <span className="text-xs font-medium text-[var(--text-primary)] whitespace-nowrap">
                {item.label}
              </span>
              {item.shortcut && (
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-[var(--bg-secondary)] text-[var(--text-muted)] font-mono">
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

// Unused imports removed
