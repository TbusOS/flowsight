/**
 * ExecutionContextPanel - Execution context annotation display
 * Shows async mechanism type, execution context, and developer notes
 */

import { useState } from 'react'
import './ExecutionContextPanel.css'

export interface ExecutionContextAnnotation {
  /** Async mechanism type */
  mechanism: 'workqueue' | 'timer' | 'irq' | 'tasklet' | 'softirq' | 'threaded' | 'completion' | 'other'
  /** Handler function name */
  handler: string
  /** Execution context type */
  executionContext: 'process' | 'softirq' | 'hardirq' | 'interrupt' | 'timer'
  /** Whether the context is sleepable */
  isSleepable: boolean
  /** Trigger timing description */
  triggerTiming: string
  /** Developer notes/warnings */
  notes: string
  /** Related trigger code */
  triggerCode?: string
}

export interface ExecutionContextPanelProps {
  /** Title of the panel */
  title?: string
  /** Annotations to display */
  annotations: ExecutionContextAnnotation[]
  /** Whether data is loading */
  loading?: boolean
  /** Error message if any */
  error?: string | null
  /** Callback when user provides feedback */
  onFeedback?: (annotation: ExecutionContextAnnotation, correction: string) => void
  /** Callback when user requests refresh */
  onRefresh?: () => void
}

