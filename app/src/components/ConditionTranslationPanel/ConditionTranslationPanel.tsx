/**
 * ConditionTranslationPanel - AI-assisted condition translation display
 * Shows AI-generated translations of constraint conditions to business semantics
 */

import { useState } from 'react'
import './ConditionTranslationPanel.css'

export interface ConditionTranslation {
  /** Original condition expression */
  original: string
  /** Business meaning translation */
  businessMeaning: string
  /** Trigger scenario description */
  triggerScenario: string
  /** AI confidence level (0-1) */
  confidence: number
}

export interface ConditionTranslationPanelProps {
  /** Title of the panel */
  title?: string
  /** Condition translations to display */
  translations: ConditionTranslation[]
  /** Function name context */
  functionName?: string
  /** Whether data is loading */
  loading?: boolean
  /** Error message if any */
  error?: string | null
  /** Callback when user provides feedback */
  onFeedback?: (original: string, correction: string) => void
  /** Callback when user requests re-translation */
  onRetry?: () => void
}

export function ConditionTranslationPanel({
  title = '条件翻译',
  translations,
  functionName,
  loading = false,
  error = null,
  onFeedback,
  onRetry
}: ConditionTranslationPanelProps) {
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const [feedbackMode, setFeedbackMode] = useState<string | null>(null)
  const [correctionText, setCorrectionText] = useState('')

  const handleToggle = (id: string) => {
    setExpandedId(expandedId === id ? null : id)
  }

  const handleFeedback = (original: string) => {
    if (onFeedback && correctionText.trim()) {
      onFeedback(original, correctionText)
      setFeedbackMode(null)
      setCorrectionText('')
    }
  }

  const getConfidenceLabel = (confidence: number): string => {
    if (confidence >= 0.9) return '高'
    if (confidence >= 0.7) return '中'
    return '低'
  }

  const getConfidenceClass = (confidence: number): string => {
    if (confidence >= 0.9) return 'confidence-high'
    if (confidence >= 0.7) return 'confidence-medium'
    return 'confidence-low'
  }

  // Render loading state
  if (loading) {
    return (
      <div className="condition-translation-panel">
        <div className="panel-header">
          <h3>{title}</h3>
          {functionName && <span className="function-context">{functionName}()</span>}
        </div>
        <div className="panel-loading">
          <div className="loading-spinner" />
          <span>AI 正在翻译条件...</span>
        </div>
      </div>
    )
  }

  // Render error state
  if (error) {
    return (
      <div className="condition-translation-panel">
        <div className="panel-header">
          <h3>{title}</h3>
          {functionName && <span className="function-context">{functionName}()</span>}
        </div>
        <div className="panel-error">
          <span className="error-icon">⚠</span>
          <span>{error}</span>
          {onRetry && (
            <button className="retry-btn" onClick={onRetry}>
              重试
            </button>
          )}
        </div>
      </div>
    )
  }

  // Render empty state
  if (!translations || translations.length === 0) {
    return (
      <div className="condition-translation-panel">
        <div className="panel-header">
          <h3>{title}</h3>
          {functionName && <span className="function-context">{functionName}()</span>}
        </div>
        <div className="panel-empty">
          <span className="empty-icon">🔍</span>
          <p>未检测到条件表达式</p>
        </div>
      </div>
    )
  }

  // Render translations list
  return (
    <div className="condition-translation-panel">
      <div className="panel-header">
        <h3>{title}</h3>
        {functionName && <span className="function-context">{functionName}()</span>}
        <span className="translation-count">{translations.length} 个条件</span>
      </div>

      <div className="translations-list">
        {translations.map((translation, index) => {
          const id = `translation-${index}`
          const isExpanded = expandedId === id
          const confidencePercent = Math.round(translation.confidence * 100)

          return (
            <div key={id} className={`translation-item ${isExpanded ? 'expanded' : ''}`}>
              {/* Header - always visible */}
              <div className="translation-header" onClick={() => handleToggle(id)}>
                <div className="translation-summary">
                  <code className="original-condition">
                    {translation.original}
                  </code>
                  <span className="arrow">{isExpanded ? '▼' : '▶'}</span>
                </div>
                <div className="translation-meta">
                  <span className={`confidence-badge ${getConfidenceClass(translation.confidence)}`}>
                    {getConfidenceLabel(translation.confidence)} {confidencePercent}%
                  </span>
                </div>
              </div>

              {/* Expanded content */}
              {isExpanded && (
                <div className="translation-content">
                  <div className="content-section">
                    <h4>业务含义</h4>
                    <p className="business-meaning">{translation.businessMeaning}</p>
                  </div>

                  <div className="content-section">
                    <h4>触发场景</h4>
                    <p className="trigger-scenario">{translation.triggerScenario}</p>
                  </div>

                  {/* Feedback section */}
                  <div className="feedback-section">
                    {feedbackMode === id ? (
                      <div className="feedback-form">
                        <textarea
                          value={correctionText}
                          onChange={(e) => setCorrectionText(e.target.value)}
                          placeholder="输入正确的翻译..."
                          className="correction-input"
                        />
                        <div className="feedback-actions">
                          <button
                            className="submit-btn"
                            onClick={() => handleFeedback(translation.original)}
                            disabled={!correctionText.trim()}
                          >
                            提交
                          </button>
                          <button
                            className="cancel-btn"
                            onClick={() => {
                              setFeedbackMode(null)
                              setCorrectionText('')
                            }}
                          >
                            取消
                          </button>
                        </div>
                      </div>
                    ) : (
                      <div className="feedback-buttons">
                        <span className="feedback-label">这个翻译正确吗？</span>
                        <button
                          className="thumbs-up-btn"
                          onClick={() => onFeedback?.(translation.original, '')}
                          title="正确"
                        >
                          👍
                        </button>
                        <button
                          className="thumbs-down-btn"
                          onClick={() => setFeedbackMode(id)}
                          title="有误"
                        >
                          👎
                        </button>
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          )
        })}
      </div>

      {/* Footer with info */}
      <div className="panel-footer">
        <span className="ai-badge">
          <span className="ai-icon">🤖</span>
          AI 生成
        </span>
        <span className="footer-hint">点击展开查看详情</span>
      </div>
    </div>
  )
}

export default ConditionTranslationPanel
