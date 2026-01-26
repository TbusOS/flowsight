"use client"

import * as React from "react"
import { useAtom, useAtomValue } from "jotai"
import { motion, AnimatePresence } from "framer-motion"
import { cn } from "../../lib/utils"
import { Sidebar } from "./sidebar"
import { Header } from "./header"
import { StatusBar } from "./status-bar"
import { CommandMenu } from "../../components/ui/command"
import { sidebarOpenAtom, bottomPanelOpenAtom, bottomPanelTabAtom } from "../../lib/atoms/layout-atoms"

// Placeholder components for different views
function CodeView() {
  return (
    <div className="flex h-full w-full items-center justify-center bg-[var(--bg-primary)]">
      <div className="text-center">
        <div className="mb-4 inline-flex h-12 w-12 items-center justify-center rounded-xl bg-[var(--accent)]/10">
          <svg className="h-6 w-6 text-[var(--accent)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
          </svg>
        </div>
        <h3 className="mb-2 text-sm font-medium text-[var(--text-primary)]">代码编辑器</h3>
        <p className="text-xs text-[var(--text-muted)]">选择一个文件开始编辑</p>
      </div>
    </div>
  )
}

function FlowView() {
  return (
    <div className="flex h-full w-full items-center justify-center bg-[var(--bg-primary)]">
      <div className="text-center">
        <div className="mb-4 inline-flex h-12 w-12 items-center justify-center rounded-xl bg-[var(--accent)]/10">
          <svg className="h-6 w-6 text-[var(--accent)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
        </div>
        <h3 className="mb-2 text-sm font-medium text-[var(--text-primary)]">执行流视图</h3>
        <p className="text-xs text-[var(--text-muted)]">加载代码后显示执行流程</p>
      </div>
    </div>
  )
}

function TerminalPanel() {
  return (
    <div className="h-full w-full bg-[var(--bg-primary)] p-2 font-mono text-xs">
      <div className="flex h-full flex-col">
        <div className="flex-1 overflow-y-auto">
          <p className="text-[var(--text-muted)]">$ flow analyze --project demo</p>
          <p className="mt-1 text-[var(--success)]">Loading project...</p>
          <p className="mt-1 text-[var(--text-secondary)]">Found 42 functions, 156 calls</p>
          <p className="mt-1 text-[var(--success)]">Analysis complete in 1.2s</p>
        </div>
        <div className="mt-2 flex items-center gap-2">
          <span className="text-[var(--accent)]">$</span>
          <span className="animate-pulse">_</span>
        </div>
      </div>
    </div>
  )
}

interface MainLayoutProps {
  children?: React.ReactNode
}

export function MainLayout({ children }: MainLayoutProps) {
  const [sidebarOpen] = useAtom(sidebarOpenAtom)
  const [bottomPanelOpen, setBottomPanelOpen] = useAtom(bottomPanelOpenAtom)
  const [bottomPanelTab, setBottomPanelTab] = useAtom(bottomPanelTabAtom)
  const [commandMenuOpen, setCommandMenuOpen] = useAtom(sidebarOpenAtom)  // Just for demo

  // Update command menu open state
  React.useEffect(() => {
    setCommandMenuOpen(sidebarOpen)
  }, [sidebarOpen, setCommandMenuOpen])

  return (
    <div className="flex h-screen w-full flex-col bg-[var(--bg-primary)] text-[var(--text-primary)]">
      {/* Command Menu */}
      <CommandMenu open={commandMenuOpen} onOpenChange={setCommandMenuOpen} />

      {/* Header */}
      <Header />

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left Sidebar */}
        <Sidebar />

        {/* Content Area */}
        <main className="flex flex-1 flex-col overflow-hidden">
          {/* View Area */}
          <div className="flex-1 overflow-hidden">
            <AnimatePresence mode="wait">
              <motion.div
                key="content"
                className="h-full w-full"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
              >
                {children || <CodeView />}
              </motion.div>
            </AnimatePresence>
          </div>

          {/* Bottom Panel */}
          <AnimatePresence>
            {bottomPanelOpen && (
              <motion.div
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: 200, opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                transition={{ type: "spring", damping: 25, stiffness: 300 }}
                className="border-t border-[var(--border-subtle)] bg-[var(--bg-secondary)]"
              >
                <div className="flex h-full flex-col">
                  {/* Resize Handle */}
                  <div
                    className="h-1 cursor-row-resize hover:bg-[var(--accent)]/20"
                    onMouseDown={(e) => {
                      e.preventDefault()
                      const startY = e.clientY
                      const startHeight = 200

                      const handleMouseMove = (moveEvent: MouseEvent) => {
                        const diff = startY - moveEvent.y
                        const newHeight = Math.max(100, Math.min(400, startHeight + diff))
                        // Update via CSS or state
                        const panel = document.getElementById("bottom-panel")
                        if (panel) {
                          panel.style.height = `${newHeight}px`
                        }
                      }

                      const handleMouseUp = () => {
                        document.removeEventListener("mousemove", handleMouseMove)
                        document.removeEventListener("mouseup", handleMouseUp)
                      }

                      document.addEventListener("mousemove", handleMouseMove)
                      document.addEventListener("mouseup", handleMouseUp)
                    }}
                  />
                  <div id="bottom-panel" className="flex-1 overflow-hidden">
                    {bottomPanelTab === "terminal" && <TerminalPanel />}
                  </div>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </main>
      </div>

      {/* Status Bar */}
      <StatusBar />
    </div>
  )
}
