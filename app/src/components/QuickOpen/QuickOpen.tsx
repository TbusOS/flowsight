/**
 * 快速打开文件组件
 * 
 * Ctrl+E 打开，搜索并快速跳转到最近打开的文件
 */

import { useState, useEffect, useMemo, useRef } from 'react'
import './QuickOpen.css'

interface RecentFile {
  path: string
  name: string
  timestamp: number
}

interface QuickOpenProps {
  isOpen: boolean
  onClose: () => void
  recentFiles: RecentFile[]
  onSelect: (path: string) => void
}

export function QuickOpen({ isOpen, onClose, recentFiles, onSelect }: QuickOpenProps) {
  const [query, setQuery] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)

  // 过滤文件
  const filteredFiles = useMemo(() => {
    if (!query.trim()) return recentFiles
    
    const lowerQuery = query.toLowerCase()
    return recentFiles.filter(file => 
      file.name.toLowerCase().includes(lowerQuery) ||
      file.path.toLowerCase().includes(lowerQuery)
    )
  }, [query, recentFiles])

  // 重置状态
  useEffect(() => {
    if (isOpen) {
      setQuery('')
      setSelectedIndex(0)
      setTimeout(() => inputRef.current?.focus(), 50)
    }
  }, [isOpen])

  // 键盘导航
  useEffect(() => {
    if (!isOpen) return

    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'ArrowDown':
          e.preventDefault()
          setSelectedIndex(prev => 
            Math.min(prev + 1, filteredFiles.length - 1)
          )
          break
        case 'ArrowUp':
          e.preventDefault()
          setSelectedIndex(prev => Math.max(prev - 1, 0))
          break
        case 'Enter':
          e.preventDefault()
          if (filteredFiles[selectedIndex]) {
            onSelect(filteredFiles[selectedIndex].path)
            onClose()
          }
          break
        case 'Escape':
          onClose()
          break
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, filteredFiles, selectedIndex, onSelect, onClose])

  // 更新选中索引当过滤结果变化时
  useEffect(() => {
    setSelectedIndex(0)
  }, [query])

  if (!isOpen) return null

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diffMs = now.getTime() - timestamp
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMins / 60)
    const diffDays = Math.floor(diffHours / 24)

    if (diffMins < 1) return '刚刚'
    if (diffMins < 60) return `${diffMins} 分钟前`
    if (diffHours < 24) return `${diffHours} 小时前`
    if (diffDays < 7) return `${diffDays} 天前`
    return date.toLocaleDateString()
  }

  return (
    <div className="quick-open-overlay" onClick={onClose}>
      <div className="quick-open" onClick={e => e.stopPropagation()}>
        <div className="quick-open-header">
          <input
            ref={inputRef}
            type="text"
            className="quick-open-input"
            placeholder="搜索最近打开的文件..."
            value={query}
            onChange={e => setQuery(e.target.value)}
          />
        </div>
        
        <div className="quick-open-list">
          {filteredFiles.length === 0 ? (
            <div className="quick-open-empty">
              {query ? '没有匹配的文件' : '暂无最近打开的文件'}
            </div>
          ) : (
            filteredFiles.map((file, index) => (
              <div
                key={file.path}
                className={`quick-open-item ${index === selectedIndex ? 'selected' : ''}`}
                onClick={() => {
                  onSelect(file.path)
                  onClose()
                }}
                onMouseEnter={() => setSelectedIndex(index)}
              >
                <span className="file-icon">📄</span>
                <div className="file-info">
                  <span className="file-name">{file.name}</span>
                  <span className="file-path">{file.path}</span>
                </div>
                <span className="file-time">{formatTime(file.timestamp)}</span>
              </div>
            ))
          )}
        </div>
        
        <div className="quick-open-footer">
          <span><kbd>↑↓</kbd> 导航</span>
          <span><kbd>Enter</kbd> 打开</span>
          <span><kbd>Esc</kbd> 关闭</span>
        </div>
      </div>
    </div>
  )
}

export default QuickOpen

