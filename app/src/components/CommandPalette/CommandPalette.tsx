/**
 * CommandPalette - 命令面板组件
 *
 * 类似 VS Code 的 Ctrl+P 功能
 * 支持搜索文件、符号和 Agent 命令
 *
 * 前缀：
 *   @ 搜索符号
 *   > 执行命令
 *   ? AI 对话 (Agent)
 */

import { useState, useEffect, useRef, useCallback, useMemo } from 'react'
import {
  Search,
  Command,
  File,
  FunctionSquare,
  Bot,
  ChevronRight,
  X,
  ArrowRight,
  FolderOpen,
  Zap,
} from 'lucide-react'
import './CommandPalette.css'

interface CommandItem {
  id: string
  type: 'file' | 'symbol' | 'command' | 'agent'
  name: string
  description?: string
  icon: React.ReactNode
  path?: string
  line?: number
  detail?: string
}

// Agent 命令定义
const BUILTIN_COMMANDS: CommandItem[] = [
  // 文件操作
  { id: 'cmd:openFile', type: 'command', name: '打开文件', icon: <FolderOpen className="w-4 h-4" />, description: '打开一个文件', detail: 'Ctrl+O' },
  { id: 'cmd:saveFile', type: 'command', name: '保存文件', icon: <File className="w-4 h-4" />, description: '保存当前文件', detail: 'Ctrl+S' },
  { id: 'cmd:closeTab', type: 'command', name: '关闭标签页', icon: <X className="w-4 h-4" />, description: '关闭当前标签页', detail: 'Ctrl+W' },

  // 视图操作
  { id: 'cmd:toggleSidebar', type: 'command', name: '切换侧边栏', icon: <ChevronRight className="w-4 h-4" />, description: '显示/隐藏侧边栏', detail: 'Ctrl+\\' },
  { id: 'cmd:togglePanel', type: 'command', name: '切换底部面板', icon: <Command className="w-4 h-4" />, description: '显示/隐藏底部面板', detail: 'Ctrl+`' },

  // 导航
  { id: 'cmd:goBack', type: 'command', name: '后退', icon: <ArrowRight className="w-4 h-4 rotate-180" />, description: '后退到上一个位置', detail: 'Alt+←' },
  { id: 'cmd:goForward', type: 'command', name: '前进', icon: <ArrowRight className="w-4 h-4" />, description: '前进到下一个位置', detail: 'Alt+→' },

  // 分析
  { id: 'cmd:analyzeFile', type: 'command', name: '分析当前文件', icon: <Search className="w-4 h-4" />, description: '分析当前文件的执行流', detail: 'F5' },

  // Agent 命令
  { id: 'agent:analyze', type: 'agent', name: 'AI 分析', icon: <Bot className="w-4 h-4" />, description: '让 AI 分析代码执行流', detail: 'Agent' },
  { id: 'agent:explain', type: 'agent', name: 'AI 解释', icon: <Bot className="w-4 h-4" />, description: '让 AI 解释代码逻辑', detail: 'Agent' },

  // 设置
  { id: 'cmd:shortcuts', type: 'command', name: '键盘快捷键', icon: <Command className="w-4 h-4" />, description: '查看所有快捷键', detail: 'Ctrl+K Ctrl+S' },
]

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

    // 添加内置命令
    items.push(...BUILTIN_COMMANDS)

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
        icon: s.isCallback ? <Zap className="w-4 h-4" /> : <FunctionSquare className="w-4 h-4" />,
        path: s.file,
        line: s.line,
      })
    })

    return items
  }, [files, symbols])

  // 过滤和排序结果
  const filteredItems = useMemo(() => {
    if (!query.trim()) {
      // 无查询时，显示命令和符号
      return allItems.filter(item =>
        item.type === 'command' || item.type === 'agent'
      ).slice(0, 15)
    }

    // 根据前缀判断搜索类型
    let searchQuery = query
    let typeFilter: 'file' | 'symbol' | 'command' | 'agent' | null = null

    if (query.startsWith('>')) {
      // > 搜索命令
      searchQuery = query.slice(1)
      typeFilter = 'command'
    } else if (query.startsWith('?')) {
      // ? AI 对话
      searchQuery = query.slice(1)
      typeFilter = 'agent'
    } else if (query.startsWith('@')) {
      // @ 搜索符号
      searchQuery = query.slice(1)
      typeFilter = 'symbol'
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
          <Search className="palette-icon w-4 h-4" strokeWidth={2} />
          <input
            ref={inputRef}
            type="text"
            className="palette-input"
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="搜索文件、符号或命令... (@ 符号, > 命令, ? AI)"
          />
          <span className="palette-hint">ESC 关闭</span>
        </div>

        <div className="palette-list" ref={listRef}>
          {filteredItems.length === 0 ? (
            <div className="palette-empty">
              {query ? '未找到匹配项' : '开始输入以搜索，或使用前缀：@ 符号, > 命令, ? AI'}
            </div>
          ) : (
            filteredItems.map((item, index) => (
              <div
                key={item.id}
                className={`palette-item ${index === selectedIndex ? 'selected' : ''} ${item.type}`}
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
                <div className="item-right">
                  {item.detail && <span className="item-shortcut">{item.detail}</span>}
                  <span className={`item-type type-${item.type}`}>{item.type === 'command' ? '命令' : item.type === 'file' ? '文件' : item.type === 'symbol' ? '符号' : 'AI'}</span>
                </div>
              </div>
            ))
          )}
        </div>

        <div className="palette-footer">
          <span>↑↓ 导航</span>
          <span>↵ 选择</span>
          <span>@ 符号</span>
          <span>&gt; 命令</span>
          <span>? AI</span>
        </div>
      </div>
    </div>
  )
}

// 根据文件名获取图标
function getFileIcon(name: string): React.ReactNode {
  const ext = name.split('.').pop()?.toLowerCase()
  switch (ext) {
    case 'c':
    case 'cpp':
    case 'cc':
    case 'cxx':
      return <File className="w-4 h-4 text-blue-400" />
    case 'h':
    case 'hpp':
      return <File className="w-4 h-4 text-blue-300" />
    case 'rs':
      return <File className="w-4 h-4 text-orange-500" />
    case 'py':
      return <File className="w-4 h-4 text-yellow-400" />
    case 'js':
    case 'ts':
    case 'tsx':
      return <File className="w-4 h-4 text-yellow-300" />
    case 'json':
      return <File className="w-4 h-4 text-yellow-500" />
    case 'md':
      return <File className="w-4 h-4 text-slate-400" />
    default:
      return <File className="w-4 h-4 text-slate-400" />
  }
}

export default CommandPalette

