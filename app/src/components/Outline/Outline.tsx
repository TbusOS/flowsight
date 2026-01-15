/**
 * Outline - Enhanced Code Outline Component
 *
 * Features:
 * - Search/filter symbols
 * - Keyboard navigation (Arrow keys, Enter, / to search)
 * - Hover previews
 * - Collapsible sections
 * - Quick navigation
 */

import React, { useState, useCallback, useMemo, useEffect, useRef } from 'react'
import './Outline.css'

export interface OutlineItem {
  name: string
  kind: 'function' | 'struct' | 'variable' | 'macro' | 'enum' | 'typedef'
  line: number
  endLine?: number
  isCallback?: boolean
  returnType?: string
  signature?: string
  filePath?: string
}

interface OutlineProps {
  items: OutlineItem[]
  onItemClick: (item: OutlineItem) => void
  selectedItem?: string
  filePath?: string
}

// Kind configuration
const kindConfig: Record<string, { icon: string; color: string; label: string }> = {
  function: { icon: '📦', color: '#60a5fa', label: 'Functions' },
  struct: { icon: '🏗️', color: '#c084fc', label: 'Structures' },
  variable: { icon: '📌', color: '#fbbf24', label: 'Variables' },
  macro: { icon: '🔧', color: '#34d399', label: 'Macros' },
  enum: { icon: '📋', color: '#f472b6', label: 'Enums' },
  typedef: { icon: '📝', color: '#38bdf8', label: 'Typedefs' },
}

// Sort items by line number
function sortByLine(items: OutlineItem[]): OutlineItem[] {
  return [...items].sort((a, b) => a.line - b.line)
}

// Filter items by search query
function filterItems(items: OutlineItem[], query: string): OutlineItem[] {
  if (!query.trim()) return items
  const lower = query.toLowerCase()
  return items.filter(item =>
    item.name.toLowerCase().includes(lower) ||
    item.returnType?.toLowerCase().includes(lower) ||
    item.signature?.toLowerCase().includes(lower)
  )
}

