/**
 * 标签栏组件
 * 
 * 支持多文件编辑，可切换、关闭、拖拽排序
 */

import { useRef, useState } from 'react'
import './TabBar.css'

export interface Tab {
  id: string
  filePath: string
  fileName: string
  isDirty: boolean
}

interface TabBarProps {
  tabs: Tab[]
  activeTabId: string | null
  onTabSelect: (tabId: string) => void
  onTabClose: (tabId: string) => void
  onTabReorder?: (fromIndex: number, toIndex: number) => void
}

export function TabBar({ 
  tabs, 
  activeTabId, 
  onTabSelect, 
  onTabClose,
  onTabReorder 
}: TabBarProps) {
  const [dragIndex, setDragIndex] = useState<number | null>(null)
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null)
  const tabsRef = useRef<HTMLDivElement>(null)

  const handleDragStart = (e: React.DragEvent, index: number) => {
    setDragIndex(index)
    e.dataTransfer.effectAllowed = 'move'
    // 设置拖拽图像
    const target = e.target as HTMLElement
    e.dataTransfer.setDragImage(target, 50, 15)
  }

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault()
    if (dragIndex !== null && dragIndex !== index) {
      setDragOverIndex(index)
    }
  }

  const handleDragEnd = () => {
    if (dragIndex !== null && dragOverIndex !== null && onTabReorder) {
      onTabReorder(dragIndex, dragOverIndex)
    }
    setDragIndex(null)
    setDragOverIndex(null)
  }

  const handleClose = (e: React.MouseEvent, tabId: string) => {
    e.stopPropagation()
    onTabClose(tabId)
  }

  // 获取文件图标
  const getFileIcon = (fileName: string): string => {
    const ext = fileName.split('.').pop()?.toLowerCase()
    switch (ext) {
      case 'c': return '📄'
      case 'h': return '📋'
      case 'cpp':
      case 'cc':
      case 'cxx': return '📄'
      case 'rs': return '🦀'
      case 'py': return '🐍'
      case 'js':
      case 'ts': return '📜'
      case 'md': return '📝'
      case 'json': return '⚙️'
      default: return '📄'
    }
  }

  if (tabs.length === 0) {
    return null
  }

  return (
    <div className="tab-bar" ref={tabsRef}>
      <div className="tabs-container">
        {tabs.map((tab, index) => (
          <div
            key={tab.id}
            className={`tab ${tab.id === activeTabId ? 'active' : ''} ${dragOverIndex === index ? 'drag-over' : ''}`}
            onClick={() => onTabSelect(tab.id)}
            draggable
            onDragStart={(e) => handleDragStart(e, index)}
            onDragOver={(e) => handleDragOver(e, index)}
            onDragEnd={handleDragEnd}
            onDragLeave={() => setDragOverIndex(null)}
            title={tab.filePath}
          >
            <span className="tab-icon">{getFileIcon(tab.fileName)}</span>
            <span className="tab-name">
              {tab.fileName}
              {tab.isDirty && <span className="dirty-indicator">●</span>}
            </span>
            <button 
              className="tab-close"
              onClick={(e) => handleClose(e, tab.id)}
              title="关闭"
            >
              ✕
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}

export default TabBar

