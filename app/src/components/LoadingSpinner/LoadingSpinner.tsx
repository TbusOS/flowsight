/**
 * 加载动画组件
 */

import { Loader2 } from 'lucide-react'
import './LoadingSpinner.css'

interface LoadingSpinnerProps {
  size?: 'small' | 'medium' | 'large'
  message?: string
  overlay?: boolean
  onCancel?: () => void
}

export function LoadingSpinner({
  size = 'medium',
  message,
  overlay = false,
  onCancel,
}: LoadingSpinnerProps) {
  const content = (
    <div className={`loading-spinner ${size}`}>
      <div className="spinner-container">
        <Loader2 className="spinner-icon" strokeWidth={2.5} />
      </div>
      {message && <span className="spinner-message">{message}</span>}
      {onCancel && (
        <button className="spinner-cancel" onClick={onCancel}>
          取消
        </button>
      )}
    </div>
  )

  if (overlay) {
    return (
      <div className="loading-overlay">
        {content}
      </div>
    )
  }

  return content
}

export default LoadingSpinner

