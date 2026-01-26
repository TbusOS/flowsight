"use client"

import * as React from "react"
import { useAtomValue } from "jotai"
import {
  Terminal,
  AlertCircle,
  Info,
  ChevronRight,
  GitBranch,
  Wifi,
  WifiOff,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { bottomPanelOpenAtom, bottomPanelTabAtom } from "../../lib/atoms/layout-atoms"
import { Button } from "../ui/button"

interface StatusBarProps {
  className?: string
}

export function StatusBar({ className }: StatusBarProps) {
  const bottomPanelOpen = useAtomValue(bottomPanelOpenAtom)
  const bottomPanelTab = useAtomValue(bottomPanelTabAtom)

  const tabs = [
    { id: "terminal", icon: Terminal, label: "终端" },
    { id: "problems", icon: AlertCircle, label: "问题" },
    { id: "output", icon: Info, label: "输出" },
  ] as const

  return (
    <footer
      className={cn(
        "flex h-6 items-center justify-between border-t border-[var(--border-subtle)] bg-[var(--bg-secondary)] px-3 text-xs text-[var(--text-muted)]",
        className
      )}
    >
      {/* Left: Panel Tabs */}
      <div className="flex items-center gap-1">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={cn(
              "flex items-center gap-1.5 rounded px-2 py-0.5 transition-colors",
              bottomPanelOpen && bottomPanelTab === tab.id
                ? "bg-[var(--bg-tertiary)] text-[var(--text-primary)]"
                : "hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-secondary)]"
            )}
          >
            <tab.icon className="h-3 w-3" />
            <span>{tab.label}</span>
          </button>
        ))}
      </div>

      {/* Right: Status Info */}
      <div className="flex items-center gap-4">
        {/* Git Branch */}
        <div className="flex items-center gap-1">
          <GitBranch className="h-3 w-3" />
          <span>main</span>
        </div>

        {/* Connection Status */}
        <div className="flex items-center gap-1">
          <Wifi className="h-3 w-3" />
          <span>已连接</span>
        </div>

        {/* Position Info */}
        <div className="flex items-center gap-2">
          <span>Ln 1, Col 1</span>
          <span>UTF-8</span>
          <span>C</span>
        </div>
      </div>
    </footer>
  )
}
