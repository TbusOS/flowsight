"use client"

import * as React from "react"
import { useAtom, useAtomValue, useSetAtom } from "jotai"
import { motion, AnimatePresence } from "framer-motion"
import { cn } from "../../lib/utils"
import { Sidebar } from "./sidebar"
import { Header } from "./header"
import { StatusBar } from "./status-bar"
import { OutlinePanel } from "../../components/panels/outline-panel"
import { NodeDetailPanel } from "../../components/panels/node-detail-panel"
import { FileExplorer } from "../../components/panels/file-explorer"
import { useAnalysisStore } from "../../store/analysisStore"
import { ErrorBoundary, LocalErrorBoundary } from "../ErrorBoundary/ErrorBoundary"
import { FlowViewSkeleton, EditorSkeleton, SearchResultSkeleton } from "../Skeleton/Skeleton"
import { LlvmIrPanel, LlvmIrParseResult } from "../LlvmIrPanel"
import { invoke } from "../../lib/tauri-api"

// 代码分割 - 懒加载重量级组件
const CommandMenu = React.lazy(() => import("../../components/ui/command").then(m => ({ default: m.CommandMenu })))
const SearchPanel = React.lazy(() => import("../../components/panels/search-panel").then(m => ({ default: m.SearchPanel })))
const CodeEditor = React.lazy(() => import("../../components/panels/code-editor").then(m => ({ default: m.CodeEditor })))
const FlowView = React.lazy(() => import("../../components/panels/flow-view").then(m => ({ default: m.FlowView })))
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
  const [lastProjectPath, setLastProjectPath] = React.useState<string | null>(null)
  const lastPhaseRef = React.useRef<string | null>(null)
  
  // 翻译索引阶段消息
  const translatePhase = (phase: string, message: string): string => {
    const phaseMap: Record<string, string> = {
      'scanning': '扫描中',
      'parsing': '解析中',
      'indexing': '索引中',
      'done': '完成',
      'complete': '完成',
      'error': '错误',
    }
    const phaseLabel = phaseMap[phase] || phase
    
    // 翻译常见英文消息
    let translatedMessage = message
      .replace('Scanning files...', '正在扫描文件...')
      .replace(/Found (\d+) files\.\.\./, '发现 $1 个文件...')
      .replace(/Parsing (\d+) files\.\.\./, '正在解析 $1 个文件...')
      .replace('Building index...', '正在构建索引...')
      .replace(/Indexed (\d+)\/(\d+)/, '已索引 $1/$2')
      .replace(/Done! (\d+) files, (\d+) functions/, '完成! $1 个文件, $2 个函数')
    
    return `[${phaseLabel}] ${translatedMessage}`
  }
  
  // 监听项目路径变化 - 仅在打开新项目时添加打开日志
  React.useEffect(() => {
    if (currentProject && currentProject.path !== lastProjectPath) {
      setLastProjectPath(currentProject.path)
      lastPhaseRef.current = null  // 重置阶段追踪
      setLogs(prev => [
        ...prev,
        { type: 'command', message: `flowsight open "${currentProject.path}"`, timestamp: new Date() },
      ])
    }
  }, [currentProject?.path, lastProjectPath])
  
  // 监听索引完成 - 更新统计信息
  React.useEffect(() => {
    if (currentProject?.indexed && currentProject.files_count > 0) {
      setLogs(prev => {
        // 避免重复添加相同的统计日志
        const hasStats = prev.some(log => log.message?.includes(`发现 ${currentProject.files_count} 个文件`))
        if (hasStats) {
          return prev
        }
        return [
          ...prev,
          { type: 'success', message: '项目加载成功', timestamp: new Date() },
          { type: 'info', message: `发现 ${currentProject.files_count} 个文件, ${currentProject.functions_count} 个函数, ${currentProject.structs_count} 个结构体`, timestamp: new Date() },
        ]
      })
    }
  }, [currentProject?.indexed, currentProject?.files_count, currentProject?.functions_count, currentProject?.structs_count])
  
  // 监听索引进度 - 改进：避免重复日志，只在阶段变化时添加
  React.useEffect(() => {
    if (indexProgress && indexProgress.phase && indexProgress.message) {
      const phase = indexProgress.phase
      const message = indexProgress.message
      
      // 避免重复添加相同阶段的日志（除了 done/complete）
      if (phase !== 'done' && phase !== 'complete' && phase === lastPhaseRef.current) {
        // 更新同一阶段的最新消息（替换而不是追加）
        setLogs(prev => {
          const newLogs = [...prev]
          // 找到最后一个相同阶段的日志并更新
          for (let i = newLogs.length - 1; i >= 0; i--) {
            if (newLogs[i].message?.startsWith(`[${phase === 'scanning' ? '扫描中' : phase === 'parsing' ? '解析中' : phase === 'indexing' ? '索引中' : phase}]`)) {
              newLogs[i] = { ...newLogs[i], message: translatePhase(phase, message), timestamp: new Date() }
              return newLogs
            }
          }
          // 如果没找到，添加新日志
          return [...prev, { type: 'info', message: translatePhase(phase, message), timestamp: new Date() }]
        })
        return
      }
      
      lastPhaseRef.current = phase
      
      if (phase === 'complete' || phase === 'done') {
        setLogs(prev => [...prev, { type: 'success', message: translatePhase(phase, message), timestamp: new Date() }])
      } else if (phase === 'error') {
        setLogs(prev => [...prev, { type: 'error', message: translatePhase(phase, message), timestamp: new Date() }])
      } else {
        setLogs(prev => [...prev, { type: 'info', message: translatePhase(phase, message), timestamp: new Date() }])
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
  const [viewMode, setViewMode] = useAtom(viewModeAtom)
  
  // LLVM IR 状态
  const [llvmIrResult, setLlvmIrResult] = React.useState<LlvmIrParseResult | null>(null)
  const [llvmIrLoading, setLlvmIrLoading] = React.useState(false)
  const [llvmIrError, setLlvmIrError] = React.useState<string | null>(null)
  const [selectedLlvmFunction, setSelectedLlvmFunction] = React.useState<string | null>(null)
  
  // 生成 LLVM IR
  const generateLlvmIr = React.useCallback(async (filePath: string) => {
    // 只处理 C 文件
    if (!filePath.endsWith('.c') && !filePath.endsWith('.h')) {
      setLlvmIrResult(null)
      setLlvmIrError(null)
      return
    }
    
    setLlvmIrLoading(true)
    setLlvmIrError(null)
    
    try {
      const result = await invoke<{
        module_name: string
        functions: Record<string, {
          name: string
          return_type: string
          parameters: Array<{ name: string; type_str: string }>
          blocks: Array<{
            name: string
            instructions: Array<{
              opcode: string
              dest?: string
              type_str: string
              operands: string[]
              location?: { file: string; line: number }
            }>
            predecessors: string[]
            successors: string[]
            terminator?: {
              opcode: string
              dest?: string
              type_str: string
              operands: string[]
              location?: { file: string; line: number }
            }
          }>
          is_callback: boolean
          callback_context?: string
        }>
      }>('generate_llvm_ir', { filePath })
      
      // 转换为前端格式
      const parseResult: LlvmIrParseResult = {
        moduleName: result.module_name,
        functions: Object.fromEntries(
          Object.entries(result.functions).map(([name, func]) => [
            name,
            {
              name: func.name,
              returnType: func.return_type,
              parameters: func.parameters.map(p => ({
                name: p.name,
                typeStr: p.type_str,
              })),
              blocks: func.blocks.map(b => ({
                name: b.name,
                instructions: b.instructions.map(i => ({
                  opcode: i.opcode,
                  dest: i.dest,
                  typeStr: i.type_str,
                  operands: i.operands,
                  location: i.location,
                })),
                predecessors: b.predecessors,
                successors: b.successors,
                terminator: b.terminator ? {
                  opcode: b.terminator.opcode,
                  dest: b.terminator.dest,
                  typeStr: b.terminator.type_str,
                  operands: b.terminator.operands,
                  location: b.terminator.location,
                } : undefined,
              })),
              isCallback: func.is_callback,
              callbackContext: func.callback_context,
            }
          ])
        ),
      }
      
      setLlvmIrResult(parseResult)
      
      // 自动选择第一个函数
      const funcNames = Object.keys(parseResult.functions)
      if (funcNames.length > 0 && !funcNames.includes('__compilation_error__')) {
        setSelectedLlvmFunction(funcNames[0])
      }
    } catch (error) {
      console.error('生成 LLVM IR 失败:', error)
      setLlvmIrError(String(error))
    } finally {
      setLlvmIrLoading(false)
    }
  }, [])
  
  // 处理文件选择
  const handleFileSelect = React.useCallback((path: string) => {
    console.log('选择文件:', path)
    setCurrentFile(path)
    // 触发 LLVM IR 生成
    generateLlvmIr(path)
  }, [setCurrentFile, generateLlvmIr])

  // Keyboard shortcuts
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+K or Ctrl+K - Open command palette
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault()
        setCommandMenuOpen(true)
      }
      // F1 - Also open command palette
      if (e.key === "F1") {
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
      // Cmd+1 - Switch to code view
      if ((e.metaKey || e.ctrlKey) && e.key === "1") {
        e.preventDefault()
        setViewMode("code")
      }
      // Cmd+2 - Switch to flow view
      if ((e.metaKey || e.ctrlKey) && e.key === "2") {
        e.preventDefault()
        setViewMode("flow")
      }
      // Cmd+3 - Switch to split view
      if ((e.metaKey || e.ctrlKey) && e.key === "3") {
        e.preventDefault()
        setViewMode("split")
      }
      // Cmd+Shift+O - Open outline panel
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === "o") {
        e.preventDefault()
        setRightPanelOpen(true)
        setRightPanelTab("outline")
      }
      // Cmd+Shift+D - Open detail panel
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === "d") {
        e.preventDefault()
        setRightPanelOpen(true)
        setRightPanelTab("detail")
      }
      // Cmd+` - Toggle terminal (alternative)
      if ((e.metaKey || e.ctrlKey) && e.key === "`") {
        e.preventDefault()
        setBottomPanelOpen(prev => !prev)
        if (!bottomPanelOpen) {
          setBottomPanelTab("terminal")
        }
      }
      // Cmd+W - Close current file
      if ((e.metaKey || e.ctrlKey) && e.key === "w" && !e.shiftKey) {
        e.preventDefault()
        if (currentFile) {
          setCurrentFile(null)
        }
      }
      // Cmd+Shift+F - Open search panel
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === "f") {
        e.preventDefault()
        setLeftPanelOpen(true)
        // Focus search input
      }
      // Escape - Close panels/dialogs
      if (e.key === "Escape") {
        if (commandMenuOpen) {
          setCommandMenuOpen(false)
        }
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [setSidebarOpen, setLeftPanelOpen, setRightPanelOpen, setBottomPanelOpen, setBottomPanelTab, bottomPanelOpen, setCommandMenuOpen, commandMenuOpen, setViewMode, setRightPanelTab, currentFile, setCurrentFile])

  // Render right panel based on tab
  const renderRightPanel = () => {
    switch (rightPanelTab) {
      case "outline":
        return <OutlinePanel />
      case "detail":
        return <NodeDetailPanel />
      case "llvm-ir":
        return (
          <div className="h-full overflow-hidden">
            <LlvmIrPanel
              parseResult={llvmIrResult}
              selectedFunction={selectedLlvmFunction}
              loading={llvmIrLoading}
              error={llvmIrError}
              onFunctionSelect={setSelectedLlvmFunction}
            />
          </div>
        )
      case "search":
        return (
          <React.Suspense fallback={<SearchResultSkeleton />}>
            <LocalErrorBoundary name="搜索面板">
              <SearchPanel onResultSelect={(result) => handleFileSelect(result.file_path)} />
            </LocalErrorBoundary>
          </React.Suspense>
        )
      default:
        return <OutlinePanel />
    }
  }

  return (
    <div className="flex h-screen w-full flex-col bg-[var(--bg-primary)] text-[var(--text-primary)]">
      {/* Command Menu */}
      <React.Suspense fallback={null}>
        <CommandMenu open={commandMenuOpen} onOpenChange={setCommandMenuOpen} />
      </React.Suspense>

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
                    ? (
                      <React.Suspense fallback={<FlowViewSkeleton />}>
                        <LocalErrorBoundary name="执行流视图">
                          <FlowView />
                        </LocalErrorBoundary>
                      </React.Suspense>
                    )
                    : (
                      <React.Suspense fallback={<EditorSkeleton />}>
                        <LocalErrorBoundary name="代码编辑器">
                          <CodeEditor filePath={currentFile} onClose={() => setCurrentFile(null)} />
                        </LocalErrorBoundary>
                      </React.Suspense>
                    )
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
