"use client"

import * as React from "react"
import { invoke } from "../../lib/tauri-api"
import {
  FileCode,
  FunctionSquare,
  Variable,
  Type,
  ChevronRight,
  ChevronDown,
  Box,
  Layers,
  Loader2,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { currentFileAtom, setEntryFunctionAtom } from "../../lib/atoms/layout-atoms"
import { useAtomValue, useSetAtom } from "jotai"

// 大纲项类型
export interface OutlineItem {
  id?: string
  name: string
  type?: "function" | "variable" | "type" | "struct" | "enum"
  kind?: "function" | "variable" | "type" | "struct" | "enum" | string
  line: number
  visibility?: "public" | "private" | "protected"
  children?: OutlineItem[]
  // 兼容旧版字段
  isCallback?: boolean
  returnType?: string
}

// 后端返回的函数信息
interface FunctionInfo {
  name: string
  return_type: string
  line: number
  is_callback: boolean
  callback_context: string | null
  calls: string[]
}

interface OutlinePanelProps {
  className?: string
  items?: OutlineItem[]
}

export function OutlinePanel({ className, items: propItems }: OutlinePanelProps) {
  const currentFile = useAtomValue(currentFileAtom)
  const setEntryFunction = useSetAtom(setEntryFunctionAtom)
  const [items, setItems] = React.useState<OutlineItem[]>([])
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)

  // 处理函数分析（双击触发）
  const handleAnalyzeFunction = React.useCallback((funcName: string) => {
    console.log('[OutlinePanel] Analyze function:', funcName)
    setEntryFunction(funcName)
  }, [setEntryFunction])

  // 当文件变化时加载函数列表
  React.useEffect(() => {
    if (!currentFile) {
      setItems([])
      return
    }

    const loadFunctions = async () => {
      setLoading(true)
      setError(null)
      try {
        const functions = await invoke<FunctionInfo[]>("get_functions", { path: currentFile })
        // 转换为 OutlineItem 格式
        const outlineItems: OutlineItem[] = functions.map((func, index) => ({
          id: `func-${index}`,
          name: func.name,
          type: "function" as const,
          line: func.line,
          visibility: "public" as const,
          isCallback: func.is_callback,
          returnType: func.return_type,
        }))
        setItems(outlineItems)
      } catch (err) {
        console.error("加载函数列表失败:", err)
        setError(String(err))
        setItems([])
      } finally {
        setLoading(false)
      }
    }

    loadFunctions()
  }, [currentFile])

  // 如果传入了 items prop，使用它
  const displayItems = propItems && propItems.length > 0 ? propItems : items
  const [expandedItems, setExpandedItems] = React.useState<Set<string>>(new Set())
  const [selectedItem, setSelectedItem] = React.useState<string | null>(null)

  const toggleExpand = (id: string) => {
    const newExpanded = new Set(expandedItems)
    if (newExpanded.has(id)) {
      newExpanded.delete(id)
    } else {
      newExpanded.add(id)
    }
    setExpandedItems(newExpanded)
  }

  const getTypeIcon = (type: string) => {
    switch (type) {
      case "function":
        return <FunctionSquare className="h-3.5 w-3.5 text-[var(--accent)]" />
      case "struct":
        return <Box className="h-3.5 w-3.5 text-[var(--accent-purple)]" />
      case "variable":
        return <Variable className="h-3.5 w-3.5 text-[var(--accent-emerald)]" />
      case "type":
        return <Type className="h-3.5 w-3.5 text-[var(--accent-amber)]" />
      default:
        return <FileCode className="h-3.5 w-3.5 text-[var(--text-muted)]" />
    }
  }

  const getVisibilityBadge = (visibility: string) => {
    switch (visibility) {
      case "public":
        return <span className="text-[10px] px-1 py-0.5 rounded bg-[var(--accent)]/10 text-[var(--accent)]">pub</span>
      case "private":
        return <span className="text-[10px] px-1 py-0.5 rounded bg-[var(--bg-tertiary)] text-[var(--text-muted)]">pri</span>
      default:
        return null
    }
  }

  const renderItem = (item: OutlineItem, depth = 0) => {
    const itemId = item.id ?? item.name
    const hasChildren = item.children && item.children.length > 0
    const isExpanded = expandedItems.has(itemId)
    const isSelected = selectedItem === itemId
    const itemType = item.type ?? item.kind ?? "function"

    return (
      <div key={itemId}>
        <div
          className={cn(
            "group flex items-center gap-1.5 px-3 py-1.5 cursor-pointer transition-colors rounded-md mx-2",
            isSelected && "bg-[var(--bg-tertiary)]",
            !isSelected && "hover:bg-[var(--bg-hover)]"
          )}
          style={{ paddingLeft: `${8 + depth * 12}px` }}
          onClick={() => setSelectedItem(itemId)}
          onDoubleClick={() => {
            if (itemType === "function") {
              handleAnalyzeFunction(item.name)
            }
          }}
          title={itemType === "function" ? "双击分析执行流" : undefined}
        >
          {/* Expand/Collapse */}
          <div className="w-4 flex items-center justify-center">
            {hasChildren ? (
              <button
                className="p-0.5 rounded hover:bg-[var(--bg-active)]"
                onClick={(e) => {
                  e.stopPropagation()
                  toggleExpand(itemId)
                }}
              >
                {isExpanded ? (
                  <ChevronDown className="h-3 w-3 text-[var(--text-muted)]" />
                ) : (
                  <ChevronRight className="h-3 w-3 text-[var(--text-muted)]" />
                )}
              </button>
            ) : (
              <div className="w-4" />
            )}
          </div>

          {/* Icon */}
          {getTypeIcon(itemType)}

          {/* Name */}
          <span className={cn(
            "flex-1 text-xs truncate",
            isSelected ? "text-[var(--text-primary)]" : "text-[var(--text-secondary)]"
          )}>
            {item.name}
          </span>

          {/* Line Number */}
          <span className="text-[10px] text-[var(--text-muted)] font-mono ml-2">
            {item.line}
          </span>

          {/* Visibility */}
          {item.visibility && getVisibilityBadge(item.visibility)}
        </div>

        {/* Children */}
        {hasChildren && isExpanded && (
          <div className="mt-0.5">
            {item.children!.map((child) => renderItem(child, depth + 1))}
          </div>
        )}
      </div>
    )
  }

  return (
    <div className={cn("flex flex-col h-full", className)}>
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--border-subtle)]">
        <div className="flex items-center gap-2">
          <Layers className="h-4 w-4 text-[var(--text-muted)]" />
          <span className="text-xs font-medium text-[var(--text-primary)]">大纲</span>
          <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-[var(--bg-tertiary)] text-[var(--text-muted)]">
            {displayItems.length}
          </span>
          {loading && <Loader2 className="h-3 w-3 animate-spin text-[var(--accent)]" />}
        </div>
        {currentFile && (
          <span className="text-[10px] text-[var(--text-muted)] truncate max-w-[100px]">
            {currentFile.split('/').pop()}
          </span>
        )}
      </div>

      {/* Search */}
      <div className="px-3 py-2 border-b border-[var(--border-subtle)]">
        <div className="relative">
          <input
            type="text"
            placeholder="搜索符号..."
            data-testid="outline-search"
            className="w-full h-7 px-2 pl-7 text-xs bg-[var(--bg-tertiary)] border border-[var(--border-light)] rounded-md text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)]"
          />
          <FileCode className="absolute left-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-[var(--text-muted)]" />
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto py-2">
        {loading ? (
          <div className="flex items-center justify-center h-full">
            <Loader2 className="h-5 w-5 animate-spin text-[var(--text-muted)]" />
          </div>
        ) : error ? (
          <div className="flex flex-col items-center justify-center h-full text-center px-4">
            <p className="text-xs text-red-400">{error}</p>
          </div>
        ) : displayItems.length > 0 ? (
          displayItems.map((item) => renderItem(item))
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-center px-4">
            <Layers className="h-8 w-8 text-[var(--text-muted)] mb-2" />
            <p className="text-xs text-[var(--text-muted)]">
              {currentFile ? "没有找到符号" : "打开文件查看大纲"}
            </p>
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="px-3 py-2 border-t border-[var(--border-subtle)] text-[10px] text-[var(--text-muted)]">
        按 F12 跳转定义
      </div>
    </div>
  )
}
