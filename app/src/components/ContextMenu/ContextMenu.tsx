/**
 * ContextMenu - Enhanced Context Menu Component
 *
 * Features:
 * - Keyboard navigation (Arrow keys, Enter, Escape)
 * - Nested submenus support
 * - Auto-positioning to stay in viewport
 * - Accessibility support
 */

import { useState, useCallback, useRef, useEffect, useMemo } from 'react'
import './ContextMenu.css'

// Menu item types
export interface MenuItem {
  id?: string
  label: string
  icon?: string
  shortcut?: string
  disabled?: boolean
  divider?: boolean
  submenu?: MenuItem[]
  action?: () => void
}

interface ContextMenuProps {
  isOpen: boolean
  position: { x: number; y: number }
  items: MenuItem[]
  onClose: () => void
  onAction?: (actionId: string) => void
}

// Context menu component
export function ContextMenu({ isOpen, position, items, onClose, onAction }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null)
  const [activeIndex, setActiveIndex] = useState(0)
  const [submenuOpen, setSubmenuOpen] = useState(false)
  const [submenuPosition, setSubmenuPosition] = useState({ x: 0, y: 0 })
  const [submenuActiveIndex, setSubmenuActiveIndex] = useState(0)

  // Get visible items (exclude dividers from navigation)
  const visibleItems = useMemo(() =>
    items.map((item, index) => ({ ...item, originalIndex: index }))
        .filter(item => !item.divider),
    [items]
  )

  // Adjust position to stay in viewport
  const adjustedPosition = useMemo(() => {
    if (!isOpen || !menuRef.current) return position

    const menu = menuRef.current
    const rect = menu.getBoundingClientRect()
    const viewportWidth = window.innerWidth
    const viewportHeight = window.innerHeight

    let x = position.x
    let y = position.y

    // Adjust horizontal position
    if (x + rect.width > viewportWidth - 20) {
      x = Math.max(20, viewportWidth - rect.width - 20)
    }

    // Adjust vertical position
    if (y + rect.height > viewportHeight - 20) {
      y = Math.max(20, viewportHeight - rect.height - 20)
    }

    return { x, y }
  }, [isOpen, position])

  // Calculate submenu position
  const adjustedSubmenuPosition = useMemo(() => {
    const submenuWidth = 180
    const submenuHeight = 200 // Estimate
    const viewportWidth = window.innerWidth
    const viewportHeight = window.innerHeight

    let x = submenuPosition.x
    let y = submenuPosition.y

    if (x + submenuWidth > viewportWidth - 20) {
      x = adjustedPosition.x - submenuWidth
    }

    if (y + submenuHeight > viewportHeight - 20) {
      y = Math.max(20, viewportHeight - submenuHeight - 20)
    }

    return { x, y }
  }, [submenuPosition, adjustedPosition])

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (!isOpen) return

    const itemsCount = visibleItems.length

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        setActiveIndex(prev => (prev + 1) % itemsCount)
        setSubmenuOpen(false)
        break

      case 'ArrowUp':
        e.preventDefault()
        setActiveIndex(prev => (prev - 1 + itemsCount) % itemsCount)
        setSubmenuOpen(false)
        break

      case 'ArrowRight':
        e.preventDefault()
        const currentItem = visibleItems[activeIndex]
        if (currentItem?.submenu && currentItem.submenu.length > 0) {
          setSubmenuOpen(true)
          setSubmenuPosition({
            x: adjustedPosition.x + 200,
            y: adjustedPosition.y + activeIndex * 32
          })
          setSubmenuActiveIndex(0)
        }
        break

      case 'ArrowLeft':
        if (submenuOpen) {
          setSubmenuOpen(false)
        }
        break

      case 'Enter':
        e.preventDefault()
        if (submenuOpen) {
          const subItems = visibleItems[activeIndex]?.submenu || []
          if (subItems[submenuActiveIndex] && !subItems[submenuActiveIndex].disabled) {
            if (subItems[submenuActiveIndex].action) {
              subItems[submenuActiveIndex].action!()
            } else if (onAction) {
              onAction(subItems[submenuActiveIndex].id || '')
            }
            onClose()
          }
        } else {
          const item = visibleItems[activeIndex]
          if (item && !item.disabled && !item.divider) {
            if (item.action) {
              item.action()
            } else if (onAction && item.id) {
              onAction(item.id)
            }
            onClose()
          }
        }
        break

      case 'Escape':
        e.preventDefault()
        if (submenuOpen) {
          setSubmenuOpen(false)
        } else {
          onClose()
        }
        break
    }
  }, [isOpen, visibleItems, activeIndex, onClose, onAction, submenuOpen, submenuActiveIndex, adjustedPosition])

  // Keyboard listener
  useEffect(() => {
    if (isOpen) {
      window.addEventListener('keydown', handleKeyDown)
      setActiveIndex(0)
      setSubmenuOpen(false)
    }
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, handleKeyDown])

  // Click outside to close
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (isOpen && menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose()
      }
    }

    if (isOpen) {
      document.addEventListener('click', handleClick)
    }
    return () => document.removeEventListener('click', handleClick)
  }, [isOpen, onClose])

  // Handle item click
  const handleItemClick = useCallback((item: MenuItem, index: number) => {
    if (item.disabled || item.divider) return

    if (item.submenu && item.submenu.length > 0) {
      setSubmenuOpen(true)
      setSubmenuPosition({
        x: adjustedPosition.x + 200,
        y: adjustedPosition.y + index * 32
      })
      setSubmenuActiveIndex(0)
    } else {
      if (item.action) {
        item.action()
      } else if (onAction && item.id) {
        onAction(item.id)
      }
      onClose()
    }
  }, [adjustedPosition, onClose, onAction])

  // Get current item for submenu
  const currentItem = visibleItems[activeIndex]
  const submenuItems = currentItem?.submenu || []

  if (!isOpen) return null

  return (
    <>
      {/* Backdrop */}
      <div className="context-menu-backdrop" onClick={onClose} />

      {/* Main menu */}
      <div
        ref={menuRef}
        className="context-menu"
        style={{ left: adjustedPosition.x, top: adjustedPosition.y }}
        role="menu"
        aria-orientation="vertical"
      >
        {items.map((item, index) => {
          if (item.divider) {
            return <div key={index} className="context-menu-divider" />
          }

          const isActive = !submenuOpen && index === activeIndex
          const hasSubmenu = item.submenu && item.submenu.length > 0

          return (
            <div
              key={index}
              className={`context-menu-item ${isActive ? 'active' : ''} ${item.disabled ? 'disabled' : ''}`}
              onClick={() => handleItemClick(item, index)}
              onMouseEnter={() => {
                setActiveIndex(index)
                if (hasSubmenu) {
                  setSubmenuOpen(true)
                  setSubmenuPosition({
                    x: adjustedPosition.x + 200,
                    y: adjustedPosition.y + index * 32
                  })
                } else {
                  setSubmenuOpen(false)
                }
              }}
              role="menuitem"
              aria-disabled={item.disabled}
            >
              {item.icon && <span className="context-menu-icon">{item.icon}</span>}
              <span className="context-menu-label">{item.label}</span>
              {item.shortcut && (
                <span className="context-menu-shortcut">{item.shortcut}</span>
              )}
              {hasSubmenu && <span className="context-menu-arrow">▶</span>}
            </div>
          )
        })}
      </div>

      {/* Submenu */}
      {submenuOpen && submenuItems.length > 0 && (
        <div
          className="context-menu submenu"
          style={{ left: adjustedSubmenuPosition.x, top: adjustedSubmenuPosition.y }}
          role="menu"
        >
          {submenuItems.map((item, index) => {
            if (item.divider) {
              return <div key={index} className="context-menu-divider" />
            }

            return (
              <div
                key={index}
                className={`context-menu-item ${index === submenuActiveIndex ? 'active' : ''} ${item.disabled ? 'disabled' : ''}`}
                onClick={(e) => {
                  e.stopPropagation()
                  if (!item.disabled && !item.divider) {
                    if (item.action) {
                      item.action()
                    } else if (onAction && item.id) {
                      onAction(item.id)
                    }
                    onClose()
                  }
                }}
                onMouseEnter={() => setSubmenuActiveIndex(index)}
                role="menuitem"
                aria-disabled={item.disabled}
              >
                {item.icon && <span className="context-menu-icon">{item.icon}</span>}
                <span className="context-menu-label">{item.label}</span>
                {item.shortcut && (
                  <span className="context-menu-shortcut">{item.shortcut}</span>
                )}
              </div>
            )
          })}
        </div>
      )}
    </>
  )
}

// Hook for using context menu
export function useContextMenu() {
  const [isOpen, setIsOpen] = useState(false)
  const [position, setPosition] = useState({ x: 0, y: 0 })
  const [items, setItems] = useState<MenuItem[]>([])

  const show = useCallback((x: number, y: number, menuItems: MenuItem[]) => {
    setPosition({ x, y })
    setItems(menuItems)
    setIsOpen(true)
  }, [])

  const hide = useCallback(() => {
    setIsOpen(false)
  }, [])

  const toggle = useCallback((x: number, y: number, menuItems: MenuItem[]) => {
    if (isOpen) {
      hide()
    } else {
      show(x, y, menuItems)
    }
  }, [isOpen, show, hide])

  const ContextMenuComponent = useCallback((props: Omit<ContextMenuProps, 'isOpen' | 'position' | 'items'>) => (
    <ContextMenu
      isOpen={isOpen}
      position={position}
      items={items}
      {...props}
    />
  ), [isOpen, position, items])

  return {
    isOpen,
    position,
    items,
    show,
    hide,
    toggle,
    ContextMenu: ContextMenuComponent
  }
}

export default ContextMenu
