/**
 * FileTree - 文件树浏览组件
 *
 * 支持延迟加载目录内容
 * 支持右键菜单: 新建文件/文件夹、删除、重命名
 */

import { useState, useCallback, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  Folder,
  FolderOpen,
  FileCode,
  FileJson,
  FileText,
  Settings,
  File,
  ChevronRight,
  Loader2,
  FilePlus,
  FolderPlus,
  Pencil,
  Trash2,
} from 'lucide-react'
import './Explorer.css'

export interface FileNode {
  name: string
  path: string
  is_dir: boolean
  children?: FileNode[]
  extension?: string
}

interface ContextMenuState {
  visible: boolean
  x: number
  y: number
  node: FileNode | null
}

interface FileTreeProps {
  nodes: FileNode[]
  onFileSelect: (path: string) => void
  selectedPath?: string
  onTreeUpdate?: (nodes: FileNode[]) => void
  onRefresh?: () => void
}

interface FileTreeItemProps {
  node: FileNode
  depth: number
  onFileSelect: (path: string) => void
  selectedPath?: string
  onToggle: (path: string, node: FileNode) => Promise<void>
  expanded: Set<string>
  loading: Set<string>
  onContextMenu: (e: React.MouseEvent, node: FileNode) => void
  editingPath: string | null
  editingValue: string
  onEditChange: (value: string) => void
  onEditSubmit: () => void
  onEditCancel: () => void
}

const FileTreeItem = ({ 
  node, 
  depth, 
  onFileSelect, 
  selectedPath, 
  onToggle, 
  expanded, 
  loading,
  onContextMenu,
  editingPath,
  editingValue,
  onEditChange,
  onEditSubmit,
  onEditCancel,
}: FileTreeItemProps) => {
  const isExpanded = expanded.has(node.path)
  const isSelected = selectedPath === node.path
  const isLoading = loading.has(node.path)
  const isEditing = editingPath === node.path
  const inputRef = useRef<HTMLInputElement>(null)
  
  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus()
      inputRef.current.select()
    }
  }, [isEditing])
  
  const handleClick = () => {
    if (isEditing) return
    if (node.is_dir) {
      onToggle(node.path, node)
    } else {
      onFileSelect(node.path)
    }
  }
  
  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()
    onContextMenu(e, node)
  }
  
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      onEditSubmit()
    } else if (e.key === 'Escape') {
      onEditCancel()
    }
  }
  
  const getFileIcon = () => {
    if (node.is_dir) {
      return isExpanded ? (
        <FolderOpen className="w-4 h-4 text-amber-400" strokeWidth={1.5} />
      ) : (
        <Folder className="w-4 h-4 text-amber-400" strokeWidth={1.5} />
      )
    }

    const ext = node.extension || node.name.split('.').pop()?.toLowerCase()
    switch (ext) {
      case 'c':
        return <FileCode className="w-4 h-4 text-blue-400" strokeWidth={1.5} />
      case 'h':
        return <FileCode className="w-4 h-4 text-blue-300" strokeWidth={1.5} />
      case 'cpp':
      case 'cc':
      case 'cxx':
        return <FileCode className="w-4 h-4 text-orange-400" strokeWidth={1.5} />
      case 'hpp':
      case 'hxx':
        return <FileCode className="w-4 h-4 text-orange-300" strokeWidth={1.5} />
      case 'rs':
        return <FileCode className="w-4 h-4 text-orange-500" strokeWidth={1.5} />
      case 'py':
        return <FileCode className="w-4 h-4 text-yellow-400" strokeWidth={1.5} />
      case 'js':
      case 'ts':
      case 'tsx':
        return <FileCode className="w-4 h-4 text-yellow-300" strokeWidth={1.5} />
      case 'json':
        return <FileJson className="w-4 h-4 text-yellow-500" strokeWidth={1.5} />
      case 'md':
        return <FileText className="w-4 h-4 text-slate-400" strokeWidth={1.5} />
      case 'yaml':
      case 'yml':
        return <Settings className="w-4 h-4 text-slate-500" strokeWidth={1.5} />
      default:
        return <File className="w-4 h-4 text-slate-400" strokeWidth={1.5} />
    }
  }
  
  return (
    <div className="tree-node">
      <div 
        className={`tree-item ${isSelected ? 'selected' : ''} ${node.is_dir ? 'directory' : 'file'} ${isEditing ? 'editing' : ''}`}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        onClick={handleClick}
        onContextMenu={handleContextMenu}
      >
        {node.is_dir && (
          <span className={`chevron ${isExpanded ? 'expanded' : ''} ${isLoading ? 'loading' : ''}`}>
            {isLoading ? (
              <Loader2 className="w-3 h-3 animate-spin" strokeWidth={2} />
            ) : (
              <ChevronRight className="w-3.5 h-3.5" strokeWidth={2} />
            )}
          </span>
        )}
        <span className="file-icon">{getFileIcon()}</span>
        {isEditing ? (
          <input
            ref={inputRef}
            type="text"
            className="rename-input"
            value={editingValue}
            onChange={(e) => onEditChange(e.target.value)}
            onKeyDown={handleKeyDown}
            onBlur={onEditCancel}
            onClick={(e) => e.stopPropagation()}
          />
        ) : (
          <span className="file-name">{node.name}</span>
        )}
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
              onContextMenu={onContextMenu}
              editingPath={editingPath}
              editingValue={editingValue}
              onEditChange={onEditChange}
              onEditSubmit={onEditSubmit}
              onEditCancel={onEditCancel}
            />
          ))}
        </div>
      )}
    </div>
  )
}

