"use client"

import * as React from "react"
import { invoke } from "@tauri-apps/api/core"
import {
  ChevronRight,
  ChevronDown,
  File,
  Folder,
  FolderOpen,
  FileCode,
  FileText,
  Loader2,
  RefreshCw,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { useAnalysisStore } from "../../store/analysisStore"

// 文件节点类型
interface FileNode {
  name: string
  path: string
  is_dir: boolean
  extension?: string
  children?: FileNode[]
}

interface FileExplorerProps {
  className?: string
  onFileSelect?: (path: string) => void
}

// 获取文件图标
function getFileIcon(node: FileNode) {
  if (node.is_dir) {
    return null // 目录图标在渲染时处理
  }

  const ext = node.extension?.toLowerCase()
  switch (ext) {
    case "c":
    case "h":
    case "cpp":
    case "hpp":
    case "cc":
      return <FileCode className="h-4 w-4 text-blue-400" />
    case "rs":
      return <FileCode className="h-4 w-4 text-orange-400" />
    case "ts":
    case "tsx":
    case "js":
    case "jsx":
      return <FileCode className="h-4 w-4 text-yellow-400" />
    case "json":
    case "yaml":
    case "yml":
    case "toml":
      return <FileText className="h-4 w-4 text-green-400" />
    case "md":
    case "txt":
      return <FileText className="h-4 w-4 text-gray-400" />
    default:
      return <File className="h-4 w-4 text-[var(--text-muted)]" />
  }
}

// 文件树节点组件
function FileTreeNode({
  node,
  depth = 0,
  onSelect,
  expandedPaths,
  toggleExpand,
  selectedPath,
}: {
  node: FileNode
  depth?: number
  onSelect: (path: string) => void
  expandedPaths: Set<string>
  toggleExpand: (path: string) => void
  selectedPath: string | null
}) {
  const isExpanded = expandedPaths.has(node.path)
  const isSelected = selectedPath === node.path
  const hasChildren = node.is_dir && node.children && node.children.length > 0

  const handleClick = () => {
    if (node.is_dir) {
      toggleExpand(node.path)
    } else {
      onSelect(node.path)
    }
  }

  return (
    <div>
      <div
        className={cn(
          "flex items-center gap-1 px-2 py-0.5 cursor-pointer rounded-sm transition-colors",
          "hover:bg-[var(--bg-tertiary)]",
          isSelected && "bg-[var(--accent)]/10 text-[var(--accent)]"
        )}
        style={{ paddingLeft: `${depth * 12 + 8}px` }}
        onClick={handleClick}
      >
        {/* 展开/折叠图标 */}
        {node.is_dir ? (
          <span className="w-4 h-4 flex items-center justify-center">
            {hasChildren ? (
              isExpanded ? (
                <ChevronDown className="h-3.5 w-3.5 text-[var(--text-muted)]" />
              ) : (
                <ChevronRight className="h-3.5 w-3.5 text-[var(--text-muted)]" />
              )
            ) : (
              <span className="w-3.5" />
            )}
          </span>
        ) : (
          <span className="w-4" />
        )}

        {/* 文件/文件夹图标 */}
        {node.is_dir ? (
          isExpanded ? (
            <FolderOpen className="h-4 w-4 text-[var(--accent)]" />
          ) : (
            <Folder className="h-4 w-4 text-[var(--accent)]" />
          )
        ) : (
          getFileIcon(node)
        )}

        {/* 文件名 */}
        <span className="text-[13px] truncate flex-1">{node.name}</span>
      </div>

      {/* 子节点 */}
      {node.is_dir && isExpanded && node.children && (
        <div>
          {node.children.map((child) => (
            <FileTreeNode
              key={child.path}
              node={child}
              depth={depth + 1}
              onSelect={onSelect}
              expandedPaths={expandedPaths}
              toggleExpand={toggleExpand}
              selectedPath={selectedPath}
            />
          ))}
        </div>
      )}
    </div>
  )
}

export function FileExplorer({ className, onFileSelect }: FileExplorerProps) {
  const currentProject = useAnalysisStore((state) => state.currentProject)
  const [files, setFiles] = React.useState<FileNode[]>([])
  const [loading, setLoading] = React.useState(false)
  const [expandedPaths, setExpandedPaths] = React.useState<Set<string>>(new Set())
  const [selectedPath, setSelectedPath] = React.useState<string | null>(null)

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
      onFileSelect?.(path)
    },
    [onFileSelect]
  )

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
            <Folder className="h-8 w-8 mx-auto mb-2 text-[var(--text-muted)]" />
            <p className="text-xs text-[var(--text-muted)]">
              打开一个项目开始浏览
            </p>
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
        <button
          onClick={loadFiles}
          className="p-1 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:text-[var(--text-primary)]"
          title="刷新"
        >
          <RefreshCw className={cn("h-3.5 w-3.5", loading && "animate-spin")} />
        </button>
      </div>

      {/* 文件树 */}
      <div className="flex-1 overflow-y-auto py-1">
        {loading && files.length === 0 ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-5 w-5 animate-spin text-[var(--text-muted)]" />
          </div>
        ) : files.length === 0 ? (
          <div className="flex items-center justify-center py-8">
            <p className="text-xs text-[var(--text-muted)]">没有文件</p>
          </div>
        ) : (
          files.map((node) => (
            <FileTreeNode
              key={node.path}
              node={node}
              onSelect={handleSelect}
              expandedPaths={expandedPaths}
              toggleExpand={toggleExpand}
              selectedPath={selectedPath}
            />
          ))
        )}
      </div>
    </div>
  )
}
