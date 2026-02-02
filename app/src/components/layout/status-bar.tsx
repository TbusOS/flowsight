"use client"

import * as React from "react"
import { useAtomValue, useSetAtom } from "jotai"
import {
  Terminal,
  AlertCircle,
  Info,
  Loader2,
  FolderOpen,
  FileCode,
  Cpu,
  Zap,
  GitBranch,
  Check,
  Clock,
  MousePointerClick,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { 
  bottomPanelOpenAtom, 
  bottomPanelTabAtom, 
  currentFileAtom,
  cursorPositionAtom,
  viewModeAtom,
  selectedEntryFunctionAtom,
} from "../../lib/atoms/layout-atoms"
import { useAnalysisStore } from "../../store/analysisStore"

interface StatusBarProps {
  className?: string
}

// 根据文件扩展名获取语言
function getLanguageFromPath(path: string | null): string {
  if (!path) return '-'
  const ext = path.split('.').pop()?.toLowerCase()
  switch (ext) {
    case 'c': return 'C'
    case 'h': return 'C Header'
    case 'cpp': case 'cc': case 'cxx': return 'C++'
    case 'hpp': case 'hxx': return 'C++ Header'
    case 'rs': return 'Rust'
    case 'ts': return 'TypeScript'
    case 'tsx': return 'TSX'
    case 'js': return 'JavaScript'
    case 'jsx': return 'JSX'
    case 'py': return 'Python'
    case 'json': return 'JSON'
    case 'yaml': case 'yml': return 'YAML'
    case 'md': return 'Markdown'
    case 'txt': return 'Text'
    case 'S': case 's': return 'Assembly'
    case 'ld': return 'Linker Script'
    case 'dts': case 'dtsi': return 'Device Tree'
    default: return ext?.toUpperCase() || '-'
  }
}

// 状态项组件
const StatusItem = React.memo(function StatusItem({ 
  children, 
  className,
  title,
  onClick,
}: { 
  children: React.ReactNode
  className?: string
  title?: string
  onClick?: () => void
}) {
  return (
    <div 
      className={cn(
        "flex items-center gap-1 text-[11px]",
        onClick && "cursor-pointer hover:text-[var(--text-primary)]",
        className
      )}
      title={title}
      onClick={onClick}
    >
      {children}
    </div>
  )
})

// 分隔符
const Separator = () => (
  <div className="w-px h-3.5 bg-[var(--border-subtle)]" />
)

export const StatusBar = React.memo(function StatusBar({ className }: StatusBarProps) {
  const bottomPanelOpen = useAtomValue(bottomPanelOpenAtom)
  const bottomPanelTab = useAtomValue(bottomPanelTabAtom)
  const currentFile = useAtomValue(currentFileAtom)
  const cursorPosition = useAtomValue(cursorPositionAtom)
  const viewMode = useAtomValue(viewModeAtom)
  const entryFunction = useAtomValue(selectedEntryFunctionAtom)
  
  // 从 store 获取索引进度和项目信息
  const indexProgress = useAnalysisStore((state) => state.indexProgress)
  const currentProject = useAnalysisStore((state) => state.currentProject)
  const isAnalyzing = useAnalysisStore((state) => state.executionFlowLoading)
  
  // 语言类型
  const language = getLanguageFromPath(currentFile)

  // 位置信息格式化
  const positionText = React.useMemo(() => {
    if (cursorPosition.selection) {
      const { startLine, endLine } = cursorPosition.selection
      if (startLine !== endLine) {
        return `Ln ${startLine}-${endLine}`
      }
    }
    return `Ln ${cursorPosition.line}, Col ${cursorPosition.column}`
  }, [cursorPosition])

  // 视图模式文本
  const viewModeText = React.useMemo(() => {
    switch (viewMode) {
      case 'code': return '代码'
      case 'flow': return '执行流'
      case 'split': return '分屏'
      default: return viewMode
    }
  }, [viewMode])

  const tabs = [
    { id: "terminal", icon: Terminal, label: "终端" },
    { id: "problems", icon: AlertCircle, label: "问题" },
    { id: "output", icon: Info, label: "输出" },
  ] as const

  return (
    <footer
      className={cn(
        "flex h-6 items-center justify-between border-t border-[var(--border-subtle)] bg-[var(--bg-secondary)] px-2 text-[11px] text-[var(--text-muted)]",
        className
      )}
    >
      {/* Left: Panel Tabs */}
      <div className="flex items-center gap-0.5">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={cn(
              "flex items-center gap-1 rounded px-2 py-1 min-h-[20px] transition-colors",
              bottomPanelOpen && bottomPanelTab === tab.id
                ? "bg-[var(--bg-tertiary)] text-[var(--text-primary)]"
                : "hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-secondary)]"
            )}
          >
            <tab.icon className="h-3 w-3" />
            <span>{tab.label}</span>
          </button>
        ))}
        
        {/* Index Progress */}
        {indexProgress && indexProgress.phase !== 'complete' && (
          <>
            <Separator />
            <StatusItem className="text-[var(--accent)]">
              <Loader2 className="h-3 w-3 animate-spin" />
              <span>{indexProgress.message}</span>
              {indexProgress.total > 0 && (
                <span className="text-[var(--text-muted)] tabular-nums">
                  ({indexProgress.current}/{indexProgress.total})
                </span>
              )}
            </StatusItem>
          </>
        )}
        
        {/* Analysis Status */}
        {isAnalyzing && (
          <>
            <Separator />
            <StatusItem className="text-amber-400">
              <Cpu className="h-3 w-3 animate-pulse" />
              <span>分析中...</span>
            </StatusItem>
          </>
        )}
        
        {/* Entry Function */}
        {entryFunction && !isAnalyzing && (
          <>
            <Separator />
            <StatusItem className="text-emerald-400" title={`入口函数: ${entryFunction}`}>
              <Zap className="h-3 w-3" />
              <span className="max-w-[100px] truncate">{entryFunction}</span>
            </StatusItem>
          </>
        )}
      </div>

      {/* Right: Status Info */}
      <div className="flex items-center gap-3">
        {/* Current Project */}
        {currentProject && (
          <StatusItem title={currentProject.path}>
            <FolderOpen className="h-3 w-3" />
            <span className="max-w-[120px] truncate">
              {currentProject.path.split('/').pop()}
            </span>
            {currentProject.indexed && (
              <span className="text-emerald-400 tabular-nums">
                <Check className="h-2.5 w-2.5 inline" />
                {currentProject.functions_count}
              </span>
            )}
          </StatusItem>
        )}
        
        {currentProject && currentFile && <Separator />}
        
        {/* Current File */}
        {currentFile && (
          <StatusItem title={currentFile}>
            <FileCode className="h-3 w-3" />
            <span className="max-w-[100px] truncate">
              {currentFile.split('/').pop()}
            </span>
          </StatusItem>
        )}

        {currentFile && <Separator />}

        {/* Position Info */}
        {currentFile && (
          <StatusItem className="tabular-nums">
            <MousePointerClick className="h-3 w-3" />
            <span>{positionText}</span>
          </StatusItem>
        )}
        
        {currentFile && <Separator />}
        
        {/* Language & Encoding */}
        {currentFile && (
          <>
            <StatusItem>
              <span>UTF-8</span>
            </StatusItem>
            <Separator />
            <StatusItem className="text-[var(--text-secondary)]">
              <span>{language}</span>
            </StatusItem>
          </>
        )}
        
        {/* View Mode */}
        <Separator />
        <StatusItem className="text-[var(--accent)]">
          <span>{viewModeText}</span>
        </StatusItem>
        
        {/* 无文件打开时的提示 */}
        {!currentFile && !currentProject && (
          <StatusItem className="text-[var(--text-muted)]">
            <span>⌘K 打开命令面板</span>
          </StatusItem>
        )}
      </div>
    </footer>
  )
})
