"use client"

import * as React from "react"
import { useAtom, useAtomValue, useSetAtom } from "jotai"
import { motion, AnimatePresence } from "framer-motion"
import { cn } from "../../lib/utils"
import { Sidebar } from "./sidebar"
import { Header } from "./header"
import { StatusBar } from "./status-bar"
import { CommandMenu } from "../../components/ui/command"
import { OutlinePanel } from "../../components/panels/outline-panel"
import { NodeDetailPanel } from "../../components/panels/node-detail-panel"
import { FileExplorer } from "../../components/panels/file-explorer"
import { SearchPanel } from "../../components/panels/search-panel"
import { CodeEditor } from "../../components/panels/code-editor"
import { FlowView } from "../../components/panels/flow-view"
import { useAnalysisStore } from "../../store/analysisStore"
import {
  sidebarOpenAtom,
  bottomPanelOpenAtom,
  bottomPanelTabAtom,
  bottomPanelHeightAtom,
  setBottomPanelHeightAtom,
  leftPanelOpenAtom,
  leftPanelWidthAtom,
  setLeftPanelWidthAtom,
  rightPanelOpenAtom,
  rightPanelTabAtom,
  rightPanelWidthAtom,
  setRightPanelWidthAtom,
  commandMenuOpenAtom,
  currentFileAtom,
  viewModeAtom,
} from "../../lib/atoms/layout-atoms"
import { ResizableDivider } from "../ui/resizable-divider"
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

// CodeView 和 FlowView 组件已移至独立文件

// 终端日志条目
interface LogEntry {
  type: 'command' | 'info' | 'success' | 'error' | 'warning'
  message: string
  timestamp: Date
}

