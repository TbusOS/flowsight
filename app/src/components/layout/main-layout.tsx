"use client"

import * as React from "react"
import { useAtom, useAtomValue } from "jotai"
import { motion, AnimatePresence } from "framer-motion"
import { cn } from "../../lib/utils"
import { Sidebar } from "./sidebar"
import { Header } from "./header"
import { StatusBar } from "./status-bar"
import { CommandMenu } from "../../components/ui/command"
import { OutlinePanel } from "../../components/panels/outline-panel"
import { NodeDetailPanel } from "../../components/panels/node-detail-panel"
import { FileExplorer } from "../../components/panels/file-explorer"
import {
  sidebarOpenAtom,
  bottomPanelOpenAtom,
  bottomPanelTabAtom,
  rightPanelOpenAtom,
  rightPanelTabAtom,
  rightPanelWidthAtom,
  commandMenuOpenAtom,
} from "../../lib/atoms/layout-atoms"
import {
  LayoutDashboard,
  FolderOpen,
  FileCode,
  Zap,
  Search,
  Command,
  Settings,
  Sparkles,
  FileText,
  BarChart3,
  Cpu,
  Folder,
} from "lucide-react"

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
    <div className="h-full w-full bg-[var(--bg-primary)] p-3 font-mono text-[13px] leading-relaxed">
      <div className="flex h-full flex-col">
        <div className="flex-1 overflow-y-auto space-y-1">
          {/* 命令行 */}
          <p>
            <span className="text-[var(--accent)]">$ </span>
            <span className="text-[var(--text-primary)]">flow analyze --project demo</span>
          </p>
          {/* 输出信息 */}
          <p className="text-[var(--success)]">Loading project...</p>
          <p className="text-[var(--text-secondary)]">Found 42 functions, 156 calls</p>
          <p className="text-[var(--success)]">Analysis complete in 1.2s</p>
        </div>
        {/* 输入提示符 */}
        <div className="mt-3 flex items-center">
          <span className="text-[var(--accent)]">$ </span>
          <span className="ml-1 inline-block w-2 h-4 bg-[var(--text-primary)] animate-pulse opacity-80" />
        </div>
      </div>
    </div>
  )
}

interface MainLayoutProps {
  children?: React.ReactNode
}

export function MainLayout({ children }: MainLayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useAtom(sidebarOpenAtom)
  const [bottomPanelOpen, setBottomPanelOpen] = useAtom(bottomPanelOpenAtom)
  const [bottomPanelTab, setBottomPanelTab] = useAtom(bottomPanelTabAtom)
  const [commandMenuOpen, setCommandMenuOpen] = useAtom(commandMenuOpenAtom)
  const [rightPanelOpen, setRightPanelOpen] = useAtom(rightPanelOpenAtom)
  const [rightPanelTab, setRightPanelTab] = useAtom(rightPanelTabAtom)
  const rightPanelWidth = useAtomValue(rightPanelWidthAtom)
  
  // 当前打开的文件
  const [currentFile, setCurrentFile] = React.useState<string | null>(null)
  
  // 处理文件选择
  const handleFileSelect = React.useCallback((path: string) => {
    console.log('选择文件:', path)
    setCurrentFile(path)
    // TODO: 在代码编辑器中打开文件
  }, [])

  // Keyboard shortcuts
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+K or Ctrl+K - Open command palette
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault()
        setCommandMenuOpen(true)
      }
      // Cmd+B or Ctrl+B - Toggle sidebar
      if ((e.metaKey || e.ctrlKey) && e.key === "b") {
        e.preventDefault()
        setSidebarOpen(prev => !prev)
      }
      // Cmd+J or Ctrl+J - Toggle bottom panel
      if ((e.metaKey || e.ctrlKey) && e.key === "j") {
        e.preventDefault()
        setBottomPanelOpen(prev => !prev)
      }
      // Cmd+\ or Ctrl+\ - Toggle right panel
      if ((e.metaKey || e.ctrlKey) && e.key === "\\") {
        e.preventDefault()
        setRightPanelOpen(prev => !prev)
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [setSidebarOpen, setRightPanelOpen, setBottomPanelOpen, setCommandMenuOpen])

  // Render right panel based on tab
  const renderRightPanel = () => {
    switch (rightPanelTab) {
      case "outline":
        return <OutlinePanel />
      case "detail":
        return <NodeDetailPanel />
      case "llvm-ir":
        return (
          <div className="flex items-center justify-center h-full text-[var(--text-muted)] text-sm">
            LLVM IR 面板
          </div>
        )
      case "explorer":
        return <FileExplorer onFileSelect={handleFileSelect} />
      default:
        return <OutlinePanel />
    }
  }

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
        <main id="main-content" className="flex flex-1 flex-col overflow-hidden">
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
                    className="h-1.5 cursor-row-resize hover:bg-[var(--accent)]/30 active:bg-[var(--accent)]/50 flex items-center justify-center transition-colors duration-150"
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
                  >
                    <div className="h-0.5 w-8 rounded-full bg-[var(--text-muted)] group-hover:bg-[var(--accent)] transition-colors" />
                  </div>
                  <div id="bottom-panel" className="flex-1 overflow-hidden">
                    {bottomPanelTab === "terminal" && <TerminalPanel />}
                  </div>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </main>

        {/* Right Panel */}
        <AnimatePresence>
          {rightPanelOpen && (
            <motion.div
              className="flex h-full border-l border-[var(--border-subtle)] bg-[var(--bg-secondary)]"
              initial={{ width: 0 }}
              animate={{ width: rightPanelWidth }}
              exit={{ width: 0 }}
              transition={{ type: "spring", damping: 25, stiffness: 300 }}
            >
              {/* Right Panel Tabs */}
              <div className="flex flex-col border-r border-[var(--border-subtle)] bg-[var(--bg-tertiary)]">
                {[
                  { id: "outline", label: "大纲", icon: FileText, iconClass: "h-4 w-4" },
                  { id: "detail", label: "详情", icon: BarChart3, iconClass: "h-4 w-4" },
                  { id: "llvm-ir", label: "IR", icon: Cpu, iconClass: "h-4 w-4" },
                  { id: "explorer", label: "文件", icon: Folder, iconClass: "h-4 w-4" },
                ].map((tab) => {
                  const Icon = tab.icon
                  return (
                    <button
                      key={tab.id}
                      className={cn(
                        "flex flex-col items-center gap-1 px-3 py-2 text-[10px] transition-colors cursor-pointer",
                        rightPanelTab === tab.id
                          ? "text-[var(--accent)] bg-[var(--bg-secondary)]"
                          : "text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
                      )}
                      onClick={() => {
                        setRightPanelTab(tab.id as typeof rightPanelTab)
                        setRightPanelOpen(true)
                      }}
                    >
                      <Icon className={tab.iconClass} />
                      <span>{tab.label}</span>
                    </button>
                  )
                })}
              </div>

              {/* Panel Content */}
              <div className="flex-1 overflow-hidden">
                {renderRightPanel()}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Status Bar */}
      <StatusBar />
    </div>
  )
}
