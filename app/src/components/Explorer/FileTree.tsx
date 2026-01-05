/**
 * FileTree - 文件树浏览组件
 * 
 * 支持延迟加载目录内容
 */

import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
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
  onTreeUpdate?: (nodes: FileNode[]) => void
}

interface FileTreeItemProps {
  node: FileNode
  depth: number
  onFileSelect: (path: string) => void
  selectedPath?: string
  onToggle: (path: string, node: FileNode) => Promise<void>
  expanded: Set<string>
  loading: Set<string>
}

const FileTreeItem = ({ node, depth, onFileSelect, selectedPath, onToggle, expanded, loading }: FileTreeItemProps) => {
  const isExpanded = expanded.has(node.path)
  const isSelected = selectedPath === node.path
  const isLoading = loading.has(node.path)
  
  const handleClick = () => {
    if (node.is_dir) {
      onToggle(node.path, node)
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
          <span className={`chevron ${isExpanded ? 'expanded' : ''} ${isLoading ? 'loading' : ''}`}>
            {isLoading ? '◌' : '▶'}
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
              loading={loading}
            />
          ))}
        </div>
      )}
    </div>
  )
}

export const FileTree = ({ nodes, onFileSelect, selectedPath, onTreeUpdate }: FileTreeProps) => {
  const [expanded, setExpanded] = useState<Set<string>>(new Set())
  const [loading, setLoading] = useState<Set<string>>(new Set())
  const [localNodes, setLocalNodes] = useState<FileNode[]>(nodes)
  
  // Update local nodes when prop changes
  if (nodes !== localNodes && nodes.length > 0 && localNodes.length === 0) {
    setLocalNodes(nodes)
  }
  
  // Helper to update a node's children in the tree
  const updateNodeChildren = useCallback((nodes: FileNode[], path: string, children: FileNode[]): FileNode[] => {
    return nodes.map(node => {
      if (node.path === path) {
        return { ...node, children }
      }
      if (node.children) {
        return { ...node, children: updateNodeChildren(node.children, path, children) }
      }
      return node
    })
  }, [])
  
  const handleToggle = useCallback(async (path: string, node: FileNode) => {
    // If already expanded, just collapse
    if (expanded.has(path)) {
      setExpanded(prev => {
        const next = new Set(prev)
        next.delete(path)
        return next
      })
      return
    }
    
    // If children not loaded, load them
    if (!node.children || node.children.length === 0) {
      setLoading(prev => new Set(prev).add(path))
      
      try {
        const children = await invoke<FileNode[]>('expand_directory', { path })
        
        setLocalNodes(prev => {
          const updated = updateNodeChildren(prev, path, children)
          onTreeUpdate?.(updated)
          return updated
        })
      } catch (e) {
        console.error('Failed to load directory:', e)
      } finally {
        setLoading(prev => {
          const next = new Set(prev)
          next.delete(path)
          return next
        })
      }
    }
    
    // Expand the node
    setExpanded(prev => new Set(prev).add(path))
  }, [expanded, updateNodeChildren, onTreeUpdate])
  
  const displayNodes = localNodes.length > 0 ? localNodes : nodes
  
  if (displayNodes.length === 0) {
    return (
      <div className="empty-tree">
        <p>无文件</p>
      </div>
    )
  }
  
  return (
    <div className="file-tree">
      {displayNodes.map((node) => (
        <FileTreeItem
          key={node.path}
          node={node}
          depth={0}
          onFileSelect={onFileSelect}
          selectedPath={selectedPath}
          onToggle={handleToggle}
          expanded={expanded}
          loading={loading}
        />
      ))}
    </div>
  )
}