export function ExecutionContextPanel({
  title = '执行上下文标注',
  annotations,
  loading = false,
  error = null,
  onFeedback,
  onRefresh
}: ExecutionContextPanelProps) {
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const [feedbackMode, setFeedbackMode] = useState<string | null>(null)

  const getMechanismInfo = (mechanism: string) => {
    const info: Record<string, { label: string; color: string; icon: string }> = {
      workqueue: { label: 'WorkQueue', color: '#f59e0b', icon: '📋' },
      timer: { label: 'Timer', color: '#22c55e', icon: '⏱' },
      irq: { label: 'IRQ', color: '#ef4444', icon: '⚡' },
      tasklet: { label: 'Tasklet', color: '#a855f7', icon: '🚀' },
      softirq: { label: 'SoftIRQ', color: '#ec4899', icon: '🔄' },
      threaded: { label: 'Threaded', color: '#3b82f6', icon: '🧵' },
      completion: { label: 'Completion', color: '#14b8a6', icon: '✅' },
      other: { label: 'Other', color: '#6e7681', icon: '❓' }
    }
    return info[mechanism] || info.other
  }

  const getContextInfo = (context: string) => {
    const info: Record<string, { label: string; bgColor: string; textColor: string }> = {
      process: { label: '进程上下文', bgColor: '#23863633', textColor: '#3fb950' },
      softirq: { label: '软中断上下文', bgColor: '#a855f733', textColor: '#c084fc' },
      hardirq: { label: '硬中断上下文', bgColor: '#ef444433', textColor: '#f87171' },
      interrupt: { label: '中断上下文', bgColor: '#f59e0b33', textColor: '#fbbf24' },
      timer: { label: '定时器上下文', bgColor: '#22c55e33', textColor: '#4ade80' }
    }
    return info[context] || { label: context, bgColor: '#30363d', textColor: '#8b949e' }
  }

  const getSleepIndicator = (isSleepable: boolean) => {
    if (isSleepable) {
      return { label: '可睡眠', color: '#3fb950', icon: '😴' }
    }
    return { label: '不可睡眠', color: '#f85149', icon: '⏰' }
  }

  // Render loading state
  if (loading) {
    return (
      <div className="execution-context-panel">
        <div className="panel-header">
          <h3>{title}</h3>
        </div>
        <div className="panel-loading">
          <div className="loading-spinner" />
          <span>分析执行上下文...</span>
        </div>
      </div>
    )
  }

  // Render error state
  if (error) {
    return (
      <div className="execution-context-panel">
        <div className="panel-header">
          <h3>{title}</h3>
        </div>
        <div className="panel-error">
          <span className="error-icon">⚠</span>
          <span>{error}</span>
          {onRefresh && (
            <button className="refresh-btn" onClick={onRefresh}>
              刷新
            </button>
          )}
        </div>
      </div>
    )
  }

  // Render empty state
  if (!annotations || annotations.length === 0) {
    return (
      <div className="execution-context-panel">
        <div className="panel-header">
          <h3>{title}</h3>
        </div>
        <div className="panel-empty">
          <span className="empty-icon">🔍</span>
          <p>未检测到异步回调</p>
          <span className="empty-hint">函数可能是同步执行的</span>
        </div>
      </div>
    )
  }

  // Render annotations
  return (
    <div className="execution-context-panel">
      <div className="panel-header">
        <h3>{title}</h3>
        <span className="annotation-count">{annotations.length} 个回调</span>
      </div>

      <div className="annotations-list">
        {annotations.map((annotation, index) => {
          const id = `annotation-${index}`
          const isExpanded = expandedId === id
          const mechanismInfo = getMechanismInfo(annotation.mechanism)
          const contextInfo = getContextInfo(annotation.executionContext)
          const sleepInfo = getSleepIndicator(annotation.isSleepable)

          return (
            <div key={id} className={`annotation-item ${isExpanded ? 'expanded' : ''}`}>
              {/* Summary header */}
              <div
                className="annotation-summary"
                onClick={() => setExpandedId(isExpanded ? null : id)}
              >
                <div className="mechanism-badge" style={{ backgroundColor: `${mechanismInfo.color}22`, color: mechanismInfo.color }}>
                  <span className="mechanism-icon">{mechanismInfo.icon}</span>
                  <span className="mechanism-label">{mechanismInfo.label}</span>
                </div>

                <div className="handler-info">
                  <code className="handler-name">{annotation.handler}()</code>
                  {annotation.triggerCode && (
                    <span className="trigger-code">{annotation.triggerCode}</span>
                  )}
                </div>

                <div className="context-badges">
                  <span
                    className="context-badge"
                    style={{ backgroundColor: contextInfo.bgColor, color: contextInfo.textColor }}
                  >
                    {contextInfo.label}
                  </span>
                  <span
                    className="sleep-badge"
                    style={{ backgroundColor: `${sleepInfo.color}22`, color: sleepInfo.color }}
                  >
                    <span>{sleepInfo.icon}</span>
                    <span>{sleepInfo.label}</span>
                  </span>
                </div>

                <span className="expand-icon">{isExpanded ? '▼' : '▶'}</span>
              </div>

              {/* Expanded details */}
              {isExpanded && (
                <div className="annotation-details">
                  <div className="detail-section">
                    <h4>触发时机</h4>
                    <p>{annotation.triggerTiming}</p>
                  </div>

                  <div className="detail-section">
                    <h4>开发注意事项</h4>
                    <div className="notes-content">
                      {annotation.notes.split('\n').map((line, i) => (
                        <p key={i}>{line}</p>
                      ))}
                    </div>
                  </div>

                  {/* Quick info grid */}
                  <div className="info-grid">
                    <div className="info-item">
                      <span className="info-label">执行上下文</span>
                      <span className="info-value">{contextInfo.label}</span>
                    </div>
                    <div className="info-item">
                      <span className="info-label">是否可睡眠</span>
                      <span className="info-value">
                        <span style={{ color: sleepInfo.color }}>{sleepInfo.icon} {sleepInfo.label}</span>
                      </span>
                    </div>
                    <div className="info-item">
                      <span className="info-label">异步机制</span>
                      <span className="info-value">
                        <span style={{ color: mechanismInfo.color }}>{mechanismInfo.icon} {mechanismInfo.label}</span>
                      </span>
                    </div>
                  </div>

                  {/* Feedback */}
                  {onFeedback && (
                    <div className="feedback-section">
                      {feedbackMode === id ? (
                        <div className="feedback-form">
                          <textarea
                            placeholder="输入您的更正..."
                            className="feedback-input"
                          />
                          <div className="feedback-actions">
                            <button
                              className="submit-btn"
                              onClick={() => {
                                onFeedback(annotation, '')
                                setFeedbackMode(null)
                              }}
                            >
                              提交
                            </button>
                            <button
                              className="cancel-btn"
                              onClick={() => setFeedbackMode(null)}
                            >
                              取消
                            </button>
                          </div>
                        </div>
                      ) : (
                        <div className="feedback-actions-inline">
                          <span className="feedback-label">信息准确吗？</span>
                          <button
                            className="feedback-btn yes"
                            onClick={() => onFeedback(annotation, '')}
                          >
                            👍 正确
                          </button>
                          <button
                            className="feedback-btn no"
                            onClick={() => setFeedbackMode(id)}
                          >
                            👎 有误
                          </button>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              )}
            </div>
          )
        })}
      </div>

      <div className="panel-footer">
        <span className="ai-badge">
          <span className="ai-icon">🤖</span>
          AI 标注
        </span>
        <span className="footer-hint">点击展开查看详情</span>
      </div>
    </div>
  )
}

export default ExecutionContextPanel