function TerminalPanel() {
  const currentProject = useAnalysisStore((state) => state.currentProject)
  const indexProgress = useAnalysisStore((state) => state.indexProgress)
  const executionFlow = useAnalysisStore((state) => state.executionFlow)
  const [logs, setLogs] = React.useState<LogEntry[]>([])
  
  // 监听项目状态变化，生成真实日志
  React.useEffect(() => {
    if (currentProject) {
      setLogs(prev => [
        ...prev,
        { type: 'command', message: `flowsight open "${currentProject.path}"`, timestamp: new Date() },
        { type: 'success', message: '项目加载成功', timestamp: new Date() },
        { type: 'info', message: `发现 ${currentProject.files_count} 个文件, ${currentProject.functions_count} 个函数, ${currentProject.structs_count} 个结构体`, timestamp: new Date() },
      ])
    }
  }, [currentProject?.path])
  
  // 监听索引进度
  React.useEffect(() => {
    if (indexProgress && indexProgress.phase && indexProgress.message) {
      const phase = indexProgress.phase
      const message = indexProgress.message
      
      if (phase === 'complete' || phase === 'done') {
        setLogs(prev => [...prev, { type: 'success', message, timestamp: new Date() }])
      } else if (phase === 'error') {
        setLogs(prev => [...prev, { type: 'error', message, timestamp: new Date() }])
      } else {
        setLogs(prev => [...prev, { type: 'info', message: `[${phase}] ${message}`, timestamp: new Date() }])
      }
    }
  }, [indexProgress])
  
  // 监听执行流分析
  React.useEffect(() => {
    if (executionFlow) {
      setLogs(prev => [
        ...prev,
        { type: 'command', message: `flowsight analyze "${executionFlow.entry_function}"`, timestamp: new Date() },
        { type: 'success', message: `分析完成: ${executionFlow.analysis_info?.total_nodes || 0} 个节点, ${executionFlow.analysis_info?.async_calls || 0} 个异步调用`, timestamp: new Date() },
      ])
    }
  }, [executionFlow])
  
  // 获取日志样式
  const getLogStyle = (type: LogEntry['type']) => {
    switch (type) {
      case 'command': return 'text-[var(--text-primary)]'
      case 'success': return 'text-[var(--success)]'
      case 'error': return 'text-[var(--error)]'
      case 'warning': return 'text-[var(--warning)]'
      default: return 'text-[var(--text-secondary)]'
    }
  }
  
  return (
    <div className="h-full w-full bg-[var(--bg-primary)] p-3 font-mono text-[13px] leading-relaxed">
      <div className="flex h-full flex-col">
        <div className="flex-1 overflow-y-auto space-y-1">
          {logs.length === 0 ? (
            <p className="text-[var(--text-muted)]">FlowSight 终端就绪。打开项目开始分析。</p>
          ) : (
            logs.map((log, i) => (
              <p key={i} className={getLogStyle(log.type)}>
                {log.type === 'command' ? (
                  <>
                    <span className="text-[var(--accent)]">$ </span>
                    {log.message}
                  </>
                ) : (
                  <>
                    <span className="text-[var(--text-muted)] text-[10px] mr-2">
                      {log.timestamp.toLocaleTimeString()}
                    </span>
                    {log.message}
                  </>
                )}
              </p>
            ))
          )}
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
  const bottomPanelHeight = useAtomValue(bottomPanelHeightAtom)
  const setBottomPanelHeight = useSetAtom(setBottomPanelHeightAtom)
  const [commandMenuOpen, setCommandMenuOpen] = useAtom(commandMenuOpenAtom)
  const [leftPanelOpen, setLeftPanelOpen] = useAtom(leftPanelOpenAtom)
  const leftPanelWidth = useAtomValue(leftPanelWidthAtom)
  const setLeftPanelWidth = useSetAtom(setLeftPanelWidthAtom)
  const [rightPanelOpen, setRightPanelOpen] = useAtom(rightPanelOpenAtom)
  const [rightPanelTab, setRightPanelTab] = useAtom(rightPanelTabAtom)
  const rightPanelWidth = useAtomValue(rightPanelWidthAtom)
  const setRightPanelWidth = useSetAtom(setRightPanelWidthAtom)
  
  // 当前打开的文件 (使用 Jotai 原子)
  const [currentFile, setCurrentFile] = useAtom(currentFileAtom)
  
  // 视图模式
  const viewMode = useAtomValue(viewModeAtom)
  
  // 处理文件选择
  const handleFileSelect = React.useCallback((path: string) => {
    console.log('选择文件:', path)
    setCurrentFile(path)
  }, [setCurrentFile])

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
      // Cmd+E or Ctrl+E - Toggle left panel (file explorer)
      if ((e.metaKey || e.ctrlKey) && e.key === "e") {
        e.preventDefault()
        setLeftPanelOpen(prev => !prev)
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [setSidebarOpen, setLeftPanelOpen, setRightPanelOpen, setBottomPanelOpen, setCommandMenuOpen])

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
      case "search":
        return <SearchPanel onResultSelect={(result) => handleFileSelect(result.file_path)} />
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
        {/* Left Sidebar (Icon Bar) */}
        <Sidebar />

        {/* Left Panel - File Explorer */}
        <AnimatePresence>
          {leftPanelOpen && (
            <motion.div
              className="flex h-full bg-[var(--bg-secondary)] overflow-hidden"
              initial={{ width: 0, opacity: 0 }}
              animate={{ width: leftPanelWidth, opacity: 1 }}
              exit={{ width: 0, opacity: 0 }}
              transition={{ type: "spring", damping: 25, stiffness: 300 }}
            >
              <div className="flex-1 flex flex-col border-r border-[var(--border-subtle)] overflow-hidden">
                <FileExplorer onFileSelect={handleFileSelect} />
              </div>
              {/* 左侧面板拖拽分隔条 */}
              <ResizableDivider
                direction="horizontal"
                onResize={(delta) => setLeftPanelWidth(leftPanelWidth + delta)}
              />
            </motion.div>
          )}
        </AnimatePresence>

        {/* Content Area */}
        <main id="main-content" className="flex flex-1 flex-col overflow-hidden">
          {/* View Area */}
          <div className="flex-1 overflow-hidden" data-view-mode={viewMode}>
            <AnimatePresence mode="wait">
              <motion.div
                key={viewMode}
                className="h-full w-full"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
              >
                {children || (
                  viewMode === "flow" 
                    ? <FlowView /> 
                    : <CodeEditor filePath={currentFile} onClose={() => setCurrentFile(null)} />
                )}
              </motion.div>
            </AnimatePresence>
          </div>

          {/* Bottom Panel */}
          <AnimatePresence>
            {bottomPanelOpen && (
              <motion.div
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: bottomPanelHeight, opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                transition={{ type: "spring", damping: 25, stiffness: 300 }}
                className="border-t border-[var(--border-subtle)] bg-[var(--bg-secondary)]"
              >
                <div className="flex h-full flex-col">
                  {/* Resize Handle */}
                  <ResizableDivider
                    direction="vertical"
                    onResize={(delta) => setBottomPanelHeight(bottomPanelHeight - delta)}
                  />
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
              className="flex h-full bg-[var(--bg-secondary)]"
              initial={{ width: 0 }}
              animate={{ width: rightPanelWidth }}
              exit={{ width: 0 }}
              transition={{ type: "spring", damping: 25, stiffness: 300 }}
            >
              {/* 右侧面板拖拽分隔条 */}
              <ResizableDivider
                direction="horizontal"
                onResize={(delta) => setRightPanelWidth(rightPanelWidth - delta)}
              />
              {/* Right Panel Tabs */}
              <div className="flex flex-col border-l border-r border-[var(--border-subtle)] bg-[var(--bg-tertiary)]">
                {[
                  { id: "outline", label: "大纲", icon: FileText, iconClass: "h-4 w-4" },
                  { id: "detail", label: "详情", icon: BarChart3, iconClass: "h-4 w-4" },
                  { id: "llvm-ir", label: "IR", icon: Cpu, iconClass: "h-4 w-4" },
                  { id: "search", label: "搜索", icon: Search, iconClass: "h-4 w-4" },
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