export const FileTree = ({ nodes, onFileSelect, selectedPath, onTreeUpdate, onRefresh }: FileTreeProps) => {
  const [expanded, setExpanded] = useState<Set<string>>(new Set())
  const [loading, setLoading] = useState<Set<string>>(new Set())
  const [localNodes, setLocalNodes] = useState<FileNode[]>(nodes)
  
  // Context menu state
  const [contextMenu, setContextMenu] = useState<ContextMenuState>({
    visible: false,
    x: 0,
    y: 0,
    node: null,
  })
  
  // Editing state (for rename)
  const [editingPath, setEditingPath] = useState<string | null>(null)
  const [editingValue, setEditingValue] = useState('')
  const [editingAction, setEditingAction] = useState<'rename' | 'new-file' | 'new-folder' | null>(null)
  
  // Close context menu on click outside
  useEffect(() => {
    const handleClick = () => setContextMenu(prev => ({ ...prev, visible: false }))
    document.addEventListener('click', handleClick)
    return () => document.removeEventListener('click', handleClick)
  }, [])
  
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
  
  // Context menu handlers
  const handleContextMenu = useCallback((e: React.MouseEvent, node: FileNode) => {
    setContextMenu({
      visible: true,
      x: e.clientX,
      y: e.clientY,
      node,
    })
  }, [])
  
  const handleNewFile = useCallback(async () => {
    if (!contextMenu.node) return
    const parentPath = contextMenu.node.is_dir ? contextMenu.node.path : contextMenu.node.path.substring(0, contextMenu.node.path.lastIndexOf('/'))
    
    // Start editing for new file
    setEditingPath(parentPath + '/__new_file__')
    setEditingValue('untitled.c')
    setEditingAction('new-file')
    setContextMenu(prev => ({ ...prev, visible: false }))
    
    // Expand parent if directory
    if (contextMenu.node.is_dir && !expanded.has(contextMenu.node.path)) {
      await handleToggle(contextMenu.node.path, contextMenu.node)
    }
  }, [contextMenu.node, expanded, handleToggle])
  
  const handleNewFolder = useCallback(async () => {
    if (!contextMenu.node) return
    const parentPath = contextMenu.node.is_dir ? contextMenu.node.path : contextMenu.node.path.substring(0, contextMenu.node.path.lastIndexOf('/'))
    
    setEditingPath(parentPath + '/__new_folder__')
    setEditingValue('new_folder')
    setEditingAction('new-folder')
    setContextMenu(prev => ({ ...prev, visible: false }))
    
    if (contextMenu.node.is_dir && !expanded.has(contextMenu.node.path)) {
      await handleToggle(contextMenu.node.path, contextMenu.node)
    }
  }, [contextMenu.node, expanded, handleToggle])
  
  const handleRename = useCallback(() => {
    if (!contextMenu.node) return
    setEditingPath(contextMenu.node.path)
    setEditingValue(contextMenu.node.name)
    setEditingAction('rename')
    setContextMenu(prev => ({ ...prev, visible: false }))
  }, [contextMenu.node])
  
  const handleDelete = useCallback(async () => {
    if (!contextMenu.node) return
    
    const confirmMsg = contextMenu.node.is_dir 
      ? `确定要删除文件夹 "${contextMenu.node.name}" 及其所有内容吗？`
      : `确定要删除文件 "${contextMenu.node.name}" 吗？`
    
    if (!window.confirm(confirmMsg)) return
    
    try {
      await invoke('delete_file_or_dir', { path: contextMenu.node.path })
      onRefresh?.()
    } catch (e) {
      alert(`删除失败: ${e}`)
    }
    setContextMenu(prev => ({ ...prev, visible: false }))
  }, [contextMenu.node, onRefresh])
  
  const handleEditSubmit = useCallback(async () => {
    if (!editingPath || !editingValue.trim()) {
      setEditingPath(null)
      setEditingAction(null)
      return
    }
    
    try {
      if (editingAction === 'rename') {
        const newPath = editingPath.substring(0, editingPath.lastIndexOf('/') + 1) + editingValue
        await invoke('rename_file', { oldPath: editingPath, newPath })
      } else if (editingAction === 'new-file') {
        const parentPath = editingPath.replace('/__new_file__', '')
        const filePath = parentPath + '/' + editingValue
        await invoke('create_file', { path: filePath })
      } else if (editingAction === 'new-folder') {
        const parentPath = editingPath.replace('/__new_folder__', '')
        const dirPath = parentPath + '/' + editingValue
        await invoke('create_directory', { path: dirPath })
      }
      onRefresh?.()
    } catch (e) {
      alert(`操作失败: ${e}`)
    }
    
    setEditingPath(null)
    setEditingAction(null)
  }, [editingPath, editingValue, editingAction, onRefresh])
  
  const handleEditCancel = useCallback(() => {
    setEditingPath(null)
    setEditingAction(null)
  }, [])
  
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
          onContextMenu={handleContextMenu}
          editingPath={editingPath}
          editingValue={editingValue}
          onEditChange={setEditingValue}
          onEditSubmit={handleEditSubmit}
          onEditCancel={handleEditCancel}
        />
      ))}
      
      {/* Context Menu */}
      {contextMenu.visible && contextMenu.node && (
        <div
          className="file-context-menu"
          style={{
            position: 'fixed',
            left: contextMenu.x,
            top: contextMenu.y,
            zIndex: 1000,
          }}
          onClick={(e) => e.stopPropagation()}
        >
          <button onClick={handleNewFile}>
            <FilePlus className="w-3.5 h-3.5" strokeWidth={2} />
            新建文件
          </button>
          <button onClick={handleNewFolder}>
            <FolderPlus className="w-3.5 h-3.5" strokeWidth={2} />
            新建文件夹
          </button>
          <div className="menu-divider" />
          <button onClick={handleRename}>
            <Pencil className="w-3.5 h-3.5" strokeWidth={2} />
            重命名
          </button>
          <button onClick={handleDelete} className="danger">
            <Trash2 className="w-3.5 h-3.5" strokeWidth={2} />
            删除
          </button>
        </div>
      )}
    </div>
  )
}

