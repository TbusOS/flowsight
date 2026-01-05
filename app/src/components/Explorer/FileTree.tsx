/**
 * FileTree - 文件树浏览组件
 */

import { useState, useCallback } from 'react'
import './Explorer.css'

export interface FileNode {
  name: string
  path: string
  is_dir: boolean
  children?: FileNode[]
  extension?: string
}

interface FileTreeProps {
  nodes: FileNode[]
  onFileSelect: (path: string) => void
  selectedPath?: string
}

interface FileTreeItemProps {
  node: FileNode
  depth: number
  onFileSelect: (path: string) => void
  selectedPath?: string
  onToggle: (path: string) => void
  expanded: Set<string>
}

const FileTreeItem = ({ node, depth, onFileSelect, selectedPath, onToggle, expanded }: FileTreeItemProps) => {
  const isExpanded = expanded.has(node.path)
  const isSelected = selectedPath === node.path
  
  const handleClick = () => {
    if (node.is_dir) {
      onToggle(node.path)
    } else {
      onFileSelect(node.path)
    }
  }
  
  const getFileIcon = () => {
    if (node.is_dir) {
      return isExpanded ? '📂' : '📁'
    }
    
    const ext = node.extension || node.name.split('.').pop()?.toLowerCase()
    switch (ext) {
      case 'c': return '🔷'
      case 'h': return '📘'
      case 'cpp':
      case 'cc':
      case 'cxx': return '🔶'
      case 'hpp':
      case 'hxx': return '📙'
      case 'rs': return '🦀'
      case 'py': return '🐍'
      case 'js':
      case 'ts':
      case 'tsx': return '💛'
      case 'json': return '📋'
      case 'md': return '📝'
      case 'yaml':
      case 'yml': return '⚙️'
      default: return '📄'
    }
  }
  
  return (
    <div className="tree-node">
      <div 
        className={`tree-item ${isSelected ? 'selected' : ''} ${node.is_dir ? 'directory' : 'file'}`}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        onClick={handleClick}
      >
        {node.is_dir && (
          <span className={`chevron ${isExpanded ? 'expanded' : ''}`}>
            ▶
          </span>
        )}
        <span className="file-icon">{getFileIcon()}</span>
        <span className="file-name">{node.name}</span>
      </div>
      
      {node.is_dir && isExpanded && node.children && (
        <div className="tree-children">
          {node.children.map((child) => (
            <FileTreeItem
              key={child.path}
              node={child}
              depth={depth + 1}
              onFileSelect={onFileSelect}
              selectedPath={selectedPath}
              onToggle={onToggle}
              expanded={expanded}
            />
          ))}
        </div>
      )}
    </div>
  )
}

export const FileTree = ({ nodes, onFileSelect, selectedPath }: FileTreeProps) => {
  const [expanded, setExpanded] = useState<Set<string>>(new Set())
  
  const handleToggle = useCallback((path: string) => {
    setExpanded(prev => {
      const next = new Set(prev)
      if (next.has(path)) {
        next.delete(path)
      } else {
        next.add(path)
      }
      return next
    })
  }, [])
  
  if (nodes.length === 0) {
    return (
      <div className="empty-tree">
        <p>无文件</p>
      </div>
    )
  }
  
  return (
    <div className="file-tree">
      {nodes.map((node) => (
        <FileTreeItem
          key={node.path}
          node={node}
          depth={0}
          onFileSelect={onFileSelect}
          selectedPath={selectedPath}
          onToggle={handleToggle}
          expanded={expanded}
        />
      ))}
    </div>
  )
}

