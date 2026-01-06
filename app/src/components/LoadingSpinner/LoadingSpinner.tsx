/**
 * 加载动画组件
 */

import './LoadingSpinner.css'

interface LoadingSpinnerProps {
  size?: 'small' | 'medium' | 'large'
  message?: string
  overlay?: boolean
}

export function LoadingSpinner({ 
  size = 'medium', 
  message, 
  overlay = false 
}: LoadingSpinnerProps) {
  const content = (
    <div className={`loading-spinner ${size}`}>
      <div className="spinner-icon">
        <div className="spinner-ring"></div>
        <div className="spinner-ring"></div>
        <div className="spinner-ring"></div>
      </div>
      {message && <span className="spinner-message">{message}</span>}
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

