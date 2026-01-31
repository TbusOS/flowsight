"use client"

import * as React from "react"
import { invoke } from "../../lib/tauri-api"
import {
  Search,
  FunctionSquare,
  Box,
  Hash,
  Loader2,
  FileCode,
  X,
  Filter,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { currentFileAtom } from "../../lib/atoms/layout-atoms"
import { useAtom } from "jotai"

// 搜索结果类型
export interface SearchResult {
  name: string
  kind: string
  file_path: string
  line: number
  preview: string
}

// 搜索类型过滤
type SearchKind = "all" | "function" | "struct" | "macro"

interface SearchPanelProps {
  className?: string
  onResultSelect?: (result: SearchResult) => void
}

export function SearchPanel({ className, onResultSelect }: SearchPanelProps) {
  const [currentFile, setCurrentFile] = useAtom(currentFileAtom)
  const [query, setQuery] = React.useState("")
  const [results, setResults] = React.useState<SearchResult[]>([])
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)
  const [selectedIndex, setSelectedIndex] = React.useState(0)
  const [searchKind, setSearchKind] = React.useState<SearchKind>("all")
  const [showFilters, setShowFilters] = React.useState(false)
  
  const inputRef = React.useRef<HTMLInputElement>(null)
  const resultsRef = React.useRef<HTMLDivElement>(null)

  // 搜索函数 - 防抖处理
  const performSearch = React.useCallback(async (searchQuery: string, kind: SearchKind) => {
    if (!searchQuery.trim()) {
      setResults([])
      return
    }

    setLoading(true)
    setError(null)

    try {
      // 调用 Tauri 后端搜索命令
      const searchResults = await invoke<SearchResult[]>("search_symbols", {
        query: searchQuery,
        kind: kind === "all" ? null : kind,
      })
      setResults(searchResults)
      setSelectedIndex(0)
    } catch (err) {
      console.error("搜索失败:", err)
      setError(String(err))
      setResults([])
    } finally {
      setLoading(false)
    }
  }, [])

  // 防抖搜索
  React.useEffect(() => {
    const timer = setTimeout(() => {
      performSearch(query, searchKind)
    }, 200)

    return () => clearTimeout(timer)
  }, [query, searchKind, performSearch])

  // 键盘导航
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!inputRef.current?.contains(document.activeElement) && 
          !resultsRef.current?.contains(document.activeElement)) {
        return
      }

      switch (e.key) {
        case "ArrowDown":
          e.preventDefault()
          setSelectedIndex(prev => Math.min(prev + 1, results.length - 1))
          break
        case "ArrowUp":
          e.preventDefault()
          setSelectedIndex(prev => Math.max(prev - 1, 0))
          break
        case "Enter":
          e.preventDefault()
          if (results[selectedIndex]) {
            handleResultClick(results[selectedIndex])
          }
          break
        case "Escape":
          e.preventDefault()
          setQuery("")
          setResults([])
          break
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [results, selectedIndex])

  // 自动滚动到选中项
  React.useEffect(() => {
    if (resultsRef.current && results.length > 0) {
      const selectedElement = resultsRef.current.querySelector(`[data-index="${selectedIndex}"]`)
      if (selectedElement) {
        selectedElement.scrollIntoView({ block: "nearest" })
      }
    }
  }, [selectedIndex, results.length])

  // 快捷键聚焦 (Ctrl/Cmd + F)
  React.useEffect(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "f") {
        e.preventDefault()
        inputRef.current?.focus()
      }
    }

    window.addEventListener("keydown", handleGlobalKeyDown)
    return () => window.removeEventListener("keydown", handleGlobalKeyDown)
  }, [])

  // 处理结果点击
  const handleResultClick = (result: SearchResult) => {
    // 设置当前文件
    setCurrentFile(result.file_path)
    // 调用外部回调
    onResultSelect?.(result)
  }

  // 获取类型图标
  const getKindIcon = (kind: string) => {
    switch (kind) {
      case "function":
        return <FunctionSquare className="h-3.5 w-3.5 text-[var(--accent)]" />
      case "struct":
        return <Box className="h-3.5 w-3.5 text-[var(--accent-purple)]" />
      case "macro":
        return <Hash className="h-3.5 w-3.5 text-[var(--accent-amber)]" />
      default:
        return <FileCode className="h-3.5 w-3.5 text-[var(--text-muted)]" />
    }
  }

  // 获取类型标签
  const getKindLabel = (kind: string) => {
    const labels: Record<string, string> = {
      function: "函数",
      struct: "结构体",
      macro: "宏",
    }
    return labels[kind] || kind
  }

  // 过滤按钮列表
  const filterButtons: { kind: SearchKind; label: string; icon: React.ReactNode }[] = [
    { kind: "all", label: "全部", icon: <Search className="h-3 w-3" /> },
    { kind: "function", label: "函数", icon: <FunctionSquare className="h-3 w-3" /> },
    { kind: "struct", label: "结构体", icon: <Box className="h-3 w-3" /> },
    { kind: "macro", label: "宏", icon: <Hash className="h-3 w-3" /> },
  ]

  return (
    <div className={cn("flex flex-col h-full", className)}>
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--border-subtle)]">
        <div className="flex items-center gap-2">
          <Search className="h-4 w-4 text-[var(--text-muted)]" />
          <span className="text-xs font-medium text-[var(--text-primary)]">搜索</span>
          {results.length > 0 && (
            <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-[var(--bg-tertiary)] text-[var(--text-muted)]">
              {results.length}
            </span>
          )}
          {loading && <Loader2 className="h-3 w-3 animate-spin text-[var(--accent)]" />}
        </div>
        <button
          className={cn(
            "p-1 rounded hover:bg-[var(--bg-tertiary)] transition-colors",
            showFilters && "bg-[var(--bg-tertiary)]"
          )}
          onClick={() => setShowFilters(!showFilters)}
          aria-label="切换过滤器"
        >
          <Filter className="h-3.5 w-3.5 text-[var(--text-muted)]" />
        </button>
      </div>

      {/* Search Input */}
      <div className="px-3 py-2 border-b border-[var(--border-subtle)]">
        <div className="relative">
          <input
            ref={inputRef}
            type="text"
            placeholder="搜索符号..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            data-testid="search-input"
            className="w-full h-7 px-2 pl-7 pr-7 text-xs bg-[var(--bg-tertiary)] border border-[var(--border-light)] rounded-md text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)]"
          />
          <Search className="absolute left-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-[var(--text-muted)]" />
          {query && (
            <button
              className="absolute right-2 top-1/2 -translate-y-1/2 p-0.5 rounded hover:bg-[var(--bg-active)]"
              onClick={() => {
                setQuery("")
                setResults([])
                inputRef.current?.focus()
              }}
              aria-label="清除搜索"
            >
              <X className="h-3 w-3 text-[var(--text-muted)]" />
            </button>
          )}
        </div>
      </div>

      {/* Filters */}
      {showFilters && (
        <div className="px-3 py-2 border-b border-[var(--border-subtle)] flex gap-1 flex-wrap">
          {filterButtons.map((filter) => (
            <button
              key={filter.kind}
              className={cn(
                "flex items-center gap-1 px-2 py-1 rounded text-[10px] transition-colors",
                searchKind === filter.kind
                  ? "bg-[var(--accent)] text-white"
                  : "bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:bg-[var(--bg-active)]"
              )}
              onClick={() => setSearchKind(filter.kind)}
            >
              {filter.icon}
              <span>{filter.label}</span>
            </button>
          ))}
        </div>
      )}

      {/* Results */}
      <div 
        ref={resultsRef}
        className="flex-1 overflow-y-auto py-2"
        data-testid="search-results"
      >
        {loading ? (
          <div className="flex items-center justify-center h-full">
            <Loader2 className="h-5 w-5 animate-spin text-[var(--text-muted)]" />
          </div>
        ) : error ? (
          <div className="flex flex-col items-center justify-center h-full text-center px-4">
            <p className="text-xs text-red-400">{error}</p>
          </div>
        ) : results.length > 0 ? (
          results.map((result, index) => (
            <div
              key={`${result.file_path}-${result.name}-${result.line}`}
              data-index={index}
              data-testid="search-result-item"
              className={cn(
                "group flex flex-col gap-1 px-3 py-2 cursor-pointer transition-colors mx-2 rounded-md",
                selectedIndex === index && "bg-[var(--bg-tertiary)]",
                selectedIndex !== index && "hover:bg-[var(--bg-hover)]"
              )}
              onClick={() => handleResultClick(result)}
            >
              {/* Result Header */}
              <div className="flex items-center gap-2">
                {getKindIcon(result.kind)}
                <span className={cn(
                  "flex-1 text-xs font-medium truncate",
                  selectedIndex === index ? "text-[var(--text-primary)]" : "text-[var(--text-secondary)]"
                )}>
                  {result.name}
                </span>
                <span className="text-[10px] px-1 py-0.5 rounded bg-[var(--bg-tertiary)] text-[var(--text-muted)]">
                  {getKindLabel(result.kind)}
                </span>
              </div>

              {/* File Path & Line */}
              <div className="flex items-center gap-2 text-[10px] text-[var(--text-muted)]">
                <span className="truncate max-w-[180px]">{result.file_path.split('/').pop()}</span>
                <span>:</span>
                <span className="font-mono">{result.line}</span>
              </div>

              {/* Preview */}
              {result.preview && (
                <div className="mt-1 p-1.5 rounded bg-[var(--bg-primary)] border border-[var(--border-subtle)]">
                  <code className="text-[10px] font-mono text-[var(--text-secondary)] line-clamp-2">
                    {result.preview}
                  </code>
                </div>
              )}
            </div>
          ))
        ) : query ? (
          <div className="flex flex-col items-center justify-center h-full text-center px-4">
            <Search className="h-8 w-8 text-[var(--text-muted)] mb-2" />
            <p className="text-xs text-[var(--text-muted)]">
              没有找到匹配 "{query}" 的结果
            </p>
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-center px-4">
            <Search className="h-8 w-8 text-[var(--text-muted)] mb-2" />
            <p className="text-xs text-[var(--text-muted)]">
              输入关键词搜索符号
            </p>
            <p className="text-[10px] text-[var(--text-muted)] mt-1">
              支持函数、结构体、宏
            </p>
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="px-3 py-2 border-t border-[var(--border-subtle)] text-[10px] text-[var(--text-muted)] flex items-center justify-between">
        <span>⌘F 聚焦搜索</span>
        <span>↑↓ 导航 · Enter 跳转</span>
      </div>
    </div>
  )
}