export function Outline({ items, onItemClick, selectedItem }: OutlineProps) {
  // State
  const [searchQuery, setSearchQuery] = useState('')
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(['function', 'struct']))
  const [activeIndex, setActiveIndex] = useState(0)
  const [isSearching, setIsSearching] = useState(false)
  const listRef = useRef<HTMLUListElement>(null)
  const searchInputRef = useRef<HTMLInputElement>(null)

  // Get filtered and sorted items
  const filteredItems = useMemo(() =>
    sortByLine(filterItems(items, searchQuery)),
    [items, searchQuery]
  )

  // Group items by kind
  const groupedItems = useMemo(() => {
    const groups: Record<string, OutlineItem[]> = {}
    filteredItems.forEach(item => {
      if (!groups[item.kind]) groups[item.kind] = []
      groups[item.kind].push(item)
    })
    return groups
  }, [filteredItems])

  // Get all visible items for keyboard navigation
  const allVisibleItems = useMemo(() => {
    const result: OutlineItem[] = []
    Object.keys(kindConfig).forEach(kind => {
      if (expandedSections.has(kind) && groupedItems[kind]) {
        result.push(...groupedItems[kind])
      }
    })
    return result
  }, [groupedItems, expandedSections])

  // Toggle section expansion
  const toggleSection = useCallback((kind: string) => {
    setExpandedSections(prev => {
      const next = new Set(prev)
      if (next.has(kind)) {
        next.delete(kind)
      } else {
        next.add(kind)
      }
      return next
    })
  }, [])

  // Handle item click
  const handleItemClick = useCallback((item: OutlineItem) => {
    onItemClick(item)
  }, [onItemClick])

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    // Only handle if outline is focused or we're searching
    if (!isSearching && document.activeElement !== listRef.current && document.activeElement !== searchInputRef.current) {
      return
    }

    // "/" to focus search
    if (e.key === '/' && !isSearching && document.activeElement !== searchInputRef.current) {
      e.preventDefault()
      setIsSearching(true)
      setTimeout(() => searchInputRef.current?.focus(), 10)
      return
    }

    // Escape to cancel search
    if (e.key === 'Escape' && isSearching) {
      e.preventDefault()
      setIsSearching(false)
      setSearchQuery('')
      searchInputRef.current?.blur()
      return
    }

    // Arrow navigation
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault()
      const direction = e.key === 'ArrowDown' ? 1 : -1
      setActiveIndex(prev => {
        const next = prev + direction
        if (next < 0) return allVisibleItems.length - 1
        if (next >= allVisibleItems.length) return 0
        return next
      })
    }

    // Enter to select
    if (e.key === 'Enter' && allVisibleItems[activeIndex]) {
      e.preventDefault()
      handleItemClick(allVisibleItems[activeIndex])
    }
  }, [isSearching, allVisibleItems, activeIndex, handleItemClick])

  // Add keyboard listener
  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleKeyDown])

  // Scroll active item into view
  useEffect(() => {
    const activeElement = listRef.current?.children[activeIndex] as HTMLElement
    if (activeElement) {
      activeElement.scrollIntoView({ block: 'nearest' })
    }
  }, [activeIndex])

  // Clear selection when items change
  useEffect(() => {
    setActiveIndex(0)
  }, [filteredItems.length])

  // Empty state
  if (items.length === 0) {
    return (
      <div className="outline outline-empty">
        <div className="empty-icon">📋</div>
        <p>No symbols found</p>
        <span className="empty-hint">Open a source file to see its outline</span>
      </div>
    )
  }

  // No results state
  if (filteredItems.length === 0) {
    return (
      <div className="outline outline-empty">
        <div className="empty-icon">🔍</div>
        <p>No matching symbols</p>
        <span className="empty-hint">Try a different search term</span>
        {searchQuery && (
          <button
            className="clear-search-btn"
            onClick={() => setSearchQuery('')}
          >
            Clear search
          </button>
        )}
      </div>
    )
  }

  return (
    <div className="outline">
      {/* Search bar */}
      <div className="outline-search">
        <input
          ref={searchInputRef}
          type="text"
          placeholder="🔍 / Search..."
          value={searchQuery}
          onChange={(e) => {
            setSearchQuery(e.target.value)
            setActiveIndex(0)
          }}
          onFocus={() => setIsSearching(true)}
          onBlur={() => setIsSearching(false)}
        />
        {searchQuery && (
          <button
            className="clear-btn"
            onClick={() => {
              setSearchQuery('')
              searchInputRef.current?.focus()
            }}
            title="Clear search"
          >
            ✕
          </button>
        )}
      </div>

      {/* Keyboard hint */}
      {!isSearching && !searchQuery && (
        <div className="outline-hint">
          Press <kbd>/</kbd> to search
        </div>
      )}

      {/* Outline sections */}
      <div className="outline-content" ref={listRef as any}>
        {Object.entries(kindConfig).map(([kind, config]) => {
          const sectionItems = groupedItems[kind]
          if (!sectionItems || sectionItems.length === 0) return null

          const isExpanded = expandedSections.has(kind)

          return (
            <div key={kind} className="outline-section">
              <button
                className={`outline-section-header ${isExpanded ? 'expanded' : ''}`}
                onClick={() => toggleSection(kind)}
              >
                <span className="section-icon">{config.icon}</span>
                <span className="section-label">{config.label}</span>
                <span className="section-count">{sectionItems.length}</span>
                <span className={`section-arrow ${isExpanded ? 'rotated' : ''}`}>
                  ▶
                </span>
              </button>

              {isExpanded && (
                <ul className="outline-list">
                  {sectionItems.map((item, index) => {
                    const globalIndex = allVisibleItems.indexOf(item)
                    const isActive = globalIndex === activeIndex
                    const isSelected = selectedItem === item.name

                    return (
                      <li
                        key={`${item.name}-${item.line}`}
                        className={`outline-item ${isSelected ? 'selected' : ''} ${isActive ? 'active' : ''} ${item.isCallback ? 'callback' : ''}`}
                        onClick={() => handleItemClick(item)}
                        onMouseEnter={() => setActiveIndex(globalIndex)}
                        style={{ '--item-index': index } as React.CSSProperties}
                      >
                        <span
                          className="item-icon"
                          style={{ color: config.color }}
                          title={config.label}
                        >
                          {item.isCallback ? '⚡' : config.icon}
                        </span>
                        <span className="item-name">{item.name}</span>
                        {item.returnType && (
                          <span className="item-type" title={item.returnType}>
                            {item.returnType}
                          </span>
                        )}
                        <span className="item-line">:{item.line}</span>

                        {/* Quick actions on hover */}
                        <div className="item-actions">
                          <button
                            className="action-btn"
                            title="Go to definition"
                            onClick={(e) => {
                              e.stopPropagation()
                              handleItemClick(item)
                            }}
                          >
                            →
                          </button>
                        </div>
                      </li>
                    )
                  })}
                </ul>
              )}
            </div>
          )
        })}
      </div>

      {/* Status bar */}
      <div className="outline-status">
        <span>{filteredItems.length} / {items.length} symbols</span>
        {searchQuery && (
          <span className="search-badge">Filtered</span>
        )}
      </div>
    </div>
  )
}

export default Outline
