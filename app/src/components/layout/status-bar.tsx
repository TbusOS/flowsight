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
  Loader2,
  FolderOpen,
  FileCode,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { bottomPanelOpenAtom, bottomPanelTabAtom, currentFileAtom } from "../../lib/atoms/layout-atoms"
import { useAnalysisStore } from "../../store/analysisStore"
import { Button } from "../ui/button"

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
    default: return ext?.toUpperCase() || '-'
  }
}

export function StatusBar({ className }: StatusBarProps) {
  const bottomPanelOpen = useAtomValue(bottomPanelOpenAtom)
  const bottomPanelTab = useAtomValue(bottomPanelTabAtom)
  const currentFile = useAtomValue(currentFileAtom)
  
  // 从 store 获取索引进度和项目信息
  const indexProgress = useAnalysisStore((state) => state.indexProgress)
  const currentProject = useAnalysisStore((state) => state.currentProject)
  
  // 编辑器状态（未来可以从 Monaco 编辑器获取）
  const [editorState, setEditorState] = React.useState({
    line: 1,
    column: 1,
    encoding: 'UTF-8',
  })
  
  // 语言类型
  const language = getLanguageFromPath(currentFile)

  const tabs = [
    { id: "terminal", icon: Terminal, label: "终端" },
    { id: "problems", icon: AlertCircle, label: "问题" },
    { id: "output", icon: Info, label: "输出" },
  ] as const

  return (
    <footer
      className={cn(
        "flex h-7 items-center justify-between border-t border-[var(--border-subtle)] bg-[var(--bg-secondary)] px-3 text-xs text-[var(--text-muted)]",
        className
      )}
    >
      {/* Left: Panel Tabs - WCAG 触摸目标至少 24px */}
      <div className="flex items-center gap-1">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={cn(
              "flex items-center gap-1.5 rounded px-3 py-1.5 min-h-[24px] transition-colors",
              bottomPanelOpen && bottomPanelTab === tab.id
                ? "bg-[var(--bg-tertiary)] text-[var(--text-primary)]"
                : "hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-secondary)]"
            )}
          >
            <tab.icon className="h-3.5 w-3.5" />
            <span className="text-[12px]">{tab.label}</span>
          </button>
        ))}
      </div>

      {/* Center: Index Progress */}
      {indexProgress && indexProgress.phase !== 'complete' && (
        <div className="flex items-center gap-2 text-[var(--accent)]">
          <Loader2 className="h-3 w-3 animate-spin" />
          <span>{indexProgress.message}</span>
          {indexProgress.total > 0 && (
            <span className="text-[var(--text-muted)]">
              ({indexProgress.current}/{indexProgress.total})
            </span>
          )}
        </div>
      )}

      {/* Right: Status Info */}
      <div className="flex items-center gap-4">
        {/* Current Project */}
        {currentProject && (
          <div className="flex items-center gap-1 text-[var(--text-secondary)]">
            <FolderOpen className="h-3 w-3" />
            <span className="max-w-[150px] truncate">
              {currentProject.path.split('/').pop()}
            </span>
            {currentProject.indexed && (
              <span className="text-[var(--text-muted)]">
                ({currentProject.functions_count} 函数)
              </span>
            )}
          </div>
        )}
        
        {/* Current File Info */}
        {currentFile && (
          <div className="flex items-center gap-1 text-[var(--text-secondary)]">
            <FileCode className="h-3 w-3" />
            <span className="max-w-[100px] truncate">
              {currentFile.split('/').pop()}
            </span>
          </div>
        )}

        {/* Position Info - 只在有文件打开时显示 */}
        {currentFile && (
          <div className="flex items-center gap-2">
            <span>Ln {editorState.line}, Col {editorState.column}</span>
            <span>{editorState.encoding}</span>
            <span>{language}</span>
          </div>
        )}
        
        {/* 无文件打开时的提示 */}
        {!currentFile && !currentProject && (
          <span className="text-[var(--text-muted)]">打开项目开始</span>
        )}
      </div>
    </footer>
  )
}
