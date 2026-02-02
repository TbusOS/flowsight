"use client"

import * as React from "react"
import { invoke, openDialog } from "../../lib/tauri-api"
import {
  Folder,
  Loader2,
  RefreshCw,
  FolderPlus,
  Search,
  X,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { useAnalysisStore } from "../../store/analysisStore"
import { useSetAtom } from "jotai"
import { currentFileAtom } from "../../lib/atoms/layout-atoms"
import { VirtualFileTree, FileNode } from "../VirtualList/VirtualFileTree"
import { useDebounce } from "../../hooks/usePerformance"

interface FileExplorerProps {
  className?: string
  onFileSelect?: (path: string) => void
}

// 使用 React.memo 优化
export const FileExplorer = React.memo(function FileExplorer({ 
  className, 
  onFileSelect 
}: FileExplorerProps) {
  const currentProject = useAnalysisStore((state) => state.currentProject)
  const openProject = useAnalysisStore((state) => state.openProject)
  const setCurrentFile = useSetAtom(currentFileAtom)
  const [files, setFiles] = React.useState<FileNode[]>([])
  const [loading, setLoading] = React.useState(false)
  const [expandedPaths, setExpandedPaths] = React.useState<Set<string>>(new Set())
  const [selectedPath, setSelectedPath] = React.useState<string | null>(null)
  const [searchText, setSearchText] = React.useState("")
  const [showSearch, setShowSearch] = React.useState(false)
  const containerRef = React.useRef<HTMLDivElement>(null)
  const [containerHeight, setContainerHeight] = React.useState(400)
  
  // 搜索防抖
  const debouncedSearchText = useDebounce(searchText, 200)

  // 监听容器高度变化
  React.useEffect(() => {
    if (!containerRef.current) return
    
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setContainerHeight(entry.contentRect.height)
      }
    })
    
    observer.observe(containerRef.current)
    return () => observer.disconnect()
  }, [])

  // 加载项目文件
  const loadFiles = React.useCallback(async () => {
    if (!currentProject?.path) return

    setLoading(true)
    try {
      const result = await invoke<FileNode[]>("list_directory", {
        path: currentProject.path,
        recursive: true,
      })
      setFiles(result)
      // 默认展开根目录的第一层
      if (result.length > 0) {
        const rootPaths = result.filter((f) => f.is_dir).map((f) => f.path)
        setExpandedPaths(new Set(rootPaths.slice(0, 3)))
      }
    } catch (error) {
      console.error("加载文件列表失败:", error)
    } finally {
      setLoading(false)
    }
  }, [currentProject?.path])

  // 项目变化时加载文件
  React.useEffect(() => {
    loadFiles()
  }, [loadFiles])

  // 切换展开状态
  const toggleExpand = React.useCallback((path: string) => {
    setExpandedPaths((prev) => {
      const next = new Set(prev)
      if (next.has(path)) {
        next.delete(path)
      } else {
        next.add(path)
      }
      return next
    })
  }, [])

  // 选择文件
  const handleSelect = React.useCallback(
    (path: string) => {
      setSelectedPath(path)
      // 只有文件才打开编辑器
      const isFile = !files.some(f => f.path === path && f.is_dir)
      if (isFile) {
        setCurrentFile(path)
        onFileSelect?.(path)
      }
    },
    [files, onFileSelect, setCurrentFile]
  )
  
  // 双击打开文件
  const handleDoubleClick = React.useCallback(
    (path: string) => {
      setCurrentFile(path)
      onFileSelect?.(path)
    },
    [onFileSelect, setCurrentFile]
  )

  // 打开项目对话框
  const handleOpenProject = async () => {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "选择项目目录",
      })
      if (selected) {
        await openProject(selected as string)
      }
    } catch (error) {
      console.error("打开项目失败:", error)
    }
  }
  
  // 切换搜索框
  const toggleSearch = React.useCallback(() => {
    setShowSearch(prev => !prev)
    if (showSearch) {
      setSearchText("")
    }
  }, [showSearch])

  // 没有项目时显示空状态
  if (!currentProject) {
    return (
      <div className={cn("flex flex-col h-full", className)}>
        <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--border-subtle)]">
          <span className="text-xs font-medium text-[var(--text-muted)] uppercase tracking-wider">
            资源管理器
          </span>
        </div>
        <div className="flex-1 flex items-center justify-center p-4">
          <div className="text-center">
            <Folder className="h-10 w-10 mx-auto mb-3 text-[var(--text-muted)]" />
            <p className="text-sm text-[var(--text-muted)] mb-4">
              打开一个项目开始浏览
            </p>
            <button
              onClick={handleOpenProject}
              className="inline-flex items-center justify-center px-4 py-3 min-h-[32px] rounded-md bg-[var(--accent)] text-white text-sm font-medium hover:bg-[var(--accent)]/90 transition-colors"
            >
              <FolderPlus className="h-4 w-4 mr-2" />
              打开项目
            </button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className={cn("flex flex-col h-full", className)}>
      {/* 标题栏 */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--border-subtle)]">
        <span className="text-xs font-medium text-[var(--text-muted)] uppercase tracking-wider truncate">
          {currentProject.path.split("/").pop()}
        </span>
        <div className="flex items-center gap-1">
          <button
            onClick={toggleSearch}
            className={cn(
              "p-1 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:text-[var(--text-primary)]",
              showSearch && "bg-[var(--bg-tertiary)] text-[var(--text-primary)]"
            )}
            title="搜索文件"
          >
            <Search className="h-3.5 w-3.5" />
          </button>
          <button
            onClick={loadFiles}
            className="p-1 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:text-[var(--text-primary)]"
            title="刷新"
          >
            <RefreshCw className={cn("h-3.5 w-3.5", loading && "animate-spin")} />
          </button>
        </div>
      </div>
      
      {/* 搜索框 */}
      {showSearch && (
        <div className="px-2 py-1.5 border-b border-[var(--border-subtle)]">
          <div className="relative">
            <Search className="absolute left-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-[var(--text-muted)]" />
            <input
              type="text"
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
              placeholder="搜索文件..."
              className="w-full pl-7 pr-7 py-1 text-xs bg-[var(--bg-tertiary)] border border-[var(--border-subtle)] rounded focus:outline-none focus:border-[var(--accent)] text-[var(--text-primary)] placeholder:text-[var(--text-muted)]"
              autoFocus
            />
            {searchText && (
              <button
                onClick={() => setSearchText("")}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-[var(--text-muted)] hover:text-[var(--text-primary)]"
              >
                <X className="h-3.5 w-3.5" />
              </button>
            )}
          </div>
        </div>
      )}

      {/* 虚拟文件树 */}
      <div ref={containerRef} className="flex-1 overflow-hidden">
        {loading && files.length === 0 ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-5 w-5 animate-spin text-[var(--text-muted)]" />
          </div>
        ) : files.length === 0 ? (
          <div className="flex items-center justify-center py-8">
            <p className="text-xs text-[var(--text-muted)]">没有文件</p>
          </div>
        ) : (
          <VirtualFileTree
            nodes={files}
            selectedPath={selectedPath}
            expandedPaths={expandedPaths}
            onSelect={handleSelect}
            onToggle={toggleExpand}
            onDoubleClick={handleDoubleClick}
            height={containerHeight}
            filterText={debouncedSearchText}
          />
        )}
      </div>
    </div>
  )
})
