/**
 * 右键上下文菜单组件
 */

import { useEffect, useRef, useCallback } from 'react'
import './ContextMenu.css'

export interface MenuItem {
  label: string
  icon?: string
  shortcut?: string
  onClick: () => void
  disabled?: boolean
  divider?: boolean
}

interface ContextMenuProps {
  x: number
  y: number
  items: MenuItem[]
  onClose: () => void
}

export function ContextMenu({ x, y, items, onClose }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null)

  // 调整位置确保菜单在视口内
  const adjustPosition = useCallback(() => {
    if (!menuRef.current) return { x, y }
    
    const rect = menuRef.current.getBoundingClientRect()
    let adjustedX = x
    let adjustedY = y
    
    if (x + rect.width > window.innerWidth) {
      adjustedX = window.innerWidth - rect.width - 8
    }
    if (y + rect.height > window.innerHeight) {
      adjustedY = window.innerHeight - rect.height - 8
    }
    
    return { x: adjustedX, y: adjustedY }
  }, [x, y])

  // 点击外部关闭
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose()
      }
    }
    
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose()
      }
    }
    
    // 延迟添加监听，防止立即触发
    const timer = setTimeout(() => {
      document.addEventListener('click', handleClick)
      document.addEventListener('keydown', handleEsc)
    }, 0)
    
    return () => {
      clearTimeout(timer)
      document.removeEventListener('click', handleClick)
      document.removeEventListener('keydown', handleEsc)
    }
  }, [onClose])

  const pos = adjustPosition()

  return (
    <div 
      ref={menuRef}
      className="context-menu"
      style={{ left: pos.x, top: pos.y }}
    >
      {items.map((item, index) => (
        item.divider ? (
          <div key={index} className="menu-divider" />
        ) : (
          <button
            key={index}
            className={`menu-item ${item.disabled ? 'disabled' : ''}`}
            onClick={() => {
              if (!item.disabled) {
                item.onClick()
                onClose()
              }
            }}
            disabled={item.disabled}
          >
            {item.icon && <span className="menu-icon">{item.icon}</span>}
            <span className="menu-label">{item.label}</span>
            {item.shortcut && <span className="menu-shortcut">{item.shortcut}</span>}
          </button>
        )
      ))}
    </div>
  )
}

export default ContextMenu

