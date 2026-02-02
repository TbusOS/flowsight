import React, { useMemo, useRef, useEffect } from 'react'
import { List, ListImperativeAPI, RowComponentProps } from 'react-window'
import { File, Folder, FolderOpen, ChevronRight, ChevronDown, FileCode, FileText } from 'lucide-react'
import { cn } from '../../lib/utils/cn'

export interface FileNode {
  name: string
  path: string
  is_dir: boolean
  extension?: string
  children?: FileNode[]
}

interface FlattenedNode {
  node: FileNode
  depth: number
  isExpanded: boolean
  hasChildren: boolean
}

interface VirtualFileTreeProps {
  nodes: FileNode[]
  selectedPath: string | null
  expandedPaths: Set<string>
  onSelect: (path: string) => void
  onToggle: (path: string) => void
  onDoubleClick?: (path: string) => void
  height: number
  width?: number | string
  itemSize?: number
  className?: string
  /** 搜索过滤词 */
  filterText?: string
}

interface RowProps {
  flattenedNodes: FlattenedNode[]
  selectedPath: string | null
  onSelect: (path: string) => void
  onToggle: (path: string) => void
  onDoubleClick?: (path: string) => void
}

// 文件图标颜色映射
const FILE_COLOR_MAP: Record<string, string> = {
  c: 'text-blue-400',
  h: 'text-green-400',
  cpp: 'text-blue-400',
  hpp: 'text-green-400',
  cc: 'text-blue-400',
  rs: 'text-orange-400',
  ts: 'text-blue-500',
  tsx: 'text-blue-500',
  js: 'text-yellow-400',
  jsx: 'text-yellow-400',
  json: 'text-yellow-500',
  yaml: 'text-green-400',
  yml: 'text-green-400',
  md: 'text-gray-400',
  txt: 'text-gray-400',
  py: 'text-yellow-500',
  sh: 'text-green-500',
}

// 获取文件图标 - 使用 memo 优化
const FileIcon = React.memo(function FileIcon({ 
  node, 
  isExpanded 
}: { 
  node: FileNode
  isExpanded: boolean 
}) {
  if (node.is_dir) {
    return isExpanded ? (
      <FolderOpen className="w-4 h-4 text-[var(--accent)] shrink-0" />
    ) : (
      <Folder className="w-4 h-4 text-[var(--accent)] shrink-0" />
    )
  }
  
  const ext = node.extension?.toLowerCase() || ''
  const colorClass = FILE_COLOR_MAP[ext] || 'text-[var(--text-muted)]'
  
  // 代码文件使用 FileCode 图标
  const isCode = ['c', 'h', 'cpp', 'hpp', 'cc', 'rs', 'ts', 'tsx', 'js', 'jsx', 'py'].includes(ext)
  
  if (isCode) {
    return <FileCode className={cn('w-4 h-4 shrink-0', colorClass)} />
  }
  
  // 文本文件使用 FileText 图标
  const isText = ['json', 'yaml', 'yml', 'md', 'txt', 'toml'].includes(ext)
  if (isText) {
    return <FileText className={cn('w-4 h-4 shrink-0', colorClass)} />
  }
  
  return <File className={cn('w-4 h-4 shrink-0', colorClass)} />
})

