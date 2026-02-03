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
  Search,
  X,
  Zap,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { currentFileAtom, setEntryFunctionAtom, jumpTargetAtom } from "../../lib/atoms/layout-atoms"
import { useAtomValue, useSetAtom } from "jotai"
import { useDebounce } from "../../hooks/usePerformance"
import { List, ListImperativeAPI, RowComponentProps } from 'react-window'

// 大纲项类型
export interface OutlineItem {
  id?: string
  name: string
  type?: "function" | "variable" | "type" | "struct" | "enum"
  kind?: "function" | "variable" | "type" | "struct" | "enum" | string
  line: number
  visibility?: "public" | "private" | "protected"
  children?: OutlineItem[]
  // 扩展字段
  isCallback?: boolean
  returnType?: string
  callbackContext?: string | null
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

// 图标组件 - memoized
const TypeIcon = React.memo(function TypeIcon({ type }: { type: string }) {
  switch (type) {
    case "function":
      return <FunctionSquare className="h-3.5 w-3.5 text-[var(--accent)]" />
    case "struct":
      return <Box className="h-3.5 w-3.5 text-purple-400" />
    case "variable":
      return <Variable className="h-3.5 w-3.5 text-emerald-400" />
    case "type":
      return <Type className="h-3.5 w-3.5 text-amber-400" />
    case "enum":
      return <Layers className="h-3.5 w-3.5 text-pink-400" />
    default:
      return <FileCode className="h-3.5 w-3.5 text-[var(--text-muted)]" />
  }
})

// 单个大纲项组件 - memoized
interface OutlineItemRowProps {
  item: OutlineItem
  depth: number
  isSelected: boolean
  isExpanded: boolean
  hasChildren: boolean
  onSelect: (id: string, name: string, line: number, type: string) => void
  onToggle: (id: string) => void
}

const OutlineItemRow = React.memo(function OutlineItemRow({
  item,
  depth,
  isSelected,
  isExpanded,
  hasChildren,
  onSelect,
  onToggle,
}: OutlineItemRowProps) {
  const itemId = item.id ?? item.name
  const itemType = item.type ?? item.kind ?? "function"
  
  const handleClick = React.useCallback(() => {
    onSelect(itemId, item.name, item.line, itemType)
  }, [itemId, item.name, item.line, itemType, onSelect])
  
  const handleToggle = React.useCallback((e: React.MouseEvent) => {
    e.stopPropagation()
    onToggle(itemId)
  }, [itemId, onToggle])

  return (
    <div
      className={cn(
        "group flex items-center gap-1.5 px-3 py-1.5 cursor-pointer transition-colors rounded-md mx-2",
        isSelected && "bg-[var(--accent)]/10 ring-1 ring-[var(--accent)]/30",
        !isSelected && "hover:bg-[var(--bg-hover)]"
      )}
      style={{ paddingLeft: `${8 + depth * 12}px` }}
      onClick={handleClick}
      title={itemType === "function" ? `点击分析 ${item.name}` : undefined}
    >
      {/* Expand/Collapse */}
      <div className="w-4 flex items-center justify-center shrink-0">
        {hasChildren ? (
          <button
            className="p-0.5 rounded hover:bg-[var(--bg-active)]"
            onClick={handleToggle}
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
      <TypeIcon type={itemType} />

      {/* Name */}
      <span className={cn(
        "flex-1 text-xs truncate",
        isSelected ? "text-[var(--text-primary)] font-medium" : "text-[var(--text-secondary)]"
      )}>
        {item.name}
      </span>
      
      {/* Callback indicator */}
      {item.isCallback && (
        <span title="回调函数">
          <Zap className="h-3 w-3 text-amber-400 shrink-0" />
        </span>
      )}

      {/* Line Number */}
      <span className="text-[10px] text-[var(--text-muted)] font-mono shrink-0 tabular-nums">
        :{item.line}
      </span>
    </div>
  )
})

// 虚拟化行组件的属性
interface VirtualRowProps {
  items: OutlineItem[]
  selectedItem: string | null
  expandedItems: Set<string>
  onSelect: (id: string, name: string, line: number, type: string) => void
  onToggle: (id: string) => void
}

// 虚拟化行组件
function VirtualOutlineRow({ 
  index, 
  style,
  items,
  selectedItem,
  expandedItems,
  onSelect,
  onToggle,
}: RowComponentProps<VirtualRowProps>) {
  const item = items[index]
  const itemId = item.id ?? item.name
  const hasChildren = item.children && item.children.length > 0
  const isExpanded = expandedItems.has(itemId)
  const isSelected = selectedItem === itemId

  return (
    <div style={style}>
      <OutlineItemRow
        item={item}
        depth={0}
        isSelected={isSelected}
        isExpanded={isExpanded}
        hasChildren={hasChildren || false}
        onSelect={onSelect}
        onToggle={onToggle}
      />
    </div>
  )
}

// 主组件
export const OutlinePanel = React.memo(function OutlinePanel({ 
  className, 
  items: propItems 
}: OutlinePanelProps) {
  const currentFile = useAtomValue(currentFileAtom)
  const setEntryFunction = useSetAtom(setEntryFunctionAtom)
  const setJumpTarget = useSetAtom(jumpTargetAtom)
  const [items, setItems] = React.useState<OutlineItem[]>([])
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)
  const [expandedItems, setExpandedItems] = React.useState<Set<string>>(new Set())
  const [selectedItem, setSelectedItem] = React.useState<string | null>(null)
  const [searchText, setSearchText] = React.useState("")
  const containerRef = React.useRef<HTMLDivElement>(null)
  const listRef = React.useRef<ListImperativeAPI | null>(null)
  const [containerHeight, setContainerHeight] = React.useState(300)
  
  // 搜索防抖
  const debouncedSearch = useDebounce(searchText, 150)

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

  // 当文件变化时加载函数列表
  React.useEffect(() => {
    if (!currentFile) {
      setItems([])
      setSelectedItem(null)
      return
    }

    const loadFunctions = async () => {
      setLoading(true)
      setError(null)
      try {
        const functions = await invoke<FunctionInfo[]>("get_functions", { path: currentFile })
        // 转换为 OutlineItem 格式，按行号排序
        const outlineItems: OutlineItem[] = functions
          .map((func, index) => ({
            id: `func-${index}`,
            name: func.name,
            type: "function" as const,
            line: func.line,
            isCallback: func.is_callback,
            returnType: func.return_type,
            callbackContext: func.callback_context,
          }))
          .sort((a, b) => a.line - b.line)
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

  // 过滤后的项目
  const filteredItems = React.useMemo(() => {
    const displayItems = propItems && propItems.length > 0 ? propItems : items
    if (!debouncedSearch.trim()) return displayItems
    
    const search = debouncedSearch.toLowerCase()
    return displayItems.filter(item => 
      item.name.toLowerCase().includes(search)
    )
  }, [propItems, items, debouncedSearch])

  // 处理选择
  const handleSelect = React.useCallback((id: string, name: string, line: number, type: string) => {
    setSelectedItem(id)
    
    // 跳转到对应行
    if (currentFile) {
      setJumpTarget({
        filePath: currentFile,
        line,
        column: 1,
      })
    }
    
    // 函数类型自动触发分析
    if (type === "function") {
      console.log('[OutlinePanel] Analyze function:', name)
      setEntryFunction(name)
    }
  }, [currentFile, setJumpTarget, setEntryFunction])

  // 切换展开
  const handleToggle = React.useCallback((id: string) => {
    setExpandedItems(prev => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }, [])

  // 选中项变化时滚动到可见
  React.useEffect(() => {
    if (selectedItem && listRef.current) {
      const index = filteredItems.findIndex(item => (item.id ?? item.name) === selectedItem)
      if (index >= 0) {
        listRef.current.scrollToRow({ index, align: 'smart' })
      }
    }
  }, [selectedItem, filteredItems])

  // 虚拟列表行属性
  const rowProps = React.useMemo<VirtualRowProps>(() => ({
    items: filteredItems,
    selectedItem,
    expandedItems,
    onSelect: handleSelect,
    onToggle: handleToggle,
  }), [filteredItems, selectedItem, expandedItems, handleSelect, handleToggle])

  // 清除搜索
  const clearSearch = React.useCallback(() => {
    setSearchText("")
  }, [])

  // 统计信息
  const stats = React.useMemo(() => {
    const callbacks = filteredItems.filter(i => i.isCallback).length
    return { total: filteredItems.length, callbacks }
  }, [filteredItems])

  return (
    <div className={cn("flex flex-col h-full", className)}>
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--border-subtle)]">
        <div className="flex items-center gap-2">
          <Layers className="h-4 w-4 text-[var(--text-muted)]" />
          <span className="text-xs font-medium text-[var(--text-primary)]">大纲</span>
          <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-[var(--bg-tertiary)] text-[var(--text-muted)] tabular-nums">
            {stats.total}
          </span>
          {stats.callbacks > 0 && (
            <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-amber-500/10 text-amber-400 tabular-nums" title="回调函数数量">
              <Zap className="h-2.5 w-2.5 inline mr-0.5" />
              {stats.callbacks}
            </span>
          )}
          {loading && <Loader2 className="h-3 w-3 animate-spin text-[var(--accent)]" />}
        </div>
        {currentFile && (
          <span className="text-[10px] text-[var(--text-muted)] truncate max-w-[100px]" title={currentFile}>
            {currentFile.split('/').pop()}
          </span>
        )}
      </div>

      {/* Search */}
      <div className="px-2 py-1.5 border-b border-[var(--border-subtle)]">
        <div className="relative">
          <Search className="absolute left-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-[var(--text-muted)]" />
          <input
            type="text"
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            placeholder="搜索函数..."
            data-testid="outline-search"
            className="w-full h-7 px-7 text-xs bg-[var(--bg-tertiary)] border border-[var(--border-light)] rounded-md text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)]"
          />
          {searchText && (
            <button
              onClick={clearSearch}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-[var(--text-muted)] hover:text-[var(--text-primary)]"
            >
              <X className="h-3.5 w-3.5" />
            </button>
          )}
        </div>
      </div>

      {/* Content - 虚拟化渲染 */}
      <div ref={containerRef} className="flex-1 overflow-hidden">
        {loading ? (
          <div className="flex items-center justify-center h-full">
            <Loader2 className="h-5 w-5 animate-spin text-[var(--text-muted)]" />
          </div>
        ) : error ? (
          <div className="flex flex-col items-center justify-center h-full text-center px-4">
            <p className="text-xs text-red-400">{error}</p>
          </div>
        ) : filteredItems.length > 0 ? (
          <List<VirtualRowProps>
            listRef={listRef}
            defaultHeight={containerHeight}
            rowCount={filteredItems.length}
            rowHeight={36}
            rowComponent={VirtualOutlineRow}
            rowProps={rowProps}
            overscanCount={5}
            role="listbox"
          />
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-center px-4">
            <Layers className="h-8 w-8 text-[var(--text-muted)] mb-2" />
            <p className="text-xs text-[var(--text-muted)]">
              {searchText 
                ? "无匹配结果" 
                : currentFile 
                  ? "没有找到函数" 
                  : "打开文件查看大纲"
              }
            </p>
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="px-3 py-1.5 border-t border-[var(--border-subtle)] text-[10px] text-[var(--text-muted)] flex items-center justify-between">
        <span>点击函数分析执行流</span>
        {selectedItem && (
          <span className="text-[var(--accent)]">已选: {selectedItem.replace('func-', '')}</span>
        )}
      </div>
    </div>
  )
})
