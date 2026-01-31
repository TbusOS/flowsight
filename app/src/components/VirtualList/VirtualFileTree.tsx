import React, { useCallback, useMemo } from 'react'
import { List, RowComponentProps } from 'react-window'
import { File, Folder, FolderOpen, ChevronRight, ChevronDown } from 'lucide-react'
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
  itemSize?: number
  className?: string
}

interface RowProps {
  flattenedNodes: FlattenedNode[]
  selectedPath: string | null
  onSelect: (path: string) => void
  onToggle: (path: string) => void
  onDoubleClick?: (path: string) => void
}

// 获取文件图标
function getFileIcon(node: FileNode, isExpanded: boolean) {
  if (node.is_dir) {
    return isExpanded ? (
      <FolderOpen className="w-4 h-4 text-yellow-500 shrink-0" />
    ) : (
      <Folder className="w-4 h-4 text-yellow-500 shrink-0" />
    )
  }
  
  // 根据扩展名返回不同颜色
  const ext = node.extension?.toLowerCase()
  const colorMap: Record<string, string> = {
    c: 'text-blue-400',
    h: 'text-green-400',
    rs: 'text-orange-400',
    ts: 'text-blue-500',
    tsx: 'text-blue-500',
    js: 'text-yellow-400',
    json: 'text-yellow-500',
    md: 'text-gray-400',
  }
  
  return <File className={cn('w-4 h-4 shrink-0', colorMap[ext || ''] || 'text-gray-400')} />
}

// 行组件
function FileTreeRow({ index, style, ...props }: RowComponentProps<RowProps>) {
  const { flattenedNodes, selectedPath, onSelect, onToggle, onDoubleClick } = props
  const { node, depth, isExpanded, hasChildren } = flattenedNodes[index]
  const isSelected = selectedPath === node.path
  
  return (
    <div
      style={style}
      className={cn(
        'flex items-center gap-1 px-2 cursor-pointer hover:bg-[var(--bg-hover)] transition-colors',
        isSelected && 'bg-[var(--bg-selected)] text-[var(--text-primary)]'
      )}
      onClick={() => {
        if (node.is_dir) {
          onToggle(node.path)
        }
        onSelect(node.path)
      }}
      onDoubleClick={() => onDoubleClick?.(node.path)}
      role="treeitem"
      aria-selected={isSelected}
      aria-expanded={node.is_dir ? isExpanded : undefined}
    >
      {/* 缩进 */}
      <span style={{ width: depth * 16 }} className="shrink-0" />
      
      {/* 展开/收起箭头 */}
      {hasChildren ? (
        <button
          className="p-0.5 hover:bg-[var(--bg-tertiary)] rounded shrink-0"
          onClick={(e) => {
            e.stopPropagation()
            onToggle(node.path)
          }}
          aria-label={isExpanded ? '收起' : '展开'}
        >
          {isExpanded ? (
            <ChevronDown className="w-3 h-3" />
          ) : (
            <ChevronRight className="w-3 h-3" />
          )}
        </button>
      ) : (
        <span className="w-4 shrink-0" />
      )}
      
      {/* 图标 */}
      {getFileIcon(node, isExpanded)}
      
      {/* 文件名 */}
      <span className="truncate text-sm">{node.name}</span>
    </div>
  )
}

/**
 * 虚拟化文件树组件
 * 使用 react-window 实现，支持大量文件的高性能渲染
 */
export function VirtualFileTree({
  nodes,
  selectedPath,
  expandedPaths,
  onSelect,
  onToggle,
  onDoubleClick,
  height,
  itemSize = 28,
  className,
}: VirtualFileTreeProps) {
  // 扁平化树结构
  const flattenedNodes = useMemo(() => {
    const result: FlattenedNode[] = []
    
    function flatten(items: FileNode[], depth: number) {
      for (const node of items) {
        const hasChildren = node.is_dir && node.children && node.children.length > 0
        const isExpanded = expandedPaths.has(node.path)
        
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
  }, [nodes, expandedPaths])

  if (flattenedNodes.length === 0) {
    return (
      <div className={cn('flex items-center justify-center h-full text-[var(--text-muted)] text-sm', className)}>
        无文件
      </div>
    )
  }

  return (
    <List
      className={className}
      defaultHeight={height}
      rowCount={flattenedNodes.length}
      rowHeight={itemSize}
      rowComponent={FileTreeRow}
      rowProps={{
        flattenedNodes,
        selectedPath,
        onSelect,
        onToggle,
        onDoubleClick,
      }}
      role="tree"
    />
  )
}

export default VirtualFileTree
