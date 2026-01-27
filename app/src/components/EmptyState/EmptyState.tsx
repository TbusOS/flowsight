/**
 * 空状态组件
 */

import { Inbox, Plus } from 'lucide-react'
import './EmptyState.css'

interface EmptyStateProps {
  icon?: React.ReactNode
  title: string
  description?: string
  action?: {
    label: string
    onClick: () => void
  }
}

export function EmptyState({ icon, title, description, action }: EmptyStateProps) {
  const defaultIcon = <Inbox className="w-12 h-12" strokeWidth={1} />

  return (
    <div className="empty-state">
      <div className="empty-icon">{icon || defaultIcon}</div>
      <h3 className="empty-title">{title}</h3>
      {description && <p className="empty-description">{description}</p>}
      {action && (
        <button className="empty-action" onClick={action.onClick}>
          <Plus className="w-4 h-4" strokeWidth={2} />
          {action.label}
        </button>
      )}
    </div>
  )
}

export default EmptyState

