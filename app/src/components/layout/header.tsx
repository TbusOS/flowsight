"use client"

import * as React from "react"
import { useAtom, useSetAtom } from "jotai"
import {
  Menubar,
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarTrigger,
} from "@radix-ui/react-menubar"
import {
  Zap,
  Terminal,
  FileCode,
  Command,
  Settings,
  Maximize2,
  Minimize2,
  X,
  FolderOpen,
  FilePlus,
  Save,
  Play,
  Link2,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { viewModeAtom, commandMenuOpenAtom, bottomPanelOpenAtom, bottomPanelTabAtom } from "../../lib/atoms/layout-atoms"
import { Button } from "../ui/button"
import { ThemeSelector } from "../ThemeSelector/ThemeSelector"

interface HeaderProps {
  className?: string
  isMaximized?: boolean
  onToggleMaximize?: () => void
  onClose?: () => void
}

// macOS 风格菜单样式 - 更宽敞的行间距和列间距
const menuContentClass = "min-w-[240px] rounded-lg border border-[var(--border-light)] bg-[var(--bg-secondary)]/95 backdrop-blur-xl py-2 shadow-2xl animate-in fade-in-0 zoom-in-95"
const menuItemClass = "relative flex cursor-pointer select-none items-center rounded-md mx-2 px-4 py-[10px] text-[13px] text-[var(--text-primary)] outline-none hover:bg-[var(--accent)] hover:text-white focus:bg-[var(--accent)] focus:text-white transition-colors"
const menuIconClass = "h-4 w-4 mr-3.5 opacity-70"
const menuShortcutClass = "ml-auto pl-8 text-[12px] tracking-wide text-[var(--text-muted)] group-hover:text-white/70"
const menuSeparatorClass = "my-2 mx-3 h-px bg-[var(--border-light)]"
// 菜单触发器样式 - 更舒适的间距
const menuTriggerClass = "px-4 py-2 text-[13px] font-normal text-[var(--text-secondary)] hover:text-[var(--text-primary)] cursor-pointer rounded-md hover:bg-[var(--bg-tertiary)] transition-colors data-[state=open]:bg-[var(--bg-tertiary)] data-[state=open]:text-[var(--text-primary)]"

export function Header({ className, isMaximized, onToggleMaximize, onClose }: HeaderProps) {
  const [viewMode, setViewMode] = useAtom(viewModeAtom)
  const setCommandMenuOpen = useSetAtom(commandMenuOpenAtom)
  const [bottomPanelOpen, setBottomPanelOpen] = useAtom(bottomPanelOpenAtom)
  const [bottomPanelTab, setBottomPanelTab] = useAtom(bottomPanelTabAtom)

  const handleToggleTerminal = () => {
    setBottomPanelOpen(!bottomPanelOpen)
    if (!bottomPanelOpen) {
      setBottomPanelTab("terminal")
    }
  }

  return (
    <header
      className={cn(
        "flex h-11 items-center justify-between border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)] px-3",
        "drag-region",
        className
      )}
    >
      {/* Left: App Title + Menubar */}
      <div className="flex items-center gap-4">
        {/* App Name */}
        <div className="flex items-center gap-2">
          <div className="flex h-6 w-6 items-center justify-center rounded-md bg-[var(--accent)] text-white">
            <Zap className="h-3.5 w-3.5" />
          </div>
          <span className="text-[13px] font-medium text-[var(--text-primary)]">FlowSight</span>
        </div>

        {/* Menubar - macOS style with spacing between items */}
        <Menubar className="hidden md:flex border-none bg-transparent gap-1">
          {/* 文件菜单 */}
          <MenubarMenu>
            <MenubarTrigger className={menuTriggerClass}>
              文件
            </MenubarTrigger>
            <MenubarContent className={menuContentClass}>
              <MenubarItem className={cn(menuItemClass, "group")}>
                <FilePlus className={menuIconClass} />
                <span>新建项目</span>
                <span className={menuShortcutClass}>⌘N</span>
              </MenubarItem>
              <MenubarItem className={cn(menuItemClass, "group")}>
                <FolderOpen className={menuIconClass} />
                <span>打开文件夹</span>
                <span className={menuShortcutClass}>⌘O</span>
              </MenubarItem>
              <MenubarSeparator className={menuSeparatorClass} />
              <MenubarItem className={cn(menuItemClass, "group")}>
                <Save className={menuIconClass} />
                <span>保存</span>
                <span className={menuShortcutClass}>⌘S</span>
              </MenubarItem>
            </MenubarContent>
          </MenubarMenu>

          {/* 视图菜单 */}
          <MenubarMenu>
            <MenubarTrigger className={menuTriggerClass}>
              视图
            </MenubarTrigger>
            <MenubarContent className={menuContentClass}>
              <MenubarItem className={cn(menuItemClass, "group")} onClick={() => setViewMode("code")}>
                <FileCode className={menuIconClass} />
                <span>代码视图</span>
              </MenubarItem>
              <MenubarItem className={cn(menuItemClass, "group")} onClick={() => setViewMode("flow")}>
                <Zap className={menuIconClass} />
                <span>执行流视图</span>
              </MenubarItem>
              <MenubarSeparator className={menuSeparatorClass} />
              <MenubarItem className={cn(menuItemClass, "group")} onClick={handleToggleTerminal}>
                <Terminal className={menuIconClass} />
                <span>终端面板</span>
                <span className={menuShortcutClass}>⌘J</span>
              </MenubarItem>
            </MenubarContent>
          </MenubarMenu>

          {/* 命令菜单 */}
          <MenubarMenu>
            <MenubarTrigger className={menuTriggerClass}>
              命令
            </MenubarTrigger>
            <MenubarContent className={menuContentClass}>
              <MenubarItem className={cn(menuItemClass, "group")} onClick={() => setCommandMenuOpen(true)}>
                <Command className={menuIconClass} />
                <span>命令面板</span>
                <span className={menuShortcutClass}>⌘K</span>
              </MenubarItem>
              <MenubarSeparator className={menuSeparatorClass} />
              <MenubarItem className={cn(menuItemClass, "group")}>
                <Play className={menuIconClass} />
                <span>运行分析</span>
                <span className={menuShortcutClass}>⇧⌘A</span>
              </MenubarItem>
              <MenubarItem className={cn(menuItemClass, "group")}>
                <Link2 className={menuIconClass} />
                <span>查找引用</span>
                <span className={menuShortcutClass}>⇧⌘F</span>
              </MenubarItem>
            </MenubarContent>
          </MenubarMenu>
        </Menubar>
      </div>

      {/* Right: Actions */}
      <div className="flex items-center gap-1.5 no-drag">
        {/* Theme Selector */}
        <ThemeSelector />

        {/* Command Palette Button */}
        <Button
          variant="ghost"
          size="sm"
          className="h-7 gap-1.5 rounded-md bg-[var(--bg-tertiary)] px-2 text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
          onClick={() => setCommandMenuOpen(true)}
        >
          <Command className="h-3.5 w-3.5" />
          <span className="hidden sm:inline text-[12px]">搜索</span>
          <kbd className="pointer-events-none inline-flex h-4 select-none items-center gap-0.5 rounded border border-[var(--border-light)] bg-[var(--bg-secondary)] px-1 font-mono text-[10px] font-medium">
            <span className="text-[10px]">⌘</span>K
          </kbd>
        </Button>

        {/* Terminal Toggle */}
        <Button
          variant="ghost"
          size="icon"
          className={cn(
            "h-7 w-7 rounded-md",
            bottomPanelOpen && bottomPanelTab === "terminal" && "bg-[var(--accent)]/10 text-[var(--accent)]"
          )}
          onClick={handleToggleTerminal}
        >
          <Terminal className="h-3.5 w-3.5" />
        </Button>

        {/* Settings */}
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7 rounded-md"
        >
          <Settings className="h-3.5 w-3.5" />
        </Button>

        {/* Window Controls (macOS style) */}
        <div className="ml-1.5 hidden md:flex items-center gap-1.5">
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 rounded-full bg-[var(--bg-tertiary)] hover:bg-[var(--bg-hover)]"
            onClick={onClose}
          >
            <X className="h-3 w-3" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 rounded-full bg-[var(--bg-tertiary)] hover:bg-[var(--bg-hover)]"
            onClick={onToggleMaximize}
          >
            {isMaximized ? (
              <Minimize2 className="h-3 w-3" />
            ) : (
              <Maximize2 className="h-3 w-3" />
            )}
          </Button>
        </div>
      </div>
    </header>
  )
}
