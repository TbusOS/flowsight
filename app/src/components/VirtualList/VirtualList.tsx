/**
 * VirtualList - 通用虚拟滚动列表组件
 * 
 * 基于 react-window 实现，支持固定高度的列表项虚拟化渲染
 * 适用于：大纲面板、搜索结果、调用者列表等
 */

import React, { useRef, useEffect, useMemo, useCallback } from 'react'
import { List, ListImperativeAPI, RowComponentProps } from 'react-window'
import { cn } from '../../lib/utils/cn'

export interface VirtualListProps<T> {
  /** 列表数据 */
  items: T[]
  /** 选中项的 key */
  selectedKey?: string | null
  /** 获取每项的唯一 key */
  getItemKey: (item: T) => string
  /** 渲染单项内容 */
  renderItem: (item: T, index: number, isSelected: boolean) => React.ReactNode
  /** 点击项回调 */
  onItemClick?: (item: T, index: number) => void
  /** 容器高度 */
  height: number
  /** 每项高度（默认 32） */
  itemHeight?: number
  /** 额外类名 */
  className?: string
  /** 空状态内容 */
  emptyContent?: React.ReactNode
  /** 自定义项容器类名 */
  itemClassName?: string | ((item: T, isSelected: boolean) => string)
  /** 过扫描行数（默认 5） */
  overscanCount?: number
}

interface RowProps<T> {
  items: T[]
  selectedKey: string | null
  getItemKey: (item: T) => string
  renderItem: (item: T, index: number, isSelected: boolean) => React.ReactNode
  onItemClick?: (item: T, index: number) => void
  itemClassName?: string | ((item: T, isSelected: boolean) => string)
}

// 通用行组件
function VirtualListRow<T>({ 
  index, 
  style,
  items,
  selectedKey,
  getItemKey,
  renderItem,
  onItemClick,
  itemClassName,
}: RowComponentProps<RowProps<T>>) {
  const item = items[index]
  const key = getItemKey(item)
  const isSelected = selectedKey === key
  
  const handleClick = useCallback(() => {
    onItemClick?.(item, index)
  }, [item, index, onItemClick])
  
  const className = typeof itemClassName === 'function' 
    ? itemClassName(item, isSelected) 
    : itemClassName
  
  return (
    <div
      style={style}
      className={cn(
        'cursor-pointer transition-colors',
        className,
        isSelected && 'bg-[var(--accent)]/10'
      )}
      onClick={handleClick}
      role="option"
      aria-selected={isSelected}
    >
      {renderItem(item, index, isSelected)}
    </div>
  )
}

/**
 * 通用虚拟滚动列表组件
 */
export function VirtualList<T>({
  items,
  selectedKey = null,
  getItemKey,
  renderItem,
  onItemClick,
  height,
  itemHeight = 32,
  className,
  emptyContent,
  itemClassName,
  overscanCount = 5,
}: VirtualListProps<T>) {
  const listRef = useRef<ListImperativeAPI | null>(null)

  // 选中项变化时滚动到可见
  useEffect(() => {
    if (selectedKey && listRef.current) {
      const index = items.findIndex(item => getItemKey(item) === selectedKey)
      if (index >= 0) {
        listRef.current.scrollToRow({ index, align: 'smart' })
      }
    }
  }, [selectedKey, items, getItemKey])

  // 行属性（稳定引用）
  const rowProps = useMemo<RowProps<T>>(() => ({
    items,
    selectedKey,
    getItemKey,
    renderItem,
    onItemClick,
    itemClassName,
  }), [items, selectedKey, getItemKey, renderItem, onItemClick, itemClassName])

  if (items.length === 0) {
    return (
      <div className={cn('flex items-center justify-center h-full', className)}>
        {emptyContent || (
          <span className="text-xs text-[var(--text-muted)]">无数据</span>
        )}
      </div>
    )
  }

  return (
    <List<RowProps<T>>
      listRef={listRef}
      className={className}
      defaultHeight={height}
      rowCount={items.length}
      rowHeight={itemHeight}
      rowComponent={VirtualListRow<T>}
      rowProps={rowProps}
      overscanCount={overscanCount}
      role="listbox"
    />
  )
}

export default VirtualList
