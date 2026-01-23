/**
 * BusinessSemanticsPanel - AI-powered function business semantics explanation
 * Explains what a function does in business terms
 */

import { useState } from 'react'
import './BusinessSemanticsPanel.css'

export interface BusinessExplanation {
  /** Function trigger condition */
  triggerCondition: string
  /** Business meaning description */
  businessMeaning: string
  /** Expected execution results */
  executionResult: string
  /** Related functions */
  relatedFunctions: string[]
  /** Common error scenarios */
  commonErrors: string[]
}

export interface BusinessSemanticsPanelProps {
  /** Title of the panel */
  title?: string
  /** Function name */
  functionName?: string
  /** Business explanation data */
  explanation?: BusinessExplanation | null
  /** Whether data is loading */
  loading?: boolean
  /** Error message if any */
  error?: string | null
  /** Source code context */
  codeSnippet?: string
  /** Callback when user provides feedback */
  onFeedback?: (field: string, correction: string) => void
  /** Callback when user requests re-explanation */
  onRetry?: () => void
}

export function BusinessSemanticsPanel({
  title = '业务语义解释',
  functionName,
  explanation,
  loading = false,
  error = null,
  codeSnippet,
  onFeedback,
  onRetry
}: BusinessSemanticsPanelProps) {
  const [expanded, setExpanded] = useState(true)
  const [editingField, setEditingField] = useState<string | null>(null)
  const [editText, setEditText] = useState('')

  const handleEdit = (field: string, currentValue: string) => {
    setEditingField(field)
    setEditText(currentValue)
  }

  const handleSave = (field: string) => {
    if (onFeedback && editText.trim()) {
      onFeedback(field, editText)
    }
    setEditingField(null)
    setEditText('')
  }

  const handleCancel = () => {
    setEditingField(null)
    setEditText('')
  }

  const getFieldIcon = (field: string): string => {
    switch (field) {
      case 'trigger': return '⚡'
      case 'meaning': return '💡'
      case 'result': return '✅'
      case 'related': return '🔗'
      case 'errors': return '⚠'
      default: return '📝'
    }
  }

  // Render loading state
  if (loading) {
    return (
      <div className="business-semantics-panel">
        <div className="panel-header">
          <h3>{title}</h3>
          {functionName && <span className="function-badge">{functionName}()</span>}
        </div>
        <div className="panel-loading">
          <div className="loading-spinner" />
          <div className="loading-text">
            <span>AI 正在分析业务语义...</span>
            <span className="loading-hint">这可能需要几秒钟</span>
          </div>
        </div>
      </div>
    )
  }

  // Render error state
  if (error) {
    return (
      <div className="business-semantics-panel">
        <div className="panel-header">
          <h3>{title}</h3>
          {functionName && <span className="function-badge">{functionName}()</span>}
        </div>
        <div className="panel-error">
          <span className="error-icon">❌</span>
          <div className="error-content">
            <span className="error-message">{error}</span>
            {onRetry && (
              <button className="retry-btn" onClick={onRetry}>
                重新分析
              </button>
            )}
          </div>
        </div>
      </div>
    )
  }

  // Render empty state
  if (!explanation) {
    return (
      <div className="business-semantics-panel">
        <div className="panel-header">
          <h3>{title}</h3>
          {functionName && <span className="function-badge">{functionName}()</span>}
        </div>
        <div className="panel-empty">
          <span className="empty-icon">🔍</span>
          <p>暂无业务语义信息</p>
          <span className="empty-hint">请先执行函数分析</span>
        </div>
      </div>
    )
  }

  // Render explanation content
  return (
    <div className="business-semantics-panel">
      <div className="panel-header">
        <h3>{title}</h3>
        {functionName && <span className="function-badge">{functionName}()</span>}
        <button
          className="toggle-btn"
          onClick={() => setExpanded(!expanded)}
          title={expanded ? '收起' : '展开'}
        >
          {expanded ? '▼' : '▶'}
        </button>
      </div>

      {expanded && (
        <div className="panel-content">
          {/* Trigger condition */}
          <div className="semantics-section trigger-section">
            <div className="section-header">
              <span className="section-icon">{getFieldIcon('trigger')}</span>
              <h4>触发条件</h4>
              {onFeedback && editingField !== 'trigger' && (
                <button
                  className="edit-btn"
                  onClick={() => handleEdit('trigger', explanation.triggerCondition)}
                  title="编辑"
                >
                  ✏
                </button>
              )}
            </div>
            {editingField === 'trigger' ? (
              <div className="edit-form">
                <textarea
                  value={editText}
                  onChange={(e) => setEditText(e.target.value)}
                  className="edit-input"
                />
                <div className="edit-actions">
                  <button className="save-btn" onClick={() => handleSave('trigger')}>保存</button>
                  <button className="cancel-btn" onClick={handleCancel}>取消</button>
                </div>
              </div>
            ) : (
              <p className="section-content">{explanation.triggerCondition}</p>
            )}
          </div>

          {/* Business meaning */}
          <div className="semantics-section meaning-section">
            <div className="section-header">
              <span className="section-icon">{getFieldIcon('meaning')}</span>
              <h4>业务含义</h4>
              {onFeedback && editingField !== 'meaning' && (
                <button
                  className="edit-btn"
                  onClick={() => handleEdit('meaning', explanation.businessMeaning)}
                  title="编辑"
                >
                  ✏
                </button>
              )}
            </div>
            {editingField === 'meaning' ? (
              <div className="edit-form">
                <textarea
                  value={editText}
                  onChange={(e) => setEditText(e.target.value)}
                  className="edit-input"
                />
                <div className="edit-actions">
                  <button className="save-btn" onClick={() => handleSave('meaning')}>保存</button>
                  <button className="cancel-btn" onClick={handleCancel}>取消</button>
                </div>
              </div>
            ) : (
              <p className="section-content">{explanation.businessMeaning}</p>
            )}
          </div>

          {/* Execution result */}
          <div className="semantics-section result-section">
            <div className="section-header">
              <span className="section-icon">{getFieldIcon('result')}</span>
              <h4>执行结果</h4>
              {onFeedback && editingField !== 'result' && (
                <button
                  className="edit-btn"
                  onClick={() => handleEdit('result', explanation.executionResult)}
                  title="编辑"
                >
                  ✏
                </button>
              )}
            </div>
            {editingField === 'result' ? (
              <div className="edit-form">
                <textarea
                  value={editText}
                  onChange={(e) => setEditText(e.target.value)}
                  className="edit-input"
                />
                <div className="edit-actions">
                  <button className="save-btn" onClick={() => handleSave('result')}>保存</button>
                  <button className="cancel-btn" onClick={handleCancel}>取消</button>
                </div>
              </div>
            ) : (
              <p className="section-content">{explanation.executionResult}</p>
            )}
          </div>

          {/* Related functions */}
          <div className="semantics-section related-section">
            <div className="section-header">
              <span className="section-icon">{getFieldIcon('related')}</span>
              <h4>相关函数</h4>
            </div>
            <ul className="related-list">
              {explanation.relatedFunctions.length > 0 ? (
                explanation.relatedFunctions.map((func, index) => (
                  <li key={index} className="related-item">
                    <code>{func}</code>
                  </li>
                ))
              ) : (
                <li className="empty-list">暂无相关信息</li>
              )}
            </ul>
          </div>

          {/* Common errors */}
          <div className="semantics-section errors-section">
            <div className="section-header">
              <span className="section-icon">{getFieldIcon('errors')}</span>
              <h4>常见错误</h4>
            </div>
            <ul className="errors-list">
              {explanation.commonErrors.length > 0 ? (
                explanation.commonErrors.map((error, index) => (
                  <li key={index} className="error-item">
                    <span className="error-bullet">●</span>
                    <span>{error}</span>
                  </li>
                ))
              ) : (
                <li className="empty-list">暂无相关信息</li>
              )}
            </ul>
          </div>

          {/* Code snippet preview (optional) */}
          {codeSnippet && (
            <div className="code-preview">
              <div className="code-preview-header">
                <span>代码片段</span>
              </div>
              <pre className="code-preview-content">
                <code>{codeSnippet}</code>
              </pre>
            </div>
          )}

          {/* Footer */}
          <div className="panel-footer">
            <div className="ai-info">
              <span className="ai-icon">🤖</span>
              <span>AI 生成</span>
            </div>
            <div className="footer-actions">
              {onFeedback && (
                <button className="feedback-btn" onClick={() => onFeedback('general', '')}>
                  反馈
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default BusinessSemanticsPanel
