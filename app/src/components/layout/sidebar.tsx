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
  ChevronRight,
  ChevronLeft,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { sidebarOpenAtom, sidebarWidthAtom, commandMenuOpenAtom, viewModeAtom } from "../../lib/atoms/layout-atoms"
import { Button } from "../ui/button"

interface NavItem {
  id: string
  icon: React.ElementType
  label: string
  shortcut?: string
  action?: () => void
}

const navItems: NavItem[] = [
  { id: "dashboard", icon: LayoutDashboard, label: "项目" },
  { id: "explorer", icon: FolderOpen, label: "文件" },
  { id: "outline", icon: FileCode, label: "大纲" },
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

  const handleNavClick = (item: NavItem) => {
    if (item.id === "command") {
      setCommandMenuOpen(true)
    } else if (item.id === "flow") {
      setViewMode("flow")
    } else if (item.id === "dashboard") {
      setViewMode("code")
    }
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
        "flex flex-col border-r border-[var(--border-subtle)] bg-[var(--bg-secondary)] transition-all duration-200",
        className
      )}
      style={{ width: sidebarOpen ? sidebarWidth : 0 }}
    >
      <div className="flex flex-col items-center py-3">
        {/* Logo / App Icon */}
        <div className="mb-3 flex h-10 w-10 items-center justify-center rounded-xl bg-[var(--accent)] text-white shadow-lg shadow-[var(--accent)]/20">
          <Zap className="h-5 w-5" />
        </div>

        {/* Nav Items */}
        <nav className="flex flex-col gap-1 px-2">
          {navItems.map((item) => (
            <SidebarButton
              key={item.id}
              item={item}
              isActive={item.id === "flow" ? viewMode === "flow" : item.id === "dashboard" ? viewMode === "code" : false}
              onClick={() => handleNavClick(item)}
            />
          ))}
        </nav>

        {/* Spacer */}
        <div className="flex-1" />

        {/* Bottom Items */}
        <nav className="flex flex-col gap-1 px-2 pb-2">
          {bottomItems.map((item) => (
            <SidebarButton
              key={item.id}
              item={item}
              onClick={() => {}}
            />
          ))}
        </nav>
      </div>

      {/* Resize Handle */}
      {sidebarOpen && (
        <div
          className="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-[var(--accent)]/20"
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

  return (
    <Button
      variant="ghost"
      className={cn(
        "group relative flex h-10 w-10 items-center justify-center rounded-xl transition-all duration-200",
        isActive && "bg-[var(--bg-tertiary)] text-[var(--accent)]"
      )}
      onClick={onClick}
    >
      <Icon className="h-5 w-5" />
      {/* Tooltip */}
      <span className="absolute left-full ml-2 px-2 py-1 rounded-md bg-[var(--bg-tertiary)] text-xs text-[var(--text-primary)] opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none z-50">
        {item.label}
        {item.shortcut && (
          <span className="ml-2 text-[var(--text-muted)]">{item.shortcut}</span>
        )}
      </span>
      {/* Active Indicator */}
      {isActive && (
        <span className="absolute left-0 top-1/2 h-4 w-0.5 -translate-y-1/2 rounded-r-full bg-[var(--accent)]" />
      )}
    </Button>
  )
}
