/**
 * ProgressIndicator - 索引进度指示器
 */

import { useEffect, useState } from 'react'
import './ProgressIndicator.css'

export interface ProgressEvent {
  phase: string
  current: number
  total: number
  message: string
  current_file?: string
}

interface ProgressIndicatorProps {
  progress: ProgressEvent | null
  onCancel?: () => void
}

const phaseLabels: Record<string, string> = {
  Scanning: '🔍 扫描文件',
  Parsing: '📝 解析代码',
  Analyzing: '🔬 分析模式',
  Indexing: '📊 构建索引',
  BuildingGraph: '🔗 构建调用图',
  Complete: '✅ 完成',
}

export function ProgressIndicator({ progress, onCancel }: ProgressIndicatorProps) {
  const [elapsed, setElapsed] = useState(0)

  useEffect(() => {
    if (!progress || progress.phase === 'Complete') {
      setElapsed(0)
      return
    }

    const start = Date.now()
    const timer = setInterval(() => {
      setElapsed(Math.floor((Date.now() - start) / 1000))
    }, 1000)

    return () => clearInterval(timer)
  }, [progress?.phase])

  if (!progress) {
    return null
  }

  const percentage = progress.total > 0 
    ? Math.round((progress.current / progress.total) * 100) 
    : 0

  const phaseLabel = phaseLabels[progress.phase] || progress.phase

  const formatTime = (seconds: number) => {
    if (seconds < 60) return `${seconds}s`
    const mins = Math.floor(seconds / 60)
    const secs = seconds % 60
    return `${mins}m ${secs}s`
  }

  if (progress.phase === 'Complete') {
    return (
      <div className="progress-indicator complete">
        <div className="progress-header">
          <span className="phase-label">{phaseLabel}</span>
          <span className="time">{formatTime(elapsed)}</span>
        </div>
        <div className="progress-message success">{progress.message}</div>
      </div>
    )
  }

  return (
    <div className="progress-indicator">
      <div className="progress-header">
        <span className="phase-label">{phaseLabel}</span>
        <span className="percentage">{percentage}%</span>
        <span className="time">{formatTime(elapsed)}</span>
      </div>

      <div className="progress-bar-container">
        <div 
          className="progress-bar-fill" 
          style={{ width: `${percentage}%` }}
        />
      </div>

      <div className="progress-details">
        <span className="count">
          {progress.current.toLocaleString()} / {progress.total.toLocaleString()}
        </span>
        <span className="message">{progress.message}</span>
      </div>

      {progress.current_file && (
        <div className="current-file">
          <code>{progress.current_file}</code>
        </div>
      )}

      {onCancel && (
        <button className="cancel-button" onClick={onCancel}>
          取消
        </button>
      )}
    </div>
  )
}

export default ProgressIndicator

