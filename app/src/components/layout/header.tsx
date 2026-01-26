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
  ChevronDown,
  Maximize2,
  Minimize2,
  X,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { viewModeAtom, commandMenuOpenAtom, bottomPanelOpenAtom, bottomPanelTabAtom } from "../../lib/atoms/layout-atoms"
import { Button } from "../ui/button"

interface HeaderProps {
  className?: string
  isMaximized?: boolean
  onToggleMaximize?: () => void
  onClose?: () => void
}

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
        "flex h-12 items-center justify-between border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)] px-4",
        "drag-region",  // macOS 窗口拖动区域
        className
      )}
    >
      {/* Left: App Title + Menubar */}
      <div className="flex items-center gap-4">
        {/* App Name */}
        <div className="flex items-center gap-2">
          <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-[var(--accent)] text-white">
            <Zap className="h-4 w-4" />
          </div>
          <span className="text-sm font-medium text-[var(--text-primary)]">FlowSight</span>
        </div>

        {/* Menubar (hidden on small screens) */}
        <Menubar className="hidden md:flex border-none bg-transparent">
          <MenubarMenu>
            <MenubarTrigger className="flex items-center gap-1 px-2 py-1 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] cursor-pointer rounded-md hover:bg-[var(--bg-tertiary)]">
              文件
              <ChevronDown className="h-3 w-3" />
            </MenubarTrigger>
            <MenubarContent className="min-w-[180px] rounded-xl border border-[var(--border-light)] bg-[var(--bg-secondary)] p-1 shadow-xl">
              <MenubarItem className="flex cursor-pointer items-center rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] focus:bg-[var(--bg-tertiary)]">
                新建项目
                <kbd className="ml-auto text-[var(--text-muted)]">⌘N</kbd>
              </MenubarItem>
              <MenubarItem className="flex cursor-pointer items-center rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] focus:bg-[var(--bg-tertiary)]">
                打开文件
                <kbd className="ml-auto text-[var(--text-muted)]">⌘O</kbd>
              </MenubarItem>
              <MenubarSeparator className="my-1 h-px bg-[var(--border-light)]" />
              <MenubarItem className="flex cursor-pointer items-center rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] focus:bg-[var(--bg-tertiary)]">
                保存
                <kbd className="ml-auto text-[var(--text-muted)]">⌘S</kbd>
              </MenubarItem>
            </MenubarContent>
          </MenubarMenu>

          <MenubarMenu>
            <MenubarTrigger className="flex items-center gap-1 px-2 py-1 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] cursor-pointer rounded-md hover:bg-[var(--bg-tertiary)]">
              视图
              <ChevronDown className="h-3 w-3" />
            </MenubarTrigger>
            <MenubarContent className="min-w-[180px] rounded-xl border border-[var(--border-light)] bg-[var(--bg-secondary)] p-1 shadow-xl">
              <MenubarItem
                className="flex cursor-pointer items-center rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] focus:bg-[var(--bg-tertiary)]"
                onClick={() => setViewMode("code")}
              >
                <FileCode className="mr-2 h-4 w-4" />
                代码视图
              </MenubarItem>
              <MenubarItem
                className="flex cursor-pointer items-center rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] focus:bg-[var(--bg-tertiary)]"
                onClick={() => setViewMode("flow")}
              >
                <Zap className="mr-2 h-4 w-4" />
                执行流视图
              </MenubarItem>
              <MenubarItem
                className="flex cursor-pointer items-center rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] focus:bg-[var(--bg-tertiary)]"
                onClick={handleToggleTerminal}
              >
                <Terminal className="mr-2 h-4 w-4" />
                终端面板
                <kbd className="ml-auto text-[var(--text-muted)]">⌘J</kbd>
              </MenubarItem>
            </MenubarContent>
          </MenubarMenu>

          <MenubarMenu>
            <MenubarTrigger className="flex items-center gap-1 px-2 py-1 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] cursor-pointer rounded-md hover:bg-[var(--bg-tertiary)]">
              命令
              <ChevronDown className="h-3 w-3" />
            </MenubarTrigger>
            <MenubarContent className="min-w-[180px] rounded-xl border border-[var(--border-light)] bg-[var(--bg-secondary)] p-1 shadow-xl">
              <MenubarItem
                className="flex cursor-pointer items-center rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] focus:bg-[var(--bg-tertiary)]"
                onClick={() => setCommandMenuOpen(true)}
              >
                <Command className="mr-2 h-4 w-4" />
                命令面板
                <kbd className="ml-auto text-[var(--text-muted)]">⌘K</kbd>
              </MenubarItem>
              <MenubarSeparator className="my-1 h-px bg-[var(--border-light)]" />
              <MenubarItem className="flex cursor-pointer items-center rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] focus:bg-[var(--bg-tertiary)]">
                运行分析
                <kbd className="ml-auto text-[var(--text-muted)]">⌘⇧A</kbd>
              </MenubarItem>
              <MenubarItem className="flex cursor-pointer items-center rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] focus:bg-[var(--bg-tertiary)]">
                查找引用
                <kbd className="ml-auto text-[var(--text-muted)]">⌘⇧F</kbd>
              </MenubarItem>
            </MenubarContent>
          </MenubarMenu>
        </Menubar>
      </div>

      {/* Right: Actions */}
      <div className="flex items-center gap-2 no-drag">
        {/* Command Palette Button */}
        <Button
          variant="ghost"
          size="sm"
          className="h-8 gap-2 rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
          onClick={() => setCommandMenuOpen(true)}
        >
          <Command className="h-4 w-4" />
          <span className="hidden sm:inline">搜索</span>
          <kbd className="pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border border-[var(--border-light)] bg-[var(--bg-secondary)] px-1.5 font-mono text-[10px] font-medium">
            <span className="text-xs">⌘</span>K
          </kbd>
        </Button>

        {/* Terminal Toggle */}
        <Button
          variant="ghost"
          size="icon"
          className={cn(
            "h-8 w-8 rounded-lg",
            bottomPanelOpen && bottomPanelTab === "terminal" && "bg-[var(--accent)]/10 text-[var(--accent)]"
          )}
          onClick={handleToggleTerminal}
        >
          <Terminal className="h-4 w-4" />
        </Button>

        {/* Settings */}
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 rounded-lg"
        >
          <Settings className="h-4 w-4" />
        </Button>

        {/* Window Controls (macOS style) */}
        <div className="ml-2 hidden md:flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 rounded-full bg-[var(--bg-tertiary)] hover:bg-[var(--bg-hover)]"
            onClick={onClose}
          >
            <X className="h-3 w-3" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 rounded-full bg-[var(--bg-tertiary)] hover:bg-[var(--bg-hover)]"
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