// 行组件（react-window v2 API）
function FileTreeRow({ 
  index, 
  style,
  flattenedNodes,
  selectedPath,
  onSelect,
  onToggle,
  onDoubleClick,
}: RowComponentProps<RowProps>) {
  const { node, depth, isExpanded, hasChildren } = flattenedNodes[index]
  const isSelected = selectedPath === node.path
  
  const handleClick = React.useCallback(() => {
    if (node.is_dir) {
      onToggle(node.path)
    }
    onSelect(node.path)
  }, [node.is_dir, node.path, onToggle, onSelect])
  
  const handleDoubleClick = React.useCallback(() => {
    onDoubleClick?.(node.path)
  }, [node.path, onDoubleClick])
  
  const handleToggleClick = React.useCallback((e: React.MouseEvent) => {
    e.stopPropagation()
    onToggle(node.path)
  }, [node.path, onToggle])
  
  return (
    <div
      style={style}
      className={cn(
        'flex items-center gap-1 px-2 cursor-pointer transition-colors',
        'hover:bg-[var(--bg-tertiary)]',
        isSelected && 'bg-[var(--accent)]/10 text-[var(--accent)]'
      )}
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
      role="treeitem"
      aria-selected={isSelected}
      aria-expanded={node.is_dir ? isExpanded : false}
    >
      {/* 缩进 */}
      <span style={{ width: depth * 12 }} className="shrink-0" />
      
      {/* 展开/收起箭头 */}
      {hasChildren ? (
        <button
          className="p-0.5 hover:bg-[var(--bg-hover)] rounded shrink-0"
          onClick={handleToggleClick}
          aria-label={isExpanded ? '收起' : '展开'}
        >
          {isExpanded ? (
            <ChevronDown className="w-3.5 h-3.5 text-[var(--text-muted)]" />
          ) : (
            <ChevronRight className="w-3.5 h-3.5 text-[var(--text-muted)]" />
          )}
        </button>
      ) : (
        <span className="w-4 shrink-0" />
      )}
      
      {/* 图标 */}
      <FileIcon node={node} isExpanded={isExpanded} />
      
      {/* 文件名 */}
      <span className="truncate text-[13px]">{node.name}</span>
    </div>
  )
}

/**
 * 虚拟化文件树组件
 * 使用 react-window v2 实现，支持大量文件的高性能渲染
 */
export function VirtualFileTree({
  nodes,
  selectedPath,
  expandedPaths,
  onSelect,
  onToggle,
  onDoubleClick,
  height,
  itemSize = 26,
  className,
  filterText,
}: VirtualFileTreeProps) {
  const listRef = useRef<ListImperativeAPI | null>(null)
  
  // 扁平化树结构（带搜索过滤）
  const flattenedNodes = useMemo(() => {
    const result: FlattenedNode[] = []
    const filter = filterText?.toLowerCase().trim()
    
    // 检查节点或其子节点是否匹配搜索词
    function matchesFilter(node: FileNode): boolean {
      if (!filter) return true
      if (node.name.toLowerCase().includes(filter)) return true
      if (node.children) {
        return node.children.some(matchesFilter)
      }
      return false
    }
    
    function flatten(items: FileNode[], depth: number) {
      for (const node of items) {
        // 如果有过滤词且节点不匹配，跳过
        if (filter && !matchesFilter(node)) continue
        
        const hasChildren = node.is_dir && node.children && node.children.length > 0
        // 搜索时自动展开包含匹配项的目录
        const isExpanded: boolean = filter 
          ? !!(hasChildren && node.children!.some(matchesFilter))
          : expandedPaths.has(node.path)
        
        result.push({
          node,
          depth,
          isExpanded,
          hasChildren: hasChildren || node.is_dir,
        })
        
        if (hasChildren && isExpanded && node.children) {
          flatten(node.children, depth + 1)
        }
      }
    }
    
    flatten(nodes, 0)
    return result
  }, [nodes, expandedPaths, filterText])

  // 选中项变化时滚动到可见
  useEffect(() => {
    if (selectedPath && listRef.current) {
      const index = flattenedNodes.findIndex(n => n.node.path === selectedPath)
      if (index >= 0) {
        listRef.current.scrollToRow({ index, align: 'smart' })
      }
    }
  }, [selectedPath, flattenedNodes])

  // 行属性（稳定引用）
  const rowProps = useMemo<RowProps>(() => ({
    flattenedNodes,
    selectedPath,
    onSelect,
    onToggle,
    onDoubleClick,
  }), [flattenedNodes, selectedPath, onSelect, onToggle, onDoubleClick])

  if (flattenedNodes.length === 0) {
    return (
      <div className={cn('flex items-center justify-center h-full text-[var(--text-muted)] text-sm', className)}>
        {filterText ? '无匹配文件' : '无文件'}
      </div>
    )
  }

  return (
    <List<RowProps>
      listRef={listRef}
      className={className}
      defaultHeight={height}
      rowCount={flattenedNodes.length}
      rowHeight={itemSize}
      rowComponent={FileTreeRow}
      rowProps={rowProps}
      overscanCount={10}
      role="tree"
    />
  )
}

export default VirtualFileTree
