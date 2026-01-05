/**
 * 跳转行号组件
 */

import { useState, useEffect, useRef } from 'react'
import './GoToLine.css'

interface GoToLineProps {
  isOpen: boolean
  onClose: () => void
  onGoTo: (line: number) => void
  totalLines: number
}

export function GoToLine({ isOpen, onClose, onGoTo, totalLines }: GoToLineProps) {
  const [value, setValue] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus()
      inputRef.current.select()
    }
    setValue('')
  }, [isOpen])

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    const line = parseInt(value, 10)
    if (!isNaN(line) && line >= 1 && line <= totalLines) {
      onGoTo(line)
      onClose()
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose()
    }
  }

  if (!isOpen) return null

  const line = parseInt(value, 10)
  const isValid = !isNaN(line) && line >= 1 && line <= totalLines

  return (
    <div className="goto-overlay" onClick={onClose}>
      <div className="goto-panel" onClick={e => e.stopPropagation()}>
        <form onSubmit={handleSubmit}>
          <div className="goto-header">
            跳转到行号 <span className="total-hint">(1 - {totalLines})</span>
          </div>
          <input
            ref={inputRef}
            type="text"
            className={`goto-input ${value && !isValid ? 'invalid' : ''}`}
            value={value}
            onChange={e => setValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="输入行号..."
            autoFocus
          />
          {value && !isValid && (
            <div className="goto-error">
              请输入 1 到 {totalLines} 之间的行号
            </div>
          )}
        </form>
      </div>
    </div>
  )
}

export default GoToLine

