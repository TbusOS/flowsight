/**
 * 查找/替换组件
 */

import { useState, useCallback, useEffect, useRef } from 'react'
import './FindReplace.css'

interface FindReplaceProps {
  isOpen: boolean
  onClose: () => void
  content: string
  onFindResult: (matches: FindMatch[]) => void
  onReplaceAll: (newContent: string) => void
  onGoToMatch: (match: FindMatch) => void
}

export interface FindMatch {
  line: number
  column: number
  length: number
  text: string
  lineContent: string
}

export function FindReplace({ 
  isOpen, 
  onClose, 
  content, 
  onFindResult, 
  onReplaceAll,
  onGoToMatch 
}: FindReplaceProps) {
  const [searchText, setSearchText] = useState('')
  const [replaceText, setReplaceText] = useState('')
  const [caseSensitive, setCaseSensitive] = useState(false)
  const [useRegex, setUseRegex] = useState(false)
  const [wholeWord, setWholeWord] = useState(false)
  const [matches, setMatches] = useState<FindMatch[]>([])
  const [currentMatchIndex, setCurrentMatchIndex] = useState(0)
  const [showReplace, setShowReplace] = useState(false)
  
  const inputRef = useRef<HTMLInputElement>(null)

  // 聚焦到输入框
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus()
      inputRef.current.select()
    }
  }, [isOpen])

  // 执行搜索
  const doSearch = useCallback(() => {
    if (!searchText || !content) {
      setMatches([])
      onFindResult([])
      return
    }

    try {
      let pattern: RegExp
      
      if (useRegex) {
        pattern = new RegExp(searchText, caseSensitive ? 'g' : 'gi')
      } else {
        // 转义正则特殊字符
        let escaped = searchText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
        if (wholeWord) {
          escaped = `\\b${escaped}\\b`
        }
        pattern = new RegExp(escaped, caseSensitive ? 'g' : 'gi')
      }

      const lines = content.split('\n')
      const newMatches: FindMatch[] = []

      lines.forEach((lineContent, lineIndex) => {
        let match
        while ((match = pattern.exec(lineContent)) !== null) {
          newMatches.push({
            line: lineIndex + 1,
            column: match.index + 1,
            length: match[0].length,
            text: match[0],
            lineContent: lineContent.trim()
          })
          // 防止零长度匹配死循环
          if (match[0].length === 0) break
        }
      })

      setMatches(newMatches)
      setCurrentMatchIndex(0)
      onFindResult(newMatches)
    } catch (e) {
      // 正则表达式无效
      setMatches([])
      onFindResult([])
    }
  }, [searchText, content, caseSensitive, useRegex, wholeWord, onFindResult])

  // 搜索文本变化时自动搜索
  useEffect(() => {
    const timer = setTimeout(doSearch, 100)
    return () => clearTimeout(timer)
  }, [doSearch])

  // 上一个匹配
  const goToPrev = useCallback(() => {
    if (matches.length === 0) return
    const newIndex = (currentMatchIndex - 1 + matches.length) % matches.length
    setCurrentMatchIndex(newIndex)
    onGoToMatch(matches[newIndex])
  }, [matches, currentMatchIndex, onGoToMatch])

  // 下一个匹配
  const goToNext = useCallback(() => {
    if (matches.length === 0) return
    const newIndex = (currentMatchIndex + 1) % matches.length
    setCurrentMatchIndex(newIndex)
    onGoToMatch(matches[newIndex])
  }, [matches, currentMatchIndex, onGoToMatch])

  // 替换当前
  const replaceCurrent = useCallback(() => {
    if (matches.length === 0 || !content) return
    
    const match = matches[currentMatchIndex]
    const lines = content.split('\n')
    const lineIndex = match.line - 1
    const line = lines[lineIndex]
    
    // 替换当前行中的匹配
    const before = line.substring(0, match.column - 1)
    const after = line.substring(match.column - 1 + match.length)
    lines[lineIndex] = before + replaceText + after
    
    onReplaceAll(lines.join('\n'))
    // 搜索会自动更新
  }, [matches, currentMatchIndex, content, replaceText, onReplaceAll])

  // 替换全部
  const replaceAll = useCallback(() => {
    if (!searchText || !content) return
    
    try {
      let pattern: RegExp
      
      if (useRegex) {
        pattern = new RegExp(searchText, caseSensitive ? 'g' : 'gi')
      } else {
        let escaped = searchText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
        if (wholeWord) {
          escaped = `\\b${escaped}\\b`
        }
        pattern = new RegExp(escaped, caseSensitive ? 'g' : 'gi')
      }

      const newContent = content.replace(pattern, replaceText)
      onReplaceAll(newContent)
    } catch (e) {
      // 正则表达式无效
    }
  }, [searchText, replaceText, content, caseSensitive, useRegex, wholeWord, onReplaceAll])

  // 键盘快捷键
  useEffect(() => {
    if (!isOpen) return

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose()
      } else if (e.key === 'Enter') {
        if (e.shiftKey) {
          goToPrev()
        } else {
          goToNext()
        }
      } else if (e.key === 'F3') {
        e.preventDefault()
        if (e.shiftKey) {
          goToPrev()
        } else {
          goToNext()
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, onClose, goToPrev, goToNext])

  if (!isOpen) return null

  return (
    <div className="find-replace-panel">
      <div className="find-row">
        <input
          ref={inputRef}
          type="text"
          className="find-input"
          value={searchText}
          onChange={e => setSearchText(e.target.value)}
          placeholder="查找..."
        />
        <div className="find-options">
          <button 
            className={`option-btn ${caseSensitive ? 'active' : ''}`}
            onClick={() => setCaseSensitive(!caseSensitive)}
            title="区分大小写 (Aa)"
          >
            Aa
          </button>
          <button 
            className={`option-btn ${wholeWord ? 'active' : ''}`}
            onClick={() => setWholeWord(!wholeWord)}
            title="全字匹配 (Ab|)"
          >
            Ab
          </button>
          <button 
            className={`option-btn ${useRegex ? 'active' : ''}`}
            onClick={() => setUseRegex(!useRegex)}
            title="正则表达式 (.*)"
          >
            .*
          </button>
        </div>
        <div className="find-nav">
          <span className="match-count">
            {matches.length > 0 
              ? `${currentMatchIndex + 1}/${matches.length}` 
              : searchText ? '无结果' : ''
            }
          </span>
          <button 
            className="nav-btn" 
            onClick={goToPrev}
            disabled={matches.length === 0}
            title="上一个 (Shift+Enter)"
          >
            ↑
          </button>
          <button 
            className="nav-btn" 
            onClick={goToNext}
            disabled={matches.length === 0}
            title="下一个 (Enter)"
          >
            ↓
          </button>
        </div>
        <button 
          className="toggle-replace-btn"
          onClick={() => setShowReplace(!showReplace)}
          title={showReplace ? '隐藏替换' : '显示替换'}
        >
          {showReplace ? '▼' : '▶'}
        </button>
        <button className="close-btn" onClick={onClose} title="关闭 (Esc)">✕</button>
      </div>
      
      {showReplace && (
        <div className="replace-row">
          <input
            type="text"
            className="find-input"
            value={replaceText}
            onChange={e => setReplaceText(e.target.value)}
            placeholder="替换为..."
          />
          <div className="replace-actions">
            <button 
              className="replace-btn"
              onClick={replaceCurrent}
              disabled={matches.length === 0}
              title="替换当前"
            >
              替换
            </button>
            <button 
              className="replace-btn"
              onClick={replaceAll}
              disabled={matches.length === 0}
              title="替换全部"
            >
              全部替换
            </button>
          </div>
        </div>
      )}

      {/* 匹配预览列表 */}
      {matches.length > 0 && matches.length <= 100 && (
        <div className="matches-list">
          {matches.slice(0, 50).map((match, i) => (
            <div 
              key={i}
              className={`match-item ${i === currentMatchIndex ? 'current' : ''}`}
              onClick={() => {
                setCurrentMatchIndex(i)
                onGoToMatch(match)
              }}
            >
              <span className="match-line">L{match.line}</span>
              <span className="match-preview">{match.lineContent.substring(0, 60)}</span>
            </div>
          ))}
          {matches.length > 50 && (
            <div className="match-more">还有 {matches.length - 50} 个匹配...</div>
          )}
        </div>
      )}
    </div>
  )
}

export default FindReplace

