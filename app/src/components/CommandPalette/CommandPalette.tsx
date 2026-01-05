/**
 * CommandPalette - 命令面板组件
 * 
 * 类似 VS Code 的 Ctrl+P 功能
 * 支持搜索文件和符号
 */

import { useState, useEffect, useRef, useCallback, useMemo } from 'react'
import './CommandPalette.css'

interface CommandItem {
  id: string
  type: 'file' | 'symbol' | 'command'
  name: string
  description?: string
  icon: string
  path?: string
  line?: number
}

interface CommandPaletteProps {
  isOpen: boolean
  onClose: () => void
  onSelect: (item: CommandItem) => void
  files: Array<{ name: string; path: string; isDir: boolean }>
  symbols: Array<{ name: string; kind: string; file?: string; line?: number; isCallback?: boolean }>
}

// 简单的模糊匹配函数
function fuzzyMatch(query: string, text: string): { matched: boolean; score: number } {
  const lowerQuery = query.toLowerCase()
  const lowerText = text.toLowerCase()
  
  // 精确包含
  if (lowerText.includes(lowerQuery)) {
    const index = lowerText.indexOf(lowerQuery)
    // 起始位置越靠前，分数越高
    return { matched: true, score: 100 - index + (lowerQuery === lowerText ? 50 : 0) }
  }
  
  // 首字母匹配
  let queryIndex = 0
  let score = 0
  for (let i = 0; i < lowerText.length && queryIndex < lowerQuery.length; i++) {
    if (lowerText[i] === lowerQuery[queryIndex]) {
      queryIndex++
      score += 10
      // 连续匹配加分
      if (i > 0 && lowerText[i - 1] === lowerQuery[queryIndex - 2]) {
        score += 5
      }
    }
  }
  
  if (queryIndex === lowerQuery.length) {
    return { matched: true, score }
  }
  
  return { matched: false, score: 0 }
}

export function CommandPalette({ 
  isOpen, 
  onClose, 
  onSelect, 
  files, 
  symbols 
}: CommandPaletteProps) {
  const [query, setQuery] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)
  const listRef = useRef<HTMLDivElement>(null)

  // 构建搜索项
  const allItems = useMemo<CommandItem[]>(() => {
    const items: CommandItem[] = []
    
    // 添加文件
    files.forEach(f => {
      if (!f.isDir) {
        items.push({
          id: `file:${f.path}`,
          type: 'file',
          name: f.name,
          description: f.path,
          icon: getFileIcon(f.name),
          path: f.path,
        })
      }
    })
    
    // 添加符号
    symbols.forEach(s => {
      items.push({
        id: `symbol:${s.name}:${s.file}:${s.line}`,
        type: 'symbol',
        name: s.name,
        description: s.file ? `${s.file.split('/').pop()}:${s.line}` : s.kind,
        icon: s.isCallback ? '⚡' : (s.kind === 'function' ? '📦' : '🏗️'),
        path: s.file,
        line: s.line,
      })
    })
    
    return items
  }, [files, symbols])

  // 过滤和排序结果
  const filteredItems = useMemo(() => {
    if (!query.trim()) {
      // 无查询时，显示最近的或前20个符号
      return allItems.filter(item => item.type === 'symbol').slice(0, 20)
    }
    
    // 根据前缀判断搜索类型
    let searchQuery = query
    let typeFilter: 'file' | 'symbol' | null = null
    
    if (query.startsWith('@')) {
      // @ 搜索符号
      searchQuery = query.slice(1)
      typeFilter = 'symbol'
    } else if (query.startsWith('>')) {
      // > 搜索命令 (暂不实现)
      return []
    }
    
    const results: Array<CommandItem & { score: number }> = []
    
    for (const item of allItems) {
      if (typeFilter && item.type !== typeFilter) continue
      
      const match = fuzzyMatch(searchQuery, item.name)
      if (match.matched) {
        results.push({ ...item, score: match.score })
      }
    }
    
    // 按分数排序
    results.sort((a, b) => b.score - a.score)
    
    return results.slice(0, 30)
  }, [query, allItems])

  // 打开时聚焦输入框
  useEffect(() => {
    if (isOpen) {
      setQuery('')
      setSelectedIndex(0)
      setTimeout(() => inputRef.current?.focus(), 50)
    }
  }, [isOpen])

  // 重置选中索引
  useEffect(() => {
    setSelectedIndex(0)
  }, [filteredItems.length])

  // 确保选中项可见
  useEffect(() => {
    if (listRef.current) {
      const selected = listRef.current.querySelector('.palette-item.selected')
      selected?.scrollIntoView({ block: 'nearest' })
    }
  }, [selectedIndex])

  // 键盘导航
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        setSelectedIndex(prev => Math.min(prev + 1, filteredItems.length - 1))
        break
      case 'ArrowUp':
        e.preventDefault()
        setSelectedIndex(prev => Math.max(prev - 1, 0))
        break
      case 'Enter':
        e.preventDefault()
        if (filteredItems[selectedIndex]) {
          onSelect(filteredItems[selectedIndex])
          onClose()
        }
        break
      case 'Escape':
        e.preventDefault()
        onClose()
        break
    }
  }, [filteredItems, selectedIndex, onSelect, onClose])

  if (!isOpen) return null

  return (
    <div className="command-palette-overlay" onClick={onClose}>
      <div className="command-palette" onClick={e => e.stopPropagation()}>
        <div className="palette-input-container">
          <span className="palette-icon">🔍</span>
          <input
            ref={inputRef}
            type="text"
            className="palette-input"
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="搜索文件或符号... (@ 搜符号)"
          />
          <span className="palette-hint">ESC 关闭</span>
        </div>
        
        <div className="palette-list" ref={listRef}>
          {filteredItems.length === 0 ? (
            <div className="palette-empty">
              {query ? '未找到匹配项' : '开始输入以搜索...'}
            </div>
          ) : (
            filteredItems.map((item, index) => (
              <div
                key={item.id}
                className={`palette-item ${index === selectedIndex ? 'selected' : ''}`}
                onClick={() => {
                  onSelect(item)
                  onClose()
                }}
                onMouseEnter={() => setSelectedIndex(index)}
              >
                <span className="item-icon">{item.icon}</span>
                <div className="item-content">
                  <span className="item-name">{item.name}</span>
                  {item.description && (
                    <span className="item-desc">{item.description}</span>
                  )}
                </div>
                <span className="item-type">{item.type === 'file' ? '文件' : '符号'}</span>
              </div>
            ))
          )}
        </div>
        
        <div className="palette-footer">
          <span>↑↓ 导航</span>
          <span>↵ 选择</span>
          <span>@ 符号</span>
        </div>
      </div>
    </div>
  )
}

// 根据文件名获取图标
function getFileIcon(name: string): string {
  const ext = name.split('.').pop()?.toLowerCase()
  switch (ext) {
    case 'c':
    case 'cpp':
    case 'cc':
    case 'cxx':
      return '📄'
    case 'h':
    case 'hpp':
      return '📋'
    case 'rs':
      return '🦀'
    case 'py':
      return '🐍'
    case 'js':
    case 'ts':
    case 'tsx':
      return '📜'
    case 'json':
      return '📦'
    case 'md':
      return '📝'
    case 'txt':
      return '📃'
    default:
      return '📄'
  }
}

export default CommandPalette

