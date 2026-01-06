/**
 * 空状态组件
 */

import './EmptyState.css'

interface EmptyStateProps {
  icon?: string
  title: string
  description?: string
  action?: {
    label: string
    onClick: () => void
  }
}

export function EmptyState({ icon = '📭', title, description, action }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <div className="empty-icon">{icon}</div>
      <h3 className="empty-title">{title}</h3>
      {description && <p className="empty-description">{description}</p>}
      {action && (
        <button className="empty-action" onClick={action.onClick}>
          {action.label}
        </button>
      )}
    </div>
  )
}

export default EmptyState

